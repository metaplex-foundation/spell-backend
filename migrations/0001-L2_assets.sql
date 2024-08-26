-- Table that stores information about L2 nft assets
CREATE TABLE l2_assets_v1 (
	asset_pubkey BYTEA NOT NULL,
    asset_name varchar(200) NOT NULL,
    asset_owner BYTEA NOT NULL,
    asset_creator BYTEA NOT NULL,
    asset_collection BYTEA DEFAULT NULL,
    asset_authority BYTEA NOT NULL,
    asset_metadata_url varchar(2048) NOT NULL,
    asset_create_timestamp TIMESTAMP DEFAULT NOW(),
    CONSTRAINT pk_asset_pubkey PRIMARY KEY (asset_pubkey)
);
