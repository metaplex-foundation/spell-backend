use crate::config::app_config::AppConfig;
use actix_web::{App, HttpServer};
use std::io::Result;
use tracing::info;
use tracing_actix_web::TracingLogger;

pub async fn start_up_rest_server(app_config: AppConfig) -> Result<()> {
    let host_and_port = app_config.host_and_port();

    info!("Starting server");

    HttpServer::new(move || {
        App::new()
            .configure(app_config.app_configuration())
            .wrap(TracingLogger::default())
    })
    .bind(host_and_port)?
    .run()
    .await
}
