use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Username(String);

impl Username {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self(name.into())
    }

    #[allow(dead_code)]
    pub fn inner(&self) -> String {
        self.0.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
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
pub struct ApiKeys(HashMap<ApiKey, Username>);

impl ApiKeys {
    pub fn contains_api_key_then_get_username(&self, provided_api_key: &str) -> Option<Username> {
        let provided_api_key = ApiKey::new(provided_api_key);
        self.0
            .contains_key(&provided_api_key)
            .then(|| self.0.get(&provided_api_key))
            .flatten()
            .cloned()
    }
}

impl From<HashMap<ApiKey, Username>> for ApiKeys {
    fn from(value: HashMap<ApiKey, Username>) -> Self {
        Self(value)
    }
}
