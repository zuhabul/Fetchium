# Fetchium — Commercial Launch Task Checklist
## Complete product-to-market execution plan

> **Engine Status**: 100% complete (44K LoC, 883 tests, 17/17 algorithms, 20/20 systems, 26 CLI commands)
> **What's missing**: REST API auth/routes, landing page, user dashboard, SDKs, billing, docs, deployment

---

## LEGEND
- `[ ]` Not started
- `[~]` In progress
- `[x]` Complete
- `🔴` Blocker (must be done before next phase)
- `🟡` Important (affects quality)
- `🟢` Nice to have

---

## PHASE A: PRODUCTION REST API
**Goal**: hsx-api exposes the full engine via authenticated HTTP endpoints
**Stack**: Rust + axum + PostgreSQL + Redis
**ETA**: 1-2 weeks

### A1: Database & Schema 🔴
- [ ] Add `sqlx` + `tokio-postgres` to `crates/hsx-api/Cargo.toml`
- [ ] Create `infra/migrations/` directory
- [ ] Migration `001_users.sql` — `users` table (id, email, password_hash, created_at, plan_tier)
- [ ] Migration `002_api_keys.sql` — `api_keys` table (id, user_id, key_hash, name, created_at, last_used_at, revoked_at)
- [ ] Migration `003_usage_logs.sql` — `usage_logs` table (id, api_key_id, endpoint, tokens_used, duration_ms, status, created_at)
- [ ] Migration `004_rate_limits.sql` — `rate_limits` table (api_key_id, window_start, request_count)
- [ ] Connection pool setup (max 20 connections, health checks)
- [ ] `sqlx::migrate!()` macro integration

### A2: Authentication Middleware 🔴
- [ ] `POST /v1/auth/register` — email + password → user creation
- [ ] `POST /v1/auth/login` → JWT access token (7d) + refresh token
- [ ] `POST /v1/auth/refresh` → new access token
- [ ] Bearer token extractor middleware for axum
- [ ] API key format: `hsx_` + 32-char hex (256-bit security)
- [ ] Key hash: SHA-256 before DB storage (never store raw)
- [ ] `POST /v1/keys` — create key (with name, optional expiry)
- [ ] `GET /v1/keys` — list keys (masked: `hsx_xxxx...xxxx`)
- [ ] `DELETE /v1/keys/:id` — revoke key
- [ ] Middleware: extract `Authorization: Bearer hsx_xxx` → look up user in DB

### A3: Rate Limiting 🔴
- [ ] Add `redis` crate to Cargo.toml
- [ ] Sliding window counter in Redis: `rate:{key_id}:{window}` → incr + expire
- [ ] Plan limits: Free=60/min, Starter=200/min, Pro=500/min
- [ ] `429 Too Many Requests` with `Retry-After` header
- [ ] Monthly quota tracking (soft cutoff with grace period)

### A4: Core Search Endpoints 🔴
- [ ] `POST /v1/search` — wire to `SearchOrchestrator::search()`
  - Query params: `query`, `backends[]`, `max_results`, `extract_content`, `token_budget`, `pds_tier`
  - Returns: `SearchResult` JSON (meta + items array)
- [ ] `POST /v1/scrape` — wire to `ExtractionPipeline::extract()`
  - Body: `{ "url": "...", "formats": ["markdown", "html"], "token_budget": 4096 }`
  - Returns: content in requested formats
- [ ] `POST /v1/agent/search` — wire to agent-optimized output
  - Returns: `AgentSearchResult` with evidence graph + citations
- [ ] `POST /v1/crawl` — wire to recursive crawl (new feature)
  - Body: `{ "url": "...", "max_pages": 50, "include_paths": [], "exclude_paths": [] }`
  - Returns: array of scraped pages
- [ ] `GET /v1/health` — liveness + readiness + version
- [ ] `GET /v1/usage` — current key's usage stats (requests today, this month, quota remaining)

### A5: Advanced Endpoints 🟡
- [ ] `POST /v1/research` — wire to AMRS research pipeline
  - Body: `{ "query": "...", "depth": "quick|standard|deep" }`
  - Returns: research report with citations
- [ ] `POST /v1/monitor` — register URL for change monitoring
  - Body: `{ "url": "...", "check_interval_hours": 24 }`
  - Returns: monitor ID + webhook URL
- [ ] `GET /v1/monitor/:id` — get monitoring results + diffs
- [ ] `DELETE /v1/monitor/:id` — stop monitoring
- [ ] `POST /v1/youtube/search` — YouTube search + metadata
- [ ] `POST /v1/agent/research` — agent-optimized research

