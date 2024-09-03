# Spell Backend 🧙‍
Backend services for Spell Wallet.

# Setup local development environment:
You can use 2 different approaches to set up local development.
The first one using `Docker` and `SQLx`, the second one implies that everything will be installed locally. 

## Local development setup via Docker and Sqlx
Follow the steps below to set up the local development environment using `Docker` and `sqlx`. All commands should be run from the root directory.

### 1. Set Up Docker Services

To start the necessary services (Postgres and MinIO), run:
```shell
docker compose up db -d
```

### 2. Configure the Database with SQLx
Set up the database with the following commands:
```shell
sqlx database setup --source ./sqlx-migrations
sqlx database setup --database-url postgres://postgres:postgres@localhost:5432/spell-wallet
sqlx database create
```
After setting up the database, apply the migrations:
```shell
sqlx migrate run
```

### 3. Setup buckets in MinIo:

To set up buckets in MinIO:

1) Open the MinIO endpoint specified in the `docker-compose.yml` file. Typically, it is available at: http://127.0.0.1:9001.
2) Log in using the credentials from the docker-compose.yml file.
3) Create the buckets specified in the configuration file. By default, these are `asset-metadata` and `binary-assets`.



## Running via local setup
1) Make sure you have installed PosgreSQL. Create a new schema, e.g. `spell` and populate in suign .sql scripts from `migrations` directory
2) Install, run Minio and create buckets.



# Configure services
1) Create a config TOML file in `config/` directory with parameters that feet your local setup, e.g. `config/john.toml`
2) Set RUN_ENV variable equal to the filename of your config, e.g:
   ```shell
   export RUN_ENV=john
   ```

# Run JSON RPC Server
From root directory:
```shell
cd ./json-rpc
cargo r
```


# Run REST Server
From root directory:
```shell
cd ./rest-server
cargo r
```
