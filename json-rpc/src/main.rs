mod config;
mod endpoints;
mod logging;
mod rpc;

use crate::config::app_config::AppConfig;
use crate::logging::tracing::set_up_logging;
use crate::rpc::app::start_up_json_rpc;
use jsonrpc_http_server::tokio;
use std::io::Result;
use util::config::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let config_settings =
        Settings::default().unwrap_or_else(|e| panic!("Configuration failed: '{e}'!"));

    set_up_logging(&config_settings.json_rpc_server.log_level);

    start_up_json_rpc(AppConfig::from_settings(config_settings)).await
}
