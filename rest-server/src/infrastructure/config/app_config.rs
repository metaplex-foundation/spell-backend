use crate::infrastructure::auth::types::{ApiKey, ApiKeys};
use crate::infrastructure::config::app_context::ApiKeysProviderCtx;
use crate::infrastructure::endpoints::health_check::{health, secured_health};
use actix_web::web::{Data, ServiceConfig};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::{error, info};

type ConnectionPool = Pool<Postgres>;

#[derive(Clone, Debug)]
pub struct AppConfig {
    host_and_port: (Ipv4Addr, u16),
    api_keys: ApiKeys,
    #[allow(dead_code)]
    connection_pool: ConnectionPool,
}

impl AppConfig {
    const DEFAULT_CONNECTION_POOP_SIZE: u32 = 10;

    pub async fn new() -> Self {
        Self {
            host_and_port: Self::read_host_and_port(),
            api_keys: Self::read_api_keys_from_env(),
            connection_pool: Self::create_connection_pool().await,
        }
    }

    pub fn host_and_port(&self) -> (Ipv4Addr, u16) {
        self.host_and_port
    }

    pub fn app_configuration(&self) -> impl FnOnce(&mut ServiceConfig) + '_ {
        |cfg: &mut ServiceConfig| {
            let api_keys_provider_ctx = ApiKeysProviderCtx::from_memory(self.api_keys.clone());

            cfg.app_data(Data::new(api_keys_provider_ctx))
                .service(health)
                .service(secured_health);
        }
    }

    fn read_host_and_port() -> (Ipv4Addr, u16) {
        (
            dotenv::var("HOST")
                .inspect(|_| error!("Failed to read 'HOST'!"))
                .ok()
                .and_then(Self::parse)
                .unwrap_or(Ipv4Addr::new(127, 0, 0, 1)),
            dotenv::var("PORT")
                .inspect_err(|_| error!("Failed to read 'PORT'!"))
                .ok()
                .and_then(Self::parse)
                .unwrap_or(8080),
        )
    }

    fn read_database_url() -> String {
        dotenv::var("DATABASE_URL").unwrap_or_else(|_| panic!("Failed to read 'DATABASE_URL'!"))
    }

    fn connection_pool_size() -> u32 {
        dotenv::var("CONNECTION_POOP_SIZE")
            .unwrap_or_else(|_| panic!("Failed to read 'CONNECTION_POOP_SIZE'!"))
            .parse()
            .unwrap_or(Self::DEFAULT_CONNECTION_POOP_SIZE)
    }

    fn read_api_keys_from_env() -> ApiKeys {
        dotenv::var("API_KEYS")
            .expect("No 'API_KEYS' was provided.")
            .split(',')
            .map(ApiKey::new)
            .collect::<Vec<ApiKey>>()
            .into()
    }

    async fn create_connection_pool() -> ConnectionPool {
        let (size, db_url) = (Self::connection_pool_size(), Self::read_database_url());
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
