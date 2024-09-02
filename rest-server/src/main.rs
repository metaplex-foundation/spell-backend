mod auth;
mod config;
mod endpoints;
mod logging;
mod web;

use crate::config::app_config::AppConfig;
use crate::logging::tracing::set_up_logging;
use crate::web::app::start_up;
use std::io::Result;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv()
        .inspect_err(|_| eprintln!("Cannot find '.env' file!"))
        .or_else(|_| dotenv::from_filename(".env.example"))
        .unwrap_or_else(|_| panic!("Failed to read '{:?}' file!", ".env.example"));

    set_up_logging();

    let app_config = AppConfig::new().await;

    start_up(app_config).await
}
