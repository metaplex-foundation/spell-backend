use async_trait::async_trait;
use entities::l2::{L2Asset, PublicKey};

/// Storage interfaces for L2 assets managing
#[async_trait]
pub trait L2Storage {
    async fn save(&self, asset: &L2Asset) -> anyhow::Result<()>;
    async fn find(&self, pubkey: &PublicKey) -> anyhow::Result<Option<L2Asset>>;
}

#[derive(Debug, PartialEq)]
pub struct DerivationValues {
    pub account: u32,
    pub change: u32,
}

/// Represents pair of increment only counters that serve as a value source
/// for Bip44 derivation string that is used as a seed for HD wallet.
/// See: https://solanacookbook.com/references/keypairs-and-wallets.html#how-to-restore-a-keypair-from-a-mnemonic-phrase
#[async_trait]
pub trait Bip44DerivationSequence {
    /// Increments account counter
    async fn next_account(&self) -> anyhow::Result<DerivationValues>;
    /// Increments change counter and returns its value along with account value
    async fn next_change(&self) -> anyhow::Result<DerivationValues>;
}
