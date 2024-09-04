use entities::l2::PublicKey;
use json_rpc::config::app_context::ArcedAppCtx;
use json_rpc::endpoints::types::{JsonRpcError, JsonRpcResponse};
use jsonrpc_core::ErrorCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use util::publickey::PublicKeyExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub metadata_json: String,
    pub owner: String,
    pub creator: String,
    pub authority: String,
    pub collection: Option<String>,
}

pub async fn create_asset(req_params: CreateAssetRequest, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let res = ctx
        .asset_service
        .create_asset(
            &req_params.metadata_json,
            PublicKey::from_bs58(&req_params.owner).ok_or(JsonRpcError::invalid_params("Invalid owner"))?,
            PublicKey::from_bs58(&req_params.creator).ok_or(JsonRpcError::invalid_params("Invalid creator"))?,
            PublicKey::from_bs58(&req_params.authority).ok_or(JsonRpcError::invalid_params("Invalid authority"))?,
            &req_params.name,
            req_params
                .collection
                .and_then(|collection| PublicKey::from_bs58(&collection)),
        )
        .await
        .map_err(|e| JsonRpcError { code: ErrorCode::InternalError, message: e.to_string(), data: None })?;

    Ok(json!(res))
}
