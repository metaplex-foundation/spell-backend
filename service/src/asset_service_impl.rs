use crate::converter::get_metadata_uri_for_key;
use chrono::Utc;
use entities::l2::{AssetSorting, L2Asset, PublicKey};
use interfaces::{
    asset_service::{AssetService, L1MintError, L2AssetInfo},
    asset_storage::{AssetMetadataStorage, BlobStorage},
    l1_service::{L1Service, ParsedMintIxInfo},
    l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage, L2StorageError},
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use solana_sdk::{signer::Signer, transaction::Transaction};
use std::sync::Arc;
use std::time::Duration;
use std::{future::Future, str::FromStr};
use tracing::{debug, error, info};
use util::publickey::PublicKeyExt;
use util::{hd_wallet::HdWalletProducer, nft_json::validate_metadata_contains_uris};

#[derive(Clone)]
pub struct AssetServiceImpl {
    pub wallet_producer: HdWalletProducer,
    pub derivation_sequence: Arc<dyn Bip44DerivationSequence + Sync + Send>,
    pub l2_storage: Arc<dyn L2Storage + Sync + Send>,
    pub asset_metadata_storage: Arc<dyn AssetMetadataStorage + Sync + Send>,
    pub blob_storage: Arc<dyn BlobStorage + Sync + Send>,
    pub l1_service: Arc<dyn L1Service + Sync + Send>,
    pub metadata_server_base_url: String,
}

#[async_trait::async_trait]
impl AssetService for AssetServiceImpl {
    async fn create_asset(
        &self,
        metadata_json: &str,
        owner: &str,
        creator: &str,
        authority: &str,
        name: &str,
        royalty_basis_points: u16,
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
            owner: owner.to_string(),
            creator: creator.to_string(),
            collection,
            authority: authority.to_string(),
            royalty_basis_points,
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
        owner: Option<String>,
        creator: Option<String>,
        authority: Option<String>,
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
        owner_pubkey: &str,
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
        creator_pubkey: &str,
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

    async fn execute_asset_l1_mint(&self, tx: Transaction, exec_sync: bool) -> anyhow::Result<()> {
        let mint_ix = self.l1_service.parse_mint_transaction(&tx)?;
        let asset_pubkey = mint_ix.asset_pubkey;

        let Some(l2_asset) = self.l2_storage.find(&asset_pubkey).await? else {
            anyhow::bail!(L2StorageError::L2AssetNotFound(asset_pubkey));
        };

        self.validate_mint_transaction_data(&mint_ix, &l2_asset)?;

        if !self.l2_storage.lock_asset_before_minting(&asset_pubkey).await? {
            if let Some(signature) = self.is_asset_already_sent_to_mint(&asset_pubkey).await {
                if let Ok(true) | Err(_) = self.l1_service.is_asset_minted(&signature).await {
                    anyhow::bail!(
                        "Asset '{l2_asset}' has already been sent for mint!",
                        l2_asset = l2_asset.pubkey.to_string()
                    )
                }
            }
        };

        let asset_kp = self
            .wallet_producer
            .make_hd_wallet(l2_asset.bip44_account_num, l2_asset.bip44_address_num);

        let tx_signature = match self.l1_service.execute_mint_transaction(tx, &asset_kp, exec_sync).await {
            Ok(signature) => {
                info!(
                    "Mint transaction '{signature}' for asset '{asset_pubkey}' successfully sent!",
                    asset_pubkey = asset_pubkey.to_string()
                );
                signature
            }
            Err(e) => {
                self.l2_storage.mint_didnt_happen(&asset_pubkey).await?;
                anyhow::bail!(e);
            }
        };

        if !exec_sync {
            self.l2_storage
                .add_l1_asset(&asset_pubkey, &tx_signature.as_ref())
                .await?;

            Self::in_background(Self::await_for_mint_status_and_save_it(
                tx_signature,
                asset_pubkey,
                self.l1_service.clone(),
                self.l2_storage.clone(),
            ));
        }

        Ok(())
    }
}

impl AssetServiceImpl {
    const AWAIT_TIME_TO_CALL_BLOCKCHAIN: Duration = Duration::from_secs(10);
    const AMOUNT_OF_ATTEMPTS_TO_CALL_BLOCKCHAIN: u8 = 18;

