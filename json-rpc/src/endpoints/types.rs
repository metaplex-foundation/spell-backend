use entities::dto::Asset;
use entities::l2::{
    AssetSortBy as L2AssetSortBy, AssetSortDirection as L2AssetSortDirection, AssetSorting as L2AssetSorting,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AssetSorting {
    pub sort_by: AssetSortBy,
    pub sort_direction: Option<AssetSortDirection>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetSortDirection {
    Asc,
    #[default]
    Desc,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetSortBy {
    #[default]
    Created,
    Updated,
    None,
}

impl Into<L2AssetSorting> for AssetSorting {
    fn into(self) -> L2AssetSorting {
        L2AssetSorting {
            sort_by: match self.sort_by {
                AssetSortBy::Created => L2AssetSortBy::Created,
                AssetSortBy::Updated => L2AssetSortBy::Updated,
                AssetSortBy::None => L2AssetSortBy::Created,
            },
            sort_direction: self
                .sort_direction
                .map(|asset_sort_direction| match asset_sort_direction {
                    AssetSortDirection::Asc => L2AssetSortDirection::Asc,
                    AssetSortDirection::Desc => L2AssetSortDirection::Desc,
                })
                .unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAssetsByOwner {
    pub owner_address: String,
    pub sort_by: Option<AssetSorting>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub cursor: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase", default)]
pub struct AssetList {
    pub total: u32,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    pub items: Vec<Asset>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GetAssetsByCreator {
    pub creator_address: String,
    pub only_verified: Option<bool>,
    pub sort_by: Option<AssetSorting>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub cursor: Option<String>,
}
