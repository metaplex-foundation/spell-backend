use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::types::{
    GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcResponse,
};
use serde_json::json;

pub async fn get_asset(req_params: GetAsset, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let _connection = ctx.get_connection_pool();
    let _id = req_params.id;
    Ok(json!("Some Asset"))
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
