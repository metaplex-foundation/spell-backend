# Spell Backend 🧙‍
Backend services for Spell Wallet.

# Setup
Run docker compose to setup DB: 
```bash
docker compose up db -d
```

## Run REST Server
From root directory:
```bash
cd ./rest-server
```
And then:
```bash
cargo r
```

## Run JSON RPC Server
From root directory:
```bash
cd ./json-rpc
```
And then:
```bash
cargo r
```