### A6: API Middleware Stack 🔴
- [ ] CORS headers (configurable whitelist via config)
- [ ] Request ID (`X-Request-ID` header, propagated to logs)
- [ ] Structured JSON request logging (method, path, status, duration, key_id)
- [ ] 30-second timeout middleware
- [ ] 10MB payload size limit
- [ ] Gzip response compression
- [ ] OpenAPI spec auto-generation (`utoipa` crate)
- [ ] Serve spec at `GET /v1/openapi.json`

### A7: Error Handling 🔴
- [ ] RFC 7807 Problem+JSON format for all errors
  ```json
  { "type": "...", "title": "...", "status": 429, "detail": "..." }
  ```
- [ ] Map `HsxError` variants → HTTP status codes
- [ ] Validation error responses with field-level detail
- [ ] Never expose internal stack traces in production

---

## PHASE B: MONOREPO SETUP
**Goal**: Add Node.js workspace alongside Rust workspace
**Stack**: pnpm + Turborepo + Next.js 15

### B1: Workspace Config 🔴
- [x] Create `package.json` at repo root (Bun workspaces)
- [x] Create `turbo.json` with build pipeline (build, dev, test, lint)
- [x] Add `.npmrc`
- [x] Add `apps/` and `packages/` directories
- [x] Update root `.gitignore` (node_modules/, .turbo/, .next/)

### B2: Shared UI Package 🟡
- [ ] `packages/ui/package.json` (`@hsx/ui`)
- [ ] Install shadcn/ui components (button, card, badge, input, etc.)
- [ ] Design tokens: brand colors, typography, spacing
- [ ] Export from `packages/ui/index.ts`

### B3: Shared Config Package 🟡
- [ ] `packages/config/package.json` (`@hsx/config`)
- [ ] `eslint.config.js` (shared ESLint)
- [ ] `tsconfig.base.json` (shared TypeScript)
- [ ] `tailwind.config.ts` (shared Tailwind)

---

## PHASE C: LANDING PAGE
**Goal**: World-class developer-focused marketing site
**Stack**: Next.js 15 + Tailwind + Framer Motion
**Hosting**: Vercel or `fetchium.server.zuhabul.com` (traefik)

### C1: Project Setup 🔴
- [x] `apps/web/package.json` (Next.js 15, Tailwind, framer-motion, lucide-react)
- [x] `next.config.ts` — image domains, security headers
- [x] `app/layout.tsx` — root layout, Geist fonts, OG metadata
- [x] Tailwind config with brand colors

### C2: Home Page (/) 🔴
- [x] **Navbar** — responsive, mobile menu, Get API Key CTA
- [x] **Hero section** — headline, subheadline, stats, CTA buttons, quick install
- [x] **Feature grid** — 9 cards: federation, CEP, HyperFusion, PIE, evidence, monitoring, YouTube, token budget, resilience
- [x] **Comparison table** — Fetchium vs Firecrawl vs Tavily vs Exa (15 features)
- [x] **Code examples** — TypeScript / Python / curl tabs with copy button
- [x] **Pricing section** — 4-tier cards with feature lists
- [x] **Footer** — 4-column nav, GitHub/Twitter links

### C3: Pricing Page (/pricing) 🔴
- [x] Dedicated /pricing page

### C4: Comparison Pages 🟡 (SEO goldmines)
- [x] `/compare/firecrawl` — "Fetchium vs Firecrawl"
- [ ] `/compare/tavily` — "Fetchium vs Tavily"
- [ ] `/compare/exa` — "Fetchium vs Exa"
- [ ] `/alternatives/firecrawl` — "Best Firecrawl alternatives"
- [ ] Each page: feature matrix, use case guide, migration instructions

### C5: Status Page (/status) 🟡
- [ ] Live health status (API, search backends, SearXNG)
- [ ] Uptime history (90 days)
- [ ] Incident history

### C6: SEO & Analytics 🟡
- [ ] `sitemap.xml` generation
- [ ] OG meta tags on all pages
- [ ] Twitter card meta
- [ ] Structured data (JSON-LD)
- [ ] Plausible analytics (self-hosted on server) — privacy-friendly
- [ ] Google Search Console verification

---

## PHASE D: USER DASHBOARD
**Goal**: Self-service portal for API key management, usage, billing
**Stack**: Next.js 15 (App Router) + NextAuth + shadcn/ui + Recharts + Stripe

