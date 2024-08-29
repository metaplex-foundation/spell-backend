use crate::config::app_context::AppCtx;
use crate::config::method_builder::RpcMethodRegistrar;
use crate::endpoints::get_nft::{
    get_asset, get_asset_batch, get_asset_by_creator, get_asset_by_owner,
};
use crate::endpoints::health_check::health;
use jsonrpc_core::IoHandler;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing::info;
use util::config::Settings;

#[derive(Clone, Debug)]
pub struct AppConfig {
    socket_addr: SocketAddr,
    connection_pool: PgPool,
}

impl AppConfig {
    pub async fn from_settings(settings: Settings) -> Self {
        let socket_addr = SocketAddr::new(
            settings.json_rpc_server.host.into(),
            settings.json_rpc_server.port,
        );
        let connection_pool = Self::create_connection_pool(
            settings.database.connection_url,
            settings.database.max_connections,
            settings.database.min_connections,
        )
        .await;

        Self {
            socket_addr,
            connection_pool,
        }
    }

    pub fn register_rpc_methods(self) -> IoHandler {
        RpcMethodRegistrar::new(AppCtx::new(self.connection_pool).arced())
            .method_without_ctx_and_params(health)
            .method(get_asset)
            .method(get_asset_batch)
            .method(get_asset_by_owner)
            .method(get_asset_by_creator)
            .finish()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        self.socket_addr
    }

    async fn create_connection_pool(
        db_url: String,
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
