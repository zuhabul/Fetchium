# Fetchium Product Roadmap — $0 to $100M ARR

**Last updated:** 2026-02-27
**Status:** Phase 1 in progress

---

## Market Opportunity

| Metric | Value |
|--------|-------|
| AI search market (2025) | $43.6B |
| CAGR | 14% |
| Projected market (2032) | $108.9B |
| AI agents market | $8B → $12B |
| Perplexity ARR (2025) | $656M |
| Cursor ARR (12 months, zero marketing) | $100M |
| Tavily acquisition (Nebius) | $25M |

**Window:** Bing Search API retired August 2025. Every AI agent needing web data is now underserved. Fetchium fills that gap.

---

## Strategic Arc

```
Phase 1 (M1-3)   → Developer adoption, API product-market fit
Phase 2 (M4-6)   → Multi-modal differentiation, content moat
Phase 3 (M7-12)  → Knowledge OS, stickiness through memory
Phase 4 (M13-18) → Platform play, ecosystem lock-in
Phase 5 (M19-24) → Enterprise revenue, repeatable sales
Phase 6 (M25-36) → Global scale, mobile, distribution
```

**Revenue targets:**
- Month 6: $5K MRR (50 paying users @ $99/mo avg)
- Month 12: $50K MRR ($600K ARR)
- Month 18: $250K MRR ($3M ARR) — hire 2 engineers
- Month 24: $833K MRR ($10M ARR) — Series A territory
- Month 36: $8.3M MRR ($100M ARR) — Series B, 50+ team

---

## Phase 1 — Web Search MVP (Months 1–3)

**Theme:** Ship fast, prove the core, get 100 developers using it daily.

### What ships
- `fetchium search` CLI with citation verification
- `fetchium ai` — AI answer synthesis (Gemini + OpenAI backends)
- `fetchium fetch` — single-URL deep extraction
- `fetchium research` — multi-source synthesis
- REST API v1 (`/search`, `/fetch`, `/ai`, `/research`)
- Landing page + docs site
- Free tier + Pro tier ($49/mo)

### KPIs
| Metric | Target |
|--------|--------|
| Beta users | 100 |
| API integrations | 10 |
| NPS | > 40 |
| API uptime | > 99% |
| p95 search latency | < 3s |
| Weekly active users | 60+ |

### Milestones
- M1W2: SearXNG backend + search CLI working
- M1W4: REST API deployed, first 10 beta users
- M2W2: AI synthesis working with citations
- M2W4: Docs site live, 50 beta users
- M3W2: Pro billing live (Stripe)
- M3W4: 100 beta users, launch-ready

### Dependencies
- SearXNG self-hosted instance stable
- Gemini API key pool (3+ keys)
- Domain + TLS + CDN
- Stripe account approved

---

## Phase 2 — Multi-Modal (Months 4–6)

**Theme:** No competitor has video + social + research in one API. Own that.

### What ships
- `fetchium video` — YouTube transcript extraction + AI summary
- `fetchium social` — Reddit, HN, Twitter/X, Facebook aggregation
- `fetchium research` v2 — multi-agent parallel research (AMRS)
- API v1.5 — video and social endpoints
- Streaming responses (SSE)
- Webhooks for async research jobs

### KPIs
| Metric | Target |
|--------|--------|
| MRR | $15K |
| API calls/day | 50K |
| Social sources indexed | 5 |
| Video platforms | 2 (YouTube, Vimeo) |
| Research job completion rate | > 95% |

### Milestones
- M4W2: YouTube transcript pipeline live
- M4W4: Reddit + HN social aggregation
- M5W2: Twitter/X via SearXNG site: search
- M5W4: Streaming API (SSE) live
- M6W2: Multi-agent research mode (AMRS) beta
- M6W4: $15K MRR, 200+ active API users

### Dependencies
- Phase 1 API stable (99.9% uptime)
- Headless browser pool (chromiumoxide)
- Job queue (tokio channels → Redis for scale)

---

## Phase 3 — Knowledge OS (Months 7–12)

**Theme:** Transform Fetchium from a search tool into a thinking partner.

### What ships
- **Personal KB** — save, tag, and retrieve past research sessions
- **PIE engine** — cross-session learning (source trust, query prediction)
- **Learn mode** — structured curriculum generation from a topic
- **Know mode** — query against your personal knowledge base
- **Monitor mode** — track topics, get daily digests
- **Team spaces** (beta) — shared knowledge bases
- API v2 (breaking changes, versioned)
- Python SDK v1.0
- JavaScript/TypeScript SDK v1.0

### KPIs
| Metric | Target |
|--------|--------|
| MRR | $80K |
| ARR | $960K |
| KB sessions stored | 1M+ |
| DAU/MAU ratio | > 35% |
| Retention (30-day) | > 55% |
| SDK downloads/week | 5K |

### Milestones
- M7: PIE cross-session learning (SQLite)
- M8: Personal KB CRUD + search
- M9: Learn mode + Monitor mode
- M10: Python SDK released
- M11: JS/TS SDK released, API v2 stable
- M12: $80K MRR, Series A materials ready

