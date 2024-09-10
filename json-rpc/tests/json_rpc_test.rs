mod utils;

use crate::utils::{
    create_asset, create_assets_with_same_owner_requests, create_different_assets_requests, CreateAssetRequest,
};
use interfaces::asset_service::L2AssetInfo;
use json_rpc::config::app_config::AppConfig;
use json_rpc::config::app_context::{AppCtx, ArcedAppCtx};
use json_rpc::endpoints::errors::DasApiError;
use json_rpc::endpoints::get_nft::{get_asset, get_asset_batch, get_asset_by_owner};
use json_rpc::endpoints::rpc_asset_models::Asset;
use json_rpc::endpoints::types::{
    AssetList, AssetSortBy, AssetSortDirection, AssetSorting, GetAsset, GetAssetBatch, GetAssetsByOwner, JsonRpcError,
};
use json_rpc::endpoints::DEFAULT_LIMIT_FOR_PAGE;
use serde_json::Value;
use setup::TestEnvironmentCfg;
use solana_sdk::pubkey::Pubkey;
use util::publickey::PublicKeyExt;

async fn fill_database_with_test_data(
    app_ctx: ArcedAppCtx,
    asset_creation_strategy: fn() -> Vec<CreateAssetRequest>,
) -> Vec<L2AssetInfo> {
    let requests_for_asset_creation = asset_creation_strategy();
    let mut filled_data_from_db = Vec::with_capacity(requests_for_asset_creation.len());

    for asset_req in requests_for_asset_creation {
        let created_assets = create_asset(asset_req, app_ctx.clone())
            .await
            .unwrap_or_else(|e| panic!("Cannot create asset: {e}!"));
        let created_asset =
            serde_json::from_value(created_assets).unwrap_or_else(|e| panic!("Cannot serialize L2AssetInfo: {e}!"));

        filled_data_from_db.push(created_asset)
    }

    filled_data_from_db
}

fn form_asset_json_uri(pubkey: &str) -> String {
    format!("127.0.0.1:8080/asset/{pubkey}/metadata.json")
}

fn extract_asset_name_from_das_asset(asset: Asset) -> String {
    serde_json::from_value(
        asset
            .content
            .expect("Content should be present")
            .metadata
            .get_item("name")
            .expect("Content should be present")
            .clone(),
    )
    .expect("Cannot call 'from_value'!")
}

#[tokio::test]
async fn get_single_asset_positive() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let assets_data_in_db = fill_database_with_test_data(app_ctx.clone(), create_different_assets_requests).await;

    for asset in assets_data_in_db {
        let expected_asset_pubkey = asset.asset.pubkey.to_string();
        let expected_asset_json_uri = form_asset_json_uri(&expected_asset_pubkey);

        let get_asset_req = GetAsset { id: expected_asset_pubkey.clone() };

        let res = {
            let res = get_asset(get_asset_req, app_ctx.clone())
                .await
                .expect("Failed to get asset.");
            serde_json::from_value::<Asset>(res).expect("Failed serialize DAO asset..")
        };

        let actual_asset_pubkey = res.id;
        let actual_asset_json_uri = res.content.expect("No content field.").json_uri;

        assert_eq!(expected_asset_pubkey, actual_asset_pubkey);
        assert_eq!(expected_asset_json_uri, actual_asset_json_uri)
    }
}

#[tokio::test]
async fn get_single_asset_negative() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let non_existing_pubkey = Pubkey::new_unique().to_string();

    let get_asset_req = GetAsset { id: non_existing_pubkey };

    let res = get_asset(get_asset_req, app_ctx.clone()).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), DasApiError::NoDataFoundError.into());
}

#[tokio::test]
async fn get_asset_batch_negative() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let non_existing_pubkeys = vec![
        Pubkey::new_unique().to_string(),
        Pubkey::new_unique().to_string(),
        Pubkey::new_unique().to_string(),
    ];

    let get_assets_batch_req = GetAssetBatch { ids: non_existing_pubkeys };

    let expected_res = vec![Value::Null, Value::Null, Value::Null];

    let res = get_asset_batch(get_assets_batch_req, app_ctx.clone())
        .await
        .expect("Failed to get assets.");

    let actual_res = serde_json::from_value::<Vec<Value>>(res).expect("Failed to serialize values");

    assert_eq!(expected_res, actual_res);
}

