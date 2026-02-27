# Phase 1 — Web Search MVP

**Timeline:** Months 1–3 (solo developer)
**Goal:** Ship a working search + AI API, get 100 developers using it daily, prove willingness to pay.

---

## Success Criteria

| Metric | Target | How Measured |
|--------|--------|-------------|
| Beta users | 100 | User table row count |
| API integrations | 10 | Users with > 100 API calls |
| NPS | > 40 | Monthly survey (Typeform) |
| API uptime | > 99% | UptimeRobot |
| p95 search latency | < 3s | Prometheus histogram |
| p95 AI answer latency | < 8s | Prometheus histogram |
| Weekly active users | 60+ | Event log |
| Pro conversions | 5+ | Stripe dashboard |

---

## What Ships

### CLI — `fetchium` binary

```
fetchium search "quantum computing breakthroughs 2025"
fetchium search "rust async runtimes" --format json --limit 20
fetchium fetch https://example.com --format markdown
fetchium ai "what is retrieval augmented generation?" --fast
fetchium ai "compare tokio vs async-std" --sources 5
fetchium research "future of AI coding assistants" --depth deep
fetchium doctor               # env check + connectivity test
fetchium setup                # install Chrome, configure SearXNG
fetchium provider set gemini --key AIza...
```

**Install methods:**
- `curl -sSf https://install.fetchium.dev | sh` (primary)
- `npm install -g fetchium` (for JS ecosystem users)
- `cargo binstall fetchium` (for Rust developers)
- `brew install fetchium/tap/fetchium` (macOS)

### REST API v1

Base URL: `https://api.fetchium.dev/v1`

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/search` | POST | Web search with citations |
| `/fetch` | POST | Single URL deep extraction |
| `/ai` | POST | AI answer synthesis |
| `/research` | POST | Multi-source research job |
| `/health` | GET | Service health + SearXNG status |
| `/usage` | GET | Key usage stats |

**Request/response format:**
```json
POST /v1/search
{
  "query": "rust async runtimes comparison",
  "limit": 10,
  "format": "markdown"
}

Response:
{
  "meta": {
    "query": "rust async runtimes comparison",
    "backend": "searxng",
    "elapsed_ms": 1240,
    "result_count": 10
  },
  "results": [
    {
      "title": "...",
      "url": "...",
      "snippet": "...",
      "score": 0.92,
      "published_at": "2025-11-15"
    }
  ]
}
```

### Web Assets

- **Landing page:** `https://fetchium.dev`
  - Hero: benchmark vs Perplexity API / Tavily
  - Pricing: Free / Pro / Enterprise
  - Quick-start code snippet (copy/paste → working in 60 seconds)
  - Live demo embed

- **Docs site:** `https://docs.fetchium.dev`
  - Getting started guide
  - API reference (auto-generated from OpenAPI spec)
  - CLI reference
  - Cookbook: common patterns

---

## Technical Architecture

### Search Backend

```
User query
    │
    ▼
Query preprocessing (SPRE pre-ranker)
    │
    ▼
SearXNG (self-hosted, port 4040)
    │
    ├── Bing (via SearXNG engine)
    ├── Google (via SearXNG engine)
    ├── DuckDuckGo (via SearXNG engine)
    └── Brave (via SearXNG engine)
    │
    ▼
Result deduplication + HyperFusion ranking
    │
    ▼
Citation verification (URL reachability + snippet match)
    │
    ▼
Token budgeting (QATBE)
    │
    ▼
Response formatting (JSON / Markdown / Plain)
```

### AI Synthesis Backend

```
Search results (snippets)
    │
    ├── [--fast flag] Skip fetch, use snippets only
    │
    └── [default] CEP extraction (CSS → readability → headless)
    │
    ▼
QADD DOM distillation (10-20x token reduction)
    │
    ▼
AI provider chain:
  1. Gemini 2.5 Flash (key pool, 3 keys)
  2. Gemini 1.5 Pro (fallback)
  3. OpenAI GPT-4o (fallback)
    │
    ▼
Answer + citations + confidence score
```

### Infrastructure

