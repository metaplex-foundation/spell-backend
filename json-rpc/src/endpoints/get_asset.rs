use crate::endpoints::errors::DasApiError;
use crate::endpoints::types::{
    AssetList, GetAsset, GetAssetBatch, GetAssetsByCreator, GetAssetsByOwner, JsonRpcResponse,
};
use crate::endpoints::{DEFAULT_LIMIT_FOR_PAGE, DEFAULT_MAX_PAGE_LIMIT};
use crate::setup::app_context::ArcedAppCtx;
use entities::dto::{Asset, AssetExtended};
use entities::l2::PublicKey;
use interfaces::asset_service::L2AssetInfo;
use serde_json::json;
use std::collections::HashMap;
use util::base64_encode_decode::encode_timestamp_and_asset_pubkey;
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
    let sorting = req_params.sort_by.map(Into::into).unwrap_or_default();
    let limit = verify_limit(req_params.limit)?;
    let before = req_params.before;
    let after = req_params.after;
    let page = verify_page(req_params.page)?;
    let cursor = req_params.cursor;

    let is_cursor_enabled = before.is_none() && after.is_none() && page.is_none();
    let after = is_cursor_enabled.then_some(cursor).unwrap_or(after);

    let l2_assets = ctx
        .asset_service
        .fetch_assets_by_owner(&req_params.owner_address, &sorting, limit, before.as_deref(), after.as_deref())
        .await
        .map_err(|_| DasApiError::DatabaseError)?;

    Ok(json!(prepare_response(l2_assets, is_cursor_enabled, page, ctx.clone(), limit)?))
}

pub async fn get_asset_by_creator(req_params: GetAssetsByCreator, ctx: ArcedAppCtx) -> JsonRpcResponse {
    let sorting = req_params.sort_by.map(Into::into).unwrap_or_default();
    let limit = verify_limit(req_params.limit)?;
    let before = req_params.before;
    let after = req_params.after;
    let page = verify_page(req_params.page)?;
    let cursor = req_params.cursor;

    let is_cursor_enabled = before.is_none() && after.is_none() && page.is_none();
    let after = is_cursor_enabled.then_some(cursor).unwrap_or(after);

    let l2_assets = ctx
        .asset_service
        .fetch_assets_by_creator(&req_params.creator_address, &sorting, limit, before.as_deref(), after.as_deref())
        .await
        .map_err(|_| DasApiError::DatabaseError)?;

    Ok(json!(prepare_response(l2_assets, is_cursor_enabled, page, ctx.clone(), limit)?))
}

fn prepare_response(
    l2_assets: Vec<L2AssetInfo>,
    is_cursor_enabled: bool,
    page: Option<u32>,
    ctx: ArcedAppCtx,
    limit: u32,
) -> Result<AssetList, DasApiError> {
    let (before, after, cursor, page) = if is_cursor_enabled {
        (
            None,
            None,
            l2_assets.last().map(|asset_info| {
                encode_timestamp_and_asset_pubkey(asset_info.asset.create_timestamp, asset_info.asset.pubkey)
            }),
            None,
        )
    } else if let Some(page) = page {
        (None, None, None, Some(page))
    } else {
        (
            l2_assets.first().map(|asset_info| {
                encode_timestamp_and_asset_pubkey(asset_info.asset.create_timestamp, asset_info.asset.pubkey)
            }),
            l2_assets.last().map(|asset_info| {
                encode_timestamp_and_asset_pubkey(asset_info.asset.create_timestamp, asset_info.asset.pubkey)
            }),
            None,
            None,
        )
    };

    let mut das_assets = Vec::with_capacity(l2_assets.len());
    for (asset, metadata) in l2_assets.into_iter().map(|asset| (asset.asset, asset.metadata)) {
        let asset_pubkey = asset.pubkey.to_string();
        let asset_extended_and_metadata = (
            AssetExtended::new(asset, ctx.metadata_uri_base.get_metadata_uri_for_key(&asset_pubkey)),
            serde_json::to_value(metadata).map_err(|_| DasApiError::JsonMetadataParsing)?,
        );

        das_assets.push(Asset::from(asset_extended_and_metadata))
    }

    Ok(AssetList {
        total: das_assets.len() as u32,
        limit,
        page,
        before,
        after,
        items: das_assets,
        errors: vec![],
        cursor,
    })
}

fn verify_limit(limit: Option<u32>) -> Result<u32, DasApiError> {
    match limit {
        Some(limit) => (limit < DEFAULT_LIMIT_FOR_PAGE)
            .then_some(limit)
            .ok_or(DasApiError::LimitTooBig(DEFAULT_LIMIT_FOR_PAGE)),
        None => Ok(DEFAULT_LIMIT_FOR_PAGE),
    }
}

fn verify_page(limit: Option<u32>) -> Result<Option<u32>, DasApiError> {
    match limit {
        Some(limit) => {
            if limit < DEFAULT_MAX_PAGE_LIMIT {
                Ok(Some(limit))
            } else {
                Err(DasApiError::PageTooBig(DEFAULT_MAX_PAGE_LIMIT))
            }
        }
        None => Ok(None),
    }
}