### D1: Authentication 🔴
- [ ] `next-auth` v5 setup with credentials + OAuth providers
- [ ] **Email/password** provider (bcrypt hashing, min 12 chars)
- [ ] **GitHub OAuth** provider (critical for dev audience)
- [ ] **Google OAuth** provider
- [ ] Email verification flow (nodemailer → postmark)
- [ ] Password reset flow
- [ ] Session: JWT httpOnly cookies (7-day expiry)
- [ ] Protected routes middleware (`middleware.ts`)

### D2: Dashboard Layout 🔴
- [x] `app/(dashboard)/layout.tsx` — sidebar + header
- [x] Sidebar nav: Overview, API Keys, Usage, Billing, Playground, Settings
- [x] Header: user avatar, current plan badge, remaining quota bar

### D3: Overview Page (/dashboard) 🔴
- [x] Stats cards: requests today, this month, avg latency, success rate
- [x] Quota progress bar with upgrade CTA
- [x] Recent API requests table

### D4: API Keys Page (/dashboard/keys) 🔴
- [x] List all keys (masked: `hsx_xxxx...****`, name, created, last used)
- [x] Create key form → shows full key ONCE with copy button
- [x] Revoke key (confirmation dialog)

### D5: Usage Analytics (/dashboard/usage) 🔴
- [x] Bar chart: requests per day (30 days, server-rendered SVG)
- [x] Breakdown by endpoint
- [x] Summary stats (tokens, latency, success rate)

### D6: Billing Page (/dashboard/billing) 🔴
- [x] Current plan display
- [x] Upgrade options (Starter/Pro/Enterprise)
- [x] Invoice history section

### D7: Playground (/dashboard/playground) 🟡
- [x] Endpoint selector + JSON body editor
- [x] Send button with loading state
- [x] Response viewer

### D8: Settings (/dashboard/settings) 🟡
- [x] Change display name, email, password
- [x] Danger zone (delete account)

---

## PHASE E: BILLING & STRIPE
**Goal**: Working subscription system with metered overages

### E1: Stripe Setup 🔴
- [ ] Create Stripe account
- [ ] Create products: Starter ($19/mo), Pro ($79/mo), Enterprise ($299/mo)
- [ ] Create metered billing for overages (per 1,000 additional requests)
- [ ] Stripe webhook handler (`POST /webhooks/stripe`)
  - `customer.subscription.created` → set user plan
  - `customer.subscription.deleted` → downgrade to free
  - `invoice.payment_failed` → suspend key (with grace period)
  - `invoice.payment_succeeded` → reset monthly quota
- [ ] Webhook secret verification (Stripe-Signature header)

### E2: Checkout Flow 🔴
- [ ] Stripe Checkout session creation (redirect flow)
- [ ] Success page (activate plan)
- [ ] Cancel page (stay on current plan)
- [ ] Customer portal (manage payment method, cancel)

### E3: Quota Enforcement 🔴
- [ ] Check quota on every API request (via Redis counter)
- [ ] Soft limit: at 90% → email warning
- [ ] Hard limit: at 100% → 429 with "quota exceeded" error
- [ ] Overage grace: 10% buffer before hard stop (prevents single-request surprise)
- [ ] Paid overage option: auto-charge $0.50 per 1,000 extra requests

---

## PHASE F: JAVASCRIPT/TYPESCRIPT SDK
**Goal**: npm-publishable SDK (`@fetchium/sdk`)

### F1: Package Setup 🔴
- [ ] `packages/sdk-js/package.json` (`@fetchium/sdk`)
- [ ] TypeScript source + types export
- [ ] Build: tsup (ESM + CJS + `.d.ts`)
- [ ] Package: `npm publish --access public`
- [ ] CI: auto-publish on git tag

### F2: Client Implementation 🔴
```typescript
class Fetchium {
  constructor(config: { apiKey: string; baseUrl?: string })

  // Core methods
  async search(query: string | SearchOptions): Promise<SearchResult>
  async scrape(url: string, options?: ScrapeOptions): Promise<ScrapeResult>
  async crawl(url: string, options?: CrawlOptions): Promise<CrawlResult>
  async research(query: string | ResearchOptions): Promise<ResearchResult>
  async monitor(url: string, options?: MonitorOptions): Promise<Monitor>

  // Specialized
  async agentSearch(query: string, options?: AgentOptions): Promise<AgentSearchResult>
  async youtubeSearch(query: string): Promise<YouTubeResult[]>
}
```
- [ ] All TypeScript types generated from OpenAPI spec
- [ ] Automatic retry (3 attempts, exponential backoff)
- [ ] Rate limit handling (`Retry-After` header)
- [ ] Streaming support (SSE for research endpoint)
- [ ] `AbortController` / timeout support
- [ ] Detailed error types (`AuthError`, `QuotaError`, `RateLimitError`, etc.)
- [ ] README with 10+ usage examples

