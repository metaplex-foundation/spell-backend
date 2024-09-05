use entities::api_key::ApiKeys;
use interfaces::api_key_provider::ApiKeysProvider;

#[derive(Debug, Clone)]
pub struct ApiKeysProviderFromMemory {
    api_keys: ApiKeys,
}

impl ApiKeysProviderFromMemory {
    pub fn new(api_keys: ApiKeys) -> Self {
        Self { api_keys }
    }
}

impl ApiKeysProvider for ApiKeysProviderFromMemory {
    fn get_api_keys(&self) -> ApiKeys {
        self.api_keys.clone()
    }
}

#[derive(Clone, Debug)]
pub struct ApiKeysProviderService<T: ApiKeysProvider>(T);

impl ApiKeysProviderService<ApiKeysProviderFromMemory> {
    pub fn from_memory(api_keys: ApiKeys) -> ApiKeysProviderService<ApiKeysProviderFromMemory> {
        Self(ApiKeysProviderFromMemory::new(api_keys))
    }
}

impl<T: ApiKeysProvider> ApiKeysProviderService<T> {
    pub fn get_api_keys(&self) -> ApiKeys {
        self.0.get_api_keys()
    }
}
