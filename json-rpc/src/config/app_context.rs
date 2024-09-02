use service::asset_service_impl::AssetServiceImpl;
use std::sync::Arc;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use util::hd_wallet::HdWalletProducer;
use crate::config::app_config::AppConfig;

pub type ArcedAppCtx = Arc<AppCtx>;

#[derive(Clone)]
pub struct AppCtx {
    pub asset_service: AssetServiceImpl,
}

impl AppCtx {
    pub fn arced(self) -> ArcedAppCtx {
        Arc::new(self)
    }

    pub async fn new(app_config: &AppConfig) -> Self {
        let connection_pool = Self::create_connection_pool(
            &app_config.settings.database.connection_url,
            app_config.settings.database.max_connections,
            app_config.settings.database.min_connections,
        ).await;

        let s3_storage = match app_config.settings.is_production_profile() {
            true => unimplemented!(),
            false => S3Storage::mocked().await,
        };

        let (master_pubkey, wallet_producer) = app_config
            .settings
            .is_not_production_profile()
            .then(|| (Default::default(), HdWalletProducer::mocked()))
            .unwrap_or_else(|| unimplemented!());

        let l2_storage = Arc::new(L2StoragePg::new_from_pool(connection_pool));
        let s3_storage = Arc::new(s3_storage);


        Self {
            asset_service: AssetServiceImpl {
                master_pubkey,
                wallet_producer,
                derivation_sequence: l2_storage.clone(),
                l2_storage: l2_storage.clone(),
                asset_metadata_storage: s3_storage.clone(),
                blob_storage: s3_storage.clone(),
            },
        }
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
