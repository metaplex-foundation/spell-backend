use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub type AssetKey = [u8; 32];

/// Represents L2 asset
#[derive(Debug,Clone,PartialEq,Eq,Serialize,Deserialize)]
pub struct L2Asset {
    pub pubkey: Option<AssetKey>,
    pub name: String,
    pub owner: AssetKey,
    pub creator: AssetKey,
    pub collection:  Option<AssetKey>,
    pub authority: AssetKey,
    pub metadata_url: String,
    pub create_timestamp: chrono::NaiveDateTime, // Need timezone?
}


/// Storage interfaces for L2 assets managing
#[async_trait]
pub trait L2Storage {
    async fn save(&self, asset: &L2Asset) ->  anyhow::Result<()>;
    async fn find(&self, pubkey: &AssetKey) -> anyhow::Result<Option<L2Asset>>;
}