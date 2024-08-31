use entities::l2::PublicKey;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub trait PublicKeyExt {
    fn from_bs58(bs58_str: &str) -> Option<Self>
    where
        Self: Sized;

    fn new_unique() -> Self;
}

impl PublicKeyExt for PublicKey {
    fn from_bs58(bs58_str: &str) -> Option<PublicKey> {
        Pubkey::from_str(bs58_str).map(|pk| pk.to_bytes()).ok()
    }

    fn new_unique() -> PublicKey {
        Pubkey::new_unique().to_bytes()
    }
}