    fn validate_mint_transaction_data(&self, mint_ix: &ParsedMintIxInfo, l2_asset: &L2Asset) -> anyhow::Result<()> {
        let expected_metadata_url = get_metadata_uri_for_key(&self.metadata_server_base_url, l2_asset.pubkey);

        if mint_ix.uri != expected_metadata_url {
            anyhow::bail!(L1MintError::WrongMetadataUri)
        }
        if mint_ix.name != l2_asset.name {
            anyhow::bail!(L1MintError::WrongName(l2_asset.name.clone(), mint_ix.name.clone()))
        }
        if let Some(authority) = mint_ix.authority {
            let is_same = Pubkey::from_str(&l2_asset.authority)
                .map(|l2_authority| l2_authority.to_bytes() == authority)
                .unwrap_or(false);
            if !is_same {
                anyhow::bail!(L1MintError::WrongAuthority)
            }
        } else {
            anyhow::bail!(L1MintError::MissingAuthority)
        }
        if let Some(owner) = mint_ix.owner {
            let is_same = Pubkey::from_str(&l2_asset.owner)
                .map(|l2_owner| l2_owner.to_bytes() == owner)
                .unwrap_or(false);
            if !is_same {
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

    fn in_background<F>(future: F)
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future);
    }

    async fn await_for_mint_status_and_save_it(
        signature: Signature,
        asset_pubkey: PublicKey,
        solana_service: Arc<dyn L1Service + Sync + Send>,
        l2_storage: Arc<dyn L2Storage + Sync + Send>,
    ) -> anyhow::Result<()> {
        let asset_pubkey_as_str = asset_pubkey.to_string();

        for _ in 0..=Self::AMOUNT_OF_ATTEMPTS_TO_CALL_BLOCKCHAIN {
            match solana_service.is_asset_minted(&signature).await {
                Ok(true) => {
                    info!("Successfully minted asset '{asset_pubkey_as_str}' in transaction '{signature}'.");
                    return l2_storage
                        .finalize_mint(&asset_pubkey)
                        .await
                        .inspect(|_| info!("Mint for '{asset_pubkey_as_str}' successfully persisted."))
                        .inspect_err(|e| error!("Failed to finalize mint because: {e}!"));
                }
                Ok(false) => {
                    info!("Failed to mint asset '{asset_pubkey_as_str}' in transaction '{signature}'.");
                    return l2_storage
                        .mint_didnt_happen(&asset_pubkey)
                        .await
                        .inspect(|_| info!("Mint for '{asset_pubkey_as_str}' successfully rolled back."))
                        .inspect_err(|e| error!("Failed to rollback mint because: {e}!"));
                }
                Err(e) => {
                    debug!("Waiting for mint of {asset_pubkey_as_str}; {e}");
                    tokio::time::sleep(Self::AWAIT_TIME_TO_CALL_BLOCKCHAIN).await
                }
            }
        }

        info!(
            "Amount of attempts to call blockchain reached the limit - {}; Rolling back mint.",
            Self::AMOUNT_OF_ATTEMPTS_TO_CALL_BLOCKCHAIN
        );

        l2_storage
            .mint_didnt_happen(&asset_pubkey)
            .await
            .inspect(|_| info!("Mint for '{asset_pubkey_as_str}' successfully rolled back."))
            .inspect_err(|e| error!("Failed to rollback mint because: {e}!"))
    }

    async fn is_asset_already_sent_to_mint(&self, asset_pubkey: &PublicKey) -> Option<Signature> {
        self.l2_storage
            .find_l1_asset_signature(asset_pubkey)
            .await
            .and_then(Self::parse_signature)
    }

    fn parse_signature(signature: Vec<u8>) -> Option<Signature> {
        Signature::try_from(signature)
            .inspect_err(|_| error!("Failed to parse signature!"))
            .ok()
    }
}
