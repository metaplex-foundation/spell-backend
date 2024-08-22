use crate::infrastructure::auth::types::ApiKeyExtractor;
use crate::infrastructure::config::app_context::ApiKeysProviderCtx;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

impl FromRequest for ApiKeyExtractor {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            match req
                .app_data::<Data<ApiKeysProviderCtx>>()
                .map(|ctx| ctx.get_api_keys())
            {
                Some(api_keys) => {
                    let Some(provided_api_key) = req.head().headers.get("x-api-key") else {
                        return ready(Err(ErrorBadRequest("No header found.")));
                    };

                    match api_keys.contains_api_key(provided_api_key) {
                        true => Ok(ApiKeyExtractor),
                        false => Err(ErrorUnauthorized("Invalid API key.")),
                    }
                }
                None => Err(ErrorInternalServerError(
                    "Couldn't retrieve 'ApiKeysProviderCtx'!",
                )),
            },
        )
    }
}
