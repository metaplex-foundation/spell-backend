#![allow(dead_code)]
use actix_http::body::MessageBody;
use actix_http::Request;
use actix_web::dev::ServiceResponse;
use actix_web::{test, App, HttpServer};
use entities::dto::Asset;
use rest_server::rest::web_app::AppState;
use setup::TestEnvironment;
use std::str::FromStr;
use tracing::metadata::LevelFilter;
use tracing_actix_web::TracingLogger;

pub async fn init_web_app(
    t_env: &TestEnvironment,
) -> impl actix_web::dev::Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
    let cfg = t_env.make_test_cfg().await;

    let app_state = AppState::create_app_state(&cfg).await;
    let log_level = cfg.rest_server.log_level.clone();
    let log_level = LevelFilter::from_str(&log_level).unwrap_or_else(|_| panic!("Invalid 'log_level' {log_level}"));

    maybe_init_logger(log_level);

    let app = test::init_service(App::new().configure(app_state.make_endpoints(&cfg))).await;

    app
}

pub async fn init_app_as_web_server(t_env: &TestEnvironment) {
    let cfg = t_env.make_test_cfg().await;

    let app_state = AppState::create_app_state(&cfg).await;
    let host_and_port = (cfg.rest_server.host, cfg.rest_server.port);
    let log_level = cfg.rest_server.log_level.clone();
    let log_level = LevelFilter::from_str(&log_level).unwrap_or_else(|_| panic!("Invalid 'log_level' {log_level}"));

    maybe_init_logger(log_level);

    let app = HttpServer::new(move || {
        App::new()
            .configure(app_state.make_endpoints(&cfg))
            .wrap(TracingLogger::default())
    })
    .bind(host_and_port)
    .unwrap_or_else(|e| panic!("Failed to start REST server cause: {e:?}"))
    .run();

    actix_web::rt::spawn(app);
}

fn maybe_init_logger(level_filter: LevelFilter) {
    if std::env::args()
        .collect::<Vec<String>>()
        .contains(&String::from("--show-output"))
    {
        // let env_filter = EnvFilter::new("rest-server=info,sqlx=info,tokio=debug");
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_line_number(true)
            .with_max_level(level_filter)
            // .with_env_filter(env_filter)
            .pretty()
            .init();
    }
}

pub fn extract_asset_from_response(serv_resp: ServiceResponse) -> Asset {
    let resp_text = String::from_utf8(serv_resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
    serde_json::from_str(&resp_text).unwrap()
}

pub async fn extract_asset_from_reqwest_response(serv_resp: reqwest::Response) -> Asset {
    serv_resp.json().await.unwrap()
}

pub async fn extract_string_from_reqwest_response(serv_resp: reqwest::Response) -> String {
    serv_resp.text().await.unwrap()
}