### F3: SDK Tests 🟡
- [ ] Unit tests (vitest)
- [ ] Integration tests (against local API)
- [ ] Mock mode for testing (no real API calls)

---

## PHASE G: PYTHON SDK
**Goal**: PyPI-publishable SDK (`fetchium`)

### G1: Package Setup 🔴
- [ ] `packages/sdk-python/pyproject.toml`
- [ ] `pip install fetchium`
- [ ] Python 3.9+, type hints throughout
- [ ] Sync client (httpx) + async client (httpx async)
- [ ] Pydantic v2 models for all types

### G2: Implementation 🔴
```python
from fetchium import Fetchium

# Sync
hsx = Fetchium(api_key="hsx_...")
results = hsx.search("rust async programming")

# Async
async with Fetchium(api_key="hsx_...") as hsx:
    results = await hsx.search("rust async programming")
```
- [ ] All endpoints matching JS SDK
- [ ] Pydantic models with validation
- [ ] Retry + rate limit handling
- [ ] Context manager support
- [ ] `pyproject.toml` + `uv publish`

---

## PHASE H: DEVELOPER DOCS
**Goal**: Mintlify-quality docs. First API call in <5 minutes.
**Stack**: Nextra or Mintlify (self-hosted) or Docusaurus

### H1: Docs Site Setup 🔴
- [ ] `apps/docs/` — Nextra (Next.js-based docs)
- [ ] `apps/docs/pages/index.mdx` — landing
- [ ] Navigation config (sidebar, top nav)
- [ ] Code syntax highlighting
- [ ] Light/dark mode
- [ ] Search (Pagefind or Algolia)

### H2: Content — Core 🔴
- [ ] **Quickstart** — 3 steps: get key, install SDK, first request (5 min max)
- [ ] **Authentication** — bearer tokens, key management, security
- [ ] **Rate Limits** — per-plan limits, headers, retry guidance
- [ ] **Errors** — all error codes with examples and fixes

### H3: Content — API Reference 🔴
- [ ] Auto-generated from OpenAPI spec (every endpoint)
- [ ] `/v1/search` — full params, response schema, examples
- [ ] `/v1/scrape` — formats, options, response
- [ ] `/v1/crawl` — limits, pagination, async jobs
- [ ] `/v1/research` — depth levels, response format
- [ ] `/v1/monitor` — webhooks, diff format, pricing
- [ ] Request/response examples in 3 languages (curl / TS / Python)

### H4: Content — Guides 🟡
- [ ] **Self-hosting guide** — Docker compose, environment vars, scaling
- [ ] **Migrating from Firecrawl** — API compatibility, feature mapping
- [ ] **Migrating from Tavily** — use cases, pricing comparison
- [ ] **Building a RAG pipeline** — step-by-step with LangChain
- [ ] **Building an AI agent** — with Claude, OpenAI, CrewAI
- [ ] **Monitoring competitor prices** — use case tutorial
- [ ] **Academic research** — ArXiv + Wikipedia + Scholar integration

### H5: SDK Docs 🟡
- [ ] TypeScript SDK reference (auto-generated from JSDoc)
- [ ] Python SDK reference (auto-generated from docstrings)
- [ ] Installation instructions for each

---

## PHASE I: INFRASTRUCTURE & DEPLOYMENT
**Goal**: Production-ready deployment on server.zuhabul.com

### I1: Docker Stack 🔴
- [ ] `Dockerfile` for hsx-api (multi-stage: builder + runtime, ~50MB image)
- [ ] `infra/docker-compose.yml` — development stack
  ```yaml
  services:
    hsx-api:     # port 8000
    postgres:    # port 5432 (internal only)
    redis:       # port 6379 (internal only)
    searxng:     # port 4040 (already running)
  ```
- [ ] `infra/docker-compose.prod.yml` — production stack (volumes, resource limits)
- [ ] `infra/nginx/fetchium.conf` — reverse proxy + SSL
- [ ] Health checks on all containers
- [ ] Volume mounts for PostgreSQL data persistence
- [ ] `.env.example` with all required environment variables

