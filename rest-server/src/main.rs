use crate::config::app_config::AppConfig;
use crate::logging::tracing::set_up_logging;
use crate::web::app::start_up_rest_server;
use std::io::Result;
use util::config::Settings;

pub mod auth;
pub mod config;
pub mod endpoints;
pub mod logging;
pub mod web;

#[actix_web::main]
async fn main() -> Result<()> {
    let config_settings =
        Settings::default().unwrap_or_else(|e| panic!("Configuration failed: '{e}'!"));

    set_up_logging(&config_settings.http_server.log_level);

    start_up_rest_server(AppConfig::from_settings(config_settings).await).await
}
