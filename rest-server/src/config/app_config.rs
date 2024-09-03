use crate::config::app_context::ApiKeysProviderCtx;
use crate::endpoints::health_check::{health, secured_health};
use crate::endpoints::l2_assets::{create_asset, get_asset, get_metadata, update_asset};
use actix_web::web::{Data, ServiceConfig};
use entities::api_key::{ApiKey, ApiKeys};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::env::var;
use std::net::Ipv4Addr;
use tracing::info;
use util::config::Settings;

#[derive(Clone, Debug)]
pub struct AppConfig {
    host_and_port: (Ipv4Addr, u16),
    api_keys: ApiKeys,
    #[allow(dead_code)]
    connection_pool: PgPool,
}

impl AppConfig {
    pub const API_KEY_HEADER: &'static str = "x-api-key";
    const API_KEY_SEPARATOR: char = ',';
    const API_KEY_ENV_NAME: &'static str = "app__API_KEY";

    pub async fn from_settings(settings: Settings) -> Self {
        let api_keys = settings
            .is_production_profile()
            .then(Self::read_api_keys_from_env)
            .unwrap_or_else(Self::mocked_api_keys);

        let host_and_port = (settings.http_server.host, settings.http_server.port);

        let connection_pool = Self::create_connection_pool(
            settings.database.connection_url,
            settings.database.max_connections,
            settings.database.min_connections,
        )
        .await;

        Self { host_and_port, api_keys, connection_pool }
    }

    pub fn host_and_port(&self) -> (Ipv4Addr, u16) {
        self.host_and_port
    }

    pub fn app_configuration(&self) -> impl FnOnce(&mut ServiceConfig) + '_ {
        |cfg: &mut ServiceConfig| {
            let api_keys_provider_ctx = ApiKeysProviderCtx::from_memory(self.api_keys.clone());

            cfg.app_data(Data::new(api_keys_provider_ctx))
                .service(health)
                .service(create_asset)
                .service(update_asset)
                .service(get_asset)
                .service(get_metadata)
                .service(secured_health);
        }
    }

    fn read_api_keys_from_env() -> ApiKeys {
        var(Self::API_KEY_ENV_NAME)
            .unwrap_or_else(|_| panic!("No '{}' was provided.", Self::API_KEY_ENV_NAME))
            .split(Self::API_KEY_SEPARATOR)
            .map(ApiKey::new)
            .collect::<Vec<ApiKey>>()
            .into()
    }

    fn mocked_api_keys() -> ApiKeys {
        info!("Using mocked api keys for local development: 111, 222, 333");
        "111,222,333"
            .split(Self::API_KEY_SEPARATOR)
            .map(ApiKey::new)
            .collect::<Vec<ApiKey>>()
            .into()
    }

    async fn create_connection_pool(db_url: String, max_size_pool: u32, min_size_pool: u32) -> PgPool {
        info!(
            "Creating connection pool from: '{}', with max_size: '{}', and min_size: '{}' connections.",
            db_url, max_size_pool, min_size_pool,
        );
        PgPoolOptions::new()
            .max_connections(max_size_pool)
            .min_connections(min_size_pool)
            .connect(&db_url)
            .await
            .unwrap_or_else(|e| panic!("Could not connect to db: '{}'", e))
    }
}
