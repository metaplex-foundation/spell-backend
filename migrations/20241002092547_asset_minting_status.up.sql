CREATE TABLE IF NOT EXISTS asset_minting_status (
    asset_pubkey BYTEA NOT NULL,
    current_state asset_state NOT NULL DEFAULT 'MINTING',
    signature BYTEA NOT NULL
);
