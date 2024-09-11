mod utils;

use crate::utils::{
    create_asset, create_assets_with_same_owner_requests, create_assets_with_same_owner_requests_with_random_values,
    create_different_assets_requests, CreateAssetRequest,
};
use interfaces::asset_service::L2AssetInfo;
use json_rpc::config::app_config::AppConfig;
use json_rpc::config::app_context::{AppCtx, ArcedAppCtx};
use json_rpc::endpoints::errors::DasApiError;
use json_rpc::endpoints::get_nft::{get_asset, get_asset_batch, get_asset_by_owner};
use json_rpc::endpoints::rpc_asset_models::Asset;
use json_rpc::endpoints::types::{
    AssetList, AssetSortBy, AssetSortDirection, AssetSorting, GetAsset, GetAssetBatch, GetAssetsByOwner,
};
use json_rpc::endpoints::{DEFAULT_LIMIT_FOR_PAGE, DEFAULT_MAX_PAGE_LIMIT};
use serde_json::{json, Value};
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
async fn get_assets_by_owner_with_invalid_page() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let request_params = GetAssetsByOwner {
        owner_address: Pubkey::new_unique().to_string(),
        sort_by: None,
        limit: None,
        page: Some(DEFAULT_MAX_PAGE_LIMIT + 1),
        before: None,
        after: None,
        cursor: None,
    };

    let expected_err = get_asset_by_owner(request_params, app_ctx.clone())
        .await
        .expect_err("Should fail.");

    assert_eq!(expected_err, DasApiError::PageTooBig(DEFAULT_MAX_PAGE_LIMIT).into());
}

#[tokio::test]
async fn get_assets_by_owner_using_cursor() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    // Put some assets for tests
    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests).await;

    // Get asset owner
    let asset_owner = data_from_db.first().unwrap().asset.owner.to_string();

    let mut assets_from_db_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .collect::<Vec<String>>();

    let first_asset_name = assets_from_db_by_name.pop().unwrap();
    let payload = GetAssetsByOwner {
        owner_address: asset_owner.clone(),
        sort_by: None,
        limit: Some(1),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };
    let first_res = get_asset_list_by_owner(payload, app_ctx.clone()).await;
    // Expecting to get last element from db because we're iterating from new to old
    assert_eq!(get_first_asset_name(&first_res), first_asset_name);

    let cursor_to_call_next = first_res.cursor.clone().unwrap();
    let mut payload = GetAssetsByOwner {
        owner_address: asset_owner.clone(),
        sort_by: None,
        limit: Some(1),
        page: None,
        before: None,
        after: None,
        cursor: Some(cursor_to_call_next),
    };

    // Iterate be cursor for the remaining elements.
    // Using reversed `assets_from_db_by_name` because we cannot use `.pop()` inside loop
    for asset_from_db_by_name in assets_from_db_by_name.into_iter().rev() {
        let first_res = get_asset_list_by_owner(payload, app_ctx.clone()).await;

        assert_eq!(get_first_asset_name(&first_res), asset_from_db_by_name);

        payload = GetAssetsByOwner {
            owner_address: asset_owner.clone(),
            sort_by: None,
            limit: Some(1),
            page: None,
            before: None,
            after: None,
            cursor: first_res.cursor,
        };
    }

    // Expected that next element by cursor will be empty
    let empty_list = get_asset_list_by_owner(payload, app_ctx.clone()).await.items;
    assert!(empty_list.is_empty());
}

#[tokio::test]
async fn get_assets_by_owner_with_pagination() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    // Put some assets for tests
    let data_from_db =
        fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_owner_requests_with_random_values).await;

    assert!(data_from_db.len().eq(&100));

    let asset_owner_address = data_from_db.first().unwrap().asset.owner.to_string();

    check_pagination(app_ctx.clone(), asset_owner_address).await;
}

async fn check_pagination(app_ctx: ArcedAppCtx, asset_owner: String) {
    let payload = GetAssetsByOwner {
        limit: Some(10),
        page: None,
        before: None,
        after: None,
        owner_address: asset_owner.clone(),
        sort_by: None,
        cursor: None,
    };
    let first_10 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    println!("second_10 start here ");
    println!("{}", json!(&first_10));
    println!("second_10 end here ");

    let payload = GetAssetsByOwner {
        limit: Some(10),
        page: None,
        before: None,
        owner_address: asset_owner.clone(),
        cursor: first_10.cursor.clone(),
        sort_by: None,
        after: None,
    };
    let second_10 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let payload = GetAssetsByOwner {
        limit: Some(20),
        page: None,
        before: None,
        after: None,
        owner_address: asset_owner.clone(),
        sort_by: None,
        cursor: None,
    };
    let first_20 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let mut first_two_resp = first_10.items;
    first_two_resp.extend(second_10.items.clone());

    assert_eq!(first_20.items, first_two_resp);

    let payload = GetAssetsByOwner {
        limit: Some(9),
        owner_address: asset_owner.clone(),
        before: first_20.cursor.clone(),
        after: None,
        sort_by: None,
        page: None,
        cursor: None,
    };
    let first_10_reverse = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let reversed = first_10_reverse.items;
    let mut second_10_resp = second_10.items.clone();
    // pop because we select 9 assets
    // select 9 because request with before do not return asset which is in before parameter
    second_10_resp.remove(0);
    assert_eq!(reversed, second_10_resp);

    let payload = GetAssetsByOwner {
        owner_address: asset_owner.clone(),
        sort_by: None,
        limit: None,
        after: first_10.cursor.clone(),
        before: first_20.cursor,
        cursor: None,
        page: None,
    };
    let first_10_before_after = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    assert_eq!(first_10_before_after.items, second_10.items);

    let payload = GetAssetsByOwner {
        limit: Some(10),
        page: None,
        owner_address: asset_owner.clone(),
        after: first_10.cursor,
        sort_by: None,
        before: None,
        cursor: None,
    };
    let after_first_10 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let payload = GetAssetsByOwner {
        limit: Some(10),
        page: None,
        owner_address: asset_owner.clone(),
        after: after_first_10.after,
        sort_by: None,
        before: None,
        cursor: None,
    };
    let after_first_20 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let payload = GetAssetsByOwner {
        limit: Some(30),
        page: None,
        before: None,
        after: None,
        owner_address: asset_owner.clone(),
        sort_by: None,
        cursor: None,
    };
    let first_30 = get_asset_list_by_owner(payload, app_ctx.clone()).await;

    let mut combined_10_30 = after_first_10.items;
    combined_10_30.extend(after_first_20.items.clone());

    assert_eq!(combined_10_30, first_30.items[10..]);
}

async fn get_asset_list_by_owner(req_params: GetAssetsByOwner, ctx: ArcedAppCtx) -> AssetList {
    serde_json::from_value::<AssetList>(
        get_asset_by_owner(req_params, ctx.clone())
            .await
            .expect("Failed to get assets."),
    )
    .expect("Failed serialize DAO assets..")
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
