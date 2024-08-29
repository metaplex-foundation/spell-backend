use std::sync::Arc;

use chrono::Local;
use entities::l2::{L2Asset, PublicKey};
use interfaces::{asset_service::AssetService, asset_storage::{AssetMetadataStorage, BlobStorage}, l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage}};
use solana_sdk::signer::Signer;
use util::{hd_wallet::HdWalletProducer, nft_json::validate_metadata_contains_uris};


pub struct AssetServiceImpl {
    master_pubkey: PublicKey,
    wallet_producer: HdWalletProducer,
    derivation_sequence: Arc<dyn Bip44DerivationSequence + Sync + Send>,
    l2_storage: Arc<dyn L2Storage + Sync + Send>,
    asset_metadata_storage: Arc<dyn AssetMetadataStorage + Sync + Send>,
    blob_storage: Arc<dyn BlobStorage + Sync + Send>,
}

#[async_trait::async_trait]
impl AssetService for AssetServiceImpl {

    async fn create_asset(&self, asset_binary: Vec<u8>, mime: &str, metadata_json: &str, authority: PublicKey, name: &str, collection: Option<PublicKey>) -> anyhow::Result<PublicKey> {
        validate_metadata_contains_uris(metadata_json)?;

        let DerivationValues { account, change} = self.derivation_sequence.next_change().await?;
        let Some(keypair) = self.wallet_producer.make_hd_wallet(account, change) else {
            anyhow::bail!("Can't derive keypair");
        };
        let asset_pubkey = keypair.pubkey().to_bytes();

        self.blob_storage.put_binary(&asset_pubkey, asset_binary, mime).await?;
        self.asset_metadata_storage.put_json(&asset_pubkey, metadata_json).await?;

        let asset = L2Asset {
            pubkey: asset_pubkey,
            name: name.to_string(),
            owner: self.master_pubkey, // TODO: Spell of Metagrid?
            creator: self.master_pubkey, // TODO: Spell of Metagrid?
            collection: collection,
            authority: authority,
            create_timestamp: Local::now().naive_local(),
            pib44_account_num: account,
            pib44_change_num: change,
        };

        self.l2_storage.save(&asset).await?;

        Ok([1u8;32])
    }
}