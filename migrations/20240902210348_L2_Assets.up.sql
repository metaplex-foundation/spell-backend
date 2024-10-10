-- State of asset.
-- Initially assets are created as L2, but later can be minted on L1 blockchain
CREATE TYPE asset_state AS ENUM (
	'L2',
	'MINTING',
	'L1_SOLANA'
);

-- Table that stores information about L2 nft assets
CREATE TABLE IF NOT EXISTS l2_assets_v1 (
	asset_pubkey BYTEA NOT NULL,
    asset_name varchar(200) NOT NULL,
    asset_owner BYTEA NOT NULL,
    asset_creator BYTEA NOT NULL,
    asset_collection BYTEA DEFAULT NULL,
    asset_authority BYTEA NOT NULL,
    current_state asset_state NOT NULL DEFAULT 'L2',
    royalty_basis_points SMALLINT NOT NULL DEFAULT 500,
    asset_create_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    asset_last_update_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    bip44_account_num INT8 NOT NULL CHECK (bip44_account_num >= 0),
    bip44_address_num INT8 NOT NULL CHECK (bip44_address_num >= 0),
    CONSTRAINT pk_asset_pubkey PRIMARY KEY (asset_pubkey)
);

CREATE INDEX idx_asset_owner_create_timestamp ON l2_assets_v1(asset_owner, asset_create_timestamp) WHERE (current_state != 'L1_SOLANA');
CREATE INDEX idx_asset_owner_update_timestamp ON l2_assets_v1(asset_owner, asset_last_update_timestamp) WHERE (current_state != 'L1_SOLANA');

CREATE INDEX idx_asset_creator_create_timestamp ON l2_assets_v1(asset_creator, asset_create_timestamp) WHERE (current_state != 'L1_SOLANA');
CREATE INDEX idx_asset_creator_update_timestamp ON l2_assets_v1(asset_creator, asset_last_update_timestamp) WHERE (current_state != 'L1_SOLANA');

-- Used to track numbers of HD wallets
-- solana_sdk::derivation_path::DerivationPath::new_bip44 talkes u32 arguments
-- that's why max value is u32::max
CREATE SEQUENCE IF NOT EXISTS l2_bip44_sequence
    INCREMENT BY 1
    MINVALUE 1
    START WITH 1
    OWNED BY NONE;
