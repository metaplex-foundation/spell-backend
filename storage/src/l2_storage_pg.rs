use anyhow::Context;
use entities::l2::{AssetSortBy, AssetSortDirection, AssetSorting, L2Asset, PublicKey};
use interfaces::l2_storage::{Bip44DerivationSequence, DerivationValues, L2Storage};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions, PgRow},
    query, ConnectOptions, PgExecutor, PgPool, Postgres, QueryBuilder, Row,
};
use std::fmt::Display;
use std::thread::{sleep, spawn};
use std::time::Duration;
use tracing::log::LevelFilter;
use tracing::{error, info};
use util::base64_encode_decode::decode_timestamp_and_asset_pubkey;

pub struct L2StoragePg {
    pub pool: PgPool,
}

#[async_trait::async_trait]
impl L2Storage for L2StoragePg {
    async fn save(&self, asset: &L2Asset) -> anyhow::Result<()> {
        let mut query_builder = QueryBuilder::new(
            r#"
                INSERT INTO l2_assets_v1
                (
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    royalty_basis_points,
                    asset_create_timestamp,
                    asset_last_update_timestamp,
                    bip44_account_num,
                    bip44_address_num
                )
            "#,
        );
        query_builder.push_values(std::iter::once(asset), |mut builder, asset| {
            builder
                .push_bind(asset.pubkey)
                .push_bind(&asset.name)
                .push_bind(&asset.owner)
                .push_bind(&asset.creator)
                .push_bind(asset.collection)
                .push_bind(&asset.authority)
                .push_bind(asset.royalty_basis_points as i16)
                .push_bind(asset.create_timestamp)
                .push_bind(asset.update_timestamp)
                .push_bind(asset.bip44_account_num as i64)
                .push_bind(asset.bip44_address_num as i64);
        });
        query_builder.push(
            r#"
                ON CONFLICT(asset_pubkey) DO UPDATE SET
                asset_name = EXCLUDED.asset_name,
                asset_owner = EXCLUDED.asset_owner,
                asset_creator = EXCLUDED.asset_creator,
                asset_collection = EXCLUDED.asset_collection,
                asset_authority = EXCLUDED.asset_authority,
                asset_create_timestamp = EXCLUDED.asset_create_timestamp,
                asset_last_update_timestamp = EXCLUDED.asset_last_update_timestamp
                WHERE l2_assets_v1.current_state = 'L2';
            "#,
        );

        let _ = query_builder.build().execute(&self.pool).await?;
        Ok(())
    }

    async fn find(&self, pubkey: &PublicKey) -> anyhow::Result<Option<L2Asset>> {
        let mut query_builder = QueryBuilder::new(
            r#"
                SELECT
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    royalty_basis_points,
                    asset_create_timestamp,
                    asset_last_update_timestamp,
                    bip44_account_num,
                    bip44_address_num
                FROM l2_assets_v1
                WHERE current_state = 'L2' AND asset_pubkey = 
            "#,
        );

        query_builder.push_bind(pubkey);

        query_builder
            .build()
            .fetch_optional(&self.pool)
            .await?
            .map(Self::asset_from_row)
            .transpose()
            .inspect_err(|e| error!("L2Storage error: {e}"))
    }

