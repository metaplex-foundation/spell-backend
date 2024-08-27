use crate::infrastructure::auth::types::{ApiKeysProviderFromMemory, ApiKeysProviderService};

pub type ApiKeysProviderCtx = ApiKeysProviderService<ApiKeysProviderFromMemory>;
