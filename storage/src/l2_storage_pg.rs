use entities::l2::{L2Asset, PublicKey};
use sqlx::{postgres::{PgConnectOptions, PgPoolOptions, PgRow}, ConnectOptions, PgPool, Row};
use interfaces::l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage};


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

#[async_trait::async_trait]
impl L2Storage for L2StoragePg {
    async fn save(&self, asset: &L2Asset) -> anyhow::Result<()> {
        let mut query_builder = sqlx::QueryBuilder::new(r#"
                INSERT INTO l2_assets_v1
                (
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    asset_create_timestamp,
                    pib44_account_num,
                    pib44_change_num
                )
            "#);
        query_builder.push_values(std::iter::once(asset), |mut builder, a| {
            builder
                .push_bind(&a.pubkey)
                .push_bind(&a.name)
                .push_bind(&a.owner)
                .push_bind(&a.creator)
                .push_bind(&a.collection)
                .push_bind(&a.authority)
                .push_bind(&a.create_timestamp)
                .push_bind(a.pib44_account_num as i64)
                .push_bind(a.pib44_change_num as i64);
        });
        query_builder.push(r#"
                ON CONFLICT(asset_pubkey) DO UPDATE SET
                asset_name = EXCLUDED.asset_name,
                asset_owner = EXCLUDED.asset_owner,
                asset_creator = EXCLUDED.asset_creator,
                asset_collection = EXCLUDED.asset_collection,
                asset_authority = EXCLUDED.asset_authority,
                asset_create_timestamp = EXCLUDED.asset_create_timestamp;
            "#);
        
        let _ = query_builder.build()
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find(&self, pubkey: &PublicKey) -> anyhow::Result<Option<L2Asset>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            r#"
                SELECT
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    asset_create_timestamp,
                    pib44_account_num,
                    pib44_change_num
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
    let pib44_account_num: i64 = row.try_get("pib44_account_num")?;
    let pib44_change_num: i64 = row.try_get("pib44_change_num")?;

    Ok(L2Asset{
        pubkey: row.try_get("asset_pubkey")?,
        name: row.try_get("asset_name")?,
        owner: row.try_get("asset_owner")?,
        creator: row.try_get("asset_creator")?,
        collection: row.try_get("asset_collection")?,
        authority: row.try_get("asset_authority")?,
        create_timestamp: row.try_get("asset_create_timestamp")?,
        pib44_account_num: pib44_account_num as u32,
        pib44_change_num: pib44_change_num as u32,
    })
}

#[derive(sqlx::FromRow, Debug)]
struct Bip44Row {
    pub account: i64,
    pub change: i64,
}

#[async_trait::async_trait]
impl Bip44DerivationSequence for L2StoragePg {
    async fn next_account(&self) -> anyhow::Result<DerivationValues> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("ALTER SEQUENCE last_bip44_change RESTART WITH 1")
            .execute(&mut tx)
            .await?;

        let Bip44Row {account, change}  = sqlx::query_as(r#"
            SELECT nextval('last_bip44_account') as account, nextval('last_bip44_change') as change
        "#)
            .fetch_one(&mut tx)
            .await?;

        tx.commit().await?;

        Ok(DerivationValues { account: account as u32, change: change as u32 })
    }

    async fn next_change(&self) -> anyhow::Result<DerivationValues> {
        let Bip44Row {account, change} = sqlx::query_as(r#"
            SELECT last_value as account, nextval('last_bip44_change') as change
            FROM last_bip44_account
        "#)
            .fetch_one(&self.pool)
            .await?;
        Ok(DerivationValues { account: account as u32, change: change as u32 })
    }
}