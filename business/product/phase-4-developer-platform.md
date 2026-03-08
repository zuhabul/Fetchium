# Phase 4 — Developer Platform

**Timeline:** Months 13–18
**Theme:** Make Fetchium the Stripe of AI search. Embed it in every agent, workflow, and tool.
**Team:** 3 people (founder + 2 engineers hired at M18 gate)

---

## The Platform Bet

The AI agent ecosystem is exploding. LangChain has 90K+ GitHub stars. LlamaIndex processes billions of tokens. CrewAI, AutoGen, and dozens of agent frameworks are emerging weekly.

Every one of these agents needs web search. The Bing API died in August 2025. Perplexity's API is expensive and lacks extraction. Tavily was acquired. The category is wide open.

Phase 4 is about becoming the default web search layer for the agent ecosystem — through excellent SDKs, seamless framework integrations, and a marketplace that lets the community extend Fetchium.

**Benchmark goal:** When a developer says "add web search to my AI agent," Fetchium is the first result, the easiest install, and the best experience.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| ARR | $3,000,000 |
| API calls/day | 500,000 |
| SDK downloads (all, /week) | 50,000 |
| Marketplace plugins | 25+ |
| Agent framework integrations | 10+ |
| Enterprise trials | 20 |
| Developer accounts | 5,000+ |
| Docs NPS | > 60 |

---

## What Ships

### 1. API v2 — GraphQL + REST

API v2 introduces GraphQL alongside REST. REST endpoints remain stable (no breaking changes).

**GraphQL endpoint:** `***REMOVED***/v2/graphql`

```graphql
query SearchAndSynthesize($query: String!, $sources: Int) {
  search(query: $query, limit: $sources) {
    results {
      title
      url
      snippet
      score
      publishedAt
    }
    meta {
      elapsedMs
      backend
    }
  }
  ai(query: $query) {
    answer
    citations {
      url
      title
      relevance
    }
    confidence
  }
}
```

**Batch endpoints** (new in v2):
```
POST /v2/batch/search    → up to 50 queries in one request
POST /v2/batch/fetch     → up to 20 URLs in one request
POST /v2/batch/ai        → up to 10 AI queries in one request
```

Batch requests run in parallel server-side. Pricing: 20% discount vs individual calls.

