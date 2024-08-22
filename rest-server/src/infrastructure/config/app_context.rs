use crate::infrastructure::auth::types::{
    ApiKeys, ApiKeysProviderFromMemory, ApiKeysProviderService,
};

#[derive(Clone, Debug)]
pub struct ApiKeysProviderCtx {
    provider: ApiKeysProviderService<ApiKeysProviderFromMemory>,
}

impl ApiKeysProviderCtx {
    pub fn get_api_keys(&self) -> ApiKeys {
        self.provider.get_api_keys()
    }

    pub fn from_memory(api_keys: ApiKeys) -> ApiKeysProviderCtx {
        Self {
            provider: ApiKeysProviderService::<ApiKeysProviderFromMemory>::from_memory(api_keys),
        }
    }
}
