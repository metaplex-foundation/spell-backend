use actix_web::http::header::HeaderValue;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

pub struct ApiKeyExtractor;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct ApiKey(String);

impl ApiKey {
    pub fn new<T: Into<String>>(value: T) -> Self {
        Self(value.into())
    }
}

impl Debug for ApiKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiKey").field("0", &"********").finish()
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct ApiKeys(Vec<ApiKey>);

impl ApiKeys {
    pub fn new<T: Iterator<Item = ApiKey>>(keys: T) -> Self {
        Self(keys.collect())
    }

    pub fn contains_api_key(&self, provided_api_key: &HeaderValue) -> bool {
        self.0.iter().any(|api_key| api_key.0.eq(provided_api_key))
    }
}

impl From<Vec<ApiKey>> for ApiKeys {
    fn from(value: Vec<ApiKey>) -> Self {
        Self(value)
    }
}

pub trait ApiKeysProvider {
    fn get_api_keys(&self) -> ApiKeys;
}

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
