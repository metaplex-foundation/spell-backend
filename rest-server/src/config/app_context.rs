use service::api_key_provider_service::{ApiKeysProviderFromMemory, ApiKeysProviderService};

pub type ApiKeysProviderCtx = ApiKeysProviderService<ApiKeysProviderFromMemory>;
