mod endpoints;
mod logging;
mod rpc;
mod setup;

use crate::logging::tracing::set_up_logging;
use crate::rpc::app::start_up_json_rpc;
use crate::setup::app_setup::AppSetup;
use jsonrpc_http_server::tokio;
use std::io::Result;
use util::config::Settings;

#[tokio::main]
async fn main() -> Result<()> {
    let config_settings = Settings::default().unwrap_or_else(|e| panic!("Configuration failed: '{e}'!"));

    set_up_logging(&config_settings.json_rpc_server.log_level);

    start_up_json_rpc(AppSetup::from_settings(config_settings)).await
}
