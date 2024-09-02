use crate::config::app_config::AppConfig;
use jsonrpc_http_server::ServerBuilder;
use std::io::Result;
use tracing::{error, info};

pub async fn start_up_json_rpc(app_config: AppConfig) -> Result<()> {
    let bind_address = app_config.socket_addr();
    let handler = app_config.register_rpc_methods().await;

    info!("Starting Json RPC using: '{bind_address}'.");

    ServerBuilder::new(handler)
        .health_api(("/health", "health"))
        .start_http(&bind_address)
        .inspect_err(|e| error!("Failed to start http: {e}."))?
        .wait();

    Ok(())
}
