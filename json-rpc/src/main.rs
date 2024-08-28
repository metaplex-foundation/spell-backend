mod config;
mod endpoints;
mod logging;
mod rpc;

use crate::config::app_config::AppConfig;
use crate::logging::tracing::set_up_logging;
use crate::rpc::app::start_up_json_rpc;
use jsonrpc_http_server::tokio;
use std::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv()
        .inspect_err(|_| eprintln!("Cannot find '.env' file!"))
        .or_else(|_| dotenv::from_filename(".env.example"))
        .unwrap_or_else(|_| panic!("Failed to read '{:?}' file!", ".env.example"));

    set_up_logging();

    start_up_json_rpc(AppConfig::new().await)
}
