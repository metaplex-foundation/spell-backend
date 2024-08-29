
use entities::l2::PublicKey;
use rand::Rng;

pub fn rand_pubkey() -> PublicKey {
    rand::thread_rng().gen::<PublicKey>()
}
