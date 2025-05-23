use crate::setup::app_setup::AppSetup;
use crate::setup::types::MetadataUriCreator;

use interfaces::asset_service::AssetService;
use service::asset_service_impl::AssetServiceImpl;
use solana_integration::l1_service_solana::SolanaService;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    ConnectOptions, PgPool,
};
use std::sync::Arc;
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use tracing::log::LevelFilter;
use tracing::{error, info};
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

    pub async fn new(app_config: &AppSetup) -> Self {
        let metadata_uri_base = MetadataUriCreator::new(format!(
            "{}:{}",
            app_config.settings.rest_server.host, app_config.settings.rest_server.port,
        ));

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

        let solana_service = Arc::new(SolanaService::new(&app_config.settings.solana.url));

        let l2_storage = Arc::new(L2StoragePg::new_from_pool(connection_pool));
        let s3_storage = Arc::new(s3_storage);

        let asset_service = Arc::new(AssetServiceImpl {
            wallet_producer,
            derivation_sequence: l2_storage.clone(),
            l2_storage: l2_storage.clone(),
            asset_metadata_storage: s3_storage.clone(),
            blob_storage: s3_storage.clone(),
            l1_service: solana_service,
            metadata_server_base_url: app_config.settings.rest_server.base_url.clone(),
        });

        asset_service
            .process_minting_assets_on_startup()
            .await
            .unwrap_or_else(|e| error!("Failed to start 'process_minting_assets'; Cause: {e}."));

        Self { asset_service, metadata_uri_base }
    }

    async fn create_connection_pool(db_url: &str, max_size_pool: u32, min_size_pool: u32) -> PgPool {
        info!(
            "Creating connection pool from: '{}', with max_size: '{}', and min_size: '{}' connections.",
            db_url, max_size_pool, min_size_pool,
        );
        let mut options = db_url.parse::<PgConnectOptions>().unwrap();
        options.log_statements(LevelFilter::Off);
        options.log_slow_statements(LevelFilter::Off, std::time::Duration::from_secs(100));
        options = options.extra_float_digits(None); // needed for Pgbouncer

        PgPoolOptions::new()
            .max_connections(max_size_pool)
            .min_connections(min_size_pool)
            .connect_with(options)
            .await
            .unwrap_or_else(|e| panic!("Could not connect to db: '{}'!", e))
    }
}
