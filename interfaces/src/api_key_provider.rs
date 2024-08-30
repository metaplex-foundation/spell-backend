use entities::api_key::ApiKeys;

pub trait ApiKeysProvider {
    fn get_api_keys(&self) -> ApiKeys;
}
