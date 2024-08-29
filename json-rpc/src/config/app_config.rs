use crate::config::app_context::AppCtx;
use crate::config::method_builder::RpcMethodRegistrar;
use crate::endpoints::get_nft::{
    get_asset, get_asset_batch, get_asset_by_creator, get_asset_by_owner,
};
use crate::endpoints::health_check::health;
use jsonrpc_core::IoHandler;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub struct AppConfig {
    host_and_port: (Ipv4Addr, u16),
    connection_pool: PgPool,
}

impl AppConfig {
    const DEFAULT_CONNECTION_POOL_SIZE: u32 = 10;
    const DEFAULT_PORT: u16 = 3030;

    pub async fn new() -> Self {
        Self {
            host_and_port: Self::read_host_and_port(),
            connection_pool: Self::create_connection_pool().await,
        }
    }

    pub fn host(&self) -> Ipv4Addr {
        self.host_and_port.0
    }

    pub fn port(&self) -> u16 {
        self.host_and_port.1
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

    fn read_host_and_port() -> (Ipv4Addr, u16) {
        (
            dotenv::var("JSON_RPC_HOST")
                .inspect(|_| error!("Failed to read 'JSON_RPC_HOST'!"))
                .ok()
                .and_then(Self::parse)
                .unwrap_or(Ipv4Addr::LOCALHOST),
            dotenv::var("JSON_RPC_PORT")
                .inspect_err(|_| error!("Failed to read 'JSON_RPC_PORT'!"))
                .ok()
                .and_then(Self::parse)
                .unwrap_or(Self::DEFAULT_PORT),
        )
    }

    fn read_database_url() -> String {
        dotenv::var("DATABASE_URL").unwrap_or_else(|_| panic!("Failed to read 'DATABASE_URL'!"))
    }

    fn read_connection_pool_size() -> u32 {
        dotenv::var("CONNECTION_POOL_SIZE")
            .unwrap_or_else(|_| panic!("Failed to read 'CONNECTION_POOL_SIZE'!"))
            .parse()
            .unwrap_or(Self::DEFAULT_CONNECTION_POOL_SIZE)
    }

    async fn create_connection_pool() -> PgPool {
        let (size, db_url) = (Self::read_connection_pool_size(), Self::read_database_url());
        info!("Creating connection pool from: '{db_url}', with size of {size} connections.");
        PgPoolOptions::new()
            .max_connections(size)
            .connect(&db_url)
            .await
            .unwrap_or_else(|e| panic!("Could not connect to db: '{}'", e))
    }

    fn parse<T: FromStr, I: Into<String>>(value: I) -> Option<T> {
        let value = value.into();
        value
            .parse::<T>()
            .inspect_err(|_| error!("Failed to parse '{value}'!"))
            .ok()
    }
}
