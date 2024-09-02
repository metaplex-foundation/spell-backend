# Spell Backend
Backend services for Spell Wallet.

## REST Server
To set up the development environment, run the following Docker commands:

```bash
cd ./rest-server
docker compose up db -d
```

To run use:
```bash
cargo r
```

## Running locally

1) Make sure you have installed PosgreSQL. Create a new schema, e.g. `spell` and populate in suign .sql scripts from `migrations` directory
2) Install and run Minio.
3) Create a config TOML file in `config/` directory with parameters that feet your local setup, e.g. `config/john.toml`
4) Set RUN_ENV variable equal to the filename of your config, e,g,
   ```shell
   export RUN_ENV=john
   ```
5) Execute `cargo run`
