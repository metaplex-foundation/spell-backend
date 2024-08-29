use solana_sdk::{
    derivation_path::DerivationPath,
    signature::{generate_seed_from_seed_phrase_and_passphrase, Keypair},
    signer::SeedDerivable,
};

pub struct HdWalletProducer {
    /// Seed of the masterkey.
    seed: Vec<u8>,
}

impl HdWalletProducer {
    pub fn from_seed_phrase_and_passphrase(
        seed_phrase: &str,
        passphrase: &str,
    ) -> HdWalletProducer {
        let seed = generate_seed_from_seed_phrase_and_passphrase(seed_phrase, passphrase);
        HdWalletProducer { seed }
    }

    /// Generates BIP44 HD Wallet based on the seed and given offset.
    /// * `account` - account number for BIP44 derivation
    /// * `change` - change number for BIP44 derivation
    pub fn make_hd_wallet(&self, account: u32, change: u32) -> Option<Keypair> {
        // TODO: compare result with the result of JS-based implementation from
        // https://solanacookbook.com/references/keypairs-and-wallets.html#how-to-restore-a-keypair-from-a-mnemonic-phrase
        // to verify the correctness
        let derivation_path = DerivationPath::new_bip44(Some(account), Some(change));
        Keypair::from_seed_and_derivation_path(&self.seed, Some(derivation_path)).ok()
    }
}
