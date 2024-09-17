use crate::config::app_config::AppConfig;
use actix_web::{web, App, HttpServer};
use interfaces::asset_service::AssetService;
use io::Result;
use service::{asset_service_impl::AssetServiceImpl, converter::AssetDtoConverter};
use solana_integration::l1_service_solana::SolanaService;
use std::{io, sync::Arc};
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use tracing::info;
use tracing_actix_web::TracingLogger;
use util::{
    config::{DatabaseCfg, Settings},
    hd_wallet::HdWalletProducer,
};

pub async fn start_up_rest_server(app_config: AppConfig) -> Result<()> {
    let host_and_port = app_config.host_and_port();

    info!("Starting server");

    let state = init_app_state().await;
    let arc_state = Arc::new(state);

    HttpServer::new(move || {
        App::new()
            .configure(app_config.app_configuration())
            .app_data(web::Data::new(arc_state.clone()))
            .wrap(TracingLogger::default())
    })
    .bind(host_and_port)?
    .run()
    .await
}

#[derive(Clone)]
pub struct AppState {
    pub asset_service: Arc<dyn AssetService + Sync + Send>,
    pub asset_converter: AssetDtoConverter,
}

pub async fn init_app_state() -> AppState {
    let cfg = Settings::default().unwrap();
    create_app_state(cfg).await
}

pub async fn create_app_state(cfg: Settings) -> AppState {
    let l2_storage = {
        let DatabaseCfg { connection_url, min_connections, max_connections } = &cfg.database;
        let storage = L2StoragePg::new_from_url(connection_url, *min_connections, *max_connections)
            .await
            .unwrap();
        Arc::new(storage)
    };

    let obj_storage = {
        let s3_client = cfg.obj_storage.s3_client().await;
        let storage = S3Storage::new(
            &cfg.obj_storage.bucket_for_json_metadata,
            &cfg.obj_storage.bucket_for_binary_assets,
            Arc::new(s3_client),
        )
        .await;
        Arc::new(storage)
    };

    let solana_service = Arc::new(SolanaService::new(&cfg.solana.url));

    let hd_wallet_producer = HdWalletProducer::from_seed(cfg.master_key_seed());

    let asset_service = Arc::new(AssetServiceImpl {
        wallet_producer: hd_wallet_producer,
        derivation_sequence: l2_storage.clone(),
        l2_storage,
        asset_metadata_storage: obj_storage.clone(),
        blob_storage: obj_storage.clone(),
        s1_service: solana_service,
        metadata_server_base_url: cfg.rest_server.base_url.clone(),
    });

    let asset_converter = AssetDtoConverter { metadata_server_base_url: cfg.rest_server.base_url };

    AppState { asset_service, asset_converter }
}
