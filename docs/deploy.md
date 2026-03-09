# Fetchium Deployment Guide

## Prerequisites

- Rust toolchain (`rustup`, stable)
- SearXNG running at `***REMOVED***`
- SecretOps manifest at `secrets.yml`
- Runtime env in `~/.fetchium/env` (never commit this file, managed by `isecd` after cutover)

## Preferred Production Model

Fetchium now targets ***REMOVED*** delivery for all internal runtime secrets:

- Source of truth: Infisical project `fetchium`
- Checked-in contract: `secrets.yml`
- Delivered runtime file: `~/.fetchium/env`
- Restart fan-out target: `***REMOVED***`

`isecd` owns the runtime env file after cutover. Any internal secret changed in Infisical should
sync into `~/.fetchium/env`, then restart:

- `fetchium-api`
- `fetchium-admin`
- `fetchium-dashboard`
- `fetchium-web`

Third-party provider credentials are intentionally outside the internal auto-rotation bucket unless
they are moved behind a broker or explicit provider rotation flow.

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
curl -s ***REMOVED***/v1/health | jq .
```

## Environment File

Runtime values live in `~/.fetchium/env` (loaded by systemd `EnvironmentFile=`):

```
***REMOVED***=<managed-by-infisical>
***REMOVED***=
***REMOVED***=<managed-by-infisical>
***REMOVED***=<managed-by-infisical>
***REMOVED***=
AUTH_SECRET=<managed-by-infisical>
NEXTAUTH_SECRET=<managed-by-infisical>
***REMOVED***=<managed-by-infisical>
***REMOVED***=false
***REMOVED***=***REMOVED***
***REMOVED***=***REMOVED***
SEARXNG_URL=***REMOVED***
RUST_LOG=info
```

Never commit `~/.fetchium/env`. It is outside the repository by design. Use `infra/fetchium.env.production`
only as a bootstrap reference, not as the long-term source of truth.

## Systemd Units

Install the repo-managed unit files so SecretOps can restart the full stack deterministically:

```bash
sudo cp infra/fetchium-admin.service /etc/systemd/system/fetchium-admin.service
sudo cp infra/fetchium-dashboard.service /etc/systemd/system/fetchium-dashboard.service
sudo cp infra/fetchium-web.service /etc/systemd/system/fetchium-web.service
sudo cp infra/***REMOVED*** /etc/systemd/system/***REMOVED***
sudo systemctl daemon-reload
```

`fetchium-dashboard.service` and `fetchium-web.service` now read `EnvironmentFile=***REMOVED***`
so the API, admin UI, dashboard, and marketing site all share the same managed runtime contract.

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
