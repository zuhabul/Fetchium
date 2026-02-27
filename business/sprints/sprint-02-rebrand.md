# Sprint 02: Code Rebrand

**Duration:** 1 week
**Theme:** Execute the HyperSearchX → Fetchium code rebrand
**Goal:** All code references updated, CI green, deployed under the Fetchium domain
**Dependency:** Sprint 01 must be complete (domain live, GitHub org created)

---

## Context

The codebase is currently named HyperSearchX with the `hsx` binary and `hsx-*` crates.
This sprint renames everything to Fetchium / `fetchium` / `fxm` (short form) while
preserving all functionality and keeping CI green throughout.

**The rebrand touches:**
- Cargo workspace: crate names, binary names, package metadata
- Source code: module paths, type names, error types, config paths
- CLI: binary name `hsx` → `fetchium`, command names
- Config file paths: `~/.hypersearchx/` → `~/.fetchium/`
- Documentation: all references
- CI/CD: GitHub Actions workflows
- Web apps: all UI and API endpoints
- npm package name
- Docker images and Compose files

---

## Pre-Rebrand Checklist

Before touching a single file:
- [ ] All tests are currently passing: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Git status is clean: `git status` shows no uncommitted changes
- [ ] Sprint 01 deliverables complete: domain live, GitHub org created
- [ ] Create a rebrand branch: `git checkout -b feat/fetchium-rebrand`

---

## Day 1: Crate & Workspace Rename

### Task 2.1 — Rename Crates in Cargo.toml

**File: `Cargo.toml` (workspace root)**
```toml
# Before
members = ["crates/hsx-core", "crates/hsx-cli", "crates/hsx-mcp", "crates/hsx-api"]

# After
members = ["crates/fetchium-core", "crates/fetchium-cli", "crates/fetchium-mcp", "crates/fetchium-api"]
```

**File: `crates/hsx-cli/Cargo.toml`**
```toml
# Before
[package]
name = "hsx-cli"

[[bin]]
name = "hsx"

# After
[package]
name = "fetchium-cli"

[[bin]]
name = "fetchium"
```

**Rename all crate directories:**
```bash
cd crates/
mv hsx-core fetchium-core
mv hsx-cli fetchium-cli
mv hsx-mcp fetchium-mcp
mv hsx-api fetchium-api
```

**Update all inter-crate dependencies:**
```toml
# In fetchium-cli/Cargo.toml
[dependencies]
fetchium-core = { path = "../fetchium-core", workspace = true }
```

### Task 2.2 — Run Initial Compile Check
```bash
export PATH="$HOME/.cargo/bin:/usr/bin:/usr/local/bin:/bin:$PATH"
cargo check 2>&1 | head -50
```
Expect errors from `use hsx_core::` imports — these get fixed in Task 2.3.

---

## Day 2: Source Code Module Renames

### Task 2.3 — Rename `hsx_core` → `fetchium_core` in Source

Every `use hsx_core::` or `extern crate hsx_core` becomes `use fetchium_core::`.

```bash
# Find all occurrences
grep -r "hsx_core" crates/ --include="*.rs" -l

# Bulk replace (review each file after)
find crates/ -name "*.rs" -exec sed -i 's/hsx_core/fetchium_core/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/hsx_cli/fetchium_cli/g' {} \;
find crates/ -name "*.rs" -exec sed -i 's/hsx_api/fetchium_api/g' {} \;
```

### Task 2.4 — Rename Type Names and Constants

Types like `HsxConfig`, `HsxError`, `HsxResult` should become `FetchiumConfig`,
`FetchiumError`, `FetchiumResult`. This is the most labour-intensive part.

**Strategy:** Use `cargo fix` where possible; manual review for ambiguous cases.

Key types to rename:
```
HsxConfig       → FetchiumConfig
HsxError        → FetchiumError
HsxResult       → FetchiumResult
HsxAuth         → FetchiumAuth
BackendId::Hsx* → BackendId::Fetchium*  (if any)
```

**Config file path:**
```rust
// Before (in config.rs)
let config_dir = dirs::home_dir().unwrap().join(".hypersearchx");

// After
let config_dir = dirs::home_dir().unwrap().join(".fetchium");
```

