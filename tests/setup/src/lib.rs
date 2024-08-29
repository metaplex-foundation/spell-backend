//! This crate contains utilities for preparing an environment
//! for integration tests, including:
//! * docker containers

use pg::PgContainer;
use s3::S3Container;

pub mod data_gen;
mod pg;
mod s3;

pub const JSON_METADATA_S3_BUCKET: &str = "asset-metadata";

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
    fn with_all() -> Self {
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
        let pg = if cfg.pg {
            Some(pg::PgContainer::run().await.unwrap())
        } else {
            None
        };

        let s3 = if cfg.s3 {
            Some(S3Container::run().await.unwrap())
        } else {
            None
        };

        let result = TestEnvironment { pg, s3 };

        // post initialization
        if cfg.s3 {
            let s3_client = result.metadata_storage_s3_client().await;
            s3_client
                .create_bucket()
                .bucket(JSON_METADATA_S3_BUCKET)
                .send()
                .await
                .unwrap();
        }

        result
    }

    /// Returns URL of PosgreSQL instance running in container
    pub async fn l2_storage_pg_url(&self) -> String {
        self.pg.as_ref().unwrap().connection_url().await
    }

    pub async fn metadata_storage_s3_client(&self) -> aws_sdk_s3::Client {
        self.s3.as_ref().unwrap().s3_client().await.unwrap()
    }
}
