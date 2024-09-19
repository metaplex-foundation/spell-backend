use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::Data;
use actix_web::{dev::Payload, Error as ActixError, FromRequest, HttpRequest};
use entities::api_key::{ApiKeys, Username};
use futures::future::{ready, Ready};

pub const API_KEY_HEADER: &'static str = "x-api-key";

/// For now just store api keys in memory
pub struct ApiKeysProviderCtx {
    pub api_keys: ApiKeys,
}

#[allow(dead_code)]
pub struct ApiKeyExtractor {
    authorized_user: Username,
}

impl ApiKeyExtractor {
    fn new(user: Username) -> Self {
        Self { authorized_user: user }
    }
}

impl FromRequest for ApiKeyExtractor {
    type Error = ActixError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(match req.app_data::<Data<ApiKeysProviderCtx>>() {
            Some(api_keys_provider) => {
                let Some(provided_api_key) = req.head().headers.get(API_KEY_HEADER) else {
                    return ready(Err(ErrorBadRequest("No header found.")));
                };

                let Some(provided_api_key) = provided_api_key.to_str().ok() else {
                    return ready(Err(ErrorBadRequest("Invalid header string.")));
                };

                match api_keys_provider
                    .api_keys
                    .contains_api_key_then_get_username(provided_api_key)
                {
                    Some(user) => Ok(ApiKeyExtractor::new(user)),
                    None => Err(ErrorUnauthorized("Invalid API key.")),
                }
            }
            None => Err(ErrorInternalServerError("Couldn't retrieve 'ApiKeysProviderCtx'!")),
        })
    }
}
