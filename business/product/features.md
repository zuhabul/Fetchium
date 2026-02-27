# Fetchium — Complete Feature Matrix

**Last updated:** 2026-02-27
**Legend:** Built = shipped and stable | In Progress = actively being built | Planned = on roadmap | Not Planned = deliberately excluded

---

## Mode Overview

Fetchium organizes capabilities into 7 search modes, each targeting a distinct information need:

| Mode | Command | Purpose | Phase |
|------|---------|---------|-------|
| **Web** | `fetchium search` | General web search with AI synthesis | Phase 1 |
| **Video** | `fetchium video` | YouTube/video transcript extraction | Phase 2 |
| **Research** | `fetchium research` | Deep multi-source research reports | Phase 1-2 |
| **Social** | `fetchium social` | Reddit, HN, Twitter/X aggregation | Phase 2 |
| **Data** | `fetchium data` | Structured data extraction (tables, APIs) | Phase 3 |
| **Deep** | `fetchium deep` | Comprehensive site/topic crawl | Phase 3 |
| **Monitor** | `fetchium monitor` | Topic tracking + alerts | Phase 3 |

---

## Mode 1 — Web Search

Standard web search with AI synthesis and citation verification.

| Feature | Status | Notes |
|---------|--------|-------|
| Basic web search | Built | SearXNG backend |
| Multi-engine aggregation | Built | Bing, Google, DDG, Brave via SearXNG |
| Result deduplication | Built | URL normalization + content fingerprint |
| HyperFusion ranking | Built | 8-signal: BM25, semantic, temporal, authority, evidence, diversity, depth, consensus |
| Citation verification | Built | URL reachability + snippet match |
| SPRE pre-ranking | Built | Speculative pre-ranking filter |
| AI synthesis | Built | Gemini 2.5 Flash + OpenAI fallback |
| AI streaming (SSE) | Built | `--stream` flag |
| Token budget control | Built | QATBE algorithm |
| `--fast` mode | Built | Skip full-page fetch, use snippets only |
| Format: JSON | Built | `--format json` |
| Format: Markdown | Built | `--format markdown` |
| Format: Plain text | Built | `--format plain` |
| Result limit control | Built | `--limit N` (default 10, max 50) |
| Language filter | Planned | Phase 6 (CLQB) |
| Date range filter | Planned | Phase 2 |
| Domain include/exclude | Planned | Phase 2 |
| Safe search toggle | Planned | Phase 2 |
| Image search | Not Planned | Out of scope |

---

## Mode 2 — Video

Extract information from video content.

| Feature | Status | Notes |
|---------|--------|-------|
| YouTube URL extraction | Planned | Phase 2 |
| YouTube subtitle API | Planned | Phase 2, fastest method |
| yt-dlp subtitle fallback | Planned | Phase 2, open-source |
| Whisper transcription | Planned | Phase 2, last resort (audio download) |
| Video summary | Planned | Phase 2 |
| Timestamp extraction | Planned | Phase 2 |
| Full transcript | Planned | Phase 2 |
| Query-focused extraction | Planned | Phase 2 (QATBE on transcript) |
| Video search | Planned | Phase 2 (SearXNG YouTube engine) |
| Vimeo support | Planned | Phase 2 |
| Twitch VOD support | Planned | Phase 3 |
| Podcast (audio) support | Planned | Phase 3 (Whisper) |
| Multilingual transcripts | Planned | Phase 6 |
| Chapter detection | Planned | Phase 3 |

---

## Mode 3 — Research

Deep, multi-source research reports.

| Feature | Status | Notes |
|---------|--------|-------|
| Single-agent research | Built | Sequential search + synthesis |
| Multi-agent research (AMRS) | Planned | Phase 2 (4 parallel agents) |
| Async job model | Planned | Phase 2 (job_id + polling) |
| Webhook on completion | Planned | Phase 2 |
| Structured report output | Built | Executive summary + findings + citations |
| Evidence graph (EGB) | Built | Claims linked to sources |
| Confidence scoring | Built | Per-finding confidence |
| Conflicting views | Planned | Phase 2 |
| Academic paper search | Planned | Phase 4 (arXiv/Semantic Scholar plugin) |
| Citation format options | Planned | Phase 3 (APA, MLA, Chicago, URL) |
| Research depth levels | Built | `--depth fast|balanced|deep` |
| Source count control | Built | `--sources N` |
| Domain restriction | Planned | Phase 3 (`--domain site.com`) |
| Export to Notion | Planned | Phase 4 (Notion plugin) |
| Export to Obsidian | Planned | Phase 3 |
| Collaboration on reports | Planned | Phase 5 (team spaces) |

---

## Mode 4 — Social

Aggregate signal from where humans actually discuss topics.

| Feature | Status | Notes |
|---------|--------|-------|
| Reddit search | Built | Native API (OAuth) |
| Hacker News search | Built | Algolia HN API |
| Twitter/X search | Built | SearXNG site:x.com |
| Facebook search | Built | SearXNG site:facebook.com |
| LinkedIn search | Planned | Phase 2 (SearXNG site:linkedin.com) |
| Sentiment analysis | Planned | Phase 2 (AI-based) |
| Engagement metrics | Planned | Phase 2 (score, comments, shares) |
| Time series volume | Planned | Phase 3 |
| Top commenter identification | Planned | Phase 3 |
| Thread deep-dive | Planned | Phase 3 (fetch full thread) |
| Cross-platform deduplication | Planned | Phase 2 |
| Sort: hot/top/new | Planned | Phase 2 |
| Time range filter | Planned | Phase 2 |

