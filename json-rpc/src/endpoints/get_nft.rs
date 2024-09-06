use crate::config::app_context::ArcedAppCtx;
use crate::endpoints::errors::DasApiError;
use crate::endpoints::rpc_asset_models::Asset;
use crate::endpoints::types::{
    AssetExtended, AssetList, GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcResponse,
};
use crate::endpoints::DEFAULT_LIMIT_FOR_PAGE;
use entities::l2::{L2Asset, PublicKey};
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

    Ok(Asset::from(asset_extended_and_metadata).into_json())
}

pub async fn get_asset_batch(req_params: GetAssetBatch, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let ids = req_params
        .ids
        .iter()
        .map(|id| PublicKey::from_bs58(id).ok_or(DasApiError::PubkeyValidationError(id.to_string())))
        .collect::<Result<Vec<PublicKey>, _>>()?;

    let id_to_l2_asset = ctx
        .asset_service
        .fetch_assets(&ids)
        .await
        .map_err(|_| DasApiError::DatabaseError)?
        .into_iter()
        .map(|l2_asset| (l2_asset.asset.pubkey.to_string(), l2_asset))
        .collect::<HashMap<String, L2AssetInfo>>();

    let mut res = Vec::with_capacity(req_params.ids.len());

    for id in req_params.ids {
        match id_to_l2_asset
            .get(&id)
            .cloned()
            .map(|l2_asset| (l2_asset.asset, l2_asset.metadata))
        {
            Some((asset, metadata)) => {
                let asset_extended_and_metadata = (
                    AssetExtended::new(asset, ctx.metadata_uri_base.get_metadata_uri_for_key(&id)),
                    serde_json::to_value(metadata).map_err(|_| DasApiError::JsonMetadataParsing)?,
                );

                res.push(Asset::from(asset_extended_and_metadata).into_json())
            }
            None => res.push(Asset::empty_json()),
        }
    }

    Ok(json!(res))
}

pub async fn get_asset_by_owner(req_params: GetAssetsByOwner, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let owner_address = PublicKey::from_bs58(&req_params.owner_address)
        .ok_or(DasApiError::PubkeyValidationError(req_params.owner_address.to_owned()))?;
    let sorting = req_params.sort_by.map(Into::into).unwrap_or_default();
    let limit = req_params.limit.unwrap_or(DEFAULT_LIMIT_FOR_PAGE);

    let l2_assets = ctx
        .asset_service
        .fetch_assets_by_owner(owner_address, sorting, limit)
        .await
        .map_err(|_| DasApiError::DatabaseError)?
        .into_iter()
        .map(|asset| (asset.asset, asset.metadata))
        .collect::<Vec<(L2Asset, Option<String>)>>();

    let mut das_assets = Vec::with_capacity(l2_assets.len());

    for (asset, metadata) in l2_assets {
        let asset_pubkey = asset.pubkey.to_string();
        let asset_extended_and_metadata = (
            AssetExtended::new(asset, ctx.metadata_uri_base.get_metadata_uri_for_key(&asset_pubkey)),
            serde_json::to_value(metadata).map_err(|_| DasApiError::JsonMetadataParsing)?,
        );

        das_assets.push(Asset::from(asset_extended_and_metadata))
    }

    Ok(json!(AssetList { total: das_assets.len() as u32, limit, items: das_assets, ..Default::default() }))
}

pub async fn get_asset_by_creator(_req_params: GetAssetsByCreator, _ctx: ArcedAppCtx) -> JsonRpcResponse {
    Ok(json!("Some Assets"))
}
