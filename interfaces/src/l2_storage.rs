use async_trait::async_trait;
use entities::l2::{AssetSorting, L2Asset, PublicKey};

/// Storage interfaces for L2 assets managing
#[async_trait]
pub trait L2Storage {
    async fn save(&self, asset: &L2Asset) -> anyhow::Result<()>;
    async fn find(&self, pubkey: &PublicKey) -> anyhow::Result<Option<L2Asset>>;
    async fn find_batch(&self, pubkeys: &[PublicKey]) -> anyhow::Result<Vec<L2Asset>>;
    async fn find_by_owner(
        &self,
        owner_pubkey: &PublicKey,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2Asset>>;
    async fn find_by_creator(
        &self,
        creator_pubkey: &PublicKey,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2Asset>>;

    /// Should guarantee atomic status update.
    async fn lock_asset_before_minting(&self, pubkey: &PublicKey) -> anyhow::Result<bool>;
    async fn find_l1_asset_signature(&self, asset_pubkey: &PublicKey) -> Option<Vec<u8>>;
    async fn add_l1_asset(&self, pubkey: &PublicKey, tx_signature: &[u8]) -> anyhow::Result<()>;
    async fn finalize_mint(&self, pubkey: &PublicKey) -> anyhow::Result<()>;
    async fn mint_didnt_happen(&self, pubkey: &PublicKey) -> anyhow::Result<()>;
}

#[derive(Debug, PartialEq)]
pub struct DerivationValues {
    pub account: u32,
    pub address: u32,
}

/// Represents pair of increment only counters that serve as a value source
/// for Bip44 derivation string that is used as a seed for HD wallet.
/// See: https://solanacookbook.com/references/keypairs-and-wallets.html#how-to-restore-a-keypair-from-a-mnemonic-phrase
#[async_trait]
pub trait Bip44DerivationSequence {
    /// Returns next value of account and address
    async fn next_account_and_address(&self) -> anyhow::Result<DerivationValues>;
}

#[derive(thiserror::Error, Debug)]
pub enum L2StorageError {
    #[error("No asset identified by pubkey={0:?}")]
    L2AssetNotFound(PublicKey),
}
