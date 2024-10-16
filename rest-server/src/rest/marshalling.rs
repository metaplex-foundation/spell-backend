use base64::{prelude::BASE64_STANDARD, Engine};
use solana_sdk::transaction::Transaction;

pub fn decode_transaction(base64_encoded: &str) -> anyhow::Result<Transaction> {
    let bytes = BASE64_STANDARD.decode(base64_encoded)?;
    let transaction = bincode::deserialize::<Transaction>(&bytes)?;
    Ok(transaction)
}

#[cfg(test)]
mod test {
    use super::*;
    use mpl_core::instructions::CreateV1Builder;
    use solana_sdk::{hash::Hash, pubkey::Pubkey, signature::Keypair, signer::Signer};

    #[test]
    pub fn test_decode_transaction_from_base64() {
        let original = {
            let asset_pubkey = Pubkey::new_unique();
            let payer = Keypair::new();
            let authority = Keypair::new();

            let create_asset_ix = CreateV1Builder::new()
                .asset(asset_pubkey)
                .payer(payer.pubkey())
                .name("My Asset 1".to_string())
                .uri(format!("http://node1-dev.mtgrd-das.app:8080/asset/{}/metadata.json", &asset_pubkey))
                .authority(Some(authority.pubkey()))
                .owner(Some(payer.pubkey()))
                .instruction();

            let signers: Vec<&Keypair> = vec![&payer, &authority];

            let last_blockhash = Hash::from([1u8; 32]);

            let mut create_asset_tx = Transaction::new_with_payer(&[create_asset_ix], Some(&payer.pubkey()));
            create_asset_tx.partial_sign(&signers, last_blockhash);

            create_asset_tx
        };

        let bincode_serialized = bincode::serialize(&original).unwrap();
        let base64_serialized = BASE64_STANDARD.encode(bincode_serialized);

        let decoded = decode_transaction(&base64_serialized).unwrap();

        assert_eq!(original, decoded);
    }
}
