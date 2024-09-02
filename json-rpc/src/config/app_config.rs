use crate::config::app_context::AppCtx;
use crate::config::method_registrar::RpcMethodRegistrar;
use crate::endpoints::get_nft::{
    get_asset, get_asset_batch, get_asset_by_creator, get_asset_by_owner,
};
use crate::endpoints::health_check::health;
use jsonrpc_core::IoHandler;
use service::asset_service_impl::AssetServiceImpl;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use tracing::info;
use util::config::Settings;
use util::hd_wallet::HdWalletProducer;

#[derive(Clone, Debug)]
pub struct AppConfig {
    settings: Settings,
}

impl AppConfig {
    pub fn from_settings(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn register_rpc_methods(self) -> IoHandler {
        RpcMethodRegistrar::new(self.create_state_and_ctx().await.arced())
            .method_without_ctx_and_params(health)
            .method(get_asset)
            .method(get_asset_batch)
            .method(get_asset_by_owner)
            .method(get_asset_by_creator)
            .finish()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(
            self.settings.json_rpc_server.host.into(),
            self.settings.json_rpc_server.port,
        )
    }

    async fn create_state_and_ctx(&self) -> AppCtx {
        let connection_pool = Self::create_connection_pool(
            &self.settings.database.connection_url,
            self.settings.database.max_connections,
            self.settings.database.min_connections,
        )
        .await;

        let s3_storage = match self.settings.is_production_profile() {
            true => Self::connect_to_mocked_s3_storage().await,
            false => Self::connect_to_s3_storage().await,
        };

        let (master_pubkey, wallet_producer) = self
            .settings
            .is_not_production_profile()
            .then(|| (Default::default(), HdWalletProducer::mocked()))
            .unwrap_or_else(|| unimplemented!());

        let l2_storage = Arc::new(L2StoragePg::new_from_pool(connection_pool));
        let s3_storage = Arc::new(s3_storage);

        let asset_service = AssetServiceImpl {
            master_pubkey,
            wallet_producer,
            derivation_sequence: l2_storage.clone(),
            l2_storage: l2_storage.clone(),
            asset_metadata_storage: s3_storage.clone(),
            blob_storage: s3_storage.clone(),
        };

        AppCtx::new(asset_service)
    }

    async fn connect_to_mocked_s3_storage() -> S3Storage {
        S3Storage::mocked().await
    }

    async fn connect_to_s3_storage() -> S3Storage {
        unimplemented!()
    }

    async fn create_connection_pool(
        db_url: &str,
        max_size_pool: u32,
        min_size_pool: u32,
    ) -> PgPool {
        info!(
            "Creating connection pool from: '{}', with max_size: '{}', and min_size: '{}' connections.",
            db_url, max_size_pool, min_size_pool,
        );
        PgPoolOptions::new()
            .max_connections(max_size_pool)
            .min_connections(min_size_pool)
            .connect(&db_url)
            .await
            .unwrap_or_else(|e| panic!("Could not connect to db: '{}'!", e))
    }
}
