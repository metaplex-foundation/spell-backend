use crate::config::app_config::AppConfig;
use crate::config::types::MetadataUriCreator;

use interfaces::asset_service::AssetService;
use service::asset_service_impl::AssetServiceImpl;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use tracing::info;
use util::hd_wallet::HdWalletProducer;

pub type ArcedAppCtx = Arc<AppCtx>;

#[derive(Clone)]
pub struct AppCtx {
    pub asset_service: Arc<dyn AssetService + Sync + Send>,
    pub metadata_uri_base: MetadataUriCreator,
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
        )
        .await;

        let s3_storage = S3Storage::new(
            &app_config.settings.obj_storage.bucket_for_json_metadata,
            &app_config.settings.obj_storage.bucket_for_binary_assets,
            Arc::new(app_config.settings.obj_storage.s3_client().await),
        )
        .await;

        info!("Connecting to S3 Storage: '{:?}'", app_config.settings.obj_storage.endpoint);
        info!(
            "Using S3 Storage bucket for assets: '{:?}'",
            app_config.settings.obj_storage.bucket_for_binary_assets
        );
        info!(
            "Using S3 Storage bucket for metadata: '{:?}'",
            app_config.settings.obj_storage.bucket_for_json_metadata
        );

        let wallet_producer = HdWalletProducer::from_seed(app_config.settings.master_key_seed());

        let l2_storage = Arc::new(L2StoragePg::new_from_pool(connection_pool));
        let s3_storage = Arc::new(s3_storage);

        Self {
            asset_service: Arc::new(AssetServiceImpl {
                wallet_producer,
                derivation_sequence: l2_storage.clone(),
                l2_storage: l2_storage.clone(),
                asset_metadata_storage: s3_storage.clone(),
                blob_storage: s3_storage.clone(),
            }),
            metadata_uri_base: MetadataUriCreator::new(app_config.socket_addr().to_string()),
        }
    }

    async fn create_connection_pool(db_url: &str, max_size_pool: u32, min_size_pool: u32) -> PgPool {
        info!(
            "Creating connection pool from: '{}', with max_size: '{}', and min_size: '{}' connections.",
            db_url, max_size_pool, min_size_pool,
        );
        PgPoolOptions::new()
            .max_connections(max_size_pool)
            .min_connections(min_size_pool)
            .connect(db_url)
            .await
            .unwrap_or_else(|e| panic!("Could not connect to db: '{}'!", e))
    }
}