### Task 2.5 — Update Error Messages and Log Output

Search for any user-facing strings containing "HyperSearchX" or "hsx":
```bash
grep -r "HyperSearchX\|hypersearchx\|hsx" crates/ --include="*.rs" -n
```

Update:
- `"HyperSearchX v{}"` → `"Fetchium v{}"`
- Error messages referencing old name
- Log output referencing old binary name
- `"~/.hypersearchx/config.toml"` → `"~/.fetchium/config.toml"`

### Task 2.6 — Intermediate Compile & Test
```bash
cargo check && cargo test 2>&1 | tail -30
```
Fix any remaining compilation errors before proceeding.

---

## Day 3: CLI Commands & User-Facing Output

### Task 2.7 — CLI Binary and Help Text

The `hsx` binary becomes `fetchium`. Update in `fetchium-cli/src/main.rs`:

```rust
// Before
#[command(name = "hsx", about = "HyperSearchX — AI-powered search CLI")]

// After
#[command(name = "fetchium", about = "Fetchium — typed web fetch for AI agents")]
```

Update all `--help` descriptions, command names, and examples to reference `fetchium`
instead of `hsx`.

**Shell completions:** The completion scripts must be regenerated with the new binary name.
```rust
// Update the binary name in the completion generation command
```

### Task 2.8 — npm Package Rename

**File: `package.json` (if exists) or npm wrapper:**
```json
{
  "name": "fetchium",
  "bin": {
    "fetchium": "./bin/fetchium.js"
  }
}
```

Update all references from `hypersearchx` to `fetchium` in the npm package files.

### Task 2.9 — Environment Variable Names

Update all environment variable names:
```bash
# Before → After
HSX_ADMIN_SECRET → FETCHIUM_ADMIN_SECRET
HSX_CHROME_PATH  → FETCHIUM_CHROME_PATH
GEMINI_API_KEYS  → (keep — third-party)
```

Update in: `.env.example`, documentation, systemd service files, GitHub Actions secrets references.

---

## Day 4: Infrastructure & Configuration Files

### Task 2.10 — Systemd Service Files

Rename and update the three systemd services:
- `hsx-api.service` → `fetchium-api.service`
- `hsx-web.service` → `fetchium-web.service`
- `hsx-dashboard.service` → `fetchium-dashboard.service`

Update service file contents: `ExecStart`, `Description`, `WorkingDirectory`, environment variables.

### Task 2.11 — Docker & Compose Files

```yaml
# Before
services:
  hsx-searxng:
    container_name: hsx-searxng

# After
services:
  fetchium-searxng:
    container_name: fetchium-searxng
```

Update:
- All container names from `hsx-*` to `fetchium-*`
- Image names if we publish custom images
- Docker network names: `hsx-net` → `fetchium-net`

### Task 2.12 — Traefik Route Configuration

Update Traefik dynamic configuration:
```yaml
# Update router names, service names, middleware names
# hsx-api-router → fetchium-api-router
# api.hypersearchx.zuhabul.com → api.fetchium.com (once DNS propagates)
```

### Task 2.13 — GitHub Actions Workflows

Update all `.github/workflows/` files:
- Workflow names and job names referencing old brand
- Binary artifact names: `hsx-linux-x64.tar.gz` → `fetchium-linux-x64.tar.gz`
- GitHub release title format
- Cache keys containing `hsx`
- Secret references: `HSX_ADMIN_SECRET` → `FETCHIUM_ADMIN_SECRET` (update in GitHub Settings too)

---

## Day 5: Web Apps & Documentation

### Task 2.14 — Web App Content

In `apps/web/` and `apps/dashboard/`:
```bash
# Find all occurrences
grep -r "HyperSearchX\|hypersearchx\|hsx\|HsxConfig" apps/ --include="*.tsx" --include="*.ts" -l
```

Update:
- Page titles: `"HyperSearchX Docs"` → `"Fetchium Docs"`
- Meta descriptions and OG tags
- API endpoint references: `/api/hsx/` → `/api/fetchium/` (or keep routes stable for backwards compat)
- Import paths if any reference old crate names

### Task 2.15 — Documentation Files

