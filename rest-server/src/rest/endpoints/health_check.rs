use crate::rest::auth::ApiKeyExtractor;
use actix_web::{get, HttpResponse, Responder};

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().body("Server is ok.")
}

#[get("/secured_health")]
pub async fn secured_health(_: ApiKeyExtractor) -> impl Responder {
    HttpResponse::Ok().body("Server is ok.")
}