### Dependencies
- SQLite persistence layer (fetchium-core/index)
- Vector embeddings (ONNX Runtime — `embeddings` feature)
- Background job scheduler
- SDK CI/CD pipeline (GitHub Actions)

---

## Phase 4 — Developer Platform (Months 13–18)

**Theme:** Make Fetchium the Stripe of AI search. Embed it everywhere.

### What ships
- **API v2** — GraphQL + REST, batch endpoints, 99.95% SLA
- **SDK suite** — Python, JS, Go, Rust crates
- **Plugin marketplace** — community-built extractors and processors
- **Fetchium Agent** — drop-in web search for LangChain, LlamaIndex, CrewAI
- **Playground** — browser-based API explorer
- **Usage dashboard** — per-key analytics, cost forecasting
- **Webhook hub** — reliable delivery, retry, dead-letter queue
- Go SDK v1.0
- Rust crate published to crates.io

### KPIs
| Metric | Target |
|--------|--------|
| ARR | $3M |
| API daily calls | 500K |
| SDK downloads (all) | 50K/week |
| Marketplace plugins | 25+ |
| Agent integrations | 10+ (LangChain, etc.) |
| Enterprise trials | 20 |

### Milestones
- M13: GraphQL API beta
- M14: Go SDK + Rust crate
- M15: Plugin marketplace launch (10 plugins)
- M16: LangChain + LlamaIndex integrations
- M17: Usage dashboard + billing portal
- M18: $250K MRR / $3M ARR — hire 2 engineers

### Dependencies
- Phase 3 SDKs stable
- Partner relationships (LangChain, LlamaIndex teams)
- Marketplace infrastructure (plugin registry, sandboxing)
- Second engineer hired

---

## Phase 5 — Teams & Enterprise (Months 19–24)

**Theme:** Land enterprise contracts. One $10K/mo deal = 200 pro users.

### What ships
- **Team workspaces** — shared KB, roles, audit logs
- **SSO** — SAML 2.0, OIDC (Okta, Google Workspace, Azure AD)
- **SLA tiers** — 99.95% (Pro), 99.99% (Enterprise) with credits
- **On-prem option** — Docker Compose + Helm chart
- **Compliance** — SOC 2 Type II audit started, GDPR DPA
- **Enterprise dashboard** — seat management, cost controls, RBAC
- **Dedicated support** — SLA-backed, named CSM for > $2K/mo accounts
- **Custom data retention** — configurable, GDPR-compliant

### KPIs
| Metric | Target |
|--------|--------|
| ARR | $10M |
| Enterprise accounts (> $1K/mo) | 25 |
| Net Revenue Retention | > 120% |
| Gross margin | > 75% |
| Sales cycle (SMB) | < 14 days |
| Sales cycle (Enterprise) | < 60 days |

### Milestones
- M19: Team workspaces GA
- M20: SSO (SAML + OIDC)
- M21: SOC 2 audit kicked off
- M22: On-prem Docker + Helm chart
- M23: First 10 enterprise contracts signed
- M24: $833K MRR / $10M ARR — Series B ready

### Dependencies
- 3-5 person team (2 engineers, 1 sales, 1 customer success)
- SOC 2 compliance partner (Vanta / Drata)
- Legal: MSA template, DPA template, security review process

---

## Phase 6 — Mobile & Global (Months 25–36)

**Theme:** 10x distribution. Meet users where they are.

### What ships
- **iOS + Android app** — native Fetchium client (Flutter)
- **Browser extension** — Chrome + Firefox + Safari (search hijack opt-in)
- **i18n** — UI in 10 languages, multilingual search
- **Cross-lingual query expansion** (CLQB algorithm)
- **Regional deployments** — EU (Frankfurt), APAC (Singapore), US-West
- **Enterprise API gateway** — per-region data residency
- **Fetchium AI** — on-device lightweight model option

### KPIs
| Metric | Target |
|--------|--------|
| ARR | $100M |
| MAU | 5M+ |
| Mobile DAU | 500K+ |
| Browser extension installs | 1M+ |
| Countries with paying users | 50+ |
| Languages supported | 10 |

### Milestones
- M25: iOS beta (TestFlight)
- M27: Android beta
- M28: Browser extension Chrome Web Store
- M30: 5 language localizations
- M32: EU + APAC infrastructure live
- M36: $8.3M MRR / $100M ARR

### Dependencies
- Mobile team (2 engineers)
- App store approvals (Apple + Google)
- Regional data residency legal review
- i18n toolchain (translation memory, CI checks)

---

## Dependency Graph

```
Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5 ──► Phase 6
                │                        │
                └── Headless browser     └── Hire 2 engineers
                    pool stable              (M18 gate)
```

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| SearXNG blocked by search engines | Medium | High | Multiple backend fallbacks, Brave API |
| Gemini API pricing changes | Medium | Medium | Key pool, multi-provider fallback |
| Perplexity launches free API | Low | High | Differentiate on extraction quality + KB |
| Solo founder burnout | High | Critical | Hire VA by M3, first engineer by M6 |
| AWS/infra costs spike | Low | Medium | Cost alerts, reserved capacity by M12 |