#[tokio::test]
async fn get_asset_batch_positive() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let assets_pubkeys_data_in_db = fill_database_with_test_data(app_ctx.clone(), create_different_assets_requests)
        .await
        .into_iter()
        .map(|asset| asset.asset.pubkey.to_string())
        .collect::<Vec<String>>();

    let get_asset_req = GetAssetBatch { ids: assets_pubkeys_data_in_db.clone() };

    let res = {
        let res = get_asset_batch(get_asset_req, app_ctx.clone())
            .await
            .expect("Failed to get assets.");
        serde_json::from_value::<Vec<Asset>>(res).expect("Failed serialize DAO asset..")
    };

    for (expected_pubkey, actual_dao_asset) in assets_pubkeys_data_in_db.into_iter().zip(res) {
        let actual_pubkey = actual_dao_asset.id;
        assert_eq!(expected_pubkey, actual_pubkey);

        let expected_asset_json_uri = form_asset_json_uri(&expected_pubkey);
        let actual_asset_json_uri = form_asset_json_uri(&actual_pubkey);
        assert_eq!(expected_asset_json_uri, actual_asset_json_uri);
    }
}

#[tokio::test]
async fn get_asset_batch_positive_with_non_existing_key() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let mut pubkeys_for_request = fill_database_with_test_data(app_ctx.clone(), create_different_assets_requests)
        .await
        .into_iter()
        .map(|asset| asset.asset.pubkey.to_string())
        .collect::<Vec<String>>();

    let non_existing_pubkey = Pubkey::new_unique().to_string();
    pubkeys_for_request.push(non_existing_pubkey);

    let get_asset_req = GetAssetBatch { ids: pubkeys_for_request.clone() };

    let res = get_asset_batch(get_asset_req, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let mut res_as_values = serde_json::from_value::<Vec<Value>>(res).expect("Failed serialize DAO asset..");

    // Last value should be null.
    assert_eq!(res_as_values.pop(), Some(Value::Null));

    let res_as_assets = res_as_values
        .into_iter()
        .map(serde_json::from_value)
        .collect::<Result<Vec<Asset>, _>>()
        .expect("Cannot parse DAO assets.");

    // Remove last values.
    pubkeys_for_request.pop();
    for (expected_pubkey, actual_dao_asset) in pubkeys_for_request.into_iter().zip(res_as_assets) {
        let actual_pubkey = actual_dao_asset.id;
        assert_eq!(expected_pubkey, actual_pubkey);

        let expected_asset_json_uri = form_asset_json_uri(&expected_pubkey);
        let actual_asset_json_uri = form_asset_json_uri(&actual_pubkey);
        assert_eq!(expected_asset_json_uri, actual_asset_json_uri);
    }
}

#[tokio::test]
async fn get_assets_by_owner_sorting_by_created_date_desc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests).await;

    // Since we created assets in order from 1 to N and sort them by DESC then the result should be from N to 1.
    // Using name field for simplicity.
    let expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .rev()
        .collect::<Vec<String>>();

    let owner_pubkey = data_from_db.first().expect("Should be present.").asset.owner;

    let request_params = GetAssetsByOwner {
        owner_address: owner_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let actual_res = serde_json::from_value::<AssetList>(actual_res).expect("Failed serialize DAO assets..");

    assert!(actual_res.limit.eq(&DEFAULT_LIMIT_FOR_PAGE));

    let actual_res = actual_res
        .items
        .into_iter()
        .map(extract_asset_name_from_das_asset)
        .collect::<Vec<String>>();

    assert_eq!(actual_res, expected_order_of_assets_by_name);
}