```
VPS (Hetzner CX21 — 4 vCPU, 8GB RAM, ~$10/mo)
├── Traefik (TLS termination, rate limiting)
├── hsx-api (port 3050) — systemd service
├── hsx-web (port 3100) — Next.js landing
├── hsx-dashboard (port 3200) — Next.js app
└── SearXNG (port 4040) — Docker container

CDN: Cloudflare (free tier)
DB: SQLite (auth.db) — sufficient for Phase 1 scale
Secrets: /home/echo/.fetchium/env
Monitoring: UptimeRobot (free, 5-min checks)
Errors: simple file logging → upgrade to Sentry at M2
```

---

## Algorithm Stack (Phase 1)

| Algorithm | Purpose | Phase 1 Status |
|-----------|---------|---------------|
| SPRE | Speculative pre-ranking | Active |
| HyperFusion | 8-signal result ranking | Active (5 signals in P1) |
| QADD | DOM distillation | Active |
| QATBE | Token budget extraction | Active |
| CEP Layer 1-3 | CSS + readability + headless | Active |
| SCS | Semantic content segmentation | Active |
| ABS | Adaptive backend selection | Active |
| TDR | Temporal decay ranking | Active |
| PIE | Cross-session learning | Phase 3 |
| AMRS | Multi-agent research swarm | Phase 2 |

---

## Pricing (Phase 1)

### Free Tier
- 100 searches/day
- 10 AI answers/day
- 5 deep fetches/day
- No credit card required
- Rate limit: 10 req/min

### Pro — $49/month
- 5,000 searches/day
- 500 AI answers/day
- 200 deep fetches/day
- Rate limit: 60 req/min
- Email support (48h response)
- API key dashboard

### Enterprise — custom
- Unlimited usage
- 99.95% SLA
- Dedicated support
- On-prem option (Phase 5)

**Billing:** Stripe. Monthly only in Phase 1. Annual discounts in Phase 3.

---

## Week-by-Week Timeline

### Month 1

| Week | Deliverable | Done When |
|------|-------------|-----------|
| W1 | SearXNG backend stable, `fetchium search` working | CLI returns results for 10 test queries |
| W2 | REST API `/search` and `/health` live | curl to api.fetchium.dev returns results |
| W3 | `fetchium fetch` + CEP extraction working | Fetches HN front page, returns markdown |
| W4 | First 10 beta users invited (Discord + GitHub) | 10 users with API keys |

### Month 2

| Week | Deliverable | Done When |
|------|-------------|-----------|
| W5 | `fetchium ai --fast` working (snippet-only) | < 10s latency for factual queries |
| W6 | AI synthesis with full fetch + citations | Sources cited with URLs in response |
| W7 | Docs site live at docs.fetchium.dev | All Phase 1 endpoints documented |
| W8 | 50 beta users, first NPS survey sent | Survey responses collected |

### Month 3

| Week | Deliverable | Done When |
|------|-------------|-----------|
| W9 | `fetchium research` (single-agent) working | Research report with 5+ sources |
| W10 | Stripe billing live, Pro tier available | First paid subscription |
| W11 | Rate limiting + usage tracking per key | Dashboard shows per-key stats |
| W12 | 100 beta users, launch preparation | Hacker News post drafted |

---

## Acceptance Criteria (Phase 1 Complete)

- [ ] `fetchium search "test query"` returns 10 results in < 3s
- [ ] `fetchium ai "test question"` returns answer + citations in < 10s
- [ ] REST API `/search`, `/fetch`, `/ai`, `/research` all respond correctly
- [ ] 100 registered beta users with at least 1 API call
- [ ] NPS score collected from 20+ users, median > 40
- [ ] Pro billing live (Stripe), at least 5 paying subscribers
- [ ] Uptime > 99% over last 30 days (UptimeRobot)
- [ ] No P0 security issues (API keys not logged, rate limiting enforced)
- [ ] Docs cover all endpoints with working examples

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| SearXNG instability | Multiple engine fallbacks; health check + auto-restart systemd |
| Gemini rate limits | 3-key pool with shuffling; OpenAI fallback |
| Low beta user engagement | Weekly async office hours (Discord voice), personal onboarding DMs |
| Latency > 3s | Profile hot path; pre-warm connections; SearXNG result caching (60s TTL) |
| Founder time overcommit | Cap at 2 features/week; defer non-critical polish |
