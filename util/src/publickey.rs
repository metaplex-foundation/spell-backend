use entities::l2::PublicKey;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub trait PublicKeyExt {
    fn from_bs58(bs58_str: &str) -> Option<Self>
    where
        Self: Sized;

    fn to_bs58(&self) -> String;

    fn new_unique() -> Self;

    fn to_string(self) -> String;
}

impl PublicKeyExt for PublicKey {
    fn from_bs58(bs58_str: &str) -> Option<PublicKey> {
        Pubkey::from_str(bs58_str).map(|pk| pk.to_bytes()).ok()
    }

    fn to_bs58(&self) -> String {
        bs58::encode(self).into_string()
    }

    fn new_unique() -> PublicKey {
        Pubkey::new_unique().to_bytes()
    }

    fn to_string(self) -> String {
        Pubkey::new_from_array(self).to_string()
    }
}
