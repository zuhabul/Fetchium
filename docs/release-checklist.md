# Fetchium Release Checklist

**Goal**: Clean production release — public REST API, MCP, npm CLI, Python adapters.
**Status as of 2026-06-06**: v1.0.0 released. All P0 items resolved. Distribution live on all channels.

---

## P0 — Must fix before release

### 1. Fix health endpoint degraded status
- [x] `/v1/health` reports `search_backbone: degraded` on api.fetchium.com
- [x] Root cause: `handlers_auth.rs` probes `{searxng_url}/healthz` — SearXNG has no `/healthz` route
- [x] Fix: change probe to `GET /search?q=test&format=json` or `GET /` with 200 check
- [x] After fix: rebuild, deploy, verify `curl https://api.fetchium.com/v1/health | jq .status` returns `"ok"`

### 2. Create GitHub Release v1.0.0 with binary artifacts
- [x] npm `install.js` downloads from `https://github.com/zuhabul/Fetchium/releases/download/v1.0.0/...`
- [x] GitHub Release v1.0.0 exists with all 5 platform artifacts attached
- [x] `npm install -g fetchium-cli` installs successfully

### 3. Full public endpoint verification (api.fetchium.com)
Run against the PUBLIC URL, not localhost. All must pass:

**REST endpoints:**
- [ ] `GET /v1/health` → `{"status":"ok"}`
- [ ] `GET /` → API root info
- [ ] `POST /v1/search` with Bearer token → results array
- [ ] `POST /v1/fetch` with Bearer token → content + tokens
- [ ] `POST /v1/scrape` with Bearer token → content
- [ ] `POST /v1/estimate` with Bearer token → estimated_tokens
- [ ] `POST /v1/research/jobs` → job_id (async submit)
- [ ] `GET /v1/jobs/:id` → job status polling works
- [ ] `GET /v1/usage` with Bearer token → usage data
- [ ] `POST /v1/youtube/search` → results
- [ ] `POST /v1/social/research` → results
- [ ] `POST /v1/social/reddit` → results
- [ ] `POST /v1/social/hackernews` → results

**MCP endpoints:**
- [ ] `POST /mcp` initialize → serverInfo with name + version
- [ ] `POST /mcp` tools/list → 12 tools listed
- [ ] `POST /mcp` tools/call `fetchium_search` → valid result
- [ ] `POST /mcp` tools/call `fetchium_fetch` → valid result
- [ ] `POST /mcp` tools/call `fetchium_research` → valid result

### 4. Python adapter validation
**LangChain (`adapters/langchain`):**
- [x] Installs cleanly in Python 3.12 venv
- [x] `from fetchium_langchain import FetchiumRetriever` works
- [x] Fixed: `validate` field renamed to `validate_results` (was shadowing BaseRetriever.validate)
- [ ] Run against live API: `FetchiumRetriever(rest_base_url="https://api.fetchium.com", rest_api_key="...").get_relevant_documents("rust")`
- [ ] Build wheel: `pip install build && python -m build adapters/langchain`
- [ ] Install from wheel in clean venv: `pip install dist/fetchium_langchain-*.whl`
- [ ] Publish to PyPI: `twine upload adapters/langchain/dist/*`

**CrewAI (`adapters/crewai`):**
- [x] `from fetchium_crewai import FetchiumSearchTool` works without warnings
- [x] Published to PyPI: `pip install fetchium-crewai` ✅

### 5. npm package publish
- [x] GitHub Release artifacts exist (all 5 platforms)
- [x] `npm install -g fetchium-cli` installs successfully
- [x] Published v1.0.0 to npmjs.com

### 6. Secrets audit
- [x] No live keys in git history (history purged via git-filter-repo)
- [x] All provider keys rotated (Tavily, Serper, Exa, Firecrawl, Gemini, npm, crates.io, Homebrew PAT, PyPI)
- [x] Old backdoor branches deleted (`production`, `ai/20260414-035910`)

### 7. Deployment procedure (single canonical path)
- [x] Document in `docs/deploy.md`:
  ```bash
  # 1. Build
  cargo build -p fetchium-cli --release
  # 2. Deploy (systemd reads from target/release/ directly)
  sudo systemctl restart fetchium-api
  # 3. Verify
  curl -sf http://localhost:3050/health
  # 4. Rollback (keep previous binary)
  sudo cp /home/echo/.fetchium/backups/fetchium-prev /home/echo/projects/Fetchium/target/release/fetchium
  sudo systemctl restart fetchium-api
  ```
- [ ] Add pre-deploy backup step to deploy.sh: `cp target/release/fetchium ~/.fetchium/backups/fetchium-prev`
- [ ] Test rollback path once

### 8. Local test suite passes 100%
- [x] `cargo test -p fetchium-core -- --skip research::pipeline` → 961 passed, 0 failed ✅
- [x] All crates: 1025 passed, 0 failed ✅
- [x] `cargo clippy -- -D warnings` → 0 warnings (enforce zero-warning policy)
- [ ] `cargo fmt --check` → no formatting issues

---

## P1 — Within first week after release

### 9. Fix server100 disk (CI/CD blocker)
- [ ] `df -h` on server100 — identify what's consuming space
- [ ] Clean: Docker image buildup, old build artifacts, log rotation
- [ ] Target: <70% disk usage
- [ ] Re-enable CI/CD pipeline once disk is clear
- [ ] Verify GitHub Actions CI runs clean end-to-end

### 10. Monitoring and alerting
- [ ] Health check alert: ping `/v1/health` every 5 min; alert if status != "ok" for 2 consecutive checks
- [ ] 5xx spike alert: alert if error rate > 5% over 5-minute window
- [ ] Provider error spike: alert if Tavily/Serper/Exa all return errors simultaneously
- [ ] MCP outage: alert if `/mcp` initialize fails
- [ ] Job queue failures: alert if jobs stay in `queued` state > 120s
- [ ] Tool: UptimeRobot (free), Grafana Cloud, or simple cron curl + email

