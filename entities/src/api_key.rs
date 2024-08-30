use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Formatter};

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
    #[allow(dead_code)]
    pub fn new<T: Iterator<Item = ApiKey>>(keys: T) -> Self {
        Self(keys.collect())
    }

    pub fn contains_api_key(&self, provided_api_key: &impl PartialEq<String>) -> bool {
        self.0.iter().any(|api_key| provided_api_key.eq(&api_key.0))
    }
}

impl From<Vec<ApiKey>> for ApiKeys {
    fn from(value: Vec<ApiKey>) -> Self {
        Self(value)
    }
}
