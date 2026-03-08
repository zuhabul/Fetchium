# Phase 2 — Multi-Modal Search

**Timeline:** Months 4–6
**Theme:** No competitor has video + social + deep research in a single API. Own that gap.
**Team:** Solo founder (hire first contractor at M5 if MRR > $8K)

---

## Context

By the end of Phase 1, Fetchium has proven basic search + AI synthesis. Phase 2 is about building an unmistakable moat. Perplexity does web search well. Tavily does web search for agents. Nobody combines video transcripts, social signal aggregation, and multi-agent research in one coherent API.

**Phase 2 thesis:** If your query needs *context* beyond blue links — what YouTube says, what Reddit thinks, what experts have written — Fetchium is the only single-call solution.

---

## Success Criteria

| Metric | Target |
|--------|--------|
| MRR | $15,000 |
| API calls/day | 50,000 |
| Paying users | 150+ |
| Social platforms active | 5 (Reddit, HN, Twitter/X, Facebook, LinkedIn) |
| Video platforms | 2 (YouTube, Vimeo) |
| Research job completion rate | > 95% |
| p95 research latency | < 45s |
| p95 video summary latency | < 12s |
| NPS | > 50 |

---

## What Ships

### 1. Video Mode — `fetchium video`

Extract insight from video content without watching it.

```bash
fetchium video "https://youtube.com/watch?v=..." --summary
fetchium video "https://youtube.com/watch?v=..." --timestamps
fetchium video search "LLM inference optimization" --top 5
```

**API endpoint:**
```
POST /v1/video
{
  "url": "https://youtube.com/watch?v=...",
  "mode": "summary" | "timestamps" | "transcript",
  "query": "optional focus query for relevance extraction"
}
```

**Pipeline:**
```
YouTube URL
    │
    ├── Attempt 1: YouTube Data API (transcript endpoint)
    ├── Attempt 2: yt-dlp subtitle extraction
    └── Attempt 3: Whisper transcription (audio download)
    │
    ▼
QATBE: query-aware segment ranking
    │
    ▼
AI synthesis with timestamp citations
    │
    ▼
Response: summary + key timestamps + full transcript (optional)
```

**Key metric:** < 12s for a 30-minute video with existing captions.

### 2. Social Mode — `fetchium social`

Aggregate signal from where humans actually talk.

```bash
fetchium social "rust vs go 2025" --sources reddit,hn,twitter
fetchium social "best mechanical keyboard switches" --sources reddit --sort hot
```

**API endpoint:**
```
POST /v1/social
{
  "query": "rust vs go 2025",
  "sources": ["reddit", "hn", "twitter", "facebook", "linkedin"],
  "sort": "hot" | "top" | "new",
  "time_range": "week" | "month" | "year" | "all"
}
```

**Source pipelines:**

| Platform | Method | Data |
|----------|--------|------|
| Reddit | Native API (OAuth) | Posts, comments, score, subreddit |
| Hacker News | Algolia HN API | Stories, comments, points |
| Twitter/X | SearXNG site:x.com | Tweets, engagement signals |
| Facebook | SearXNG site:facebook.com | Public posts |
| LinkedIn | SearXNG site:linkedin.com | Public posts, articles |

**Output includes:**
- Sentiment distribution (positive/negative/neutral ratio)
- Top posts with engagement metrics
- Key themes extracted by AI
- Time series: discussion volume over time

### 3. Research Mode v2 — AMRS (Adaptive Multi-Agent Research Swarm)

True multi-agent research: parallel agents, synthesis, fact-checking.

```bash
fetchium research "impact of quantization on LLM accuracy" \
  --depth deep \
  --sources 20 \
  --format report
```

**AMRS Architecture:**
```
Research query
    │
    ▼
Orchestrator agent
    ├── Spawns 4 parallel Search agents (tokio tasks)
    │     ├── Agent A: academic / papers
    │     ├── Agent B: recent news / events
    │     ├── Agent C: technical documentation
    │     └── Agent D: practitioner discussion (social)
    │
    ├── Source deduplication + relevance filtering
    │
    ├── Evidence graph construction (EGB)
    │
    └── Synthesis agent → final report
    │
    ▼
Structured report:
  - Executive summary (200 words)
  - Key findings (5-10 bullet points)
  - Evidence graph (claims → sources)
  - Conflicting views
  - Confidence score per finding
  - Full citations (APA / URL format)
```

**Async job model:**
- Research jobs run asynchronously (30-120s)
- Returns `job_id` immediately
- Poll `GET /v1/research/{job_id}` for status
- Or subscribe via webhook on completion

```
POST /v1/research → { "job_id": "res_abc123", "status": "queued" }
GET /v1/research/res_abc123 → { "status": "running", "progress": 0.6 }
GET /v1/research/res_abc123 → { "status": "complete", "report": {...} }
```

### 4. Streaming API (SSE)

