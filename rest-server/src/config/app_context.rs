use crate::auth::types::{ApiKeysProviderFromMemory, ApiKeysProviderService};

pub type ApiKeysProviderCtx = ApiKeysProviderService<ApiKeysProviderFromMemory>;
