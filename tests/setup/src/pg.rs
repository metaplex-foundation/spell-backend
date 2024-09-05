use testcontainers::{core::Mount, runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

pub struct PgContainer {
    node: ContainerAsync<Postgres>,
}

impl PgContainer {
    /// Launches docker container with PostgreSQL that is prepopulated
    /// with SQL scripts from sqlx-migrations directory
    pub async fn run() -> anyhow::Result<PgContainer> {
        let container_cfg = testcontainers_modules::postgres::Postgres::default()
            .with_mount(Mount::bind_mount(ddl_path(), "/docker-entrypoint-initdb.d"));

        let node = container_cfg.start().await?;

        Ok(PgContainer { node })
    }

    /// Returns URL of PosgreSQL instance running in container
    pub async fn connection_url(&self) -> String {
        format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            self.node.get_host_port_ipv4(5432).await.unwrap()
        )
    }
}

/// Returns path to "sqlx-migrations" folder
fn ddl_path() -> String {
    std::env::current_dir()
        .unwrap() // integration tests dir
        .parent()
        .unwrap() // workspace dir
        .join("sqlx-migrations")
        .to_str()
        .unwrap()
        .to_string()
}
