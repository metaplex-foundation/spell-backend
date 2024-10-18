use actix_web::{App, HttpServer};
use interfaces::asset_service::AssetService;
use io::Result;
use service::{asset_service_impl::AssetServiceImpl, converter::AssetDtoConverter};
use solana_integration::l1_service_solana::SolanaService;
use std::{io, sync::Arc};
use storage::asset_storage_s3::S3Storage;
use storage::l2_storage_pg::L2StoragePg;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use util::{config::Settings, hd_wallet::HdWalletProducer};

use crate::rest::endpoints::l2_assets::{
    create_asset, get_asset, get_metadata, mint_status, mint_transaction, update_asset,
};
use crate::{
    rest::auth::ApiKeysProviderCtx,
    rest::endpoints::health_check::{health, secured_health},
};
use actix_web::web::{Data, ServiceConfig};

use super::endpoints::l2_assets::mint_transaction_async;

pub async fn start_up_rest_server(cfg: &Settings) -> Result<()> {
    info!("Starting server");

    let app_state = AppState::create_app_state(cfg).await;
    let cfg_clone = cfg.clone();

    HttpServer::new(move || {
        App::new()
            .configure(app_state.make_endpoints(&cfg_clone))
            .wrap(TracingLogger::default())
    })
    .bind((cfg.rest_server.host, cfg.rest_server.port))?
    .run()
    .await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub asset_service: Arc<dyn AssetService + Sync + Send>,
    pub asset_converter: AssetDtoConverter,
}

impl AppState {
    pub async fn create_app_state(cfg: &Settings) -> AppState {
        let l2_storage = {
            let storage = L2StoragePg::new_from_cfg(&cfg.database)
                .await
                .unwrap_or_else(|e| panic!("Failed to init 'L2Storage' cause: {e}"));
            Arc::new(storage)
        };

        let obj_storage = Arc::new(
            S3Storage::new(
                &cfg.obj_storage.bucket_for_json_metadata,
                &cfg.obj_storage.bucket_for_binary_assets,
                Arc::new(cfg.obj_storage.s3_client().await),
            )
            .await,
        );

        let solana_service = Arc::new(SolanaService::new(&cfg.solana.url));

        let hd_wallet_producer = HdWalletProducer::from_seed(cfg.master_key_seed());

        let asset_service = Arc::new(AssetServiceImpl {
            wallet_producer: hd_wallet_producer,
            derivation_sequence: l2_storage.clone(),
            l2_storage,
            asset_metadata_storage: obj_storage.clone(),
            blob_storage: obj_storage.clone(),
            l1_service: solana_service,
            metadata_server_base_url: cfg.rest_server.base_url.clone(),
        });

        asset_service
            .process_minting_assets_on_startup()
            .await
            .unwrap_or_else(|e| error!("Failed to start 'process_minting_assets'; Cause: {e}."));

        let asset_converter = AssetDtoConverter { metadata_server_base_url: cfg.rest_server.base_url.clone() };

        AppState { asset_service, asset_converter }
    }

    pub fn make_endpoints(&self, cfg: &Settings) -> impl FnOnce(&mut ServiceConfig) + '_ {
        let api_keys_provider_ctx = ApiKeysProviderCtx { api_keys: cfg.rest_api_keys() };
        let app_state = self.clone();

        |serv_cfg: &mut ServiceConfig| {
            serv_cfg
                .app_data(Data::new(api_keys_provider_ctx))
                .app_data(Data::new(app_state))
                .service(health)
                .service(create_asset)
                .service(update_asset)
                .service(get_asset)
                .service(get_metadata)
                .service(mint_transaction)
                .service(mint_status)
                .service(mint_transaction_async)
                .service(secured_health);
        }
    }
}
