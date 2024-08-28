use crate::config::app_config::AppConfig;
use jsonrpc_http_server::ServerBuilder;
use std::io::Result;
use std::net::{IpAddr, SocketAddr};
use tracing::info;

pub fn start_up_json_rpc(app_config: AppConfig) -> Result<()> {
    let bind_address = SocketAddr::new(IpAddr::from(app_config.host()), app_config.port());
    let handler = app_config.register_rpc_methods();

    info!("Starting Json RPC");

    Ok(ServerBuilder::new(handler)
        .health_api(("/health", "health"))
        .start_http(&bind_address)?
        .wait())
}