### I2: CI/CD Pipeline 🔴
- [ ] `.github/workflows/ci.yml`:
  - Trigger: PR + push to main
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - Build success badge in README
- [ ] `.github/workflows/release.yml`:
  - Trigger: git tag `v*`
  - Build release binary (linux-x86_64)
  - Docker build + push to ghcr.io
  - GitHub Release with binary attachment
  - Auto-deploy to server
- [ ] `.github/workflows/deploy.yml`:
  - SSH to server, `docker compose pull && docker compose up -d`

### I3: Traefik Integration 🔴
- [ ] Add `hsx-api` to `/etc/traefik/dynamic/services.yml`:
  - `api.fetchium.server.zuhabul.com` → `localhost:8000`
  - `fetchium.server.zuhabul.com` → Next.js app
- [ ] SSL certificate auto-provisioning
- [ ] Authelia bypass for API endpoints (public)
- [ ] Rate limit headers passthrough

### I4: Monitoring & Observability 🟡
- [ ] Prometheus metrics endpoint (`GET /metrics`) in hsx-api
  - Total requests counter (by endpoint, status)
  - Latency histogram (P50, P95, P99)
  - Active connections gauge
  - Queue depth gauge
- [ ] Grafana dashboard (add to existing Grafana)
- [ ] Alert rules: error rate >5%, P95 latency >5s, quota >90%
- [ ] Sentry error tracking (DSN in env var)
- [ ] Log shipping to structured file (JSON lines format)

### I5: Database Ops 🟡
- [ ] PostgreSQL automated backups (pg_dump daily → S3 or local)
- [ ] Redis persistence (RDB snapshots)
- [ ] Migration rollback scripts
- [ ] DB seeding script (dev environment)

---

## PHASE J: GO-TO-MARKET

### J1: Repository & Open Source 🔴
- [ ] GitHub README complete overhaul:
  - Badges: CI, tests passing, license, stars
  - Feature overview with GIF demo
  - Quick install: `cargo install hsx-cli`
  - Comparison table vs Firecrawl/Tavily
  - "Hosted API" badge linking to pricing
- [ ] GitHub topics: `search`, `ai`, `scraping`, `rust`, `llm`, `mcp`
- [ ] Issue templates (bug, feature, question)
- [ ] PR template
- [ ] `CONTRIBUTING.md`
- [ ] `SECURITY.md`
- [ ] `LICENSE` (Apache-2.0 for CLI/core, AGPL-3.0 for cloud service — like Firecrawl)

