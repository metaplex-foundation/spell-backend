use serde::{Deserialize, Serialize};

pub type PublicKey = [u8; 32];

/// Represents L2 asset
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct L2Asset {
    /// Asset uniqueue identifier
    pub pubkey: PublicKey,

    /// Asset name
    pub name: String,

    /// Asset owner. For L2 assets owner is us.
    /// Meaning initially all assets ownder by us, and the user who uploaded asset becomes authority.
    pub owner: PublicKey,

    /// Also we.
    pub creator: PublicKey,

    /// ID of collection the asset belongs to.
    pub collection: Option<PublicKey>,

    /// Pubkey of user who uploaded the asset.
    pub authority: PublicKey,

    /// The timestamp of the L2 asset "mint"
    pub create_timestamp: chrono::NaiveDateTime, // Need timezone?

    /// Number that had been used as account in PIB44 derivation,
    /// to generate the asset pubkey
    pub pib44_account_num: u32,

    /// Number that had been used as change in PIB44 derivation,
    /// to generate the asset pubkey
    pub pib44_address_num: u32,
}

pub struct AssetExtended {
    pub asset: L2Asset,
    pub metadata_uri: String,
    pub royalty_basis_points: u16,
}

impl AssetExtended {
    pub fn new(asset: L2Asset, metadata_uri: String) -> Self {
        Self { asset, metadata_uri, royalty_basis_points: 0 }
    }
}

pub fn pubkey_to_string(pubkey: PublicKey) -> String {
    bs58::encode(pubkey).into_string()
}
