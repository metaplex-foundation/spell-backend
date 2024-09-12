#![allow(dead_code)]

use entities::l2::PublicKey;
use interfaces::asset_service::L2AssetInfo;
use json_rpc::config::app_context::ArcedAppCtx;
use json_rpc::endpoints::rpc_asset_models::Asset;
use json_rpc::endpoints::types::{AssetList, JsonRpcError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use util::publickey::PublicKeyExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub metadata_json: String,
    pub owner: String,
    pub creator: String,
    pub authority: String,
    pub collection: Option<String>,
}

impl CreateAssetRequest {
    const METADATA_JSON: &'static str = r#"
        {"description": "An astronaut exploring distant galaxies.", "image": "https://example.com/images/galactic_explorer_1.png"}
    "#;

    fn with_name_and_owner(name: &str, owner: &str) -> Self {
        Self {
            name: name.to_string(),
            metadata_json: Self::METADATA_JSON.to_string(),
            owner: owner.to_string(),
            creator: PublicKey::new_unique().to_string(),
            authority: PublicKey::new_unique().to_string(),
            collection: Some(PublicKey::new_unique().to_string()),
        }
    }

    fn with_name_and_creator(name: &str, creator: &str) -> Self {
        Self {
            name: name.to_string(),
            metadata_json: Self::METADATA_JSON.to_string(),
            owner: PublicKey::new_unique().to_string(),
            creator: creator.to_string(),
            authority: PublicKey::new_unique().to_string(),
            collection: Some(PublicKey::new_unique().to_string()),
        }
    }
}

