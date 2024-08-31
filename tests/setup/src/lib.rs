//! This crate contains utilities for preparing an environment
//! for integration tests, including:
//! * docker containers

use pg::PgContainer;
use s3::S3Container;
use util::config::{DatabaseCfg, ObjStorageCfg};

pub mod data_gen;
pub mod pg;
pub mod s3;

pub struct TestEnvironment {
    pub pg: Option<PgContainer>,
    pub s3: Option<S3Container>,
}

#[derive(Default)]
pub struct TestEnvironmentCfg {
    pub pg: bool,
    pub s3: bool,
}

impl TestEnvironmentCfg {
    pub fn with_all() -> Self {
        TestEnvironmentCfg { pg: true, s3: true }
    }
    pub fn with_pg(mut self) -> Self {
        self.pg = true;
        self
    }
    pub fn with_s3(mut self) -> Self {
        self.s3 = true;
        self
    }
    pub async fn start(self) -> TestEnvironment {
        TestEnvironment::start_with_cfg(self).await
    }
}

impl TestEnvironment {
    pub async fn start() -> TestEnvironment {
        TestEnvironment::start_with_cfg(TestEnvironmentCfg::with_all()).await
    }

    pub fn builder() -> TestEnvironmentCfg {
        TestEnvironmentCfg::default()
    }

    pub async fn start_with_cfg(cfg: TestEnvironmentCfg) -> TestEnvironment {
        let pg = if cfg.pg { Some(pg::PgContainer::run().await.unwrap()) } else { None };

        let s3 = if cfg.s3 { Some(S3Container::run().await.unwrap()) } else { None };

        let result = TestEnvironment { pg, s3 };

        result
    }

    /// Returns URL of PosgreSQL instance running in container
    pub async fn l2_storage_pg_url(&self) -> String {
        self.pg.as_ref().unwrap().connection_url().await
    }

    pub async fn metadata_storage_s3_client(&self) -> aws_sdk_s3::Client {
        self.s3.as_ref().unwrap().s3_client().await
    }

    pub async fn database_cfg(&self) -> DatabaseCfg {
        DatabaseCfg {
            connection_url: self.l2_storage_pg_url().await,
            min_connections: 1,
            max_connections: 1,
        }
    }

    pub async fn obj_storage_cfg(&self) -> ObjStorageCfg {
        self.s3.as_ref().unwrap().obj_storage_cfg().await
    }
}
