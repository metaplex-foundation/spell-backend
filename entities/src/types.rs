use crate::l2::L2Asset;
use jsonrpc_core::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

pub type JsonRpcError = Error;
pub type JsonRpcResponse = Result<JsonValue, JsonRpcError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAsset {
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAssetBatch {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAssetsByOwner {
    pub owner_address: String,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAssetsByCreator {
    pub creator_address: String,
    pub only_verified: Option<bool>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub cursor: Option<String>,
}

// TODO: merge with other asset types
pub struct AssetExtended {
    pub asset: L2Asset,
    pub metadata_uri: String,
    pub royalty_basis_points: u16,
}

impl AssetExtended {
    // TODO: use the basis points from db
    pub fn new(asset: L2Asset, metadata_uri: String) -> Self {
        Self { asset, metadata_uri, royalty_basis_points: 0 }
    }
}
