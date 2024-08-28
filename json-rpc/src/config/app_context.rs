use sqlx::PgPool;
use std::sync::Arc;

pub type ArcedAppCtx = Arc<AppCtx>;

#[derive(Clone, Debug)]
pub struct AppCtx {
    connection_pool: PgPool,
}

impl AppCtx {
    pub fn new(connection_pool: PgPool) -> Self {
        Self { connection_pool }
    }

    pub fn get_connection_pool(&self) -> &PgPool {
        &self.connection_pool
    }

    pub fn arced(self) -> ArcedAppCtx {
        Arc::new(self)
    }
}
