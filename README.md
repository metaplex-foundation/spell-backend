# Spell Backend 🧙‍
Backend services for Spell Wallet.

# Setup local development environment:
You can use 2 different approaches to set up local development.
The first one using `Docker` and `SQLx`, the second one implies that everything will be installed locally. 

## Local development setup via Docker and SQLx
Follow the steps below to set up the local development environment using `Docker` and `sqlx`. All commands should be run from the root directory.

### 1. Set Up Docker Services

To start the necessary services (Postgres and MinIO), run:
```shell
docker compose up db -d
```

### 2. Configure the Database with SQLx
If you ***don't familiar with SQLx***, read docs:
1. https://github.com/launchbadge/sqlx/blob/main/README.md
2. https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md


Set up the database with the following commands:
```shell
sqlx database setup --database-url postgres://postgres:postgres@localhost:5432/spell-wallet
```
It will create the database specified and ***runs any pending migrations***, so it's unnecessary to run migrations manually. 


### Running SQLx migrations
When running or rolling back migrations, make sure to specify the `--database-url` option if you don't have a `.env` file configured.
For example:
```shell
sqlx migrate run --database-url postgres://postgres:postgres@localhost:5432/spell-wallet
```
### Simplifying with Environment Variables

To avoid specifying the `--database-url` option each time, you can create a `.env` file with the `DATABASE_URL` environment variable:

```shell
DATABASE_URL=postgres://postgres:postgres@localhost:5432/spell-wallet
```
 

## `manual-migrations` folder

The `manual-migrations` folder contains SQL files used specifically for deployment. If you modify any migrations in the `migrations` folder,
make sure to replicate those changes in the `manual-migrations` folder to keep both environments in sync.

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

# Run all tests

## In parallel
Before running tests, it may be necessary to configure the test runtime limits due to potential high CPU and RAM usage when running tests in a Docker environment. This ensures that tests don't fail due to exceeding time limits.

Test time limits are specified in milliseconds. For example, `300000 milliseconds equals 5 minutes`.

To configure the test timeouts, run the following commands:
```shell
export RUST_TEST_TIME_UNIT=300000,300000
export RUST_TEST_TIME_INTEGRATION=300000,300000
```

Once the environment is set, you can run all tests using:
```shell
cargo test
```

## In one thread
You can run all tests sequentially, one by one, using the following command:
```shell
cargo test -- --test-threads 1
```
This may take more time to complete, but it will reduce CPU and memory usage by preventing parallel test execution.

# Formating
Use this command for formating code:
```shell
cargo +nightly fmt 
```
