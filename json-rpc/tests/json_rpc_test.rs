mod utils;

use crate::utils::{create_asset, CreateAssetRequest};
use interfaces::asset_service::L2AssetInfo;
use json_rpc::config::app_config::AppConfig;
use json_rpc::config::app_context::{AppCtx, ArcedAppCtx};
use json_rpc::endpoints::errors::DasApiError;
use json_rpc::endpoints::get_nft::{get_asset, get_asset_batch};
use json_rpc::endpoints::rpc_asset_models::Asset;
use json_rpc::endpoints::types::{GetAsset, GetAssetBatch};
use serde_json::Value;
use setup::TestEnvironmentCfg;
use solana_sdk::pubkey::Pubkey;
use util::publickey::PublicKeyExt;

fn create_asset_requests() -> Vec<CreateAssetRequest> {
    vec![
        CreateAssetRequest {
            name: "ArtPieceOne".to_string(),
            metadata_json: r#"{"description": "First NFT", "attributes": {"rarity": "rare"}}"#.to_string(),
            owner: "5HueWz2D9f8yjXAx8eb6WY8ocE2Fy6smAt1NkJ39kF9z".to_string(),
            creator: "7YmM7NUj9wFmAkT4mJ2LD6yyYHtKBu9qMEfF7cX5mH9J".to_string(),
            authority: "3JzRkTrzYa1ZFbVRd2kVy5YoAUp4XCN8UNW5NTR4jMgP".to_string(),
            collection: None,
        },
        CreateAssetRequest {
            name: "DigitalCollectible".to_string(),
            metadata_json: r#"{"description": "Exclusive collectible", "attributes": {"series": "limited"}}"#
                .to_string(),
            owner: "8QzJK3WYaZBfX9qf9XJe8Phc7xZ8yz6kC8QzTfPyLfPb".to_string(),
            creator: "4VfFHrM7vZYg5Ezm8YQVBr9N7vHj3DFJLqPh2Qj9L9Tp".to_string(),
            authority: "9UuKzN5b2mVz9xVh8qK3Px7yYoVs8XVQfF2YkHp4jMcK".to_string(),
            collection: None,
        },
        CreateAssetRequest {
            name: "GamingAvatar".to_string(),
            metadata_json: r#"{"description": "Avatar for gaming", "attributes": {"level": 5}}"#.to_string(),
            owner: "2YhMfS7oMv2gV8hB3kJ8PmZbW6zQmLq5rK1TrF1yNq7Z".to_string(),
            creator: "3RfNrQk8zDfX5GvH9pV7cPdT3hWj2Ff8VNpF8Xm8PfJk".to_string(),
            authority: "6TpLpS8b2cZk3mZk8qW5Pc6yJoVt8XVXrK3XrF2YkNtK".to_string(),
            collection: None,
        },
    ]
}

async fn fill_database_with_test_data(app_ctx: ArcedAppCtx) -> Vec<L2AssetInfo> {
    let create_assets_requests = create_asset_requests();
    let mut res = Vec::with_capacity(create_assets_requests.len());

    for asset_req in create_assets_requests {
        let created_assets = create_asset(asset_req, app_ctx.clone())
            .await
            .unwrap_or_else(|e| panic!("Cannot create asset: {e}!"));
        let created_asset =
            serde_json::from_value(created_assets).unwrap_or_else(|e| panic!("Cannot serialize L2AssetInfo: {e}!"));

        res.push(created_asset)
    }

    res
}

fn form_asset_json_uri(pubkey: &str) -> String {
    format!("127.0.0.1:8081/asset/{pubkey}/metadata.json")
}

#[tokio::test]
async fn get_single_asset_positive() {
    let t_env = TestEnvironmentCfg::with_all().start().await;
    let app_ctx = AppCtx::new(&AppConfig::from_settings(t_env.make_test_cfg().await))
        .await
        .arced();

    let assets_data_in_db = fill_database_with_test_data(app_ctx.clone()).await;

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

    let assets_pubkeys_data_in_db = fill_database_with_test_data(app_ctx.clone())
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

    let mut pubkeys_for_request = fill_database_with_test_data(app_ctx.clone())
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
