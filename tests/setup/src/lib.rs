//! This crate contains utilities for preparing an environment
//! for integration tests, including:
//! * docker containers

use pg::PgContainer;
use s3::S3Container;
use std::net::Ipv4Addr;
use test_validator_runner::{SolanaProcess, TestValidatorRunner};
use util::config::{DatabaseCfg, JsonRpc, ObjStorageCfg, RestServerCfg, SecretCfg, SecretsCfg, Settings, SolanaCfg};

pub mod data_gen;
pub mod pg;
pub mod s3;
pub mod test_validator_runner;

pub struct TestEnvironment {
    pub pg: Option<PgContainer>,
    pub s3: Option<S3Container>,
    pub solana: Option<SolanaProcess>,
}

#[derive(Default)]
pub struct TestEnvironmentCfg {
    pub pg: bool,
    pub s3: bool,
    pub solana: bool,
}

impl TestEnvironmentCfg {
    pub fn with_all() -> Self {
        TestEnvironmentCfg { pg: true, s3: true, solana: true }
    }
    pub fn with_pg(mut self) -> Self {
        self.pg = true;
        self
    }
    pub fn with_s3(mut self) -> Self {
        self.s3 = true;
        self
    }
    pub fn with_solana(mut self) -> Self {
        self.solana = true;
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

        let solana = if cfg.solana {
            let mut validator = TestValidatorRunner::new(8062);
            validator.clone_program("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");

            Some(validator.run().unwrap())
        } else {
            None
        };

        TestEnvironment { pg, s3, solana }
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
            min_connections: 5,
            max_connections: 10,
            log_level: "DEBUG".to_string(),
        }
    }

    pub async fn obj_storage_cfg(&self) -> ObjStorageCfg {
        self.s3.as_ref().unwrap().obj_storage_cfg().await
    }

    pub fn solana_url(&self) -> String {
        self.solana.as_ref().unwrap().solana_url.clone()
    }

    pub async fn make_test_cfg(&self) -> Settings {
        let solana = if let Some(s) = self.solana.as_ref() {
            SolanaCfg { url: s.solana_url.clone() }
        } else {
            SolanaCfg { url: "".to_string() }
        };

        Settings {
            rest_server: RestServerCfg {
                port: 8080,
                host: Ipv4Addr::LOCALHOST,
                log_level: "DEBUG".to_string(),
                base_url: "http://localhost".to_string(),
            },
            database: self.database_cfg().await,
            obj_storage: self.obj_storage_cfg().await,
            env: "it".to_string(),
            json_rpc_server: JsonRpc { port: 8081, host: Ipv4Addr::LOCALHOST, log_level: "DEBUG".to_string() },
            solana,
            secrets: SecretsCfg {
                master_mnemonic: SecretCfg::Plain("".to_string()),
                rest_api_keys: SecretCfg::Plain("111:name1;222:name2;333:name3".to_string()),
            },
        }
    }
}