**SLA tiers:**
- Pro: 99.9% uptime (< 8.7h downtime/year)
- Enterprise: 99.95% uptime (< 4.4h downtime/year)
- Credits issued automatically for SLA breaches (10% of month's bill per 0.1% below SLA)

### 2. SDK Suite Expansion

**Go SDK v1.0:**
```go
go get github.com/fetchium/fetchium-go

client := fetchium.NewClient("hsx_...")

results, err := client.Search(ctx, &fetchium.SearchRequest{
    Query: "rust async runtimes 2025",
    Limit: 10,
})

answer, err := client.AI(ctx, &fetchium.AIRequest{
    Query: "explain transformers",
    Stream: true,
})
for chunk := range answer.Stream {
    fmt.Print(chunk.Content)
}
```

**Rust crate (crates.io):**
```toml
[dependencies]
fetchium = "1.0"
```

```rust
use fetchium::Fetchium;

let client = Fetchium::new("hsx_...")?;

let results = client.search()
    .query("rust async runtimes 2025")
    .limit(10)
    .send()
    .await?;

let answer = client.ai()
    .query("explain transformers")
    .stream()
    .send()
    .await?;

while let Some(chunk) = answer.next().await {
    print!("{}", chunk?.content);
}
```

**SDK quality bar:**
- 90%+ test coverage
- Zero-dependency core (HTTP + serde only)
- Async-native (tokio, asyncio, go routines)
- Auto-retry with exponential backoff
- Automatic API key rotation (pool support)
- OpenTelemetry tracing built-in

### 3. Agent Framework Integrations

**LangChain (Python):**
```python
from langchain_fetchium import FetchiumSearch, FetchiumRetriever

# As a tool
tools = [FetchiumSearch(api_key="hsx_...")]

# As a retriever
retriever = FetchiumRetriever(api_key="hsx_...", k=5)
docs = retriever.get_relevant_documents("rust async runtimes")
```

**LlamaIndex:**
```python
from llama_index.tools.fetchium import FetchiumToolSpec

tool_spec = FetchiumToolSpec(api_key="hsx_...")
tools = tool_spec.to_tool_list()
```

**CrewAI:**
```python
from fetchium.integrations.crewai import FetchiumSearchTool

search_tool = FetchiumSearchTool(api_key="hsx_...")
agent = Agent(tools=[search_tool], ...)
```

**n8n node:** Community node published at n8n.io/integrations
**Zapier action:** Published in Zapier app marketplace
**Make (Integromat) module:** Published

**Integration requirements for each:**
- Follow framework's tool/node/action interface exactly
- Include example workflows in docs
- CI: test against pinned framework versions
- Alert system when upstream framework releases breaking changes

### 4. Plugin Marketplace

Community-built extractors, processors, and data sources.

**Plugin types:**
- **Source plugin** — adds a new search backend (e.g., arXiv, PubMed, Semantic Scholar)
- **Extractor plugin** — custom content extraction for specific site types
- **Processor plugin** — post-processing step (e.g., PDF table extraction, image OCR)
- **Output plugin** — custom output format (e.g., Notion export, Obsidian sync)

**Plugin interface (Rust trait):**
```rust
pub trait FetchiumPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    async fn execute(&self, ctx: PluginContext) -> HsxResult<PluginOutput>;
}
```

**Plugin registry:** `https://plugins.fetchium.com`
- Searchable catalog
- Version history + changelog
- Install count + rating
- Security review process (automated + manual for popular plugins)

**Plugin install:**
```bash
fetchium plugin install arxiv-search
fetchium plugin install pdf-tables
fetchium plugin list
fetchium plugin update --all
```

**Revenue share:**
- Plugins are free during Phase 4 (grow ecosystem first)
- Phase 5+: paid plugins with 70/30 revenue split (author/Fetchium)

### 5. Developer Dashboard — Full Feature Set

**Usage analytics:**
- API calls by endpoint, key, time range
- Error rate breakdown (4xx vs 5xx, by endpoint)
- Latency percentiles (p50, p95, p99) over time
- Top queries (aggregated, anonymized)
- Cost breakdown by feature (search vs AI vs research)

**Cost forecasting:**
- "At your current usage, next month's bill will be $X"
- Usage alerts: notify at 80% and 100% of plan limit
- Per-key spending limits (set a hard cap per API key)

**API key management:**
- Multiple keys per account
- Per-key labels, rate limits, IP allowlists
- Key rotation without downtime (grace period overlap)
- Audit log: which key called what endpoint when

**Billing portal (Stripe Customer Portal):**
- Plan upgrade/downgrade
- Invoice history + download
- Payment method management
- Usage-based billing for enterprise (per-1K-calls)

### 6. Playground

Browser-based API explorer. No code required to test any endpoint.

URL: `https://app.fetchium.com/playground`

Features:
- Dropdown: select endpoint
- Form-based request builder (no raw JSON required)
- "Copy as cURL / Python / JS / Go" buttons
- Response viewer with syntax highlighting
- Latency timer
- Save requests to collections

---

## Webhook Hub — v2

Phase 2 introduced basic webhooks. Phase 4 makes them enterprise-grade.

**Improvements:**
- Reliable delivery: at-least-once with dead-letter queue
- Retry schedule: 1s, 5s, 30s, 2m, 10m, 1h, 6h, 24h (8 attempts)
- Delivery log: see every attempt, response code, latency
- Manual replay: replay any webhook from the dashboard
- Filtering: subscribe only to specific event subtypes
- Batch delivery: bundle up to 100 events in one HTTP call (reduces load)

---

## Infrastructure

### Scaling targets at Phase 4 peak
- 500K API calls/day = ~6 calls/second average, ~60 calls/second peak
- VPS → 2x Hetzner CCX33 (8 vCPU, 32GB RAM, ~$75/mo each) with load balancer
- PostgreSQL (Hetzner managed, ~$50/mo) — migrate from per-user SQLite
- Redis (Hetzner managed, ~$30/mo) — job queue + caching
- Cloudflare (paid plan) — DDoS protection, caching, analytics

### Observability stack
- Prometheus + Grafana (self-hosted)
- Sentry for error tracking
- OpenTelemetry traces for request path visibility
- PagerDuty for on-call alerts (escalation to Founder → Engineer 1 → Engineer 2)

---

## Week-by-Week Timeline

### Month 13
- W49: GraphQL API beta (search + AI endpoints)
- W50: Batch endpoints (`/v2/batch/*`)
- W51: LangChain integration (Python)
- W52: LlamaIndex integration (Python)

### Month 14
- W53: Go SDK v1.0
- W54: Rust crate (crates.io)
- W55: CrewAI integration
- W56: SDK integration test suite in CI

### Month 15
- W57: Plugin marketplace infrastructure (registry, install system)
- W58: 5 first-party plugins (arXiv, PubMed, Reddit enhanced, PDF tables, Notion export)
- W59: Developer playground (browser-based)
- W60: Plugin developer docs + SDK

### Month 16
- W61: n8n + Zapier + Make integrations
- W62: Marketplace grows to 15 community plugins
- W63: Webhook hub v2 (reliable delivery, replay, dead-letter)
- W64: Developer dashboard analytics full feature set

### Month 17
- W65: Usage-based enterprise billing (per-1K-calls)
- W66: Per-key rate limits + IP allowlists
- W67: SLA monitoring + automated credits
- W68: Performance: p95 < 2s for all endpoints

### Month 18
- W69: 500K API calls/day load test + fix
- W70: $250K MRR milestone review
- W71: Hire Engineer 2 (backend focus)
- W72: Series A preparation begins

---

## Pricing Updates (Phase 4)

| Tier | Price | Calls/day | Batch | GraphQL | Webhooks | SLA |
|------|-------|-----------|-------|---------|----------|-----|
| Free | $0 | 100 | No | No | No | None |
| Pro | $49/mo | 5K | No | Yes | 3 | 99% |
| Pro+ | $99/mo | 20K | Yes | Yes | 10 | 99.5% |
| Growth | $299/mo | 100K | Yes | Yes | 50 | 99.9% |
| Enterprise | Custom | Unlimited | Yes | Yes | Unlimited | 99.95% |