```bash
# Update all markdown files
find . -name "*.md" -exec grep -l "HyperSearchX\|hypersearchx\|hsx" {} \;
```

Key files:
- `README.md` — most important; complete rewrite for Fetchium brand
- `CLAUDE.md` — update crate names, paths, commands
- `TASKS.md` — update crate references
- All `tasks/phase-*.md` files — update code examples

### Task 2.16 — Config Migration Script

Users upgrading from HyperSearchX to Fetchium need their config migrated:

```rust
// In fetchium-core/src/config.rs — on startup, check for legacy config
pub fn migrate_legacy_config() -> Result<()> {
    let old_dir = dirs::home_dir()?.join(".hypersearchx");
    let new_dir = dirs::home_dir()?.join(".fetchium");

    if old_dir.exists() && !new_dir.exists() {
        fs::rename(&old_dir, &new_dir)?;
        eprintln!("Migrated config from ~/.hypersearchx to ~/.fetchium");
    }
    Ok(())
}
```

---

## Day 6–7: Testing, CI, and Deployment

### Task 2.17 — Full Test Suite

```bash
export PATH="$HOME/.cargo/bin:/usr/bin:/usr/local/bin:/bin:$PATH"
cargo test --skip research::pipeline
cargo clippy -- -D warnings
cargo fmt --check
```

All 941+ tests must pass. Zero clippy warnings.

### Task 2.18 — Manual Smoke Tests

```bash
# Test the new binary name
./target/debug/fetchium --help
./target/debug/fetchium --version

# Test key commands still work
./target/debug/fetchium doctor
./target/debug/fetchium fetch "https://example.com"
./target/debug/fetchium ai --fast "what is rust programming language"

# Test config migration
ls ~/.fetchium/   # Should exist (migrated or created fresh)
```

### Task 2.19 — Update GitHub Secrets

In GitHub Repository Settings → Secrets → Actions:
- Rename `HSX_ADMIN_SECRET` → `FETCHIUM_ADMIN_SECRET`
- Add any new secrets required by renamed workflows

### Task 2.20 — Deploy to Production

```bash
# Rebuild and deploy
cargo build -p fetchium-cli --release
cargo build -p fetchium-api --release

# Update systemd services
sudo systemctl daemon-reload
sudo systemctl restart fetchium-api fetchium-web fetchium-dashboard

# Verify
curl https://api.fetchium.com/health
```

### Task 2.21 — Create and Merge PR

```bash
git add -A
git commit -m "feat!: rebrand HyperSearchX to Fetchium

BREAKING CHANGE: binary renamed from hsx to fetchium;
config directory moved from ~/.hypersearchx to ~/.fetchium;
auto-migration runs on first startup"

git push origin feat/fetchium-rebrand
gh pr create --title "feat!: rebrand HyperSearchX to Fetchium"
```

---

## Deliverables Checklist

- [ ] All crate directories renamed from `hsx-*` to `fetchium-*`
- [ ] Binary is now `fetchium` (not `hsx`)
- [ ] `cargo test` passes with 0 failures
- [ ] `cargo clippy -- -D warnings` passes with 0 warnings
- [ ] `fetchium --help` shows Fetchium branding
- [ ] Config migrates from `~/.hypersearchx` to `~/.fetchium` on first run
- [ ] Web apps show Fetchium branding
- [ ] Docs site updated
- [ ] CI/CD workflows updated and green
- [ ] GitHub Actions secrets renamed
- [ ] Systemd services renamed and restarted
- [ ] `https://api.fetchium.com/health` returns 200

---

## Rollback Plan

If something breaks catastrophically:
1. Keep the old branch (`main` before rebrand) for 30 days
2. Systemd service rollback: `sudo systemctl restart hsx-api` (old service still on disk)
3. Database compatibility: SQLite auth.db is not renamed — both old and new binary can read it
4. Config migration is one-way: `~/.hypersearchx` folder is renamed, not deleted

---

## Breaking Change Notice

This is a `feat!` commit (BREAKING CHANGE). The release notes must document:
- Old binary `hsx` → new binary `fetchium`
- Old env vars `HSX_*` → new env vars `FETCHIUM_*`
- Config directory migration (automatic on first run)
- npm package rename from `hypersearchx` to `fetchium`
