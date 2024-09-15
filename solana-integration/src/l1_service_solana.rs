use std::sync::Arc;

use entities::l2::PublicKey;
use interfaces::l1_service::{L1MintTransactionError, L1Service};

use solana_client::nonblocking::rpc_client::{self, RpcClient};
use solana_sdk::signature::Signature;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::transaction::Transaction;

pub struct SolanaService {
    client: Arc<RpcClient>,
}

impl SolanaService {
    pub fn new(solana_url: &str) -> SolanaService {
        SolanaService { client: Arc::new(rpc_client::RpcClient::new(solana_url.to_string())) }
    }
}

#[async_trait::async_trait]
impl L1Service for SolanaService {
    fn extract_mint_asset_pubkey(&self, tx: &Transaction) -> anyhow::Result<PublicKey> {
        let instructions = &tx.message().instructions;
        if instructions.is_empty() {
            anyhow::bail!(L1MintTransactionError::NoInstruction);
        }
        if instructions.len() > 1 {
            anyhow::bail!(L1MintTransactionError::UnexpectedInstructions);
        }
        // TODO: check it's mpl-core program

        // Order of pubkeys in CreateV1 instruction:
        // 0) asset
        // 1) collection | MPL_CORE_ID
        // 2) authority | MPL_CORE_ID
        // 3) payer
        // 4) owner | MPL_CORE_ID
        // 5) update_authority | MPL_CORE_ID
        // 6) system_program
        // 7) log_wrapper | MPL_CORE_ID
        let ix_accounts = &instructions[0].accounts;

        if ix_accounts.len() < 8 {
            anyhow::bail!(L1MintTransactionError::MalformedMintAssetInstruction);
        }

        let tx_accounts = &tx.message.account_keys;

        // Instruction keeps indexes of pubkeys stored on transaction message level,
        // that's why the amount of pubkeys in transaction should not smaller
        // that than the biggest pubkey index on instuction level + 1
        if (tx_accounts.len() as u8) < ix_accounts.iter().max().unwrap() + 1 {
            anyhow::bail!(L1MintTransactionError::MalformedTransaction);
        }

        // TODO: What kind of validations we need?

        let asset_pubkey = tx_accounts[ix_accounts[0] as usize];

        Ok(asset_pubkey.to_bytes())
    }

    async fn execute_mint_transaction(
        &self,
        mut tx: Transaction,
        asset_keypair: &Keypair,
    ) -> anyhow::Result<Signature> {
        tx.sign(&[asset_keypair], tx.message.recent_blockhash);

        let tx_singnature = self.client.send_and_confirm_transaction(&tx).await.unwrap();

        Ok(tx_singnature)
    }
}