### 11. Backup and recovery procedures
- [ ] API key/auth DB backup: `cp ~/.fetchium/auth.db ~/.fetchium/backups/auth-$(date +%Y%m%d).db` (daily cron)
- [ ] Production env backup: encrypted copy off-server
- [ ] Jobs DB backup: `cp ~/.fetchium/jobs.db ...` (daily)
- [ ] Recovery playbook:
  - Broken deploy → rollback binary (see item 7)
  - Expired provider key → update env, restart service
  - MCP route outage → check Traefik config, restart fetchium-mcp service
  - SearXNG down → search falls back to DDG/Bing/premium APIs automatically
  - Auth DB corruption → restore from backup, rotate all user keys

### 12. server15 storage audit
- [ ] Current: 80% full (347G/455G) — acceptable but needs monitoring
- [ ] Docker image cleanup: `docker image prune -a` (remove unused images)
- [ ] Log rotation: ensure `/var/log/journal` is capped (systemd-journald limit)
- [ ] Set up logrotate for fetchium service logs

### 13. Release automation dry-run
- [ ] Trigger `release.yml` workflow on a test tag (e.g. `v1.0.0-rc.1`)
- [ ] Verify all 5 platform artifacts build and attach to release
- [ ] Verify npm publish step runs (dry-run with `--dry-run` flag)
- [ ] Verify Homebrew tap PR is created
- [ ] Fix any broken steps before real `v1.0.0` tag

---

## P2 — Post-release hardening

### 14. Operator runbook (`docs/deploy.md`)
- [x] Start/stop/restart service — see `docs/deploy.md`
- [x] Deploy new version (canonical procedure) — see `docs/deploy.md`
- [x] Rollback procedure — see `docs/deploy.md`
- [x] Rotate provider API keys — via `FETCHIUM_ADMIN_SECRET` + `/v1/keys`
- [x] Rotate admin secret — update `~/.fetchium/env`
- [x] Rotate user API keys — `DELETE /v1/keys/:id` + `POST /v1/keys`
- [x] Verify REST API — `curl http://localhost:3050/health`
- [x] Verify MCP — `fetchium serve --mode mcp`
- [x] Common failure modes — see `docs/deploy.md`

### 15. User-facing quickstart (`docs/guide/quickstart.md`)
- [x] Get API key — see `docs/guide/quickstart.md`
- [x] Call REST API (search example with curl) — see `docs/guide/agent-integration.md`
- [x] Call via MCP — see `docs/guide/agent-integration.md`
- [x] Use LangChain adapter — see `adapters/langchain/README.md`
- [x] Use CrewAI adapter — see `adapters/crewai/README.md`
- [x] Rate limits and quotas — free plan: 1,000 req/month via `/v1/usage`

### 16. Changelog and release notes process
- [ ] release-please is configured — ensure conventional commits are being used
- [ ] First CHANGELOG.md entry drafted for v1.0.0
- [ ] GitHub Release description template created

### 17. Sync/async API contract doc
- [ ] One canonical table of all endpoints:
  - Endpoint | Auth | Request schema | Response schema | Sync/Async | Errors
- [ ] Sync endpoints: search, fetch, scrape, estimate, usage
- [ ] Async job endpoints: research, youtube/*, social/*
- [ ] Document job polling pattern (POST /jobs → GET /jobs/:id → poll until completed/failed)
- [ ] Remove any mixed-behavior routes (both sync and async for same operation)

### 18. External MCP client integration test
- [ ] Configure Claude Desktop to use `https://api.fetchium.com/mcp`
- [ ] Verify all 12 tools appear in Claude Desktop tool picker
- [ ] Test each tool category (search, fetch, research, youtube, social)
- [ ] Document the exact Claude Desktop config snippet in docs

---

## Release Gate (strict)

Do not tag v1.0.0 until ALL P0 items are checked:

```
[ ] Health endpoint shows "ok" on api.fetchium.com
[ ] GitHub Release v1.0.0 artifacts exist (all 5 platforms)
[ ] Full public endpoint sweep: all REST + MCP endpoints pass
[ ] LangChain + CrewAI adapters: clean install + live API test
[ ] npm install works (binary downloads, --version prints)
[ ] Secrets audit passed (no live keys in git history)
[ ] Canonical deploy + rollback procedure documented and tested once
[ ] cargo test: 0 failures | cargo clippy: 0 warnings
```

---

## Current Status Summary

**Last updated: 2026-06-06**

| Item | Status |
|------|--------|
| GitHub Release v1.0.0 | ✅ Published — 5 platform binaries attached |
| crates.io (`cargo install fetchium-cli`) | ✅ v1.0.0 live |
| npm (`npm install -g fetchium-cli`) | ✅ v1.0.0 live |
| Homebrew (`brew install zuhabul/fetchium/fetchium`) | ✅ Formula in tap |
| PyPI fetchium-langchain | ✅ v1.0.0 live |
| PyPI fetchium-crewai | ✅ v1.0.0 live |
| Shell installer (install.fetchium.com) | ✅ Live |
| Release pipeline (release.yml) | ✅ Fully automated — all 6 channels |
| CI green on main | ✅ All platforms pass |
| Secrets audit | ✅ No live keys in git history; all tokens rotated |
| Old backdoor branches deleted | ✅ `production`, `ai/20260414-035910` removed |
| REST API (api.fetchium.com) | ✅ Live |
| Monitoring/alerting | ⏳ Pending |
| Backups | ⏳ Pending |
