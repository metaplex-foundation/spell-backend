use entities::l2::{L2Asset, PublicKey};
use interfaces::l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    ConnectOptions, PgPool, Row,
};

use tracing::log::LevelFilter;

pub struct L2StoragePg {
    pub pool: PgPool,
}

impl L2StoragePg {
    pub async fn new_from_url(url: &str, min_connections: u32, max_connections: u32) -> anyhow::Result<L2StoragePg> {
        let mut options: PgConnectOptions = url.parse().unwrap();
        options.log_statements(LevelFilter::Off);
        options.log_slow_statements(LevelFilter::Off, std::time::Duration::from_secs(100));

        let pool = PgPoolOptions::new()
            .min_connections(min_connections)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        Ok(L2StoragePg { pool })
    }

    pub fn new_from_pool(pool: PgPool) -> L2StoragePg {
        L2StoragePg { pool }
    }
}

#[async_trait::async_trait]
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
                    asset_create_timestamp,
                    pib44_account_num,
                    pib44_address_num
                )
            "#,
        );
        query_builder.push_values(std::iter::once(asset), |mut builder, a| {
            builder
                .push_bind(a.pubkey)
                .push_bind(&a.name)
                .push_bind(a.owner)
                .push_bind(a.creator)
                .push_bind(a.collection)
                .push_bind(a.authority)
                .push_bind(a.create_timestamp)
                .push_bind(a.pib44_account_num as i64)
                .push_bind(a.pib44_address_num as i64);
        });
        query_builder.push(
            r#"
                ON CONFLICT(asset_pubkey) DO UPDATE SET
                asset_name = EXCLUDED.asset_name,
                asset_owner = EXCLUDED.asset_owner,
                asset_creator = EXCLUDED.asset_creator,
                asset_collection = EXCLUDED.asset_collection,
                asset_authority = EXCLUDED.asset_authority,
                asset_create_timestamp = EXCLUDED.asset_create_timestamp;
            "#,
        );

        let _ = query_builder.build().execute(&self.pool).await?;
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
                    pib44_address_num
                FROM l2_assets_v1
                WHERE asset_pubkey = 
            "#,
        );

        query_builder.push_bind(pubkey);

        query_builder
            .build()
            .fetch_optional(&self.pool)
            .await?
            .map(|r| from_row(&r))
            .transpose()
    }

    async fn find_batch(&self, pubkeys: &[PublicKey]) -> anyhow::Result<Vec<L2Asset>> {
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
                    pib44_address_num
                FROM l2_assets_v1
                WHERE asset_pubkey IN(
            "#,
        );

        let mut separated = query_builder.separated(", ");
        for pubkey in pubkeys {
            separated.push_bind(pubkey);
        }

        // Complete the query
        separated.push_unseparated(")");

        Ok(query_builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| from_row(&r))
            .collect::<Result<Vec<L2Asset>, _>>()?)
    }
}

fn from_row(row: &PgRow) -> anyhow::Result<L2Asset> {
    let pib44_account_num: i64 = row.try_get("pib44_account_num")?;
    let pib44_change_num: i64 = row.try_get("pib44_address_num")?;

    Ok(L2Asset {
        pubkey: row.try_get("asset_pubkey")?,
        name: row.try_get("asset_name")?,
        owner: row.try_get("asset_owner")?,
        creator: row.try_get("asset_creator")?,
        collection: row.try_get("asset_collection")?,
        authority: row.try_get("asset_authority")?,
        create_timestamp: row.try_get("asset_create_timestamp")?,
        pib44_account_num: pib44_account_num as u32,
        pib44_address_num: pib44_change_num as u32,
    })
}

#[derive(sqlx::FromRow, Debug)]
struct Bip44Row {
    pub seq_val: i64,
}

#[async_trait::async_trait]
impl Bip44DerivationSequence for L2StoragePg {
    async fn next_account_and_address(&self) -> anyhow::Result<DerivationValues> {
        let Bip44Row { seq_val } = sqlx::query_as(
            r#"
            SELECT nextval('l2_bip44_sequence') as seq_val
        "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let (account, address) = i64_to_u32s(seq_val);

        Ok(DerivationValues { account, address })
    }
}

fn i64_to_u32s(a: i64) -> (u32, u32) {
    ((a as u64 >> 32) as u32, (a & 0xffffffff) as u32)
}

#[test]
fn test_i64_to_u32s() {
    assert_eq!((0, 0), i64_to_u32s(0));
    assert_eq!((0, 1), i64_to_u32s(1));
    assert_eq!((0, 2), i64_to_u32s(2));
    assert_eq!((0, u32::MAX), i64_to_u32s(u32::MAX as i64));
    assert_eq!((1, 0), i64_to_u32s(1i64 + u32::MAX as i64));
    assert_eq!((1, 1), i64_to_u32s(2i64 + u32::MAX as i64));
}
