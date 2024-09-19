use entities::l2::PublicKey;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::{Keypair, Signature};
use thiserror::Error;

use solana_sdk::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedMintIxInfo {
    pub asset_pubkey: PublicKey,
    pub authority: Option<PublicKey>,
    pub owner: Option<PublicKey>,
    pub payer: PublicKey,
    pub collection: Option<PublicKey>,
    pub name: String,
    pub uri: String,
}

#[async_trait::async_trait]
pub trait L1Service {
    /// Takes Transaction that contains single mpl-core CreateV1Builder instruction
    /// and extracts NTF asset pubkey from it.
    /// ## Args:
    /// * `tx` - transaction created on the client side
    fn parse_mint_transaction(&self, tx: &Transaction) -> anyhow::Result<ParsedMintIxInfo>;

    /// Accepts a transaction that contains single mpl-core CreateV1Builder instruction
    /// created on the client side and partially signed by the client,
    /// and executes it on Solana.
    /// The asset specified in the instruction should be an L2 asset that is not minted yet.
    /// The payer for the transaction should be specified by the client.
    /// ## Args:
    /// * `tx` - transaction created on the client side
    /// * `asset_keypair` - keypair for the asset specified in the transaction,
    ///    i.e. asset ID is a pubkey of this keypair.
    ///    (We use bip44 to derive this keypair from our master keypair,
    ///    when we initially an L2 asset)
    async fn execute_mint_transaction(&self, tx: Transaction, asset_keypair: &Keypair) -> anyhow::Result<Signature>;
}

#[derive(Error, Debug)]
pub enum L1MintTransactionError {
    #[error("Transaction contains no instructions")]
    NoInstruction,
    #[error("Transaction contains other unexpected instructions")]
    UnexpectedInstructions,
    #[error("Malformed transaction")]
    MalformedTransaction,
    #[error("Malformed mpl-code create v1 instruction")]
    MalformedMintAssetInstruction,
    #[error("Wrong mpl-core program id")]
    WrongMplCoreProgrmaId,
}
