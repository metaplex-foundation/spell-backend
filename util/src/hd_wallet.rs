use solana_sdk::{
    derivation_path::DerivationPath,
    signature::{generate_seed_from_seed_phrase_and_passphrase, Keypair},
    signer::SeedDerivable,
};


const SOLANA_COIN: u32 = 501;

#[derive(Clone, Debug)]
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
    /// * `address` - address number for given account
    pub fn make_hd_wallet(&self, account: u32, address: u32) -> Option<Keypair> {
        // TODO: compare result with the result of JS-based implementation from
        // https://solanacookbook.com/references/keypairs-and-wallets.html#how-to-restore-a-keypair-from-a-mnemonic-phrase
        // to verify the correctness

        // Should never ever fail
        let derivation_path = DerivationPath::from_absolute_path_str(
            format!("m/44'/{SOLANA_COIN}'/{account}'/0/{address}").as_str()
        ).unwrap();

        Keypair::from_seed_and_derivation_path(&self.seed, Some(derivation_path)).ok()
    }
}

#[test]
fn test_hd_wallet_generation() {
    let sut = HdWalletProducer::from_seed_phrase_and_passphrase("", "");

    let seed = generate_seed_from_seed_phrase_and_passphrase("", "");

    {
        let absolute_str = "m/44'/501'/1'/0/1";
        let derivation_path = DerivationPath::from_absolute_path_str(absolute_str).unwrap();
        let result = Keypair::from_seed_and_derivation_path(&seed, Some(derivation_path)).unwrap();
        assert_eq!(result, sut.make_hd_wallet(1, 1).unwrap());
    }
    {
        let absolute_str = "m/44'/501'/1'/0/5";
        let derivation_path = DerivationPath::from_absolute_path_str(absolute_str).unwrap();
        let result = Keypair::from_seed_and_derivation_path(&seed, Some(derivation_path)).unwrap();
        assert_eq!(result, sut.make_hd_wallet(1, 5).unwrap());
    }
    {
        let absolute_str = "m/44'/501'/999'/0/999";
        let derivation_path = DerivationPath::from_absolute_path_str(absolute_str).unwrap();
        let result = Keypair::from_seed_and_derivation_path(&seed, Some(derivation_path)).unwrap();
        assert_eq!(result, sut.make_hd_wallet(999, 999).unwrap());
    }
}
