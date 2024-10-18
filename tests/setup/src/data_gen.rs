use entities::l2::PublicKey;
use rand::Rng;
use util::publickey::PublicKeyExt;

pub fn rand_pubkey() -> PublicKey {
    rand::thread_rng().gen::<PublicKey>()
}

pub fn rand_pubkey_str() -> String {
    format!("spell{}", rand::thread_rng().gen::<PublicKey>().to_bs58())
}
