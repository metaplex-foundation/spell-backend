use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::errors::DasApiError;
use crate::endpoints::types::{
    GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcError, JsonRpcResponse,
};
use serde_json::json;

pub async fn get_asset(req_params: GetAsset, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let id = &req_params.id;
    let id = id
        .as_bytes()
        .try_into()
        .map_err(|_| DasApiError::PubkeyValidationError(id.to_owned()))
        .map_err(Into::<JsonRpcError>::into)?;

    let res = ctx
        .asset_service
        .l2_storage
        .find(&id)
        .await
        .map_err(|_| DasApiError::DatabaseError)
        .map_err(Into::<JsonRpcError>::into)?
        .ok_or(DasApiError::NoDataFoundError)
        .map_err(Into::<JsonRpcError>::into)?;

    Ok(json!(res))
}

pub async fn get_asset_batch(_req_params: GetAssetBatch, _ctx: ArcedAppCtx) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}

pub async fn get_asset_by_owner(
    _req_params: GetAssetsByOwner,
    _ctx: ArcedAppCtx,
) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}

pub async fn get_asset_by_creator(
    _req_params: GetAssetsByCreator,
    _ctx: ArcedAppCtx,
) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}
