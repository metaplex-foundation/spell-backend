use crate::infrastructure::config::app_config::AppConfig;
use crate::infrastructure::logging::tracing::set_up_logging;
use std::io::Result;
use crate::infrastructure::web::app::start_up;

mod infrastructure;

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
