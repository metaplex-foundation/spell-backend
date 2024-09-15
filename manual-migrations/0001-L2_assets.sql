-- Table that stores information about L2 nft assets
CREATE TABLE l2_assets_v1 (
	asset_pubkey BYTEA NOT NULL,
    asset_name varchar(200) NOT NULL,
    asset_owner BYTEA NOT NULL,
    asset_creator BYTEA NOT NULL,
    asset_collection BYTEA DEFAULT NULL,
    asset_authority BYTEA NOT NULL,
    current_state asset_state NOT NULL DEFAULT 'L2',
    asset_create_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    asset_last_update_timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    pib44_account_num INT8 NOT NULL CHECK (pib44_account_num >= 0),
    pib44_address_num INT8 NOT NULL CHECK (pib44_address_num >= 0),
    CONSTRAINT pk_asset_pubkey PRIMARY KEY (asset_pubkey)
);

CREATE INDEX idx_asset_owner_create_timestamp ON l2_assets_v1(asset_owner, asset_create_timestamp);
CREATE INDEX idx_asset_owner_update_timestamp ON l2_assets_v1(asset_owner, asset_last_update_timestamp);

CREATE INDEX idx_asset_creator_create_timestamp ON l2_assets_v1(asset_creator, asset_create_timestamp);
CREATE INDEX idx_asset_creator_update_timestamp ON l2_assets_v1(asset_creator, asset_last_update_timestamp);

-- Used to track numbers of HD wallets
-- solana_sdk::derivation_path::DerivationPath::new_bip44 talkes u32 arguments
-- that's why max value is u32::max
CREATE SEQUENCE l2_bip44_sequence
    INCREMENT BY 1
    MINVALUE 1
    START WITH 1
    OWNED BY NONE;