### J2: Community 🟡
- [ ] Discord server setup (channels: #general, #api, #self-hosting, #show-and-tell)
- [ ] Twitter/X `@fetchium` account
- [ ] Dev.to blog connected to GitHub (auto-post)

### J3: Pre-Launch Content 🔴
- [ ] Blog post 1: "Why we built Fetchium: Firecrawl isn't enough"
- [ ] Blog post 2: "Fetchium benchmarks vs Firecrawl, Tavily, Exa"
- [ ] Blog post 3: "How to build a RAG pipeline with Fetchium"
- [ ] Blog post 4: "Running your own unlimited search API for free"
- [ ] Record demo video (screen capture, voice-over): 5 min max

### J4: Launch Day 🔴
- [ ] Hacker News "Show HN" post (Tuesday-Thursday 9am ET for max visibility)
  - Title: "Show HN: Fetchium – open-source Firecrawl alternative, 11 backends, 8-signal ranking"
- [ ] Product Hunt launch (same day, cross-post community)
- [ ] Post to r/MachineLearning, r/LocalLLaMA, r/rust, r/programming
- [ ] Tweet from @fetchium with demo video
- [ ] Email beta waitlist users (early access)

### J5: Post-Launch 🟡
- [ ] Submit to awesome-selfhosted
- [ ] Submit to awesome-llm-tools
- [ ] LangChain integration (open PR to LangChain repo)
- [ ] CrewAI integration
- [ ] Anthropic MCP marketplace listing
- [ ] Respond to every HN comment within 2 hours

---

## PHASE K: ADVANCED FEATURES (POST-LAUNCH)

### K1: Streaming Responses 🟡
- [ ] SSE endpoint for long research jobs
- [ ] `POST /v1/research/stream` → `text/event-stream`
- [ ] Progress events: `search_complete`, `extract_complete`, `rank_complete`, `done`
- [ ] SDK support for streaming

### K2: Batch Processing 🟡
- [ ] `POST /v1/batch/scrape` — up to 100 URLs in one request
- [ ] Async job queue (Redis + tokio)
- [ ] `GET /v1/jobs/:id` — job status + results
- [ ] Webhook notification on completion

### K3: Structured Extraction (AI-powered) 🟡
- [ ] `POST /v1/extract` — extract structured JSON from URL using schema
  ```json
  { "url": "...", "schema": { "price": "number", "title": "string" } }
  ```
- [ ] Uses Claude/GPT to extract fields matching schema
- [ ] Confidence score per field
- [ ] Better than Firecrawl's /agent (uses our ranking + evidence)

### K4: Embeddings Endpoint 🟡
- [ ] `POST /v1/embed` — generate embeddings for text/documents
- [ ] Backend: fastembed (ONNX) for self-hosted, optional OpenAI-compatible API
- [ ] Semantic search over previously crawled content

### K5: Vector Search 🟡
- [ ] Index crawled documents automatically
- [ ] `POST /v1/search/semantic` — semantic similarity search
- [ ] Hybrid: BM25 + semantic (HyperFusion already supports this)

---

## IMMEDIATE NEXT STEPS (This Week)

### Priority Order
1. 🔴 **Phase I1** — Docker Compose for full stack (API + PG + Redis + SearXNG)
2. 🔴 **Phase A1** — PostgreSQL schema + migrations
3. 🔴 **Phase A2** — Authentication (register/login/API keys)
4. 🔴 **Phase A4** — `/v1/search` and `/v1/scrape` endpoints live
5. 🔴 **Phase B1** — Monorepo Node.js workspace setup
6. 🔴 **Phase C1** — Landing page scaffold
7. 🔴 **Phase D1** — Dashboard auth (register + login)

### In Parallel
- Landing page (Phase C) — can build without API ready
- Dashboard scaffold (Phase D) — mock data OK for initial build

---

## ENVIRONMENT VARIABLES CHECKLIST

```bash
# API Server
HSX_DATABASE_URL=postgresql://hsx:password@localhost:5432/fetchium
HSX_REDIS_URL=redis://localhost:6379
HSX_JWT_SECRET=<64-char random hex>
HSX_API_KEY_PREFIX=hsx_

# Stripe
STRIPE_SECRET_KEY=sk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_STARTER=price_...
STRIPE_PRICE_PRO=price_...
STRIPE_PRICE_ENTERPRISE=price_...

# Email
SMTP_HOST=smtp.postmarkapp.com
SMTP_PORT=587
SMTP_USER=...
SMTP_PASS=...
FROM_EMAIL=noreply@fetchium.com

# Search backends
BRAVE_API_KEY=...
GITHUB_TOKEN=...
SEARXNG_URL=***REMOVED***

# AI providers (optional - users bring their own)
HSX_ANTHROPIC_KEY=...
HSX_OPENAI_KEY=...

# Observability
SENTRY_DSN=...
RUST_LOG=info

# Next.js (dashboard)
NEXTAUTH_SECRET=<32-char random>
NEXTAUTH_URL=https://app.fetchium.com
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
GOOGLE_CLIENT_ID=...
GOOGLE_CLIENT_SECRET=...
NEXT_PUBLIC_API_URL=***REMOVED***
```

---

## TECH STACK SUMMARY

| Layer | Technology | Reason |
|-------|-----------|--------|
| **Core engine** | Rust + tokio + axum | Existing 44K LoC, production-grade |
| **Database** | PostgreSQL + sqlx | Auth, keys, usage logs |
| **Cache/Queue** | Redis | Rate limiting, job queue |
| **Self-hosted search** | SearXNG (Docker) | Free, unlimited, 9+ engines |
| **Landing page** | Next.js 15 + Tailwind | Fast to build, Vercel-deployable |
| **Dashboard** | Next.js 15 + NextAuth | Same codebase as landing |
| **Component lib** | shadcn/ui | High-quality, copy-paste |
| **Charts** | Recharts | Usage analytics |
| **Billing** | Stripe | Industry standard |
| **Docs** | Nextra | MDX-based, searchable |
| **Monorepo** | pnpm + Turborepo | Efficient builds |
| **CI/CD** | GitHub Actions | Free for open source |
| **Reverse proxy** | Nginx + Traefik | Existing infra |
| **Monitoring** | Prometheus + Grafana | Existing infra |
| **JS SDK** | TypeScript + tsup | ESM/CJS dual build |
| **Python SDK** | httpx + Pydantic | Async + sync |

---

*Version: 1.0 | Created: 2026-02-26 | Status: Active*
*Next review: when Phase A (API) is complete*
