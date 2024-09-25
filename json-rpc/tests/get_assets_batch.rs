use crate::utils::{
    create_different_assets_requests, create_single_asset_request, fill_database_with_test_data, form_asset_json_uri,
};
use entities::dto::Asset;
use json_rpc::endpoints::get_asset::get_asset_batch;
use json_rpc::endpoints::types::GetAssetBatch;
use json_rpc::setup::app_context::AppCtx;
use json_rpc::setup::app_setup::AppSetup;
use serde_json::Value;
use setup::TestEnvironmentCfg;
use solana_sdk::pubkey::Pubkey;
use util::publickey::PublicKeyExt;

mod utils;

#[tokio::test]
async fn get_asset_batch_negative() {
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
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
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
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
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
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
async fn verify_royalty_points() {
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let assets_pubkeys_data_in_db = fill_database_with_test_data(app_ctx.clone(), create_single_asset_request)
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

    let expected_royalty_basis_points = res.first().unwrap().clone().royalty.unwrap().basis_points;
    for (expected_pubkey, actual_dao_asset) in assets_pubkeys_data_in_db.into_iter().zip(res) {
        let actual_pubkey = actual_dao_asset.id;
        assert_eq!(expected_pubkey, actual_pubkey);

        let expected_asset_json_uri = form_asset_json_uri(&expected_pubkey);
        let actual_asset_json_uri = form_asset_json_uri(&actual_pubkey);
        assert_eq!(expected_asset_json_uri, actual_asset_json_uri);

        let actual_royalty_basis_points = actual_dao_asset.royalty.unwrap().basis_points;
        assert_eq!(expected_royalty_basis_points, actual_royalty_basis_points);
    }
}
