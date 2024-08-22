use crate::infrastructure::auth::types::ApiKeyExtractor;
use actix_web::{get, HttpResponse, Responder};

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("Server is ok.")
}

#[get("/secured_health")]
pub async fn secured_health(_: ApiKeyExtractor) -> impl Responder {
    HttpResponse::Ok().body("Server is ok.")
}

// fn api_key_check(guard_context: &GuardContext<'_>) -> bool {
//     match guard_context.app_data::<Data<ApiKeysProviderCtx>>() {
//         Some(ctx) => {
//             let api_keys = ctx.get_api_keys();
//             guard_context
//                 .head()
//                 .headers
//                 .get("x-api-key")
//                 .is_some_and(|provided_api_key| api_keys.contains_api_key(provided_api_key))
//         },
//         None => false
//     }
// }