Real-time streaming for AI responses — critical for UX.

```bash
# CLI: streaming by default for ai/research
fetchium ai "explain transformers" --stream

# API: add Accept: text/event-stream header
curl -N -H "Accept: text/event-stream" \
  -H "Authorization: Bearer hsx_..." \
  https://api.fetchium.com/v1/ai \
  -d '{"query": "explain transformers"}'
```

**SSE event format:**
```
data: {"type": "token", "content": "Transformers are"}
data: {"type": "token", "content": " a neural network"}
data: {"type": "citation", "url": "https://...", "title": "..."}
data: {"type": "done", "elapsed_ms": 4200}
```

### 5. Webhooks

For research jobs and monitor alerts (preview).

```
POST /v1/webhooks
{
  "url": "https://your-app.com/webhook",
  "events": ["research.complete", "monitor.alert"],
  "secret": "your-signing-secret"
}
```

Delivery: at-least-once, 3 retries with exponential backoff (1s, 5s, 30s).
Signature: `X-Fetchium-Signature: sha256=...` (HMAC-SHA256).

---

## API v1.5 — New Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/video` | POST | Video extraction + summary |
| `/social` | POST | Social platform aggregation |
| `/research` | POST | Async research job |
| `/research/{id}` | GET | Poll research job status |
| `/webhooks` | POST | Register webhook |
| `/webhooks/{id}` | DELETE | Remove webhook |

All existing v1 endpoints unchanged. No breaking changes.

---

## Infrastructure Changes

### Job Queue
- Phase 1: synchronous responses only
- Phase 2: tokio task pool for research jobs
- Queue depth: 100 concurrent research jobs (VPS limit)
- Upgrade path: Redis + background workers at 500 jobs/day

### Headless Browser Pool
- Chromiumoxide pool (3 instances) for JS-rendered pages
- Used by: Social (Facebook, Twitter/X fallback), CEP Layer 3
- Pool management: `BrowserPool::from_config()` with `resolve_chrome_path()`

### Caching Layer
- Search results: 60s TTL (Redis-compatible in-process cache Phase 1, upgrade at scale)
- Video transcripts: 24h TTL (rarely change)
- Social posts: 5-min TTL (balance freshness vs API limits)

---

## Week-by-Week Timeline

### Month 4

| Week | Deliverable |
|------|-------------|
| W13 | YouTube transcript extraction (API + yt-dlp fallback) |
| W14 | Video AI summary + timestamp mode live |
| W15 | Reddit native API integration |
| W16 | HN Algolia API integration |

### Month 5

| Week | Deliverable |
|------|-------------|
| W17 | Twitter/X + Facebook via SearXNG site: search |
| W18 | `fetchium social` CLI + `/v1/social` API endpoint |
| W19 | SSE streaming for `/v1/ai` and `/v1/research` |
| W20 | Async research job system + job polling |

### Month 6

| Week | Deliverable |
|------|-------------|
| W21 | AMRS multi-agent research (4 parallel agents) |
| W22 | Webhook delivery system (research.complete) |
| W23 | Phase 2 hardening: load testing, error handling |
| W24 | $15K MRR milestone review, Phase 3 planning |

---

## Pricing Updates (Phase 2)

| Tier | Price | Search | AI | Video | Social | Research |
|------|-------|--------|-----|-------|--------|----------|
| Free | $0 | 100/day | 10/day | 5/day | 20/day | 2/day |
| Pro | $49/mo | 5K/day | 500/day | 100/day | 1K/day | 50/day |
| Pro+ | $99/mo | 20K/day | 2K/day | 500/day | 5K/day | 200/day |
| Enterprise | Custom | Unlimited | Unlimited | Unlimited | Unlimited | Unlimited |

**Video and social credits:** 1 video summary = 5 search credits. 1 social query = 2 search credits.

---

## Competitive Differentiation at Phase 2 Complete

| Capability | Fetchium | Perplexity API | Tavily | Exa |
|------------|----------|---------------|--------|-----|
| Web search | ✅ | ✅ | ✅ | ✅ |
| AI synthesis | ✅ | ✅ | ✅ | Partial |
| Video extraction | ✅ | ❌ | ❌ | ❌ |
| Social aggregation | ✅ | ❌ | ❌ | ❌ |
| Multi-agent research | ✅ | ❌ | ❌ | ❌ |
| Streaming | ✅ | ✅ | ❌ | ❌ |
| Webhooks | ✅ | ❌ | ❌ | ❌ |

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| YouTube blocks transcript access | yt-dlp fallback + Whisper as last resort |
| Reddit API pricing changes | Cache aggressively; push-down requests per user; evaluate Pushshift |
| Research jobs timeout > 120s | Hard timeout with partial results + retry suggestion |
| Headless browser memory leaks | Pool health checks; restart on 500MB threshold |
| Solo founder capacity | Scope creep guard: no new features until core 3 are stable |
