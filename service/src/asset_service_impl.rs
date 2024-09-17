use std::sync::Arc;

use chrono::Utc;
use entities::l2::{AssetSorting, L2Asset, PublicKey};
use interfaces::{
    asset_service::{AssetService, L1MintError, L2AssetInfo},
    asset_storage::{AssetMetadataStorage, BlobStorage},
    l1_service::{L1Service, ParsedMintIxInfo},
    l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage, L2StorageError},
};
use solana_sdk::{signer::Signer, transaction::Transaction};
use tracing::info;
use util::{hd_wallet::HdWalletProducer, nft_json::validate_metadata_contains_uris};

use crate::converter::get_metadata_uri_for_key;

#[derive(Clone)]
pub struct AssetServiceImpl {
    pub wallet_producer: HdWalletProducer,
    pub derivation_sequence: Arc<dyn Bip44DerivationSequence + Sync + Send>,
    pub l2_storage: Arc<dyn L2Storage + Sync + Send>,
    pub asset_metadata_storage: Arc<dyn AssetMetadataStorage + Sync + Send>,
    pub blob_storage: Arc<dyn BlobStorage + Sync + Send>,
    pub s1_service: Arc<dyn L1Service + Sync + Send>,
    pub metadata_server_base_url: String,
}

#[async_trait::async_trait]
impl AssetService for AssetServiceImpl {
    async fn create_asset(
        &self,
        metadata_json: &str,
        owner: PublicKey,
        creator: PublicKey,
        authority: PublicKey,
        name: &str,
        collection: Option<PublicKey>,
    ) -> anyhow::Result<L2AssetInfo> {
        validate_metadata_contains_uris(metadata_json)?;

        let DerivationValues { account, address } = self.derivation_sequence.next_account_and_address().await?;
        let keypair = self.wallet_producer.make_hd_wallet(account, address);
        let asset_pubkey = keypair.pubkey().to_bytes();

        self.asset_metadata_storage
            .put_json(&asset_pubkey, metadata_json)
            .await?;

        let utc_now = Utc::now().naive_local();

        let asset = L2Asset {
            pubkey: asset_pubkey,
            name: name.to_string(),
            owner,
            creator,
            collection,
            authority,
            create_timestamp: utc_now,
            update_timestamp: utc_now,
            bip44_account_num: account,
            bip44_address_num: address,
        };

        self.l2_storage.save(&asset).await?;

        Ok(L2AssetInfo { asset, metadata: Some(metadata_json.to_string()) })
    }

    async fn update_asset(
        &self,
        asset_pubkey: PublicKey,
        metadata_json: Option<&str>,
        owner: Option<PublicKey>,
        creator: Option<PublicKey>,
        authority: Option<PublicKey>,
        name: Option<&str>,
        collection: Option<Option<PublicKey>>,
    ) -> anyhow::Result<Option<L2AssetInfo>> {
        if let Some(mut asset) = self.l2_storage.find(&asset_pubkey).await? {
            let metadata = if let Some(v) = metadata_json {
                self.asset_metadata_storage.put_json(&asset_pubkey, v).await?;
                Some(v.to_string())
            } else {
                self.asset_metadata_storage.get_json(&asset_pubkey).await?
            };
            if let Some(v) = owner {
                asset.owner = v;
            };
            if let Some(v) = creator {
                asset.creator = v;
            };
            if let Some(v) = authority {
                asset.authority = v;
            };
            if let Some(v) = name {
                asset.name = v.to_string();
            };
            if let Some(v) = collection {
                asset.collection = v;
            };

            self.l2_storage.save(&asset).await?;

            Ok(Some(L2AssetInfo { asset, metadata }))
        } else {
            Ok(None)
        }
    }

    async fn fetch_asset(&self, asset_pubkey: PublicKey) -> anyhow::Result<Option<L2AssetInfo>> {
        let metadata = self.asset_metadata_storage.get_json(&asset_pubkey).await?;
        let asset_op = self.l2_storage.find(&asset_pubkey).await?;

        Ok(asset_op.map(|asset| L2AssetInfo { asset, metadata }))
    }

    async fn fetch_assets(&self, asset_pubkeys: &[PublicKey]) -> anyhow::Result<Vec<L2AssetInfo>> {
        let (asset_op, metadata) = (
            self.l2_storage.find_batch(asset_pubkeys).await?,
            self.asset_metadata_storage.get_json_batch(asset_pubkeys).await?,
        );

        Ok(asset_op
            .into_iter()
            .zip(metadata)
            .map(|(asset, metadata)| L2AssetInfo { asset, metadata })
            .collect())
    }