---

## Mode 5 — Data

Extract structured data from web pages and APIs.

| Feature | Status | Notes |
|---------|--------|-------|
| HTML table extraction | Built | CSS selector + heuristic |
| JSON API extraction | Built | `fetchium fetch url --format json` |
| CSV download + parse | Planned | Phase 3 |
| PDF table extraction | Planned | Phase 4 (PDF tables plugin) |
| Financial data (Yahoo Finance) | Planned | Phase 4 (plugin) |
| GitHub stats extraction | Built | Via search + fetch |
| Structured schema extraction | Planned | Phase 3 (JSON-LD, OpenGraph) |
| Wikipedia infobox | Planned | Phase 3 |
| Recurring data jobs (cron) | Planned | Phase 5 |
| Data diff (change detection) | Planned | Phase 5 |

---

## Mode 6 — Deep

Comprehensive crawl of a site or topic with full knowledge extraction.

| Feature | Status | Notes |
|---------|--------|-------|
| Single-URL deep extraction | Built | CEP 5-layer cascade |
| CEP Layer 1 (CSS selectors) | Built | Fast, covers 70% of sites |
| CEP Layer 2 (readability) | Built | Mozilla Readability-style |
| CEP Layer 3 (headless JS) | Built | Chromiumoxide for SPAs |
| CEP Layer 4 (PDF) | Planned | Phase 3 |
| CEP Layer 5 (screenshot OCR) | Planned | Phase 5 |
| Multi-page site crawl | Planned | Phase 3 |
| Sitemap parsing | Planned | Phase 3 |
| Depth limit control | Planned | Phase 3 |
| Domain-scoped crawl | Planned | Phase 3 |
| Robots.txt compliance | Planned | Phase 3 |
| Rate limiting for crawls | Built | Configurable delay |
| Crawl export (JSON/JSONL) | Planned | Phase 3 |
| Incremental re-crawl | Planned | Phase 5 |

---

## Mode 7 — Monitor

Track topics over time and receive alerts on new content.

| Feature | Status | Notes |
|---------|--------|-------|
| Topic monitor creation | Planned | Phase 3 |
| Daily digest email | Planned | Phase 3 |
| Weekly digest email | Planned | Phase 3 |
| Webhook delivery | Planned | Phase 3 |
| Dashboard notification | Planned | Phase 3 |
| CLI inbox | Planned | Phase 3 |
| Monitor pause/resume | Planned | Phase 3 |
| Alert relevance scoring | Planned | Phase 3 |
| Team-shared monitors | Planned | Phase 5 |
| Multiple delivery channels | Planned | Phase 5 |
| Digest customization | Planned | Phase 4 |
| Alert frequency control | Planned | Phase 3 |

---

## API & Integration Features

| Feature | Status | Notes |
|---------|--------|-------|
| REST API v1 | Built | `/v1/search`, `/fetch`, `/ai`, `/research` |
| REST API v2 | Planned | Phase 4 (batch, GraphQL) |
| GraphQL API | Planned | Phase 4 |
| Batch endpoints | Planned | Phase 4 (up to 50 queries) |
| SSE streaming | Built | `Accept: text/event-stream` |
| Webhooks | Planned | Phase 2 |
| OpenAPI spec | Built | Auto-generated |
| Python SDK | Planned | Phase 3 |
| JavaScript SDK | Planned | Phase 3 |
| Go SDK | Planned | Phase 4 |
| Rust crate | Planned | Phase 4 |
| LangChain integration | Planned | Phase 4 |
| LlamaIndex integration | Planned | Phase 4 |
| CrewAI integration | Planned | Phase 4 |
| n8n node | Planned | Phase 4 |
| Zapier action | Planned | Phase 4 |
| Plugin marketplace | Planned | Phase 4 |

---

## Knowledge & Memory Features

| Feature | Status | Notes |
|---------|--------|-------|
| PIE cross-session learning | Built | Source trust, query history |
| Personal KB (nodes) | Planned | Phase 3 |
| KB threads | Planned | Phase 3 |
| KB semantic search | Planned | Phase 3 (BM25), Phase 5 (vectors) |
| KB export (JSON/Markdown) | Planned | Phase 3 |
| Obsidian export | Planned | Phase 3 |
| Notion export | Planned | Phase 4 (plugin) |
| Team shared KB | Planned | Phase 3 (beta), Phase 5 (GA) |
| KB version history | Planned | Phase 5 |
| Vector search in KB | Planned | Phase 5 |

---

## Auth, Billing & Admin

| Feature | Status | Notes |
|---------|--------|-------|
| API key auth | Built | `hsx_` prefix + 64 hex chars |
| Multiple keys per account | Built | Via dashboard |
| Per-key rate limits | Planned | Phase 4 |
| Per-key IP allowlist | Planned | Phase 4 |
| Usage dashboard | Built | Basic per-key stats |
| Full analytics dashboard | Planned | Phase 4 |
| Stripe billing | Built | Free + Pro tiers |
| Usage-based billing | Planned | Phase 4 |
| SSO (SAML/OIDC) | Planned | Phase 5 |
| SCIM provisioning | Planned | Phase 5 |
| Audit logs | Planned | Phase 5 |
| Team roles (RBAC) | Planned | Phase 5 |
| SOC 2 Type II | Planned | Phase 5 |
| GDPR DPA | Planned | Phase 5 |
| On-prem (Docker) | Planned | Phase 5 |
| On-prem (Helm/K8s) | Planned | Phase 5 |
| Admin API | Built | `X-Admin-Secret` header |
