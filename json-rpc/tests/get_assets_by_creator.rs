use crate::utils::{
    create_assets_with_same_creator_requests, extract_asset_name_from_das_asset, fill_database_with_test_data,
    get_first_asset_name,
};
use json_rpc::config::app_config::AppConfig;
use json_rpc::config::app_context::{AppCtx, ArcedAppCtx};
use json_rpc::endpoints::errors::DasApiError;
use json_rpc::endpoints::get_nft::get_asset_by_creator;
use json_rpc::endpoints::types::{AssetList, AssetSortBy, AssetSortDirection, AssetSorting, GetAssetsByCreator};
use json_rpc::endpoints::{DEFAULT_LIMIT_FOR_PAGE, DEFAULT_MAX_PAGE_LIMIT};
use setup::TestEnvironmentCfg;
use solana_sdk::pubkey::Pubkey;
use util::publickey::PublicKeyExt;

mod utils;

#[tokio::test]
async fn get_assets_by_creator_sorting_by_created_date_desc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_creator_requests).await;

    // Since we created assets in order from 1 to N and sort them by DESC then the result should be from N to 1.
    // Using name field for simplicity.
    let expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .rev()
        .collect::<Vec<String>>();

    let creator_pubkey = data_from_db.first().expect("Should be present.").asset.creator;

    let request_params = GetAssetsByCreator {
        creator_address: creator_pubkey.to_string(),
        only_verified: None,
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_creator(request_params, app_ctx.clone())
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
async fn get_assets_by_creator_sorting_by_created_date_asc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_creator_requests).await;

    // Since we created assets in order from 1 to N and sort them by ASC then the result should be from 1 to N.
    // Using name field for simplicity.
    let expected_order_of_assets_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .collect::<Vec<String>>();

    let creator_pubkey = data_from_db.first().expect("Should be present.").asset.creator;

    let request_params = GetAssetsByCreator {
        creator_address: creator_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Asc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
        only_verified: None,
    };

    let actual_res = get_asset_by_creator(request_params, app_ctx.clone())
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
async fn get_assets_by_creator_with_limit_and_sorting_by_creation_data_desc() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_creator_requests).await;

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

    let creator_pubkey = data_from_db.first().expect("Should be present.").asset.creator;

    let request_params = GetAssetsByCreator {
        creator_address: creator_pubkey.to_string(),
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: Some(limit as u32),
        page: None,
        before: None,
        after: None,
        cursor: None,
        only_verified: None,
    };

    let actual_res = get_asset_by_creator(request_params, app_ctx.clone())
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
async fn get_assets_by_non_existent_creator() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let non_existent_owner_pubkey = Pubkey::new_unique();

    let request_params = GetAssetsByCreator {
        creator_address: non_existent_owner_pubkey.to_string(),
        only_verified: None,
        sort_by: Some(AssetSorting { sort_by: AssetSortBy::Created, sort_direction: Some(AssetSortDirection::Desc) }),
        limit: None,
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let actual_res = get_asset_by_creator(request_params, app_ctx.clone())
        .await
        .expect("Failed to get assets.");
    let actual_res = serde_json::from_value::<AssetList>(actual_res).expect("Failed serialize DAO assets..");

    assert!(actual_res.total.eq(&0));
    assert!(actual_res.limit.eq(&DEFAULT_LIMIT_FOR_PAGE));
    assert!(actual_res.items.is_empty());
}

#[tokio::test]
async fn get_assets_by_creator_with_invalid_limit() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let request_params = GetAssetsByCreator {
        creator_address: Pubkey::new_unique().to_string(),
        only_verified: None,
        sort_by: None,
        limit: Some(DEFAULT_LIMIT_FOR_PAGE + 1),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };

    let expected_err = get_asset_by_creator(request_params, app_ctx.clone())
        .await
        .expect_err("Should fail.");

    assert_eq!(expected_err, DasApiError::LimitTooBig(DEFAULT_LIMIT_FOR_PAGE).into());
}

#[tokio::test]
async fn get_assets_by_creator_with_invalid_page() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let request_params = GetAssetsByCreator {
        creator_address: Pubkey::new_unique().to_string(),
        only_verified: None,
        sort_by: None,
        limit: None,
        page: Some(DEFAULT_MAX_PAGE_LIMIT + 1),
        before: None,
        after: None,
        cursor: None,
    };

    let expected_err = get_asset_by_creator(request_params, app_ctx.clone())
        .await
        .expect_err("Should fail.");

    assert_eq!(expected_err, DasApiError::PageTooBig(DEFAULT_MAX_PAGE_LIMIT).into());
}

#[tokio::test]
async fn get_assets_by_creator_using_cursor() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    // Put some assets for tests
    let data_from_db = fill_database_with_test_data(app_ctx.clone(), create_assets_with_same_creator_requests).await;

    // Get asset owner
    let asset_creator = data_from_db.first().unwrap().asset.creator.to_string();

    let mut assets_from_db_by_name = data_from_db
        .iter()
        .map(|asset| asset.asset.name.clone())
        .collect::<Vec<String>>();

    let first_asset_name = assets_from_db_by_name.pop().unwrap();
    let payload = GetAssetsByCreator {
        creator_address: asset_creator.clone(),
        only_verified: None,
        sort_by: None,
        limit: Some(1),
        page: None,
        before: None,
        after: None,
        cursor: None,
    };
    let first_res = get_asset_list_by_creator(payload, app_ctx.clone()).await;
    // Expecting to get last element from db because we're iterating from new to old
    assert_eq!(get_first_asset_name(&first_res), first_asset_name);

    let cursor_to_call_next = first_res.cursor.clone().unwrap();
    let mut payload = GetAssetsByCreator {
        creator_address: asset_creator.clone(),
        only_verified: None,
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
        let first_res = get_asset_list_by_creator(payload, app_ctx.clone()).await;

        assert_eq!(get_first_asset_name(&first_res), asset_from_db_by_name);

        payload = GetAssetsByCreator {
            creator_address: asset_creator.clone(),
            only_verified: None,
            sort_by: None,
            limit: Some(1),
            page: None,
            before: None,
            after: None,
            cursor: first_res.cursor,
        };
    }

    // Expected that next element by cursor will be empty
    let empty_list = get_asset_list_by_creator(payload, app_ctx.clone()).await.items;
    assert!(empty_list.is_empty());
}

async fn get_asset_list_by_creator(req_params: GetAssetsByCreator, ctx: ArcedAppCtx) -> AssetList {
    serde_json::from_value::<AssetList>(
        get_asset_by_creator(req_params, ctx.clone())
            .await
            .expect("Failed to get assets."),
    )
    .expect("Failed serialize DAO assets..")
}
