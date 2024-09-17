use std::collections::HashMap;
use std::sync::Arc;

use actix_http::Request;
use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::{test, web, App};
use entities::api_key::ApiKeys;
use entities::api_key::{ApiKey, Username};
use entities::rpc_asset_models::Asset;
use rest_server::endpoints::l2_assets::mint_transaction;
use rest_server::{
    config::app_context::ApiKeysProviderCtx,
    endpoints::l2_assets::{create_asset, get_asset, get_metadata, update_asset},
    web::app::create_app_state,
};

use setup::TestEnvironment;

#[allow(dead_code)]
pub async fn init_web_app(
    t_env: &TestEnvironment,
) -> impl actix_web::dev::Service<Request, Response = ServiceResponse, Error = actix_web::Error> {
    let cfg = t_env.make_test_cfg().await;
    let state = Arc::new(create_app_state(cfg).await);

    let api_keys_provider_ctx =
        ApiKeysProviderCtx::from_memory(ApiKeys::from(HashMap::from([(ApiKey::new("111"), Username::new(""))])));

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(api_keys_provider_ctx))
            .app_data(web::Data::new(state))
            .service(create_asset)
            .service(update_asset)
            .service(get_asset)
            .service(get_metadata)
            .service(mint_transaction),
    )
    .await;

    app
}

#[allow(dead_code)]
pub fn extract_asset_from_response(serv_resp: ServiceResponse) -> Asset {
    let resp_text = String::from_utf8(serv_resp.into_body().try_into_bytes().unwrap().to_vec()).unwrap();
    serde_json::from_str(&resp_text).unwrap()
}
