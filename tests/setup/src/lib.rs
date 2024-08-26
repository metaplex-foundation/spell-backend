//! This crate contains utilities for preparing an environment
//! for integration tests, including:
//! * docker containers

use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

mod pg;

pub struct TestEnvironment {
    pg: ContainerAsync<Postgres>,
}

impl TestEnvironment {
    pub async fn start() -> TestEnvironment {

        let pg = pg::run_pg().await.unwrap();

        TestEnvironment {
            pg
        }
    }

    /// Returns URL of PosgreSQL instance running in container
    pub async fn pg_url(&self) -> String {
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            self.pg.get_host_port_ipv4(5432).await.unwrap()
        )
    }
}