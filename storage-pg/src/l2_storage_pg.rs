use sqlx::{postgres::{PgConnectOptions, PgPoolOptions, PgRow}, ConnectOptions, PgPool, Row};
use storage::l2_storage::{AssetKey, L2Asset, L2Storage};


use async_trait::async_trait;
use tracing::log::LevelFilter;

pub struct L2StoragePg {
    pub pool: PgPool,
}

impl L2StoragePg {
    pub async fn new_from_url(url: &str, min_connections: u32, max_connections: u32,) -> anyhow::Result<L2StoragePg> {
        let mut options: PgConnectOptions = url.parse().unwrap();
        options.log_statements(LevelFilter::Off);
        options.log_slow_statements(LevelFilter::Off, std::time::Duration::from_secs(100));

        let pool = PgPoolOptions::new()
            .min_connections(min_connections)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        Ok(L2StoragePg {
            pool
        })
    }

    pub fn new_from_pool(pool: PgPool) -> L2StoragePg {
        L2StoragePg { pool }
    }
}

#[async_trait]
impl L2Storage for L2StoragePg {
    async fn save(&self, asset: &L2Asset) -> anyhow::Result<()> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
                INSERT INTO l2_assets_v1
                (
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    asset_metadata_url,
                    asset_create_timestamp
                )
            "#
        );
        query_builder.push_values(std::iter::once(asset), |mut builder, a| {
            builder
                .push_bind(&a.pubkey)
                .push_bind(&a.name)
                .push_bind(&a.owner)
                .push_bind(&a.creator)
                .push_bind(&a.collection)
                .push_bind(&a.authority)
                .push_bind(&a.metadata_url)
                .push_bind(&a.create_timestamp);
        });
        query_builder.push(" ON CONFLICT DO NOTHING");
        
        let _ = query_builder.build()
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find(&self, pubkey: &AssetKey) -> anyhow::Result<Option<L2Asset>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
                SELECT
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    asset_metadata_url,
                    asset_create_timestamp
                FROM l2_assets_v1
                WHERE asset_pubkey = 
            "#);

        query_builder.push_bind(pubkey);

        query_builder.build()
            .fetch_optional(&self.pool)
            .await?
            .map(|r| from_row(&r))
            .transpose()
    }
}

fn from_row(row: &PgRow) -> anyhow::Result<L2Asset> {
    Ok(L2Asset{
        pubkey: row.try_get("asset_pubkey")?,
        name: row.try_get("asset_name")?,
        owner: row.try_get("asset_owner")?,
        creator: row.try_get("asset_creator")?,
        collection: row.try_get("asset_collection")?,
        authority: row.try_get("asset_authority")?,
        metadata_url: row.try_get("asset_metadata_url")?,
        create_timestamp: row.try_get("asset_create_timestamp")?,
    })
}
