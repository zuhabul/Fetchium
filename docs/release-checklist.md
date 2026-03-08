# Fetchium Release Checklist

**Goal**: Clean production release — public REST API, MCP, npm CLI, Python adapters.
**Status as of 2026-03-08**: Public beta viable. Clean release blocked by items below.

---

## P0 — Must fix before release

### 1. Fix health endpoint degraded status
- [x] `/v1/health` reports `search_backbone: degraded` on api.fetchium.com
- [x] Root cause: `handlers_auth.rs` probes `{searxng_url}/healthz` — SearXNG has no `/healthz` route
- [x] Fix: change probe to `GET /search?q=test&format=json` or `GET /` with 200 check
- [x] After fix: rebuild, deploy, verify `curl ***REMOVED***/v1/health | jq .status` returns `"ok"`

### 2. Create GitHub Release v1.0.0 with binary artifacts
- [ ] npm `install.js` downloads from `https://github.com/zuhabul/fetchium/releases/download/v1.0.0/...`
- [ ] No v1.0.0 release exists → every `npm install -g fetchium` 404s (gracefully degraded but broken)
- [ ] Action: trigger release workflow OR manually create release tag and upload artifacts:
  - `fetchium-linux-x64.tar.gz`
  - `fetchium-linux-arm64.tar.gz`
  - `fetchium-darwin-x64.tar.gz`
  - `fetchium-darwin-arm64.tar.gz`
  - `fetchium-win-x64.zip`
- [ ] Verify: `npm install -g fetchium` installs and `fetchium --version` works

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
- [ ] Run against live API: `FetchiumRetriever(rest_base_url="***REMOVED***", rest_api_key="...").get_relevant_documents("rust")`
- [ ] Build wheel: `pip install build && python -m build adapters/langchain`
- [ ] Install from wheel in clean venv: `pip install dist/fetchium_langchain-*.whl`
- [ ] Publish to PyPI: `twine upload adapters/langchain/dist/*`

**CrewAI (`adapters/crewai`):**
- [ ] Verify clean install: `pip install -e adapters/crewai[rest]`
- [ ] `from fetchium_crewai import FetchiumSearchTool` works without warnings
- [ ] Run against live API
- [ ] Build wheel + install test
- [ ] Publish to PyPI

### 5. npm package publish
- [ ] GitHub Release artifacts exist (see item 2)
- [ ] `npm pack` produces clean tarball ✅ (already verified: 3.3KB)
- [ ] Test: `npm install -g ./fetchium-1.0.0.tgz` in clean Docker container
- [ ] `fetchium --version` works after install
- [ ] `npm publish` with valid NPM_TOKEN

### 6. Secrets audit
- [ ] `git log --all -p | grep -i 'api_key\|secret\|password\|token' | grep '+' | head -50` — verify no live keys in history
- [ ] `~/.fetchium/env` — confirm not committed to git
- [ ] DataImpulse credentials — confirm only in env file
- [ ] Admin secret (`***REMOVED***`) — confirm only in env file, not docs/logs
- [ ] All provider keys (Tavily, Serper, Exa, Firecrawl, Gemini) — confirm rotation policy documented

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

### 14. Operator runbook (`docs/runbook.md`)
- [ ] Start/stop/restart service
- [ ] Deploy new version (canonical procedure)
- [ ] Rollback procedure
- [ ] Rotate provider API keys
- [ ] Rotate admin secret
- [ ] Rotate user API keys
- [ ] Verify REST API
- [ ] Verify MCP
- [ ] Common failure modes and their fixes

### 15. User-facing quickstart (`docs/quickstart.md`)
- [ ] Get API key (admin curl command)
- [ ] Call REST API (search example with curl + Python)
- [ ] Call via MCP (Claude Desktop config)
- [ ] Use LangChain adapter (code snippet)
- [ ] Use CrewAI adapter (code snippet)
- [ ] Rate limits and quotas table

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
- [ ] Configure Claude Desktop to use `***REMOVED***/mcp`
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

| Item | Status |
|------|--------|
| REST API serving (localhost) | ✅ Live |
| Public API (api.fetchium.com) | ✅ Live |
| Health: auth_store | ✅ ok |
| Health: search_backbone | ⚠️ degraded (fix in progress) |
| Benchmark quality | 🔄 Running (target: beat 8.76 baseline) |
| LangChain adapter | ✅ Installs, field warning fixed |
| CrewAI adapter | 🔄 Verifying |
| npm pack | ✅ Works |
| npm install (from release) | ❌ 404 — no GitHub Release yet |
| GitHub Release v1.0.0 | ❌ Does not exist |
| Public endpoint sweep | 🔄 In progress |
| Secrets audit | ⏳ Pending |
| Canonical deploy doc | ⏳ Pending |
| server100 disk | ❌ Full, CI/CD broken |
| server15 disk | ⚠️ 80% (monitor) |
| Monitoring/alerting | ⏳ Pending |
| Backups | ⏳ Pending |
