use service::asset_service_impl::AssetServiceImpl;
use std::sync::Arc;

pub type ArcedAppCtx = Arc<AppCtx>;

#[derive(Clone)]
pub struct AppCtx {
    pub asset_service: AssetServiceImpl,
}

impl AppCtx {
    pub fn new(asset_service: AssetServiceImpl) -> Self {
        Self { asset_service }
    }

    pub fn arced(self) -> ArcedAppCtx {
        Arc::new(self)
    }
}
