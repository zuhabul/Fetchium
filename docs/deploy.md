# Fetchium Deployment Guide

## Prerequisites

- Rust toolchain (`rustup`, stable)
- SearXNG running at `http://localhost:4040`
- Secrets in `~/.fetchium/env` (never commit this file)

## Deploy Procedure

```bash
# 0. Backup current binary
cp target/release/fetchium ~/.fetchium/backups/fetchium-prev

# 1. Build
~/.cargo/bin/cargo build -p fetchium-cli --release

# 2. Verify binary
./target/release/fetchium --version

# 3. Deploy (systemd reads target/release/ directly)
sudo systemctl restart fetchium-api

# 4. Verify
curl -sf http://localhost:3050/v1/health | jq .status
```

Expected: `"ok"` (auth_store + search_backbone both healthy)

## Rollback

```bash
sudo cp ~/.fetchium/backups/fetchium-prev target/release/fetchium
sudo systemctl restart fetchium-api
curl -sf http://localhost:3050/v1/health | jq .status
```

## Verify Public Endpoint

```bash
curl -s https://api.fetchium.com/v1/health | jq .
```

## Environment File

Secrets live in `~/.fetchium/env` (loaded by systemd `EnvironmentFile=`):

```
FETCHIUM_ADMIN_SECRET=<openssl rand -hex 32>
SEARXNG_URL=http://localhost:4040
RUST_LOG=info
```

Never commit `~/.fetchium/env`. It is outside the repository by design.

## Service Management

```bash
sudo systemctl status fetchium-api     # check status
sudo systemctl restart fetchium-api    # restart
sudo journalctl -u fetchium-api -f     # tail logs
```

## Pre-deploy Backup (automate)

Add to `scripts/deploy.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail
mkdir -p ~/.fetchium/backups
cp target/release/fetchium ~/.fetchium/backups/fetchium-prev
~/.cargo/bin/cargo build -p fetchium-cli --release
sudo systemctl restart fetchium-api
sleep 2
curl -sf http://localhost:3050/v1/health | jq .status
```
