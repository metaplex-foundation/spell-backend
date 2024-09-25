use crate::utils::{create_different_assets_requests, fill_database_with_test_data, form_asset_json_uri};
use entities::dto::Asset;
use json_rpc::endpoints::errors::DasApiError;
use json_rpc::endpoints::get_asset::get_asset;
use json_rpc::endpoints::types::GetAsset;
use json_rpc::setup::app_context::AppCtx;
use json_rpc::setup::app_setup::AppSetup;
use setup::TestEnvironmentCfg;
use solana_sdk::pubkey::Pubkey;
use util::publickey::PublicKeyExt;

mod utils;

#[tokio::test]
async fn get_single_asset_positive() {
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
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
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let non_existing_pubkey = Pubkey::new_unique().to_string();

    let get_asset_req = GetAsset { id: non_existing_pubkey };

    let res = get_asset(get_asset_req, app_ctx.clone()).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), DasApiError::NoDataFoundError.into());
}

#[tokio::test]
async fn get_single_asset_using_invalid_pubkey() {
    let t_env = TestEnvironmentCfg::default().with_pg().with_s3().start().await;
    let app_ctx = AppCtx::new(&AppSetup::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let invalid_pubkey = String::from("Something that is not a public key.");

    let get_asset_req = GetAsset { id: invalid_pubkey.clone() };

    let res = get_asset(get_asset_req, app_ctx.clone()).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), DasApiError::PubkeyValidationError(invalid_pubkey).into());
}
