use entities::{
    dto::{Asset, AssetExtended},
    l2::{L2Asset, PublicKey},
};
use util::publickey::PublicKeyExt;

const ROYALTY_BASIS_POINTS: u16 = 0;

#[derive(Debug, Clone)]
pub struct AssetDtoConverter {
    pub metadata_server_base_url: String,
}

impl AssetDtoConverter {
    pub fn to_response_asset_dto(&self, entity: &L2Asset, metadata: Option<String>) -> Asset {
        let asset_ex = AssetExtended {
            asset: entity.clone(),
            metadata_uri: get_metadata_uri_for_key(&self.metadata_server_base_url, entity.pubkey),
            royalty_basis_points: ROYALTY_BASIS_POINTS,
        };

        let metadata_json_text = metadata.unwrap_or("{}".to_string());
        let metadata_json_value = serde_json::from_str(&metadata_json_text).unwrap_or_default();

        Asset::from((asset_ex, metadata_json_value))
    }
}

pub fn get_metadata_uri_for_key_str(metadata_server_base_url: &str, public_key: &str) -> String {
    format!("{}/asset/{}/metadata.json", metadata_server_base_url, public_key)
}

pub fn get_metadata_uri_for_key(metadata_server_base_url: &str, public_key: PublicKey) -> String {
    get_metadata_uri_for_key_str(metadata_server_base_url, public_key.to_bs58().as_str())
}
