use testcontainers::{core::Mount, runners::AsyncRunner, ContainerAsync, ImageExt};
use testcontainers_modules::postgres::Postgres;

/// Launches docker container with PostgreSQL that is prepopulated
/// with SQL scripts from migrations directory
pub async fn run_pg() -> anyhow::Result<ContainerAsync<Postgres>> {
    let container_cfg = testcontainers_modules::postgres::Postgres::default()
        .with_mount(Mount::bind_mount(ddl_path(), "/docker-entrypoint-initdb.d"));

    let node = container_cfg.start().await?;

    Ok(node)
}

/// Returns path to "migrations" folder
fn ddl_path() -> String {
    std::env::current_dir().unwrap() // integration tests dir
        .parent().unwrap() // workspace dir
        .join("migrations")
        .to_str().unwrap()
        .to_string()
}
