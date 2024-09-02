use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::errors::DasApiError;
use crate::endpoints::types::{
    GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcError, JsonRpcResponse,
};
use serde_json::json;
use crate::endpoints::rpc_asset_models::{Asset, Ownership};

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

    // let res = Asset {
    //     interface: "".to_string(),
    //     id: "".to_string(),
    //     content: None,
    //     authorities: None,
    //     compression: None,
    //     grouping: None,
    //     royalty: None,
    //     creators: None,
    //     ownership: Ownership {
    //         frozen: false,
    //         delegated: false,
    //         delegate: None,
    //         ownership_model: OwnershipModel::Single,
    //         owner: "".to_string(),
    //     },
    //     uses: None,
    //     supply: None,
    //     mutable: false,
    //     burnt: false,
    //     lamports: None,
    //     executable: None,
    //     metadata_owner: None,
    //     rent_epoch: None,
    //     plugins: None,
    //     unknown_plugins: None,
    //     mpl_core_info: None,
    //     external_plugins: None,
    //     unknown_external_plugins: None,
    //     spl20: None,
    // };

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
