use crate::config::app_context::AppCtx;
use crate::config::method_registrar::RpcMethodRegistrar;
use crate::endpoints::get_nft::{get_asset, get_asset_batch, get_asset_by_creator, get_asset_by_owner};
use crate::endpoints::health_check::health;
use jsonrpc_core::IoHandler;
use std::net::SocketAddr;
use util::config::Settings;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub settings: Settings,
}

impl AppConfig {
    pub fn from_settings(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn register_rpc_methods(&self) -> IoHandler {
        RpcMethodRegistrar::new(AppCtx::new(self).await.arced())
            .method_without_ctx_and_params(health)
            .method(get_asset)
            .method(get_asset_batch)
            .method(get_asset_by_owner)
            .method(get_asset_by_creator)
            .finish()
    }

    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.settings.json_rpc_server.host.into(), self.settings.json_rpc_server.port)
    }
}
