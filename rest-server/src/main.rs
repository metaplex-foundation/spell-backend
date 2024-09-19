mod logging;
mod rest;

use crate::logging::set_up_logging;
use std::io::Result;
use util::config::Settings;

#[actix_web::main]
async fn main() -> Result<()> {
    let cfg = Settings::default().unwrap_or_else(|e| panic!("Configuration failed: '{e}'!"));

    set_up_logging(&cfg.rest_server.log_level);

    rest::web_app::start_up_rest_server(&cfg).await
}
