use async_trait::async_trait;
use entities::l2::PublicKey;

#[async_trait]
pub trait AssetMetadataStorage {
    async fn put_json(&self, pubkey: &PublicKey, json_metadata: &str) -> anyhow::Result<()>;
    async fn get_json(&self, pubkey: &PublicKey) -> anyhow::Result<String>;
}

#[async_trait]
pub trait BlobStorage {
    async fn put_binary(&self, pubkey: &PublicKey, bytes: Vec<u8>, mime: &str) -> anyhow::Result<()>;
    async fn get_binary(&self, pubkey: &PublicKey) -> anyhow::Result<(Vec<u8>, String)>;
}