#[tokio::test]
async fn get_assets_by_owner_sorting_by_created_date_asc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests).await;

    // Since we created assets in order from 1 to N and sort them by ASC then the result should be from 1 to N.
    // Using name field for simplicity.
    let expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .collect::<Vec<String>>();

    let owner_pubkey = data_from_db.first().expect("Should be present.").asset.owner;

    let request_params = GetAssetsByOwner {
        owner_address: owner_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Asc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let actual_res = serde_json::from_value::<AssetList>(actual_res).expect("Failed serialize DAO assets..");

    assert!(actual_res.limit.eq(&DEFAULT_LIMIT_FOR_PAGE));

    let actual_res = actual_res
        .items
        .into_iter()
        .map(extract_asset_name_from_das_asset)
        .collect::<Vec<String>>();

    assert_eq!(actual_res, expected_order_of_assets_by_name);
}

#[tokio::test]
async fn get_assets_by_owner_with_limit_and_sorting_by_creation_data_desc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests).await;

    let limit = 3;

    // Since we created assets in order from 1 to N and sort them by DESC then the result should be from N to 1.
    // Using name field for simplicity.
    // Taking only first 3 items after reversing because of limit test.
    let expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .rev()
        .take(limit)
        .collect::<Vec<String>>();

    let owner_pubkey = data_from_db.first().expect("Should be present.").asset.owner;

    let request_params = GetAssetsByOwner {
        owner_address: owner_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: Some(limit as u32),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let actual_res = serde_json::from_value::<AssetList>(actual_res).expect("Failed serialize DAO assets..");

    assert!(actual_res.limit.eq(&(limit as u32)));
    assert!(actual_res.total.eq(&(limit as u32)));
    assert!(actual_res.items.len().eq(&limit));

    let actual_res = actual_res
        .items
        .into_iter()
        .map(extract_asset_name_from_das_asset)
        .collect::<Vec<String>>();

    assert_eq!(actual_res, expected_order_of_assets_by_name);
}

#[tokio::test]
async fn get_assets_by_non_existent_owner() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let non_existent_owner_pubkey = Pubkey::new_unique();

    let request_params = GetAssetsByOwner {
        owner_address: non_existent_owner_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let actual_res = serde_json::from_value::<AssetList>(actual_res).expect("Failed serialize DAO assets..");

    assert!(actual_res.total.eq(&0));
    assert!(actual_res.limit.eq(&DEFAULT_LIMIT_FOR_PAGE));
    assert!(actual_res.items.is_empty());
}

#[tokio::test]
async fn get_assets_by_owner_with_invalid_limit() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let request_params = GetAssetsByOwner {
        owner_address: Pubkey::new_unique().to_string(),
        sort_by: None,
        limit: Some(DEFAULT_LIMIT_FOR_PAGE + 1),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let expected_err = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect_err("Should fail.");

    assert_eq!(expected_err, DasApiError::LimitTooBig(DEFAULT_LIMIT_FOR_PAGE).into());
}

#[tokio::test]
async fn get_assets_owner_by_cursor() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests).await;

    let asset_owner = data_from_db.first().unwrap().asset.owner.to_string();

    let mut expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .collect::<Vec<String>>();

    let first_asset_owner = expected_order_of_assets_by_name.pop().unwrap();
    let request_params = GetAssetsByOwner {
        owner_address: asset_owner.clone(),
        sort_by: None,
        limit: Some(1),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let first_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let first_res = serde_json::from_value::<AssetList>(first_res).expect("Failed serialize DAO assets..");

    assert_eq!(get_first_asset_name(&first_res), first_asset_owner);

    let cursor_to_call_next = first_res.cursor.clone().unwrap();
    let request_params = GetAssetsByOwner {
        owner_address: asset_owner,
        sort_by: None,
        limit: Some(1),
        page: None,
        before: None,
        after: None,
        cursor: Some(cursor_to_call_next),
    };

    let second_res = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let first_res = serde_json::from_value::<AssetList>(second_res).expect("Failed serialize DAO assets..");

    assert_eq!(get_first_asset_name(&first_res), expected_order_of_assets_by_name.pop().unwrap());
}

fn get_first_asset_name(asset_list: &AssetList) -> String {
    serde_json::from_value(
        asset_list
            .items
            .first()
            .cloned()
            .unwrap()
            .content
            .unwrap()
            .metadata
            .get_item("name")
            .unwrap()
            .clone(),
    )
    .unwrap()
}