pub fn create_assets_with_same_owner_requests() -> Vec<CreateAssetRequest> {
    vec![
        CreateAssetRequest {
            name: "Galactic Explorer #1".to_string(),
            metadata_json: r#"{"description": "An astronaut exploring distant galaxies.", "image": "https://example.com/images/galactic_explorer_1.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "6iDkdEY9HVY2XM4T6MbS6fpnX4AGv4sq9bhp28kRSrHf".to_string(),
            collection: Some("2VST7tMfRf7X2YqAfoem5BzyrWn2CrZqQG8om4Fh9K6R".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #2".to_string(),
            metadata_json: r#"{"description": "A futuristic spaceship navigating through nebulae.", "image": "https://example.com/images/galactic_explorer_2.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "6iDkdEY9HVY2XM4T6MbS6fpnX4AGv4sq9bhp28kRSrHf".to_string(),
            collection: Some("3j4P4Xq3xZ7FbZB6PHTrFQs6rCnzZ7BfqTp55pSt77rV".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #3".to_string(),
            metadata_json: r#"{"description": "A cosmic landscape with distant stars.", "image": "https://example.com/images/galactic_explorer_3.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "8L5jqJ9mY9btqQF8XM6hpv9Y1rM3K6jF3y5qfJ7bH7v9".to_string(),
            authority: "7T3prz4G8HT8JnY3n3K8GR7n7s3yzM2u5b6pqWfD6WnP".to_string(),
            collection: Some("4kR9SmmSp4T86PvPLg3Jr7Erz5fD8sD83MwFzGJ4v4cD".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #4".to_string(),
            metadata_json: r#"{"description": "A robotic explorer on a distant planet.", "image": "https://example.com/images/galactic_explorer_4.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "5Fw2t5H9L6CpZqB8H9vL5pJt4N9E7bHk9N5qR7yK2L8H".to_string(),
            authority: "8YkF9K6cLg77zSxVpN9rX4cqj6Jr7L6QK9Hg2PzQb1tP".to_string(),
            collection: Some("5vWdY6Ht9F7RQ6J7j7C8rzKHs7G79NjZ8Jk4g3N2tPp5".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #5".to_string(),
            metadata_json: r#"{"description": "A space station orbiting a vibrant star.", "image": "https://example.com/images/galactic_explorer_5.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "6C9vX6D8c2T7D9pM3Y5jK6BzR9V2Jh8X8W3L6Q5Z4NxG".to_string(),
            authority: "9FgR2z8vZ8N4D7qH6K7hL2W7X8uR3vT2b6qW3L9J9Pt".to_string(),
            collection: Some("6y8P8QcG4T6RfL9FpL8zTrR5D2JvK4gH5D7wG5s8FkY3".to_string()),
        },
    ]
}

pub fn create_assets_with_same_owner_requests_with_random_values() -> Vec<CreateAssetRequest> {
    let name_prefix = "Galactic Explorer #".to_string();
    let owner = "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string();

    (1..=100)
        .map(|iteration| {
            let (name, owner) = (format!("{name_prefix}{iteration}"), owner.clone());
            CreateAssetRequest::with_name_and_owner(&name, &owner)
        })
        .collect()
}

pub fn create_different_assets_requests() -> Vec<CreateAssetRequest> {
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

pub fn create_assets_with_same_creator_requests() -> Vec<CreateAssetRequest> {
    vec![
        CreateAssetRequest {
            name: "Galactic Explorer #1".to_string(),
            metadata_json: r#"{"description": "An astronaut exploring distant galaxies.", "image": "https://example.com/images/galactic_explorer_1.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "6iDkdEY9HVY2XM4T6MbS6fpnX4AGv4sq9bhp28kRSrHf".to_string(),
            collection: Some("2VST7tMfRf7X2YqAfoem5BzyrWn2CrZqQG8om4Fh9K6R".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #2".to_string(),
            metadata_json: r#"{"description": "A futuristic spaceship navigating through nebulae.", "image": "https://example.com/images/galactic_explorer_2.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "6iDkdEY9HVY2XM4T6MbS6fpnX4AGv4sq9bhp28kRSrHf".to_string(),
            collection: Some("3j4P4Xq3xZ7FbZB6PHTrFQs6rCnzZ7BfqTp55pSt77rV".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #3".to_string(),
            metadata_json: r#"{"description": "A cosmic landscape with distant stars.", "image": "https://example.com/images/galactic_explorer_3.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "7T3prz4G8HT8JnY3n3K8GR7n7s3yzM2u5b6pqWfD6WnP".to_string(),
            collection: Some("4kR9SmmSp4T86PvPLg3Jr7Erz5fD8sD83MwFzGJ4v4cD".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #4".to_string(),
            metadata_json: r#"{"description": "A robotic explorer on a distant planet.", "image": "https://example.com/images/galactic_explorer_4.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "8YkF9K6cLg77zSxVpN9rX4cqj6Jr7L6QK9Hg2PzQb1tP".to_string(),
            collection: Some("5vWdY6Ht9F7RQ6J7j7C8rzKHs7G79NjZ8Jk4g3N2tPp5".to_string()),
        },
        CreateAssetRequest {
            name: "Galactic Explorer #5".to_string(),
            metadata_json: r#"{"description": "A space station orbiting a vibrant star.", "image": "https://example.com/images/galactic_explorer_5.png"}"#.to_string(),
            owner: "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string(),
            creator: "4R7zW4cV9D6x2nZyZ2DtCzF8H9jtyqG7nD8P8ZJ8ZxkM".to_string(),
            authority: "9FgR2z8vZ8N4D7qH6K7hL2W7X8uR3vT2b6qW3L9J9Pt".to_string(),
            collection: Some("6y8P8QcG4T6RfL9FpL8zTrR5D2JvK4gH5D7wG5s8FkY3".to_string()),
        },
    ]
}

pub fn create_assets_with_same_creator_requests_with_random_values() -> Vec<CreateAssetRequest> {
    let name_prefix = "Galactic Explorer #".to_string();
    let creator = "9hfHbS34pV8eDPi8F3B9N6N9hvX2MjLs1B3fKm6vQeEq".to_string();

    (1..=100)
        .map(|iteration| {
            let (name, creator) = (format!("{name_prefix}{iteration}"), creator.clone());
            CreateAssetRequest::with_name_and_creator(&name, &creator)
        })
        .collect()
}

pub async fn fill_database_with_test_data(
    app_ctx: ArcedAppCtx,
    asset_creation_strategy: fn() -> Vec<CreateAssetRequest>,
) -> Vec<L2AssetInfo> {
    let requests_for_asset_creation = asset_creation_strategy();
    let mut filled_data_from_db = Vec::with_capacity(requests_for_asset_creation.len());

    for asset_req in requests_for_asset_creation {
        filled_data_from_db.push(
            create_asset(asset_req, app_ctx.clone())
                .await
                .unwrap_or_else(|e| panic!("Cannot create asset: {e}!")),
        );
    }

    filled_data_from_db
}

pub fn form_asset_json_uri(pubkey: &str) -> String {
    format!("127.0.0.1:8080/asset/{pubkey}/metadata.json")
}

pub fn extract_asset_name_from_das_asset(asset: Asset) -> String {
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

async fn create_asset(req_params: CreateAssetRequest, ctx: ArcedAppCtx) -> Result<L2AssetInfo, Box<dyn Error>> {
    Ok(ctx
        .asset_service
        .create_asset(
            &req_params.metadata_json,
            PublicKey::from_bs58(&req_params.owner).ok_or(JsonRpcError::invalid_params("Invalid owner"))?,
            PublicKey::from_bs58(&req_params.creator).ok_or(JsonRpcError::invalid_params("Invalid creator"))?,
            PublicKey::from_bs58(&req_params.authority).ok_or(JsonRpcError::invalid_params("Invalid authority"))?,
            &req_params.name,
            req_params
                .collection
                .and_then(|collection| PublicKey::from_bs58(&collection)),
        )
        .await?)
}

pub fn get_first_asset_name(asset_list: &AssetList) -> String {
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
