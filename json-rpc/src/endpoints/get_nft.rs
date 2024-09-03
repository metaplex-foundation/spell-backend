use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::errors::DasApiError;
use crate::endpoints::rpc_asset_models::Asset;
use crate::endpoints::types::{GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcResponse};
use entities::l2::{AssetExtended, PublicKey};
use interfaces::asset_service::L2AssetInfo;
use serde_json::json;
use std::collections::HashMap;
use util::publickey::PublicKeyExt;

pub async fn get_asset(req_params: GetAsset, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let id =
        PublicKey::from_bs58(&req_params.id).ok_or(DasApiError::PubkeyValidationError(req_params.id.to_owned()))?;

    let (asset, metadata) = ctx
        .asset_service
        .fetch_asset(id)
        .await
        .map_err(|_| DasApiError::DatabaseError)?
        .ok_or(DasApiError::NoDataFoundError)
        .map(|l2_info| (l2_info.asset, l2_info.metadata))?;

    let asset_extended_and_metadata = (
        AssetExtended::new(asset, ctx.metadata_uri_base.get_metadata_uri_for_key(&req_params.id)),
        serde_json::to_value(metadata).map_err(|_| DasApiError::JsonMetadataParsing)?,
    );

    Ok(Asset::from(asset_extended_and_metadata).to_json())
}

pub async fn get_asset_batch(req_params: GetAssetBatch, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let ids = req_params
        .ids
        .iter()
        .map(|id| PublicKey::from_bs58(id).ok_or(DasApiError::PubkeyValidationError(id.to_string())))
        .collect::<Result<Vec<PublicKey>, _>>()?;

    let l2_assets = ctx
        .asset_service
        .fetch_assets(&ids)
        .await
        .map_err(|_| DasApiError::DatabaseError)?
        .into_iter()
        .map(|l2_asset| (l2_asset.asset.pubkey.clone().to_string(), l2_asset))
        .collect::<HashMap<String, L2AssetInfo>>();

    let mut res = Vec::with_capacity(req_params.ids.len());

    for id in req_params.ids {
        match l2_assets
            .get(&id)
            .cloned()
            .map(|l2_asset| (l2_asset.asset, l2_asset.metadata))
        {
            Some((asset, metadata)) => {
                let asset_extended_and_metadata = (
                    AssetExtended::new(asset, ctx.metadata_uri_base.get_metadata_uri_for_key(&id)),
                    serde_json::to_value(metadata).map_err(|_| DasApiError::JsonMetadataParsing)?,
                );

                res.push(Asset::from(asset_extended_and_metadata).to_json())
            }
            None => res.push(Asset::empty_json()),
        }
    }

    Ok(json!(res))
}

pub async fn get_asset_by_owner(_req_params: GetAssetsByOwner, _ctx: ArcedAppCtx) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}

pub async fn get_asset_by_creator(_req_params: GetAssetsByCreator, _ctx: ArcedAppCtx) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}
