# Fetchium API Remediation Plan

Date: 2026-03-07

This document tracks the API issues found in public testing and the fixes applied in code.

## Issues

- [x] `/v1/estimate` was too expensive for an estimate endpoint because it performed a full page fetch.
- [x] Expensive endpoints could hang until an outer proxy/client timeout rather than failing with a clear API timeout.
- [x] Added async job submission and polling endpoints for long-running research, social, and YouTube routes.
- [x] `/v1/fetch` and `/v1/scrape` shared one handler and one usage bucket, which made alias behavior harder to reason about operationally.
- [x] Legacy MCP tool naming was removed so the active surface is Fetchium-branded only.
- [x] Add async job mode for long-running routes such as research, social, and YouTube.
- [x] Add a canonical response envelope with `request_id`, `status`, `duration_ms`, and endpoint metadata.
- [x] Add release automation and CI coverage for SDK packaging and publication readiness.
- [x] Add a canonical production deployment path in-repo.
- [x] Add app-level request tracing on REST and HTTP MCP servers.
- [ ] Add richer request controls: domain include/exclude, recency filters, geography, vertical/source selection.
- [ ] Add structured output schemas and field-level grounding/citations.
- [ ] Add deployment-side tracing for public timeout diagnosis.
- [x] Expose an HTTP MCP transport in addition to stdio.
- [ ] Deploy the HTTP MCP transport publicly.

## Fixes applied in this patch

### 1. Lightweight estimate path

`/v1/estimate` now uses an HTTP `HEAD` request and content-length heuristics with fallback defaults, instead of downloading the full document body.

Expected effect:
- Faster first byte
- Lower origin load
- Fewer estimate timeouts

### 2. App-level timeouts for expensive routes

The REST layer now wraps these endpoints in explicit timeouts and returns a clean `504 request_timeout` JSON error instead of hanging indefinitely:

- `/v1/search`
- `/v1/scrape`
- `/v1/fetch`
- `/v1/estimate`
- `/v1/research`
- `/v1/social/research`
- `/v1/social/reddit`
- `/v1/social/hackernews`
- `/v1/youtube/search`
- `/v1/youtube/analyze`

### 3. Async job endpoints for long-running work

Added async submission + polling endpoints:

- `POST /v1/research/jobs`
- `POST /v1/social/research/jobs`
- `POST /v1/social/reddit/jobs`
- `POST /v1/social/hackernews/jobs`
- `POST /v1/youtube/search/jobs`
- `POST /v1/youtube/analyze/jobs`
- `GET /v1/jobs/:id`

This keeps the existing sync routes intact while providing a production-safe path for longer operations.

### 4. Explicit scrape/fetch handlers

`/v1/scrape` and `/v1/fetch` now route to distinct wrappers with separate usage endpoint labels, while sharing the same extraction implementation.

Expected effect:
- Better observability
- Easier debugging if a proxy or deployment treats the paths differently

### 5. Fetchium MCP naming cleanup

The MCP tool surface now uses Fetchium-branded names only:

- `fetchium_search`
- `fetchium_fetch`
- `fetchium_research`
- `fetchium_estimate`
- `fetchium_expand`

### 6. Consistent response metadata

REST responses now include a shared `meta` object with:

- `request_id`
- `status`
- `endpoint`
- `duration_ms`

Search and research keep their richer metadata fields, while fetch, estimate, usage, job, admin, YouTube, and social responses now expose the same response metadata contract.

### 7. SDK release readiness

The release and CI pipelines now cover:

- npm dry-run packaging
- Python adapter builds for `fetchium-langchain` and `fetchium-crewai`
- Python adapter tests
- PyPI publication in the release workflow
- release-please version syncing for the adapter `pyproject.toml` files

### 8. Canonical deployment path

The repo now contains an explicit production deployment path:

- `infra/docker-compose.prod.yml`
- `scripts/deploy.sh`
- `.github/workflows/deploy.yml`

This removes the previous ambiguity around how the public host should be rolled out.

### 9. Request tracing

Both the REST API and HTTP MCP server now use explicit request tracing middleware so rollout failures can be diagnosed from application logs with latency and failure events.

## Next fixes

1. Deploy the HTTP MCP transport publicly.
2. Add public benchmark/evals harness across Fetchium, Tavily, Exa, Firecrawl, and Serper.
3. Diagnose and fix public deployment timeouts using deployment-side tracing.
