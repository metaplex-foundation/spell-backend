use std::sync::Arc;

use chrono::Local;
use entities::l2::{L2Asset, PublicKey};
use interfaces::{
    asset_service::{AssetService, L2AssetInfo},
    asset_storage::{AssetMetadataStorage, BlobStorage},
    l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage},
};
use solana_sdk::signer::Signer;
use util::{hd_wallet::HdWalletProducer, nft_json::validate_metadata_contains_uris};

pub struct AssetServiceImpl {
    pub wallet_producer: HdWalletProducer,
    pub derivation_sequence: Arc<dyn Bip44DerivationSequence + Sync + Send>,
    pub l2_storage: Arc<dyn L2Storage + Sync + Send>,
    pub asset_metadata_storage: Arc<dyn AssetMetadataStorage + Sync + Send>,
    pub blob_storage: Arc<dyn BlobStorage + Sync + Send>,
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
        let Some(keypair) = self.wallet_producer.make_hd_wallet(account, address) else {
            anyhow::bail!("Can't derive keypair");
        };
        let asset_pubkey = keypair.pubkey().to_bytes();

        self.asset_metadata_storage
            .put_json(&asset_pubkey, metadata_json)
            .await?;

        let asset = L2Asset {
            pubkey: asset_pubkey,
            name: name.to_string(),
            owner: owner,
            creator: creator,
            collection: collection,
            authority: authority,
            create_timestamp: Local::now().naive_local(),
            pib44_account_num: account,
            pib44_address_num: address,
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
                println!("UPDATING METADA");
                self.asset_metadata_storage.put_json(&asset_pubkey, v).await?;
                Some(v.to_string())
            } else {
                println!("NOT UPDATING METADA");
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
}
