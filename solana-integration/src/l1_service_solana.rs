use std::sync::Arc;

use interfaces::l1_service::{L1MintTransactionError, L1Service, ParsedMintIxInfo};

use mpl_core::instructions::CreateV1InstructionArgs;
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
    fn extract_mint_asset_pubkey(&self, tx: &Transaction) -> anyhow::Result<ParsedMintIxInfo> {
        let instructions = &tx.message().instructions;
        if instructions.is_empty() {
            anyhow::bail!(L1MintTransactionError::NoInstruction);
        }
        if instructions.len() > 1 {
            anyhow::bail!(L1MintTransactionError::UnexpectedInstructions);
        }

        let mint_ix = &instructions[0];
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

        if tx_accounts[mint_ix.program_id_index as usize] != mpl_core::ID {
            anyhow::bail!(L1MintTransactionError::WrongMplCoreProgrmaId);
        }

        let asset_pubkey = tx_accounts[ix_accounts[0] as usize];
        let collection = Some(tx_accounts[ix_accounts[1] as usize]).filter(|pk| *pk != mpl_core::ID);
        let authority = tx_accounts[ix_accounts[2] as usize];
        let payer = tx_accounts[ix_accounts[3] as usize];
        let owner = tx_accounts[ix_accounts[4] as usize];

        let (name, uri) = {
            use borsh::de::BorshDeserialize;
            let Ok(mint_args) = CreateV1InstructionArgs::try_from_slice(&mint_ix.data[1..]) else {
                anyhow::bail!(L1MintTransactionError::MalformedMintAssetInstruction);
            };
            (mint_args.name, mint_args.uri)
        };

        Ok(ParsedMintIxInfo {
            asset_pubkey: asset_pubkey.to_bytes(),
            authority: authority.to_bytes(),
            owner: owner.to_bytes(),
            payer: payer.to_bytes(),
            collection: collection.map(|pk| pk.to_bytes()),
            name,
            uri,
        }) // asset_pubkey.to_bytes()
    }

    async fn execute_mint_transaction(
        &self,
        mut tx: Transaction,
        asset_keypair: &Keypair,
    ) -> anyhow::Result<Signature> {
        tx.sign(&[asset_keypair], tx.message.recent_blockhash);

        let tx_singnature = self.client.send_and_confirm_transaction(&tx).await?;

        Ok(tx_singnature)
    }
}