    async fn find_batch(&self, pubkeys: &[PublicKey]) -> anyhow::Result<Vec<L2Asset>> {
        let mut query_builder = QueryBuilder::new(
            r#"
                SELECT
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    asset_create_timestamp,
                    asset_last_update_timestamp,
                    royalty_basis_points,
                    bip44_account_num,
                    bip44_address_num
                FROM l2_assets_v1
                WHERE asset_pubkey IN(
            "#,
        );

        let mut separated = query_builder.separated(", ");

        for pubkey in pubkeys {
            separated.push_bind(pubkey);
        }

        separated.push_unseparated(")");

        query_builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(Self::asset_from_row)
            .collect::<Result<Vec<L2Asset>, _>>()
    }

    async fn find_by_owner(
        &self,
        owner_pubkey: &str,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2Asset>> {
        self.find_by("asset_owner", owner_pubkey, sorting, limit, before, after)
            .await
    }

    async fn find_by_creator(
        &self,
        creator_pubkey: &str,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2Asset>> {
        self.find_by("asset_creator", creator_pubkey, sorting, limit, before, after)
            .await
    }

    async fn lock_asset_before_minting(&self, asset_pubkey: &PublicKey) -> anyhow::Result<bool> {
        let update_result = QueryBuilder::new(
            r#"
                UPDATE l2_assets_v1
                SET current_state = 'MINTING', asset_last_update_timestamp = NOW()
                WHERE current_state = 'L2' AND asset_pubkey = "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .execute(&self.pool)
        .await?;

        Ok(update_result.rows_affected() > 0)
    }

    async fn find_l1_asset_signature(&self, asset_pubkey: &PublicKey) -> Option<Vec<u8>> {
        QueryBuilder::new(
            r#"
            SELECT
                signature
            FROM asset_minting_status
            WHERE asset_pubkey = 
        "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .fetch_one(&self.pool)
        .await
        .ok()
        .and_then(Self::try_get_signature_from_row)
    }

    async fn add_l1_asset(&self, asset_pubkey: &PublicKey, tx_signature: &[u8]) -> anyhow::Result<()> {
        query(
            r#"
                INSERT INTO asset_minting_status
                (
                    asset_pubkey,
                    signature
                )
                VALUES ($1, $2)
                "#,
        )
        .bind(asset_pubkey)
        .bind(tx_signature)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn finalize_mint(&self, pubkey: &PublicKey) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        Self::finalize_mint_of_l2_asset(pubkey, &mut tx).await?;
        Self::finalize_mint_of_asset_minting_status(pubkey, &mut tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn mint_didnt_happen(&self, pubkey: &PublicKey) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        Self::cancel_l2_minting(pubkey, &mut tx).await?;
        Self::cancel_l1_minting(pubkey, &mut tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

impl L2StoragePg {
    const LOGGING_INTERVAL_FOR_POOL_STATUS_IN_TESTS: Duration = Duration::from_secs(5);

    pub async fn new_from_url(url: &str, min_connections: u32, max_connections: u32) -> anyhow::Result<L2StoragePg> {
        let mut options = url.parse::<PgConnectOptions>()?;
        options.log_statements(LevelFilter::Off);
        options.log_slow_statements(LevelFilter::Off, std::time::Duration::from_secs(100));
        options = options.extra_float_digits(None); // needed for Pgbouncer

        let pool = PgPoolOptions::new()
            .min_connections(min_connections)
            .max_connections(max_connections)
            .connect_with(options)
            .await?;

        Ok(Self::new_from_pool(pool))
    }

    pub fn new_from_pool(pool: PgPool) -> L2StoragePg {
        if cfg!(test) {
            Self::log_connection_pool_status_in_background(pool.clone());
        }

        L2StoragePg { pool }
    }

    fn log_connection_pool_status_in_background(pool: PgPool) {
        spawn(|| Self::log_connection_pool_status(pool));
    }

    fn log_connection_pool_status(pool: PgPool) {
        loop {
            info!("Pool size: {}", pool.size());
            info!("Idle connections: {}", pool.num_idle());
            sleep(Self::LOGGING_INTERVAL_FOR_POOL_STATUS_IN_TESTS)
        }
    }

    async fn finalize_mint_of_l2_asset(asset_pubkey: &PublicKey, executor: impl PgExecutor<'_>) -> anyhow::Result<()> {
        QueryBuilder::new(
            r#"
            UPDATE l2_assets_v1
            SET current_state = 'L1_SOLANA', asset_last_update_timestamp = NOW()
            WHERE asset_pubkey =
        "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .execute(executor)
        .await?;

        Ok(())
    }

    async fn finalize_mint_of_asset_minting_status(
        asset_pubkey: &PublicKey,
        executor: impl PgExecutor<'_>,
    ) -> anyhow::Result<()> {
        QueryBuilder::new(
            r#"
            UPDATE asset_minting_status
            SET current_state = 'L1_SOLANA'
            WHERE asset_pubkey =
        "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .execute(executor)
        .await?;

        Ok(())
    }

    async fn cancel_l2_minting(asset_pubkey: &PublicKey, executor: impl PgExecutor<'_>) -> anyhow::Result<()> {
        QueryBuilder::new(
            r#"
                    UPDATE l2_assets_v1
                    SET current_state = 'L2', asset_last_update_timestamp = NOW()
                    WHERE asset_pubkey = 
                "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .execute(executor)
        .await?;

        Ok(())
    }

    async fn cancel_l1_minting(asset_pubkey: &PublicKey, executor: impl PgExecutor<'_>) -> anyhow::Result<()> {
        QueryBuilder::new(
            r#"
                    UPDATE asset_minting_status
                    SET current_state = 'L2'
                    WHERE asset_pubkey =
                "#,
        )
        .push_bind(asset_pubkey)
        .build()
        .execute(executor)
        .await?;

        Ok(())
    }

    async fn find_by(
        &self,
        column_name: &str,
        primary_key: &str,
        sorting: &AssetSorting,
        limit: u32,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<Vec<L2Asset>> {
        let mut query_builder = QueryBuilder::new(
            r#"
                SELECT
                    asset_pubkey,
                    asset_name,
                    asset_owner,
                    asset_creator,
                    asset_collection,
                    asset_authority,
                    royalty_basis_points,
                    asset_create_timestamp,
                    asset_last_update_timestamp,
                    bip44_account_num,
                    bip44_address_num
                FROM l2_assets_v1
                WHERE
            "#,
        );

        query_builder.push(column_name).push(" = ").push_bind(primary_key);

        Self::add_timestamp_and_pubkey_comparison(&mut query_builder, &sorting, before, after)?;

        let is_order_reversed = before.is_some() && after.is_none();

        let direction = match (&sorting.sort_direction, is_order_reversed) {
            (AssetSortDirection::Asc, true) | (AssetSortDirection::Desc, false) => " DESC ",
            (AssetSortDirection::Asc, false) | (AssetSortDirection::Desc, true) => " ASC ",
        };

        query_builder
            .push(" ORDER BY ")
            .push(sorting.sort_by.to_string())
            .push(direction)
            .push(", asset_pubkey ")
            .push(direction);

        query_builder.push(" LIMIT ").push_bind(limit as i64);

        match is_order_reversed {
            true => query_builder
                .build()
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(Self::asset_from_row)
                .rev()
                .collect::<Result<Vec<L2Asset>, _>>(),
            false => query_builder
                .build()
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(Self::asset_from_row)
                .collect::<Result<Vec<L2Asset>, _>>(),
        }
    }

    fn add_timestamp_and_pubkey_comparison(
        mut query_builder: &mut QueryBuilder<'_, Postgres>,
        asset_sorting: &AssetSorting,
        before: Option<&str>,
        after: Option<&str>,
    ) -> anyhow::Result<()> {
        match &asset_sorting.sort_by {
            AssetSortBy::Created | AssetSortBy::Updated => {
                if let Some(before) = before {
                    let comparison = match asset_sorting.sort_direction {
                        AssetSortDirection::Asc => " < ",
                        AssetSortDirection::Desc => " > ",
                    };

                    Self::add_timestamp_and_key_comparison(
                        &before,
                        comparison,
                        &asset_sorting.sort_by,
                        &mut query_builder,
                    )?;
                }

                if let Some(after) = after {
                    let comparison = match asset_sorting.sort_direction {
                        AssetSortDirection::Asc => " > ",
                        AssetSortDirection::Desc => " < ",
                    };

                    Self::add_timestamp_and_key_comparison(
                        &after,
                        comparison,
                        &asset_sorting.sort_by,
                        &mut query_builder,
                    )?;
                }
            }
        }

        Ok(())
    }

    fn add_timestamp_and_key_comparison(
        key: &str,
        comparison: &str,
        order_field: &AssetSortBy,
        query_builder: &mut QueryBuilder<'_, Postgres>,
    ) -> anyhow::Result<()> {
        let (timestamp, pubkey) = decode_timestamp_and_asset_pubkey(key)?;

        let order_field = order_field.to_string();
        let comparison = comparison.to_string();

        query_builder
            .push(" AND (")
            .push(order_field.clone())
            .push(comparison.clone())
            .push_bind(timestamp);

        query_builder
            .push(" OR (")
            .push(order_field)
            .push(" = ")
            .push_bind(timestamp);

        query_builder
            .push(" AND asset_pubkey ")
            .push(comparison)
            .push_bind(pubkey)
            .push("))");

        Ok(())
    }

    fn log_from_row_err(row_name: &str, err: impl Display) {
        error!("FromRowError for: '{row_name}'. Cause: {err}")
    }

    fn try_get_signature_from_row(row: PgRow) -> Option<Vec<u8>> {
        row.try_get("signature")
            .inspect_err(|e| Self::log_from_row_err("signature", e))
            .ok()
    }

    fn asset_from_row(row: PgRow) -> anyhow::Result<L2Asset> {
        Ok(L2Asset {
            pubkey: Self::try_get_asset_from_row(&row, "asset_pubkey")?,
            name: Self::try_get_asset_from_row(&row, "asset_name")?,
            owner: Self::try_get_asset_from_row(&row, "asset_owner")?,
            creator: Self::try_get_asset_from_row(&row, "asset_creator")?,
            collection: Self::try_get_asset_from_row(&row, "asset_collection")?,
            authority: Self::try_get_asset_from_row(&row, "asset_authority")?,
            royalty_basis_points: Self::try_get_asset_from_row::<i16>(&row, "royalty_basis_points")? as u16,
            create_timestamp: Self::try_get_asset_from_row(&row, "asset_create_timestamp")?,
            update_timestamp: Self::try_get_asset_from_row(&row, "asset_last_update_timestamp")?,
            bip44_account_num: Self::try_get_asset_from_row::<i64>(&row, "bip44_account_num")? as u32,
            bip44_address_num: Self::try_get_asset_from_row::<i64>(&row, "bip44_address_num")? as u32,
        })
    }

    fn try_get_asset_from_row<'a, T>(row: &'a PgRow, index: &str) -> anyhow::Result<T>
    where
        T: sqlx::Decode<'a, Postgres> + sqlx::Type<Postgres>,
    {
        row.try_get::<'a, T, _>(index)
            .inspect_err(|e| Self::log_from_row_err(index, e))
            .context("FromRowError")
    }
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
