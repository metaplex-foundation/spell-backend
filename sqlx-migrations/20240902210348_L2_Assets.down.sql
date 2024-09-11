-- Add down migration script here
DROP TABLE IF EXISTS l2_assets_v1;

DROP INDEX IF EXISTS idx_asset_owner_create_timestamp;

DROP SEQUENCE IF EXISTS l2_bip44_sequence;