    async fn fetch_metadata(&self, asset_pubkey: PublicKey) -> anyhow::Result<Option<String>> {
        self.asset_metadata_storage.get_json(&asset_pubkey).await
    }

    async fn fetch_assets_by_owner(
        &self,
        owner_pubkey: PublicKey,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2AssetInfo>> {
        let l2_assets = self
            .l2_storage
            .find_by_owner(&owner_pubkey, sorting, limit, before, after)
            .await?;
        let l2_asset_pubkeys = l2_assets.iter().map(|asset| asset.pubkey).collect::<Vec<PublicKey>>();
        let l2_assets_metadata = self.asset_metadata_storage.get_json_batch(&l2_asset_pubkeys).await?;

        Ok(l2_assets
            .into_iter()
            .zip(l2_assets_metadata)
            .map(|(asset, metadata)| L2AssetInfo { asset, metadata })
            .collect())
    }

    async fn fetch_assets_by_creator(
        &self,
        creator_pubkey: PublicKey,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2AssetInfo>> {
        let l2_assets = self
            .l2_storage
            .find_by_creator(&creator_pubkey, sorting, limit, before, after)
            .await?;
        let l2_asset_pubkeys = l2_assets.iter().map(|asset| asset.pubkey).collect::<Vec<PublicKey>>();
        let l2_assets_metadata = self.asset_metadata_storage.get_json_batch(&l2_asset_pubkeys).await?;

        Ok(l2_assets
            .into_iter()
            .zip(l2_assets_metadata)
            .map(|(asset, metadata)| L2AssetInfo { asset, metadata })
            .collect())
    }

    async fn execute_asset_l1_mint(&self, tx: Transaction) -> anyhow::Result<()> {
        let mint_ix = self.s1_service.parse_mint_transaction(&tx)?;
        let asset_pubkey = mint_ix.asset_pubkey;

        let Some(l2_asset) = self.l2_storage.find(&asset_pubkey).await? else {
            anyhow::bail!(L2StorageError::L2AssetNotFound(asset_pubkey));
        };

        self.validate_mint_transaction_data(&mint_ix, &l2_asset)?;

        if !self.l2_storage.lock_asset_before_minting(&asset_pubkey).await? {
            anyhow::bail!(L1MintError::NotUnlockedL2Asset)
        };

        let asset_kp = self
            .wallet_producer
            .make_hd_wallet(l2_asset.bip44_account_num, l2_asset.bip44_address_num);

        let tx_signature = match self.s1_service.execute_mint_transaction(tx, &asset_kp).await {
            Ok(signature) => signature,
            Err(e) => {
                self.l2_storage.mint_didnt_happen(&asset_pubkey).await?;
                anyhow::bail!(e);
            }
        };

        info!("Successfully minted asset={asset_pubkey:?} in transaction={tx_signature}");

        let _ = self.l2_storage.finilize_minted(&asset_pubkey).await;

        Ok(())
    }
}

impl AssetServiceImpl {
    fn validate_mint_transaction_data(&self, mint_ix: &ParsedMintIxInfo, l2_asset: &L2Asset) -> anyhow::Result<()> {
        let expected_metadata_url = get_metadata_uri_for_key(&self.metadata_server_base_url, l2_asset.pubkey);

        if mint_ix.uri != expected_metadata_url {
            anyhow::bail!(L1MintError::WrongMetadataUri)
        }
        if mint_ix.name != l2_asset.name {
            anyhow::bail!(L1MintError::WrongName(l2_asset.name.clone(), mint_ix.name.clone()))
        }
        if let Some(authority) = mint_ix.authority {
            if authority != l2_asset.authority {
                anyhow::bail!(L1MintError::WrongAuthority)
            }
        } else {
            anyhow::bail!(L1MintError::MissingAuthority)
        }
        if let Some(owner) = mint_ix.owner {
            if owner != l2_asset.owner {
                anyhow::bail!(L1MintError::WrongOwner)
            }
        } else {
            anyhow::bail!(L1MintError::MissingOwner)
        }
        if mint_ix.collection != l2_asset.collection {
            anyhow::bail!(L1MintError::WrongOwner)
        }
        Ok(())
    }
}
