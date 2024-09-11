use entities::{l2::L2Asset, rpc_asset_models::Asset, types::AssetExtended};
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
            metadata_uri: self.get_metadata_uri_for_key(entity.pubkey.to_bs58().as_str()),
            royalty_basis_points: ROYALTY_BASIS_POINTS,
        };

        let metadata_json_text = metadata.unwrap_or("{}".to_string());
        let metadata_json_value = serde_json::from_str(&metadata_json_text).unwrap_or_default();

        Asset::from((asset_ex, metadata_json_value))
    }

    pub fn get_metadata_uri_for_key(&self, public_key: &str) -> String {
        format!("{}/asset/{}/metadata.json", self.metadata_server_base_url, public_key)
    }
}
