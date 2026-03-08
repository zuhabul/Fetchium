# Fetchium — Brand Vision Document

> **"Make all the world's information instantly accessible, deeply understood, and reliably verified."**

*Formerly Fetchium. Now Fetchium: the universal retrieval layer for the internet.*

---

## Table of Contents

1. [Mission](#1-mission)
2. [Vision — The Universal Retrieval Layer](#2-vision--the-universal-retrieval-layer)
3. [Core Values](#3-core-values)
4. [The Problem](#4-the-problem)
5. [The Solution — 7 Fetch Modes](#5-the-solution--7-fetch-modes)
6. [The 5-Layer Information Stack](#6-the-5-layer-information-stack)
7. [Technical Moat — 20+ Novel Algorithms](#7-technical-moat--20-novel-algorithms)
8. [Market Opportunity](#8-market-opportunity)
9. [Competitive Landscape](#9-competitive-landscape)
10. [Differentiation](#10-differentiation)
11. [Brand Identity](#11-brand-identity)
12. [Path to $100M ARR](#12-path-to-100m-arr)
13. [Go-To-Market Wedges](#13-go-to-market-wedges)
14. [Revenue Architecture](#14-revenue-architecture)
15. [World Impact Thesis](#15-world-impact-thesis)
16. [Brand Evolution Arc](#16-brand-evolution-arc)

---

## 1. Mission

**Make all the world's information instantly accessible, deeply understood, and reliably verified.**

This is not a product tagline. It is an engineering constraint, a business model, and a moral commitment simultaneously.

### What "Instantly Accessible" Means

The gap between a question forming in a human mind and a verified answer landing in front of them should be measured in seconds — not minutes, not tabs, not copy-paste cycles. The current average workflow for a knowledge worker to fully answer a non-trivial question involves:

- 3–7 Google searches
- 4–12 browser tabs
- 2–4 manual copy-paste operations into an AI assistant
- 1–3 follow-up searches to cross-reference claims
- An average of 11–18 minutes per question

That is a broken loop. "Instantly accessible" means the entire chain — search, fetch, extract, verify, synthesize — completes in under 15 seconds with a single command.

### What "Deeply Understood" Means

A link is not an answer. A snippet is not understanding. "Deeply understood" means:

- Content is fully extracted, not just previewed (5-layer cascade: CSS → readability → headless JS → PDF → OCR)
- Key claims are identified and weighted by evidence density
- The answer is structured by relevance to the specific query, not by the order the source chose to present it
- Token budgets are respected without losing the signal — the most important 10% of a 50,000-word document can be surfaced in under 800 tokens
- Multiple sources are synthesized into a single coherent response with individual claims attributed to their origin

### What "Reliably Verified" Means

Every claim Fetchium surfaces can be traced back to a source. Not a URL — a specific passage in a specific document at a specific point in time. The evidence graph is not decorative; it is the primary output. Unverifiable assertions are flagged. Sources are scored for authority, recency, and consistency with other sources. Contradiction is surfaced as contradiction, not silently resolved.

This mission applies equally to:
- A student in Dhaka writing a thesis
- An analyst on Wall Street building a brief
- A developer integrating search into an AI agent
- A founder researching a market before pitching
- A journalist verifying a claim before publishing

The same tool. The same quality. The same guarantee.

---

## 2. Vision — The Universal Retrieval Layer

### The One-Sentence Vision

**Fetchium is the universal retrieval layer for the internet: a single intelligence engine that finds, fetches, understands, and acts on any information from anywhere.**

### The Analogy That Explains Everything

Stripe did not invent online payments. PayPal existed. Braintree existed. Direct bank integrations existed. But every team building a product had to re-implement the same broken payment flow from scratch. Stripe said: *this is infrastructure. Abstract it. Make it reliable. Make it one API call.*

Fetchium is to information retrieval what Stripe is to payments.

Every AI application being built today needs to retrieve information from the internet. Every AI agent needs to search, extract content, verify facts, and synthesize across sources. Every developer building on top of LLMs is re-implementing the same broken retrieval stack from scratch — scraping, parsing, rate-limit handling, fallbacks, token management, citation tracking.

Fetchium says: **this is infrastructure. Abstract it. Make it reliable. Make it one API call.**

### What "Universal" Actually Means

Universal has five dimensions:

**1. Source Universality**
Not just web pages. Video transcripts. Academic papers. Reddit threads. Twitter conversations. GitHub repositories. PDF documents. Data tables. Forum posts. Hacker News discussions. The full spectrum of where human knowledge actually lives in 2026.

**2. Format Universality**
Not just HTML. JavaScript-rendered pages. PDFs. Images with text (OCR). Paywalled content (where legally accessible). API responses. Raw data files. Structured databases. Whatever format the information is stored in, Fetchium extracts signal from it.

**3. Query Universality**
Not just keyword search. Natural language questions. Comparison queries ("A vs B"). Verification queries ("Is this claim true?"). Research queries that require synthesis across dozens of sources. Monitoring queries that need to run continuously. Code queries that need repository context. Opinion queries that need aggregated sentiment.

**4. Consumer Universality**
Not just humans. AI agents running autonomously. Pipelines that process thousands of queries per hour. Developer tools that need search as a building block. MCP-compatible applications that treat Fetchium as a tool-calling endpoint. The CLI. The TUI. The REST API. The SDK.

**5. Temporal Universality**
Not just now. Cross-session memory. Query history. Source trust that improves over time. The ability to monitor a topic and surface changes. The persistent intelligence engine that learns your research patterns and improves results for your specific domain.

### The Retrieval Layer Is the Missing Infrastructure

The current AI application stack has three layers:
- **Foundation models** — OpenAI, Anthropic, Google, Mistral — commoditizing fast
- **Application layer** — the products being built on top of models
- **Retrieval layer** — **this is the gap**

Every meaningful AI application needs grounded, current, verified information. The model alone cannot provide this. The retrieval layer is not optional — it is the difference between an AI that hallucinates and one that answers correctly.

Fetchium is purpose-built to be this layer: open-source, Rust-native, composable, and designed from day one to be integrated into AI pipelines rather than to be a standalone product that happens to have an API.

### What Success Looks Like

In five years, "I'll Fetchium it" is said the way "I'll Google it" was said in 2004. Not because we replaced Google — Google does not want to be infrastructure for AI agents. We become infrastructure precisely because we are not trying to be the consumer product. We are the thing every AI agent uses to know things.

- **1M developers** have integrated the Fetchium API
- **10,000 organizations** run Fetchium on-premise for compliance
- **100,000 AI agents** use Fetchium as their primary retrieval tool
- **$100M ARR** from a combination of API usage, Pro subscriptions, and Enterprise contracts
- **Every major AI framework** (LangChain, LlamaIndex, AutoGen, CrewAI) has a Fetchium integration in its standard library

---

## 3. Core Values

Fetchium is built on four non-negotiable values. These are not aspirations. They are engineering decisions that were made before writing the first line of code, and they constrain every subsequent decision.

---

### Value 1: Speed — Rust-Native, Zero Cold Start

**Why it matters:** Slow retrieval is not just inconvenient. It breaks AI pipelines. An agent that waits 3 seconds for a search result introduces latency that compounds across tool calls. A developer who waits 8 seconds for CLI output stops using the CLI.

**What we built:** Fetchium is written entirely in Rust. Not "the performance-critical parts in Rust." All of it.

- **Zero cold start**: The binary starts in under 50ms on any machine. No JVM warmup. No Python interpreter startup. No Node.js module resolution.
- **Async-native**: Built on Tokio with fully async I/O throughout. Search, fetch, extract, analyze — all concurrent, none blocking.
- **Zero-copy parsing**: HTML parsing, JSON serialization, and token counting are designed to minimize allocations. The fastest path through the system is the common path.
- **Connection pooling**: HTTP connections are pooled and reused. Repeated queries to the same backends do not pay TCP handshake costs.
- **Lock-free telemetry**: Metrics collection uses atomic operations, never mutexes. Measuring performance does not cost performance.

**The result:** On a standard developer laptop, `fetchium ai "what is the capital of France"` completes in under 3 seconds end-to-end (search: 800ms, fetch: 200ms, AI synthesis: 1.5s). On a server with SearXNG colocated, total latency drops to under 1.5 seconds.

**The commitment:** Every new feature is benchmarked. If it makes the happy path slower, it does not ship in its current form.

---

### Value 2: Truth — Evidence-Backed, Mandatory Citations

**Why it matters:** The AI industry has a hallucination problem. LLMs confidently assert false information. Retrieval-augmented generation helps, but only if the retrieval layer is honest about what it found and where. A citation to a URL is not a citation to a claim. A citation to a claim is a pointer to a specific passage in a specific document at a specific moment in time.

**What we built:**

- **Evidence graphs**: Every synthesized response is backed by a graph of sources → passages → claims. The graph is not cosmetic — it is the primary data structure that the synthesis algorithm operates on.
- **Mandatory attribution**: The system architecture does not have a code path for generating an assertion without attribution. If a fact cannot be attributed, it is flagged as unattributed, not silently included.
- **Contradiction detection**: When two sources make conflicting claims, the system surfaces the contradiction. It does not silently pick one. It shows both and scores them by authority.
- **Source authority scoring**: Sources are scored on domain authority, publication recency, author credibility (where available), and consistency with the corpus of other sources on the same topic.
- **Temporal validity**: Claims are tagged with the recency of their source. A fact from 2018 is labeled as such, not presented alongside 2026 facts as equivalent.

**The commitment:** We will never optimize for confident-sounding answers at the expense of accurate answers. We would rather surface "insufficient evidence" than fabricate confidence.

---

### Value 3: Privacy — Local-First, No Tracking

**Why it matters:** Search queries are among the most intimate data a person generates. They reveal what you do not know, what you are afraid of, what you are planning. Sending every query to a cloud service is a data liability that many individuals, teams, and organizations cannot accept.

**What we built:**

- **Local-first by default**: The core binary runs entirely on your machine. No query is sent anywhere except the search backends you explicitly configure. The CLI does not phone home.
- **No telemetry without consent**: Zero telemetry is collected by default. The opt-in telemetry (if enabled) sends only aggregate performance metrics — never query content, never results, never user behavior.
- **Self-hostable everything**: The REST API, the SearXNG backend, the dashboard — all can be deployed on your own infrastructure. We provide Docker images, Helm charts, and systemd unit files.
- **Local AI**: Full support for Ollama, LM Studio, llama.cpp, and any OpenAI-compatible local endpoint. Your queries can be answered entirely on your own hardware with no external API calls.
- **Encrypted credential storage**: API keys and credentials are stored with OS-level encryption, not in plaintext config files.
- **GDPR-native**: The persistent intelligence engine (cross-session learning) stores data locally and provides a single command to delete it entirely.

**The commitment:** We will never build a business model that depends on user data. Revenue comes from software subscriptions and API usage, not from selling or leveraging query data.

---

### Value 4: Openness — Open Source, Community-Driven

**Why it matters:** Infrastructure that the ecosystem depends on cannot be a black box. When an AI agent's ability to retrieve information is gated behind a proprietary API with opaque ranking and unknown data practices, the agent is not trustworthy. Open source is not idealism — it is the correct engineering choice for infrastructure.

**What we built:**

- **MIT-licensed core**: `fetchium-core` and `fetchium-cli` are MIT-licensed. Use them in commercial products. Modify them. Build on them. No restrictions.
- **Public algorithms**: All 20+ novel algorithms are documented in the PRD. The implementation is in the open. Competitors can read our code. We improve by being challenged, not by being opaque.
- **Plugin architecture**: The system is designed for extensibility. Custom backends, custom extractors, custom ranking signals — all can be added without forking the core.
- **Community governance**: RFC process for major changes. Public roadmap. Community voting on feature priorities. No decisions made in private that affect the open-source users.
- **Self-hostable first**: Every feature ships with self-hosted support before SaaS support. The cloud offering is a convenience, not a requirement.

**The commitment:** We will never close-source a feature that was previously open. We will never add artificial limitations to the open-source version to push users toward paid tiers. The distinction between open and paid will always be about infrastructure costs (compute, storage, support) not features.

---

## 4. The Problem

### The Internet Is Fragmented Across Incompatible Silos

Human knowledge in 2026 does not live in one place. It has never lived in one place. But the fragmentation has become so severe that accessing knowledge comprehensively now requires an expert's knowledge of where to look before you can find out what you are looking for.

| Silo | What It Contains | What It Lacks |
|------|-----------------|---------------|
| Google | Links to web pages | Content, synthesis, verification, recency |
| YouTube | Video knowledge | Text extraction, searchability, timestamps |
| Reddit | Community wisdom, lived experience | Authority scoring, recency, accuracy filtering |
| Academic databases | Peer-reviewed research | Accessibility, readability, connection to practice |
| Twitter/X | Real-time discourse, expert opinions | Context, permanence, accuracy |
| GitHub | Code, technical documentation | Discoverability, explanation, non-code context |
| HN | Curated technical discussion | Archive depth, non-tech topics |
| LinkedIn | Professional context, career data | Personal data sensitivity, limited API |
| Substack | Long-form expert writing | Discoverability, paywall access |
| Data sources | Statistics, datasets | Interpretation, context |

**The result:** Nobody unifies them. Every knowledge worker in 2026 is a human middleware layer, manually switching between tools, copying text between tabs, and trying to hold a mental model of a question together while the tools fight them.

### The "5-Tool Problem"

To go from a non-trivial question to a trusted, well-sourced answer in 2026, the average knowledge worker needs:

1. **A search engine** (Google, Bing) — to find what exists
2. **A content reader** (browser, reader mode) — to extract what matters
3. **An AI assistant** (ChatGPT, Claude) — to synthesize what was found
4. **A fact-checker** (another search engine, Snopes, primary sources) — to verify the synthesis
5. **A citation manager** (Zotero, Notion) — to record what was used

This is five context switches, five authentication barriers, five different data models, and five places where information can be lost, misinterpreted, or inadequately attributed.

The tools were not designed to work together. They are competitors, not collaborators. Each optimizes for its own engagement metrics, not for the quality of the knowledge worker's output.

### The AI Agent Problem

The problem is worse for AI agents. An autonomous agent running in 2026 needs to retrieve current information to ground its reasoning. The options available to it are:

- **Web search APIs**: Bing's Search API was retired in August 2025. Google's APIs are expensive and designed for consumer product integration, not agent pipelines. SerpAPI and similar proxies exist but are expensive and have rate limits.
- **Scraping**: DIY scraping works until it doesn't — CAPTCHAs, JavaScript rendering, rate limits, and legal ambiguity make it unreliable for production systems.
- **Tavily**: Good developer adoption (800K+), but acquired by Nebius in February 2026, limited to web search, no multi-source capability, no extraction pipeline beyond snippets.
- **Exa**: Neural search is interesting but one-dimensional — semantic similarity is not the same as comprehensive retrieval. No social, no video, no academic.
- **Firecrawl**: Excellent for scraping, zero search capability. Solves the wrong half of the problem.

The gap is not in any one capability. The gap is in the pipeline: search + fetch + extract + verify + synthesize, working together, reliably, at the speed an agent needs.

### The Developer Tax

Every developer building an AI application today is paying what we call the "developer tax" — the engineering cost of building retrieval infrastructure that has nothing to do with their core product:

- Setting up and managing multiple search API accounts
- Writing and maintaining HTML parsers, readability extractors, PDF parsers
- Handling rate limits, circuit breakers, and backend failover
- Managing token budgets for LLM context windows
- Building citation and source tracking
- Dealing with headless browser infrastructure for JavaScript-rendered content

This tax is paid repeatedly by every team. It is not a competitive differentiator — it is undifferentiated infrastructure work that every team builds and every team does slightly wrong.

Fetchium eliminates the developer tax. One integration. One API. One binary. The entire pipeline is solved.

### The Misinformation Problem

Traditional search returns links. LLMs generate text. Neither system is architected around the question "is this actually true?"

Google PageRank was designed to surface popular content, not accurate content. LLMs are trained to generate plausible text, not necessarily factual text. RAG (retrieval-augmented generation) helps, but only if:

1. The retrieval is comprehensive (finds the right sources)
2. The extraction is accurate (pulls the right content from those sources)
3. The attribution is precise (connects claims to passages, not just URLs)
4. The synthesis is honest (surfaces contradiction, not just agreement)

No existing tool does all four. Fetchium is the first system architected with verification as a primary output, not an afterthought.

---

## 5. The Solution — 7 Fetch Modes

Fetchium unifies information retrieval across every domain through a single interface with 7 specialized modes. Each mode is optimized for its domain's specific challenges while sharing the same underlying infrastructure: multi-backend search, content extraction pipeline, AI synthesis, evidence graph generation.

**The unified interface:**
```bash
fetchium fetch --mode web "your question"
fetchium fetch --mode video "your question"
fetchium fetch --mode research "your question"
fetchium fetch --mode social "your question"
fetchium fetch --mode data "your question"
fetchium fetch --mode deep "your question"
fetchium fetch --mode monitor "your topic" --interval 1h
```

Or through the API:
```json
POST /v1/fetch
{
  "query": "your question",
  "mode": "research",
  "options": { "depth": "detailed", "sources": 20 }
}
```

---

### Mode 1: Web Fetch

**What it solves:** The core web research problem — finding, extracting, and synthesizing information from regular web pages at speed and scale.

**Sources:** SearXNG (unified access to Google, Bing, DuckDuckGo, Brave), direct site search, custom domains, RSS feeds.

**Pipeline:**
1. Multi-backend search with automatic failover (ABS — Adaptive Backend Selector)
2. Results ranked by HyperFusion (8-signal ranking: BM25 + semantic + temporal + authority + evidence + diversity + depth + consensus)
3. Full content extraction via CEP (Content Extraction Protocol: CSS selectors → readability → headless JS → PDF → screenshot OCR)
4. QATBE token budget management — fit maximum signal into any context window
5. AI synthesis with mandatory source attribution

**Performance:** 10–50 results searched, top 5–10 fetched and extracted, synthesized response in under 15 seconds.

**Unique capability:** Not just snippets. Full-page extraction with JavaScript rendering for SPAs, lazy-loaded content, and paywalled-but-cached articles.

---

### Mode 2: Video Fetch

**What it solves:** YouTube and video content is increasingly where technical knowledge, expert opinions, and educational content lives. It is completely unsearchable by content — only by title, description, and channel.

**Sources:** YouTube (transcript API), Vimeo, Loom (transcript extraction), Twitch VODs, conference recordings.

**Pipeline:**
1. Video search across platforms
2. Transcript extraction (YouTube API where available, Whisper for transcripts not otherwise available)
3. Timestamp-indexed segment extraction — find the specific 2-minute section that answers the question without watching the full 3-hour conference talk
4. Speaker identification and attribution
5. Cross-video synthesis ("what do 5 different experts say about X?")

**Unique capability:** Query-aware timestamp extraction. Ask "how does the speaker explain backpressure in this Tokio talk?" and get the 90 seconds of transcript that directly answers it, with a timestamp link.

---

### Mode 3: Research Fetch

**What it solves:** Academic and scientific research is locked behind paywalls, formatted for human reading rather than machine processing, and disconnected from the broader web of evidence.

**Sources:** arXiv, Semantic Scholar, PubMed, CrossRef, OpenAlex, Google Scholar (via search), preprint servers, institutional repositories.

**Pipeline:**
1. Semantic query expansion for academic terminology (CLQB — Cross-Lingual Query Boost)
2. Paper retrieval with citation graph traversal
3. Abstract and full-text extraction (PDF pipeline: pdfium → text extraction → structure parsing)
4. Citation network analysis — find the papers that cite and are cited by the most relevant results
5. Evidence synthesis with academic citation format (APA, MLA, Chicago, BibTeX)

**Unique capability:** Citation network traversal. Starting from one relevant paper, Fetchium can automatically find the 10 most important related papers by following citation graphs forward and backward, surfacing the academic consensus on a topic.

---

### Mode 4: Social Fetch

**What it solves:** The most real-time, human signal on the internet — what people actually think, not what they have published — lives on social platforms that are notoriously difficult to query programmatically.

**Sources:** Twitter/X (via SearXNG site:x.com search), Reddit (native API), Hacker News (Algolia API), Facebook (via SearXNG site:facebook.com), TikTok (tikwm.com API), LinkedIn (via search).

**Pipeline:**
1. Platform-specific query optimization (each social platform has different search semantics)
2. Sentiment analysis per source and per claim
3. Engagement-weighted ranking (viral content is weighted differently from niche expert content)
4. Bot and low-quality content filtering
5. Opinion aggregation with attribution to user segments, not individuals

**Unique capability:** Cross-platform opinion synthesis. "What does the developer community think about X?" pulls from Reddit's programming communities, Hacker News, Twitter technical circles, and aggregates a weighted view with specific representative quotes.

---

### Mode 5: Data Fetch

**What it solves:** Statistical claims, market data, and quantitative information are scattered across government databases, data repositories, financial data providers, and research institution publications — all in incompatible formats.

**Sources:** World Bank API, Our World in Data, Statista (via search), government open data portals (data.gov, eurostat), financial data (Yahoo Finance, FRED), GitHub (CSV/JSON datasets).

**Pipeline:**
1. Data source discovery for the query domain
2. Format normalization (CSV, JSON, XML, HTML tables → unified schema)
3. Unit standardization and time series alignment
4. Statistical context — the data point plus the trend, the comparison, and the source methodology
5. Inline visualization descriptions (for TUI/CLI) or chart data export (for API consumers)

**Unique capability:** Automatic data provenance. Every number comes with its source, its date, its methodology notes, and where to find the raw dataset. Not "the unemployment rate is 4.2%" but "the unemployment rate was 4.2% (U.S. Bureau of Labor Statistics, January 2026, seasonally adjusted — see https://fred.stlouisfed.org/series/UNRATE)."

---

### Mode 6: Deep Fetch

**What it solves:** Some questions cannot be answered by surface-level retrieval. A due diligence report. A competitive analysis. A technical research brief. These require depth, synthesis, and multi-step reasoning across many sources.

**Pipeline:**
1. AMRS (Adaptive Multi-Agent Research Swarm) — 4 concurrent agent types:
   - **Scout agents**: fast parallel search across all sources and modes
   - **Analyst agents**: deep extraction and cross-source synthesis
   - **Critic agents**: contradiction detection, claim verification
   - **Synthesist agents**: final narrative construction with full evidence graph
2. RAR (Retry-and-Refine) — 5-checkpoint self-correction loop
3. PIE (Persistent Intelligence Engine) — draws on past queries and learned source trust
4. Multi-tier PDS output: key_facts (~200 tokens) → summary (~1000) → detailed (~5000) → complete

**Time budget:** Deep Fetch is designed for queries where 1–5 minutes is acceptable for comprehensively-researched, deeply-verified output. Not fast — thorough.

**Unique capability:** The evidence graph output. Every claim in a Deep Fetch response is a node in a graph. Edges connect claims to sources, sources to authors, authors to institutions, institutions to potential conflicts of interest. The full provenance chain is exportable as JSON-LD for downstream processing.

---

### Mode 7: Monitor

**What it solves:** Information is not static. Prices change. Research is published. Companies make announcements. Laws are passed. A retrieval tool that only answers point-in-time queries misses the most valuable use case: staying current on a topic over time.

**Pipeline:**
1. Query registration with configurable interval (1h, 6h, 24h, 7d)
2. Differential analysis — what has changed since the last run?
3. Significance scoring — is this change important enough to notify?
4. Configurable delivery: webhook, email, Slack, CLI output, API polling endpoint
5. Trend tracking — not just "what changed" but "how is this topic evolving"

**Use cases:**
- Track a competitor's pricing page and be notified of changes
- Monitor arXiv for new papers on a specific topic
- Watch Reddit/HN for emerging sentiment shifts on your product
- Track government agency pages for regulatory updates
- Monitor a client's press coverage and competitor mentions

**Unique capability:** Semantic diffing. Monitor does not just detect page changes (like a website uptime monitor). It detects *meaningful* changes — a price change in a table, a new section in a document, a new claim in a regularly-updated report — and ignores cosmetic changes like nav bar updates or advertisement changes.

---

## 6. The 5-Layer Information Stack

Fetchium is not a search engine. It is a complete information processing stack. Understanding its architecture requires understanding each layer and how they compose.

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 5: ACT                                               │
│  API · Agent Integration · MCP · Monitoring · Automation    │
├─────────────────────────────────────────────────────────────┤
│  Layer 4: KNOW                                              │
│  PIE · Cross-Session Memory · Source Trust · Query History  │
├─────────────────────────────────────────────────────────────┤
│  Layer 3: LEARN                                             │
│  AI Synthesis · Evidence Graphs · Contradiction Detection   │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: FETCH                                             │
│  CEP · QADD · Browser Automation · PDF · OCR               │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: FIND                                              │
│  Multi-Backend Search · ABS · Circuit Breakers · Resilience │
└─────────────────────────────────────────────────────────────┘
```

---

### Layer 1: FIND — Multi-Backend Search

The foundation. Before anything can be extracted, analyzed, or verified, it must be found. Fetchium's FIND layer is architected around two realities: no single search backend is reliable, and no single search backend covers the full range of sources.

**Components:**

- **ABS (Adaptive Backend Selector)**: Intelligently routes queries to the optimal backend(s) based on query intent, backend health, latency history, and result quality scoring. A code query goes to GitHub and Stack Overflow; a news query goes to SearXNG with recency weighting; a research query goes to arXiv and Semantic Scholar.

- **Multi-backend resilience**: Circuit breakers prevent cascading failures when a backend goes down. Rate limiters prevent API key exhaustion. Bulkheads isolate backend failures so one slow backend does not block results from fast ones.

- **SPRE (Speculative Pre-Ranking)**: While results are still arriving from backends, SPRE begins pre-scoring early results. When the last results arrive, the top candidates have already been partially ranked, shaving 200–400ms off total latency.

- **QFD (Query Fingerprinting)**: Classifies query intent (10 variants: Factual, HowTo, Comparison, Verification, CurrentEvents, DeepAnalysis, Code, Academic, Opinion, Data) to optimize backend selection and extraction strategy.

- **HyperFusion (8-signal ranking)**: Once results are collected, HyperFusion ranks them by combining BM25 text relevance, semantic similarity, temporal decay, domain authority, evidence density, result diversity, content depth, and cross-source consensus. No single signal dominates; the weights are dynamically adjusted by query intent.

**Resilience SLA:** If the primary backend is unavailable, fallback to secondary within 200ms. If all backends for a source type are unavailable, surface a clear error with source-specific guidance, never a silent empty result set.

---

### Layer 2: FETCH — Content Extraction

Finding a URL is 10% of the work. Getting the content — the actual text, structured data, or media that answers the question — is the other 90%, and it is the part that every DIY solution gets wrong.

**CEP (Content Extraction Protocol) — 5-layer cascade:**

Each layer is tried in sequence. If a layer succeeds (extracts content above the quality threshold), subsequent layers are skipped. This minimizes latency while guaranteeing maximum coverage.

```
Layer 1: CSS Selectors
    ↓ (if insufficient content)
Layer 2: Mozilla Readability (port in Rust)
    ↓ (if insufficient content or JavaScript-rendered page)
Layer 3: Headless Chrome (chromiumoxide)
    ↓ (if PDF or document)
Layer 4: PDF Extraction (pdfium)
    ↓ (if image or OCR needed)
Layer 5: Screenshot OCR
```

- **Layer 1 (CSS):** Fastest path. For well-structured sites (news, documentation, blogs), CSS-based extraction finds the article body in under 50ms.
- **Layer 2 (Readability):** Mozilla's Readability algorithm, ported to Rust. Handles most content sites, strips navigation, ads, footers. 80–200ms.
- **Layer 3 (Headless JS):** For SPAs, React apps, lazy-loaded content, dynamic tables, and pages that require JavaScript to render meaningful content. 1–5s. Budget managed by the browser pool.
- **Layer 4 (PDF):** pdfium-based text extraction for PDFs with embedded text. Structure-aware: tables are extracted as tables, not as runs of text.
- **Layer 5 (OCR):** Last resort for scanned PDFs, image-based documents, or any content that exists as pixels. Uses Tesseract with layout analysis.

**QADD (Query-Aware DOM Distillation):** Before extraction, the DOM is pruned to remove content irrelevant to the query. Navigation menus, footers, comment sections, related articles, advertisements — all removed. The query is used to score DOM subtrees, keeping only what is relevant. This achieves 10–20x token reduction on complex pages without losing signal.

---

### Layer 3: LEARN — AI Analysis and Evidence Graphs

Raw extracted content is not yet an answer. LEARN is the layer that transforms content into understanding.

**AI Synthesis Pipeline:**

- **Multi-provider support**: Gemini (default, gemini-2.5-flash), OpenAI (GPT-4o, o3), Anthropic (Claude 3.5 Sonnet, Claude Opus 4), Ollama (fully local), OpenRouter (access to 50+ models).
- **Key pooling**: Multiple API keys per provider are pooled and cycled to handle rate limits without interruption. A 429 response triggers automatic key rotation, not a user-visible error.
- **QATBE (Query-Aware Token-Budgeted Extraction)**: BM25-scored segment ranking with greedy knapsack packing. The most relevant 10% of content fits into a tight token budget without losing the key signal. PDS (Progressive Detail Streaming) provides 4 output tiers: key_facts (~200 tokens), summary (~1000), detailed (~5000), complete.
- **Structured prompting**: The synthesis prompt is dynamically constructed from query intent, source authority scores, and requested output format. A "verification" query generates a different prompt than a "howto" query.

**Evidence Graphs:**

The evidence graph is the primary innovation in LEARN. It is a directed graph where:
- **Nodes** are claims (atomic factual assertions extracted from sources)
- **Edges** connect claims to sources, sources to domains, and claims to contradicting claims
- **Weights** encode confidence, source authority, and recency

The synthesis algorithm does not write prose and add citations afterward. It walks the evidence graph and constructs a response that mirrors the structure of evidence: well-supported claims are stated confidently, contested claims are qualified, unsupported claims are flagged.

**EGB (Evidence Graph Builder):** Constructs the graph incrementally as sources arrive. Claims are deduplicated by semantic similarity, not string matching. The same fact from three different sources becomes one high-confidence node with three supporting edges, not three separate assertions.

---

### Layer 4: KNOW — Persistent Intelligence Engine

Information retrieval that improves over time is qualitatively different from retrieval that treats every query in isolation. KNOW is the layer that makes Fetchium a learning system.

**PIE (Persistent Intelligence Engine):**

- **Source trust learning**: Every source interaction is scored. Pages that consistently contain high-quality, accurate content are scored higher. Sources that are frequently found misleading or thin are downranked. This learning is per-user, stored locally in SQLite.
- **Query history**: Past queries inform future ones. If you have researched a topic before, Fetchium knows which sources were most useful for your query style and pre-weights them.
- **Query prediction**: Based on query history and in-progress query context, Fetchium can prefetch likely follow-up sources before you ask the follow-up question.
- **Failure pattern learning**: When a backend fails, extraction fails, or AI synthesis produces a low-quality result, the failure is recorded. Future queries avoid the same failure modes.

**Privacy guarantee:** All PIE data is stored locally in `~/.fetchium/pie.db`. A single `fetchium forget --all` command deletes it. The data is never sent to any external service.

**LP (Latency Predictor):** Predicts expected latency for a given query based on backend health history, recent latency measurements, and query complexity. Informs timeout settings and user-facing progress indicators.

---

### Layer 5: ACT — Integration and Automation

Knowledge that cannot be acted on is not useful. ACT is the layer that makes Fetchium a building block for larger systems.

**REST API:**
- Full OpenAPI 3.1 specification, auto-generated from the Axum handlers
- Endpoints: `/v1/search`, `/v1/fetch`, `/v1/research`, `/v1/monitor`
- Authentication: API key (`X-API-Key` header) or session token
- Rate limiting: Configurable per key, per IP, per endpoint
- Response formats: JSON (default), NDJSON (streaming), Markdown, plain text

**MCP Server:**
- Implements the Model Context Protocol (MCP), making Fetchium a first-class tool for Claude, GPT-4, and any MCP-compatible AI system
- Tools exposed: `web_search`, `fetch_page`, `research_topic`, `verify_claim`, `monitor_topic`
- Works with Claude Desktop, Claude Code, and any MCP client

**CLI:**
- `fetchium` binary — zero dependencies, single statically-linked binary
- Full feature parity with the API
- Shell completion for bash, zsh, fish
- Structured output modes: JSON, table, markdown, plain
- TUI mode for interactive exploration

**SDK (planned, Phase 4):**
- Rust crate (`fetchium-core`) — use the full pipeline in your Rust application
- Python bindings (`fetchium-py`) — via PyO3
- Node.js bindings (`fetchium-node`) — via Neon
- TypeScript types for the REST API (auto-generated)

**Agent Integration:**
- LangChain tool integration (Python and JS)
- LlamaIndex reader integration
- AutoGen tool wrapper
- CrewAI tool integration
- Standard OpenAI function calling format

---

## 7. Technical Moat — 20+ Novel Algorithms

Fetchium's competitive advantage is not a proprietary dataset, a closed model, or a locked-in platform. It is 20+ novel algorithms that took 18 months to design, implement, test, and tune. Each one solves a specific problem that every retrieval system faces and that no other open-source system has solved at this level of sophistication.

This is code that can be read, but cannot be easily replicated. The implementation is open — the insight that produced it is not easily reproduced without the same depth of problem analysis.

---

### Algorithm Group 1: Search and Routing

**ABS — Adaptive Backend Selector**
Dynamically routes queries to the optimal set of backends based on query intent classification, real-time backend health, recent latency distribution, and result quality feedback. Not round-robin. Not static priority. Adaptive, per-query optimization.

- Query intent → backend affinity matrix
- Real-time health scoring (exponential moving average of success rates)
- Latency budget allocation across parallel backends
- Quality feedback loop: result click-through and synthesis quality inform future routing

**SPRE — Speculative Pre-Ranking**
Reduces effective ranking latency by beginning to rank results as they arrive rather than waiting for all backends to complete. Partial rankings are computed speculatively and merged as new results arrive.

- Merge-heap based incremental ranking
- Confidence-bounded early termination
- Reduces p50 total latency by 200–400ms

**QFD — Query Fingerprinting**
Classifies query intent into 10 categories (Factual, HowTo, Comparison, Verification, CurrentEvents, DeepAnalysis, Code, Academic, Opinion, Data) with sub-millisecond latency using a local rule-based classifier augmented with lightweight ML features.

- Zero network calls — runs entirely locally
- Intent classification propagates through all subsequent layers
- Different intent classes produce different backend routing, extraction depth, and synthesis prompts

**HyperFusion — 8-Signal Ranking**
The core ranking algorithm. Eight signals, each normalized to [0,1], weighted by query intent:

1. **BM25** — term frequency relevance (classic IR)
2. **Semantic** — embedding-based similarity (optional, requires `embeddings` feature)
3. **Temporal** — recency decay (configurable half-life per domain category)
4. **Authority** — domain authority and author credibility scoring
5. **Evidence** — density of citable claims in the content
6. **Diversity** — penalizes results that are too similar to already-selected results
7. **Depth** — rewards comprehensive content over thin content
8. **Consensus** — rewards content that agrees with the majority of other high-authority sources

---

### Algorithm Group 2: Content Extraction

**CEP — Content Extraction Protocol**
The 5-layer cascade described in Layer 2. The key innovation is the quality threshold system: each layer produces a content quality score (based on text density, DOM structure, content-to-noise ratio, and query relevance). The cascade stops at the first layer that exceeds the threshold, minimizing latency without sacrificing coverage.

**QADD — Query-Aware DOM Distillation**
5-step DOM pruning process:
1. DOM tree construction from raw HTML
2. Query-aware relevance scoring of DOM subtrees (BM25 on text content)
3. Low-relevance subtree removal (below configurable threshold)
4. Structural normalization (collapse single-child nodes, merge adjacent text)
5. Token counting and threshold enforcement

Achieves 10–20x token reduction on complex pages with under 5% information loss (measured by recall of key claims from unprocessed content).

**QATBE — Query-Aware Token-Budgeted Extraction**
After content extraction, QATBE fits the maximum relevant signal into a configurable token budget using:
1. BM25 scoring of extracted segments against the query
2. Greedy knapsack allocation (highest score-per-token segments are included first)
3. Coherence constraints (avoids including segments that lose meaning without context)
4. Budget-aware summarization for segments too long for the budget (if AI summarization is enabled)

---

### Algorithm Group 3: Ranking and Scoring

**TDR — Temporal Decay Ranking**
Recency is not a binary property. Content published 1 day ago is not equally preferable to content published 30 days ago for all query types. TDR computes per-query, per-domain half-lives:

- News queries: 6-hour half-life
- Technology queries: 30-day half-life
- Scientific queries: 1-year half-life (but flags as outdated if newer research exists)
- Historical queries: no decay
- Market data queries: 1-hour half-life

**RCE — Result Clustering Engine**
Groups results by semantic similarity before final ranking, ensuring that HyperFusion's diversity signal has accurate clusters to work with. Uses locality-sensitive hashing for near-linear time complexity even at scale.

**QXE — Query Expansion Engine**
Expands the original query with semantically related terms, synonyms, and domain-specific terminology. Different expansion strategies for different query intents:
- Factual: expand with known aliases and common phrasings
- Academic: expand with domain-specific terminology and citation keywords
- Code: expand with language-specific syntax and common variable names
- Comparison: generate separate queries for each compared entity

**RDO — Result Diversity Optimizer**
Implements Maximal Marginal Relevance (MMR) to balance relevance and diversity in the final result set. Ensures that the top 10 results cover the information space rather than all pointing to the same cluster.

---

### Algorithm Group 4: Quality and Verification

**RQE — Result Quality Estimator**
Scores each result before full content extraction to determine whether the expense of extraction is justified. Features:
- Domain trust score (pre-computed from historical quality)
- Content length estimate (from HTTP headers and HTML structure)
- Age and recency
- Structural signals (presence of dates, author attribution, citations)

Allows Fetchium to skip low-quality results early, spending extraction budget only on likely high-quality sources.

**STP — Source Trust Pipeline**
Maintains a per-domain trust model that is updated with every interaction. Trust factors:
- Factual accuracy (agreement with high-consensus sources)
- Content quality (readability score, citation density)
- Recency maintenance (regularly updated vs. stale)
- Bias indicators (language sentiment, one-sided framing)

Trust scores are stored locally and improve over time. A source that has consistently provided accurate, well-cited content gets a higher prior in ranking.

**EGB — Evidence Graph Builder**
Constructs the evidence graph described in Layer 3. Technical implementation:
- Claims are extracted as atomic assertions using dependency parsing
- Semantic deduplication using cosine similarity on claim embeddings
- Contradiction detection: claims with high similarity but opposing sentiment
- Source attribution: each claim node has a provenance edge to its extraction location

---

### Algorithm Group 5: Intelligence and Research

**AMRS — Adaptive Multi-Agent Research Swarm**
4 agent types coordinated over Tokio channels:
- **Scout agents** (parallelism: N, where N = number of backends): Fan out to all configured backends simultaneously
- **Analyst agents** (parallelism: 3–5): Deep extraction and cross-source synthesis
- **Critic agents** (parallelism: 2): Contradiction detection, claim verification
- **Synthesist agents** (parallelism: 1): Final narrative construction

Agents communicate via message-passing (never shared state). The Synthesist waits for a quorum signal from Critics before finalizing output.

**PIE — Persistent Intelligence Engine**
Cross-session learning via SQLite. Schema:
- `source_trust`: per-domain trust history
- `query_history`: past queries with quality scores
- `failure_patterns`: backend/extraction failures for avoidance
- `query_predictions`: prefetch candidates based on in-progress queries

**RAR — Retry-and-Refine**
5-checkpoint self-correction loop:
1. Initial synthesis check (quality score below threshold → retry with different prompt)
2. Citation coverage check (unsupported claims > threshold → request additional sources)
3. Contradiction resolution check (unresolved contradictions → explicit qualification)
4. Completeness check (key query aspects unaddressed → targeted sub-queries)
5. Confidence calibration (overconfident assertions → hedging pass)

**ATB — Adaptive Token Budget**
Dynamically allocates token budget across the information extraction pipeline based on:
- Available model context window
- Query complexity score
- Number of sources being synthesized
- Requested output detail tier (PDS: key_facts → summary → detailed → complete)

**AXE — Answer Extraction Engine**
For factual queries with definitive answers, AXE identifies the specific sentence or data point that directly answers the question and surfaces it prominently before the full synthesis. Converts "find the passage that answers this" from a linear search into a relevance-weighted binary search.

---

### Algorithm Group 6: Query Enhancement

**QCE — Query Complexity Estimator**
Estimates the expected complexity of answering a query before execution. Used to:
- Allocate agent count for AMRS
- Set timeout budgets per extraction layer
- Determine whether to invoke RAR self-correction
- Choose appropriate PDS output tier

**CLQB — Cross-Lingual Query Boost**
Expands queries into multiple languages for sources that may have better coverage in non-English languages. Particularly valuable for academic queries (much research is published in non-English journals) and region-specific topics.

**LP — Latency Predictor**
Predicts end-to-end latency for a given query configuration. Used to:
- Set user-facing progress estimates
- Adapt timeout thresholds dynamically
- Trigger early termination when budget is exceeded
- Recommend alternative modes for time-sensitive use cases

**SSE — Smart Snippet Engine**
Extracts optimal context snippets from retrieved content — not just the first 200 characters, not just the title and meta description. The snippet is selected by finding the passage with the highest query-relevance score that also contains enough surrounding context to be interpretable in isolation.

---

## 8. Market Opportunity

### The Size of the Prize

Fetchium is entering a market at the moment of its maximum growth and maximum disruption simultaneously.

**AI Search Engine Market:**
- 2025 size: **$43.6 billion**
- 2032 projection: **$108.9 billion**
- CAGR: **14%**
- Primary drivers: AI agent proliferation, developer tool adoption, enterprise AI initiatives

**AI Agents Market (primary customer):**
- 2025 size: **$8 billion**
- 2026 projection: **$12 billion** (+50% YoY)
- Every AI agent needs retrieval infrastructure — Fetchium is that infrastructure

**Developer Tools Market (adjacent):**
- Cursor's trajectory: $100M ARR in 12 months, zero marketing budget
- GitHub Copilot: 1M+ paying users within months of launch
- JetBrains: $500M+ ARR from developer tools
- The developer productivity market is large, growing, and price-insensitive when the tool is genuinely valuable

### The Bing API Retirement Gap

Microsoft retired the Bing Search API in **August 2025**. This was the primary search API used by:
- LangChain's web search integration
- LlamaIndex's search tools
- Dozens of enterprise RAG pipelines
- Hundreds of smaller AI applications

This created a vacuum. The options teams were forced to migrate to are each inadequate:
- **SerpAPI**: Expensive ($50–$130/mo for meaningful volume), no extraction pipeline
- **Brave Search API**: Limited rate limits, no multi-source, no extraction
- **Tavily**: Good but acquired and single-source
- **Google Custom Search**: Expensive, 100 queries/day on free tier, no extraction

Fetchium is positioned to be the obvious migration target: open-source, self-hostable (zero per-query cost), multi-source, and with a full extraction pipeline.

### The Perplexity Ceiling

Perplexity AI has demonstrated enormous demand ($656M ARR projected, 45M+ MAUs, $20B valuation). But Perplexity is built as a consumer product, not developer infrastructure:
- No meaningful API offering at scale
- Closed-source, cannot be self-hosted
- No extraction pipeline (snippets only)
- No multi-source (primarily web)
- No evidence graph or citation verification
- No social, video, academic specialization

Perplexity proves the market. Fetchium serves the market that Perplexity does not: developers, AI agents, enterprises requiring control, and users requiring verification.

### The Open Source Advantage

Supabase went from 1M to 4.5M developers in under a year. Their primary acquisition channel: open source, community, and word-of-mouth from developers who found it genuinely useful.

Open source compounds:
- **Trust**: Security-conscious developers and enterprises can audit the code
- **Integration**: Community contributes integrations (LangChain, LlamaIndex, etc.)
- **Distribution**: GitHub stars generate discovery; stars generate press; press generates users
- **Enterprise**: "We started with the open-source version" is the most common enterprise sales path

The total addressable market for Fetchium is every developer, team, and organization that needs to retrieve, process, and act on internet information — which is, increasingly, all of them.

---

## 9. Competitive Landscape

### Perplexity AI

**What they are:** A consumer AI search product. Users type questions, get AI-synthesized answers with source links.

**Revenue:** $656M ARR projected (2026). $20B valuation. 45M+ MAUs.

**Strengths:**
- Beautiful consumer UX
- Strong brand recognition ("Perplexity" has become a verb in some circles)
- Fast response times for simple queries
- Good web coverage
- Significant VC funding for infrastructure investment

**Fatal weaknesses for the developer/enterprise market:**
- **No meaningful API**: Their API is expensive, rate-limited, and designed for building Perplexity-like interfaces, not for powering arbitrary AI pipelines
- **Closed source**: Cannot be audited, cannot be self-hosted, complete vendor lock-in
- **No extraction pipeline**: Returns snippets and citations, not full content. Cannot get the full page content of a result.
- **No social/video/academic specialization**: One mode — web search
- **No evidence graph**: Citations link to pages, not to specific claims within pages
- **No developer story**: Perplexity is a product, not a platform

**Fetchium position vs. Perplexity:** We are not competing for Perplexity's users. We are competing for the developers and enterprises who need what Perplexity cannot provide: a programmable, verifiable, multi-source retrieval layer with full content access.

---

### Tavily

**What they are:** A search API designed for AI agents, founded by Assaf Elovic (former RAG researcher). Acquired by Nebius in February 2026.

**Revenue/Scale:** 800K+ developers using the free tier. $25M in funding pre-acquisition.

**Strengths:**
- Good developer adoption and documentation
- LangChain and LlamaIndex integrations out of the box
- Simple API (one call, get results with snippets)
- Reasonable pricing for low volume

**Fatal weaknesses:**
- **Acquired**: Now part of Nebius's infrastructure play. Direction uncertain. Risk of deprecation or pricing changes.
- **Web-only**: No social, no video, no academic, no data sources
- **Snippets only**: No full content extraction pipeline
- **No self-hosting**: Cloud-only, per-query pricing
- **No verification layer**: Returns results, no evidence graph, no contradiction detection
- **No persistent intelligence**: No cross-session learning

**Fetchium position vs. Tavily:** Tavily is the comparison we want. For developers who are currently using Tavily and want to go deeper — more sources, full content, self-hosting, verification — Fetchium is the natural next step.

---

### Exa

**What they are:** A neural search API that uses embeddings to find semantically similar content rather than keyword matching. $17M Series A from Lightspeed, Nvidia, YC.

**Strengths:**
- Genuinely differentiated semantic search
- Strong funding
- Interesting neural retrieval approach
- Good for "find content similar to X" use cases

**Fatal weaknesses:**
- **One-dimensional**: Semantic similarity is powerful but not sufficient. Many queries need recency, authority, contradiction detection, or specific source type matching that pure semantic search does not provide.
- **Web-only**: No multi-source capability
- **No extraction pipeline**: Returns URLs and snippets
- **No social/video/academic/data**: Single mode
- **Expensive**: At scale, neural search costs add up quickly

**Fetchium position vs. Exa:** Exa wins at "find pages semantically similar to this." Fetchium wins at "answer this question using every relevant source on the internet, fully extracted and verified."

---

### Firecrawl

**What they are:** A web scraping and extraction API. Converts any website to clean markdown. $1M ARR in short time post-launch.

**Strengths:**
- Excellent at what it does: clean content extraction from web pages
- Good developer UX
- Fast
- Handles JavaScript rendering

**Fatal weaknesses:**
- **No search**: Cannot find relevant pages — must be given a URL
- **No multi-source**: Only web pages
- **No synthesis**: Returns extracted content, no analysis
- **No verification**: No evidence graph, no citation verification
- **No monitoring**: Point-in-time only

**Fetchium position vs. Firecrawl:** Firecrawl solves Layer 2 (FETCH). Fetchium solves Layers 1–5. They are complementary, not competitors. CEP's extraction capabilities match or exceed Firecrawl's for most use cases, and Fetchium adds four more layers on top.

---

### Jina AI (Reader)

**What they are:** An AI infrastructure company with reader.jina.ai — a service that converts any URL to clean text with a simple prefix (`r.jina.ai/https://...`).

**Strengths:**
- Extremely simple UX (URL prefix)
- Good adoption for simple extraction use cases
- Fast

**Fatal weaknesses:**
- **No search**: URL-only
- **No multi-source**: Web pages only
- **Rate limited**: Free tier severely limited
- **No extraction depth**: Does not handle complex JavaScript SPAs well
- **No analysis or verification**: Pure extraction, no intelligence

**Fetchium position vs. Jina Reader:** Jina Reader is a convenience tool. Fetchium is infrastructure. Different use cases, different scales, different value propositions.

---

### Google (and Google Search API)

**What they are:** The dominant search engine. Google does not have a meaningful developer API for AI integration — the Custom Search API is throttled, expensive, and returns 10 results per query.

**Strengths:**
- Best index in the world (200+ billion pages)
- Infrastructure that cannot be replicated
- Dominant consumer brand

**Fatal weaknesses for AI/developer use:**
- **Ad-driven**: Search results are influenced by advertising, not pure relevance
- **No extraction**: Returns links only
- **Expensive and throttled API**: $5 per 1000 queries, 100 free/day
- **No developer API for AI**: No content extraction, no evidence graphs, no multi-source
- **Privacy concerns**: All queries pass through Google's surveillance infrastructure
- **No self-hosting possible**: Cannot be replicated

**Fetchium position vs. Google:** We use SearXNG as our primary search backend, which itself aggregates Google, Bing, DuckDuckGo, and others — without the tracking, without the throttling, and on our own infrastructure. Fetchium users get the breadth of Google's index without the limitations of Google's API.

---

### The Competitive Matrix

| Capability | Fetchium | Perplexity | Tavily | Exa | Firecrawl | Jina |
|-----------|----------|------------|--------|-----|-----------|------|
| Multi-source (web+social+video+academic) | YES | No | No | No | No | No |
| Full content extraction | YES | No | No | No | YES | YES |
| Evidence graph + verification | YES | No | No | No | No | No |
| Self-hostable | YES | No | No | No | No | No |
| Open source | YES | No | No | No | Partial | Partial |
| AI synthesis | YES | YES | Limited | No | No | No |
| Developer API | YES | Limited | YES | YES | YES | YES |
| Social media intelligence | YES | No | No | No | No | No |
| Persistent learning (PIE) | YES | No | No | No | No | No |
| Multi-agent research (AMRS) | YES | No | No | No | No | No |
| MCP server | YES | No | No | No | No | No |
| Local AI (Ollama) | YES | No | No | No | No | No |
| Monitoring / alerts | YES | No | No | No | No | No |

Fetchium is the only platform that ticks all boxes. There is no other single tool that does what Fetchium does.

---

## 10. Differentiation

### The Only Platform at the Intersection

Fetchium's differentiation is not about being better at one thing. It is about being the only platform that operates at the intersection of four capabilities that every other tool treats as separate concerns:

```
        SEARCH
           |
    ┌──────┴──────┐
    │             │
EXTRACTION    ANALYSIS
    │             │
    └──────┬──────┘
           │
      VERIFICATION
```

Every competitor is strong at one or two of these. No competitor is strong at all four. And the value is not additive — it is multiplicative. Search without extraction gives you links. Extraction without analysis gives you text dumps. Analysis without verification gives you confident hallucinations. Verification without search cannot find new evidence.

The four together, working in a unified pipeline, produce something qualitatively different from any of the four in isolation: **trusted knowledge**.

---

### Unique Differentiator 1: Multi-Source Breadth

**What we have that nobody else has:** Seven distinct source domains, each with specialized handling, all queryable through a single interface.

Perplexity searches the web and some curated sources. Tavily searches the web. Exa searches the web (semantically). Firecrawl scrapes given URLs. None of them combine web + social + video + academic + data + code in a unified query interface.

This matters because the answer is often not on any single type of source:

- "Is this startup worth investing in?" — web (news, coverage), social (founder reputation, community sentiment), data (market size statistics), academic (research on the problem domain)
- "What do developers think of this library?" — social (Reddit, HN, Twitter), code (GitHub issues, stars), web (blog posts, tutorials)
- "What is the current scientific consensus on X?" — academic (papers, citation networks), web (science journalism), social (expert Twitter)

Fetchium is the only tool that answers all three of these queries correctly, because it is the only tool that goes to all the right sources.

---

### Unique Differentiator 2: Verification Layer

**What we have that nobody else has:** An evidence graph that makes every claim traceable to a specific passage in a specific source at a specific time.

The AI industry has a trust deficit. Users who have been burned by LLM hallucinations have learned to verify AI-generated content manually. This reduces the value of AI assistants — if you have to fact-check everything anyway, what did you save?

Fetchium's mandatory citation architecture changes this. The output is not "here is an answer, trust me." The output is "here is a claim, here is the passage it comes from, here is the source, here is the date, here is the authority score of the source." Verification is built in, not bolted on.

For enterprise use cases — legal research, competitive intelligence, financial analysis, healthcare information — this is not a nice-to-have. It is a requirement.

---

### Unique Differentiator 3: Personal Knowledge OS

**What we have that nobody else has:** A retrieval system that gets better over time for your specific use case.

PIE (Persistent Intelligence Engine) tracks which sources have been most reliable for your queries, which query patterns lead to high-quality results, and which backends perform best for your domain. Over time, your Fetchium instance is more valuable than a colleague's instance who uses it differently, because it has learned your research patterns.

This is what Google should have been for power users. Personalized retrieval, not personalized advertising.

---

### Unique Differentiator 4: Developer API with Full Content Access

**What we have that nobody else has:** An API that returns not just search results and snippets, but fully extracted, parsed, and token-budgeted content from every source.

Tavily returns snippets. Exa returns snippets. SerpAPI returns snippets. Developers who need the full content of search results have to implement their own extraction layer on top of these APIs — which means they are paying the developer tax we described in Section 4.

Fetchium's API returns fully extracted content. A developer building an AI pipeline calls `/v1/fetch`, passes a query and a token budget, and gets back structured content ready to pass directly to an LLM context window. No additional scraping layer needed.

---

### The Developer Story

The single best articulation of Fetchium's differentiation for a developer audience:

> "You are building an AI agent that needs to research a topic. You want it to search the web, check Reddit for community opinions, find academic papers, get the latest statistics, and synthesize all of this into a brief. With any other tool, you need five API integrations and a custom extraction layer. With Fetchium, you need one API call."

```python
import fetchium

result = fetchium.research(
    query="best practices for Rust async error handling",
    modes=["web", "social", "code", "academic"],
    depth="detailed",
    token_budget=4000
)

# result.synthesis: AI-generated brief with source attribution
# result.evidence_graph: Full provenance graph
# result.sources: All extracted sources with full content
# result.contradictions: Any conflicting claims surfaced
```

This is the API that nobody else offers. This is the developer story.

---

## 11. Brand Identity

### The Name: Fetchium

**"Fetch"** — the verb that precisely describes what we do. Not "search" (which implies finding links). Not "browse" (which implies exploration). Not "query" (which is too technical). "Fetch" means: you asked, we went and got it. The precision and completion implied by "fetch" is exactly what we deliver.

**"-ium"** — the suffix that transforms a verb into an element. Helium. Uranium. Titanium. Chromium. These are not products — they are fundamental materials. "-ium" signals: this is not a consumer app. This is infrastructure. It is a constituent element of whatever you build with it.

**Combined:** Fetchium is a fundamental retrieval element. Engineered. Precise. Reliable. A building block, not a destination.

---

### The Tagline: "You ask. Fetchium fetches."

This tagline does three things simultaneously:

1. **Defines the value**: Your role is to ask. Our role is to fetch. The division of labor is clear and the implied promise is complete.
2. **Asserts reliability**: "Fetchium fetches" is a guarantee. Not "Fetchium tries" or "Fetchium searches." It fetches.
3. **Makes the brand a verb**: "Fetchium fetches" primes the pump for "I'll Fetchium it" as a natural language construction.

Alternative taglines under consideration:
- "Fetch anything. Verified. Fast."
- "The internet, retrieved."
- "Find. Fetch. Know."
- "Information, extracted."

The primary tagline remains: **"You ask. Fetchium fetches."**

---

### The Goal: Make "Fetchium it" a Verb

In 1997, nobody said "Google it." By 2002, it was ubiquitous. The path from brand to verb is: do one thing so reliably and so well that the action becomes synonymous with the product name.

The action we want to own: **retrieving information from the internet and getting a verified, synthesized answer**.

Current language: "Let me search for that and then figure out what's actually true."
Target language: "Let me Fetchium that."

This goal is not just marketing aspiration. It is a product constraint. The product has to be good enough that people instinctively reach for it as the answer to any information need. That means:
- Fast enough that it beats manual searching before the reflex kicks in
- Accurate enough that users trust the output without verification
- Comprehensive enough that users never need to go elsewhere for the same query

---

### Brand Voice: Confident, Technical, Accessible

**What we are:**
- Confident but not arrogant ("here is the answer" vs. "we think maybe possibly")
- Technical but not alienating (correct terminology, explained clearly)
- Accessible but not dumbed down (complexity acknowledged, not hidden)
- Direct but not cold (we care about the outcome, not just the mechanism)

**What we are not:**
- Corporate (no "leverage synergies to unlock value" language)
- Cutesy (no unnecessary emoji, no quirky mascots, no "Hey there!" energy)
- Academic (no jargon without explanation)
- Over-promising (we do not claim to know things we do not know)

**Voice examples:**

*Wrong:* "Fetchium leverages cutting-edge AI technologies to revolutionize the information retrieval paradigm, enabling stakeholders to extract actionable insights from diverse data sources."

*Right:* "Fetchium searches 20+ sources, pulls the full content, and gives you a verified answer with every claim linked to its source."

*Wrong:* "Oops! We couldn't find that! 🙈 Try searching again later!"

*Right:* "No results matched. The query may be too narrow or the sources may be temporarily unavailable. Try broadening the query or checking `fetchium doctor`."

*Wrong:* "Our proprietary algorithms ensure maximum retrieval quality."

*Right:* "We use BM25 + semantic + temporal + authority scoring. You can read the algorithm source in `crates/fetchium-core/src/rank/fusion.rs`."

---

### Visual Identity Principles

*(For design system implementation — detailed specifications in separate brand guide)*

**Color:** Deep space black as primary. Electric blue-white as accent (Fetchium blue: `#4A9EFF`). Trust signal: clean white backgrounds with maximum contrast.

**Typography:** Monospace for technical content (code, data, metrics). Clean sans-serif for prose. No decorative fonts.

**Iconography:** Minimal, technical. The Fetchium mark should suggest retrieval, not search — a pointer going outward and returning with something, not a magnifying glass.

**Motion:** Fast, precise, functional animations. Loading states that show real progress, not spinning indicators. No animations that obscure information.

**Density:** Information-dense UIs are appropriate for power users. The dashboard and TUI should pack information efficiently. Consumer-facing pages should be clean and spacious.

---

## 12. Path to $100M ARR

### The Cursor Playbook

Cursor achieved $100M ARR in 12 months with zero marketing budget. The mechanism:
1. Build a tool so good that developers tell other developers
2. Start with the individual developer workflow (not enterprise)
3. 36% conversion rate from free to paid (industry average: 2–5%)
4. Word of mouth compounds — every satisfied user generates 3–5 new users

Fetchium follows this playbook precisely. The product is the marketing. The developer experience is the sales team.

---

### Phase 1: Developer Adoption (Months 1–18)
**Goal: 100,000 developers using the free tier**

**How:**
- Launch CLI on Homebrew, `cargo install fetchium`, npm install fetchium
- Submit to Hacker News Show HN, Reddit r/rust, r/MachineLearning, r/LocalLLaMA
- Publish technical deep-dives: "How we built a 5-layer content extraction pipeline in Rust"
- LangChain and LlamaIndex integrations in the standard library
- Open-source the project fully — GitHub stars are the growth mechanism
- Developer documentation that is exceptional, not adequate

**Metrics:**
- 50K GitHub stars (target: 18 months)
- 100K CLI downloads
- 10K active API users
- 500 developers with > 100 API calls/month

**Revenue at this stage:** $300K–$500K ARR (from Pro conversions and API usage of active developers)

---

### Phase 2: Teams (Months 12–30)
**Goal: 1,000 paying teams**

**The expansion motion:** A developer on a team uses Fetchium personally. They demo it to the team. The team wants shared workspaces, shared knowledge bases, usage analytics, and centralized billing.

**Features required:**
- Shared persistent intelligence (team-level PIE)
- Usage dashboard and per-user analytics
- Team API key management
- Shared query history and saved searches
- Collaborative research workspaces (multiple people contributing to the same research session)

**Pricing:** $49/month/seat (minimum 3 seats = $147/month/team)

**Metrics:**
- 1,000 teams on paid plans
- Average team size: 4 seats
- = $2.4M ARR from Teams tier alone

---

### Phase 3: Enterprise (Months 24–42)
**Goal: 100 enterprise customers at $50K–$500K ACV**

**The enterprise motion:** A team using Fetchium proves internal ROI. IT/Legal/Compliance gets involved. Requirements emerge for SSO, data residency, audit logging, SLA guarantees, and on-premise deployment.

**Features required:**
- SAML/SSO integration (Okta, Azure AD, Google Workspace)
- On-premise deployment (Docker Compose → Kubernetes Helm chart)
- Data residency guarantees (query data never leaves the customer's VPC)
- Audit logging (every query, every API call, every user action)
- SLA guarantees (99.9% uptime, < 200ms search response p95)
- Enterprise support (dedicated Slack channel, SLA response times)
- Custom model integration (bring your own LLM endpoint)
- Volume pricing for API usage

**Target customers:**
- Law firms and legal research teams (citation-critical)
- Financial analysis teams (market data, corporate research)
- Healthcare organizations (clinical research, drug information)
- Government and defense (OSINT, policy research)
- Consulting firms (competitive intelligence, client research)
- Academic institutions (research assistance at scale)

**Pricing:** $50K–$500K ACV depending on scale and configuration

**Metrics:**
- 100 enterprise customers
- Average ACV: $120K
- = $12M ARR from Enterprise alone

---

### Phase 4: Platform (Months 36–60)
**Goal: $100M ARR**

**The platform motion:** Fetchium becomes the infrastructure layer. Third-party plugins extend it. Marketplace economics emerge. Specialized AI agent builders use Fetchium as the retrieval primitive for their products.

**Platform components:**
- **Plugin marketplace**: Custom backends, custom extractors, custom ranking algorithms — sold by third-party developers, with Fetchium taking 20% revenue share
- **Fetch-as-a-Service**: Metered API for third-party AI applications. Volume pricing makes Fetchium economically viable as infrastructure for SaaS products.
- **Embed Fetchium**: White-label the extraction pipeline for enterprise customers who want to power their own products with Fetchium infrastructure
- **Fetchium Cloud**: Managed cloud version with global edge deployment — for teams that want Fetchium's capabilities without managing infrastructure

**ARR at $100M:**
- Free-to-Pro conversions: $15M (100K Pro users at $144/year)
- Teams: $25M (5K teams, avg $5K/year)
- API usage: $30M (volume-scaled, high-usage customers)
- Enterprise: $25M (100 customers, avg $250K ACV)
- Platform/marketplace: $5M

**Total: $100M ARR**

---

### Key Growth Assumptions

| Metric | Conservative | Target | Stretch |
|--------|-------------|--------|---------|
| Free-to-Pro conversion | 3% | 10% | 25% |
| Monthly active CLI users at 18mo | 50K | 100K | 250K |
| API calls per paying customer/month | 10K | 50K | 200K |
| Enterprise customers at 36mo | 30 | 100 | 300 |
| Average Enterprise ACV | $50K | $120K | $300K |

The target column is achievable with a product that matches Cursor's level of developer satisfaction. The key variable is conversion rate. Cursor converts 36% of free users to paid. That is possible only if the product is genuinely excellent at the free tier — good enough that paying feels obvious, not coerced.

---

## 13. Go-To-Market Wedges

### Wedge 1: Students and Academic Researchers

**The audience:** 250M+ university students globally. 8M+ academic researchers. All of them spend significant time on literature review, source verification, and research synthesis.

**The pain:** Academic research is slow, fragmented, and citation-intensive. The average PhD student spends 30–40% of their research time on retrieval and citation management. The tools are terrible: Google Scholar has no synthesis, Zotero is a citation manager not a retrieval tool, traditional databases are siloed.

**The Fetchium offer:**
- Free tier: 50 research queries/day (generous for student use)
- Academic mode with arXiv, PubMed, Semantic Scholar, CrossRef integration
- BibTeX and APA/MLA export
- Citation graph exploration
- "Prove this claim" mode — takes an assertion, finds supporting or contradicting academic literature

**GTM tactics:**
- University partnerships: offer institutional licenses to university libraries
- ResearchGate, Academia.edu, and r/PhD community presence
- Tutorial content: "How I cut my literature review time by 80% with Fetchium"
- The citation features are genuinely innovative — researchers will share them

**Value of this wedge:** Students become researchers. Researchers become professors. Professors become institutional buyers. The habit formed in grad school follows the researcher through their career. This is the same playbook that made Overleaf (LaTeX editor) a $50M/year business.

---

### Wedge 2: Content Creators and Journalists

**The audience:** 50M+ professional content creators globally. 200K+ journalists and editorial staff. All producing content that requires research, verification, and source attribution.

**The pain:** Research is the bottleneck in content production. A skilled journalist spends 60–70% of their time on research and verification. Misinformation risk is high. Citation management is manual. Finding the right expert quote or statistic requires hours of searching.

**The Fetchium offer:**
- Social intelligence (what is the community saying about X?)
- Citation verification (is this statistic actually from the claimed source?)
- Trend monitoring (what is emerging that my audience will care about?)
- "Find the expert": academic + social search to identify credible voices on a topic
- Draft research brief: 5-source synthesis with all claims attributed

**GTM tactics:**
- Journalism school partnerships
- Partnership with tools journalists already use: Notion, Roam, Obsidian
- "Fact-check this article" mode demo — instantly viral for the journalism community
- Case studies with individual journalists who can share results publicly

---

### Wedge 3: Small Business and SMB Competitive Intelligence

**The audience:** 50M+ SMBs globally. Every SMB owner needs competitive intelligence — what are competitors charging? What do their customers say? What is the regulatory environment? What are the trends in their market?

**The pain:** Competitive intelligence is expensive at the enterprise level (Nielsen, Gartner, IDC cost tens of thousands of dollars per report). SMBs cannot access this level of research and do not have staff to replicate it manually.

**The Fetchium offer:**
- "Research my competitor" mode: web + social + data analysis of a named company
- Pricing intelligence: monitor competitor pricing pages
- Market trends: what are customers in my industry saying about their problems?
- Local market research: what is the regulatory environment in my target city?

**GTM tactics:**
- Shopify App Store integration (for e-commerce competitive intelligence)
- Small business communities: Indie Hackers, r/entrepreneur, Product Hunt
- Content: "How I researched my competitor's pricing strategy with a free tool"
- The free tier provides real value — conversion to Pro comes when usage limits are hit

---

### Wedge 4: Enterprise — Legal and Compliance

**The audience:** 150K+ law firms globally. 500K+ compliance officers. All in an industry where incorrect information has severe consequences and citation accuracy is not optional.

**The pain:** Legal research is expensive (Westlaw costs thousands per month), slow, and specialized. But not all legal research requires Westlaw — general legal information, regulatory research, and case preparation research often happens across general web sources, court documents, and legal databases that are publicly accessible.

**The Fetchium offer:**
- Citation-grade source attribution (every claim linked to specific passage, not just URL)
- Regulatory monitoring (watch agency pages for rule changes)
- Case law discovery (court document extraction and citation)
- Contradiction flagging (surface conflicting interpretations or rulings)
- On-premise deployment for client data confidentiality

**GTM tactics:**
- Legal tech conference presence (ILTA, LegalWeek)
- Partnership with legal research tools (Clio, MyCase, PracticePanther)
- Case study: "How a boutique law firm replaced $50K/year in research tools with Fetchium Enterprise"
- The compliance and evidence features are designed for this exact market

---

## 14. Revenue Architecture

### Design Philosophy

The revenue model follows three principles:

1. **Value-aligned**: The more value you get from Fetchium, the more you pay. But the value-per-dollar stays consistent across tiers — we never create artificial limitations to force upgrades.
2. **Developer-first**: The free tier must be genuinely useful for building and testing. A developer who cannot evaluate the product without paying will not become a customer.
3. **Infrastructure pricing**: API usage follows the model of cloud infrastructure — low per-unit cost, volume discounts, predictable pricing. This enables Fetchium to be embedded in commercial AI products without destroying their unit economics.

---

### Tier 1: Free — "Try It, Love It"

**Who it is for:** Individual developers evaluating the product. Students and researchers. Open-source maintainers. Journalists on a budget.

**What it includes:**
- 10 fetches per day (resets at midnight UTC)
- All 7 Fetch Modes (Web, Video, Research, Social, Data, Deep, Monitor)
- CLI, REST API, MCP server
- All output formats (JSON, Markdown, plain text)
- Community support (GitHub Discussions, Discord)
- Self-hosting with no limitations (self-hosted is always unlimited)

**What it excludes:**
- High-volume API usage
- Priority queue (queued behind Pro/Teams when under load)
- Advanced monitoring (max 1 active monitor, 24h minimum interval)
- Team features (shared workspaces, shared PIE)

**Price:** $0 forever. No credit card. No trial period.

**Why 10 fetches:** Generous enough to evaluate all features and build a meaningful integration. Restrictive enough that production workloads require a paid tier. The number may be adjusted based on conversion data.

---

### Tier 2: Pro — "Power User"

**Who it is for:** Individual developers and power users in production. Freelance researchers. Solo content creators. Independent consultants.

**What it includes:**
- Unlimited fetches
- All 7 Fetch Modes with maximum depth
- Priority queue (always ahead of free tier)
- Advanced monitoring (unlimited monitors, down to 1-hour intervals)
- PIE persistence (unlimited cross-session memory)
- Export formats (PDF, Word, BibTeX, structured JSON-LD)
- Email support with 48-hour response guarantee
- API key management (multiple keys, usage analytics per key)

**Price:** **$12/month** (annual: $10/month, billed $120/year)

**Why $12:** Below the psychological "is this worth a subscription" threshold ($15/month). Comparable to a single ChatGPT Plus subscription ($20/month) but significantly less than the value provided. The goal at Pro tier is revenue per user, not maximum extraction — users who feel well-priced tell others.

---

### Tier 3: Teams — "Collaborative Intelligence"

**Who it is for:** Engineering teams, research teams, content teams, small agencies.

**What it includes:**
- Everything in Pro, for every seat
- Shared persistent intelligence (team-level PIE that learns from all team members' queries)
- Shared workspaces (collaborative research sessions, shared saved searches)
- Centralized billing with per-seat usage breakdown
- Admin dashboard (user management, usage analytics, spend controls)
- Custom API key policies (rate limits per team member, project-level keys)
- Priority support with 24-hour response guarantee

**Price:** **$49/seat/month** (annual: $40/seat/month, billed annually)

**Minimum:** 3 seats = $147/month minimum

**Why $49/seat:** In the range of developer tools like Linear ($8/seat), Notion ($8/seat), Figma ($12/seat), GitHub ($4/seat) — but justified by the productivity leverage. If Fetchium saves a developer 1 hour/week, at a loaded cost of $100/hour, the breakeven is less than 30 minutes of use per month. At $49/seat, this is an easy ROI conversation.

---

### Tier 4: API — "Build With Fetchium"

**Who it is for:** Developers embedding Fetchium in their products. AI application builders. Automation and pipeline builders.

**Pricing:**

| Volume | Price per Fetch |
|--------|----------------|
| 0–10,000/month | $0.01 |
| 10,000–100,000/month | $0.007 |
| 100,000–1M/month | $0.005 |
| 1M+/month | Custom |

**"Fetch" is defined as:** One complete pipeline execution — search + extraction + synthesis. Sub-operations (search-only, extract-only) are priced at 40% of a full fetch.

**Included in API tier:**
- Full API access with no mode restrictions
- Webhook delivery for async fetches
- Response caching (configurable TTL — pay once, receive multiple times)
- Priority infrastructure (dedicated compute pool for > 100K fetches/month)
- SLA: 99.9% uptime, < 200ms p95 latency for search

**Volume discounts explained:** At $0.005/fetch with 500K fetches/month, monthly API cost is $2,500. At an AI application serving 10,000 active users, this is $0.25/user/month — economically viable for embedding in any SaaS product.

---

### Tier 5: Enterprise — "Infrastructure-Grade"

**Who it is for:** Organizations with compliance requirements, data residency needs, high volume, or requirements for on-premise deployment.

**What it includes:**
- Everything in Teams, for the entire organization
- On-premise deployment (Kubernetes Helm chart, Docker Compose)
- Data residency guarantee (no query data leaves designated geography)
- Custom model integration (bring your own LLM endpoint, including locally-deployed models)
- SSO integration (SAML 2.0 — Okta, Azure AD, Google Workspace, LDAP)
- Audit logging (every API call, every query, every user action, tamper-evident)
- SLA: 99.9% uptime with SLA credits
- Dedicated support channel (Slack Connect or Teams channel)
- Custom rate limits and quotas
- Volume pricing for API usage (negotiated)
- Professional services: deployment assistance, custom integration, training

**Price:** **Custom, starting at $50K/year**

**Typical deal size:** $50K–$500K/year depending on organization size, volume, and support requirements.

**Sales motion:** Enterprise deals do not close through self-serve. The path: free/Pro user → Team admin → IT/legal contact → vendor review → contract negotiation. This cycle takes 3–9 months for large enterprises. The product has to be in use before the deal starts.

---

### Revenue Model Summary

| Tier | Price | Target Customers | ARR per Customer |
|------|-------|-----------------|-----------------|
| Free | $0 | 500K users | $0 |
| Pro | $144/year | 100K users | $144 |
| Teams | $588–$2,400/year | 5K teams | $1,200 avg |
| API | Variable | 1K customers | $2,500 avg/month |
| Enterprise | Custom | 100 orgs | $120K avg |

**Path to $100M ARR:**
- Pro: 100K × $144 = $14.4M
- Teams: 5K × $1,200 = $6M
- API: 1K × $30K/year = $30M
- Enterprise: 100 × $120K = $12M
- Platform/ecosystem: $5M
- **Total: $67.4M** (conservative) → stretch to $100M with platform revenue and higher API volume

---

## 15. World Impact Thesis

### Democratize Research

The most important research question you have ever had — the one that changed your career, your health, your relationships, your business — was answered by your ability to find and evaluate information. The quality of your information access is a massive determinant of the quality of your life outcomes.

Right now, information access is unequal in ways that compound inequality:

- A student at Harvard has access to institutional database subscriptions (Scopus, Web of Science, JSTOR, LexisNexis) that cost tens of thousands of dollars per year. A student at a university in Bangladesh has access to Google.
- A senior analyst at McKinsey has Bloomberg Terminal, Gartner research, and proprietary data tools. A small business owner in Lagos has Google and gut instinct.
- A pharmaceutical company has teams of researchers and access to every clinical database. A patient trying to understand their diagnosis has WebMD.

These gaps are not natural. They are the result of information being treated as a product rather than a commons.

**Fetchium's impact:** An open-source, self-hostable intelligence engine changes this equation. A university in Dhaka with a Linux server and an internet connection can deploy Fetchium and give every student access to a research capability that matches what a Harvard student gets from their institutional subscriptions. Free. Unlimited. Maintained by the community.

A small business owner in Lagos can ask Fetchium the same market research question that the McKinsey analyst can — and get a similar quality of synthesized, verified answer.

This is not charity. This is infrastructure. Open infrastructure is how the internet became universal. Open infrastructure is how Linux runs the world. Fetchium follows the same playbook.

---

### Anti-Misinformation Infrastructure

The misinformation crisis is not primarily a content moderation problem. It is an epistemological infrastructure problem. The tools that billions of people use to find information are not designed to surface truth — they are designed to maximize engagement. Engagement correlates with outrage and novelty, not accuracy.

Fetchium's mandatory citation architecture is a structural intervention:

- Every claim is attributed to a specific source, with the option to view the exact passage
- Conflicting claims are surfaced as conflicts, not resolved silently in favor of the more engaging version
- Source authority is scored and visible — a claim from a CDC publication and a claim from an anonymous blog are both accessible, but their authority is not presented as equivalent
- The evidence graph makes the reasoning process transparent — users can see not just what was concluded, but how the conclusion was reached and what evidence supports it

At scale, this changes the epistemological infrastructure of information retrieval. When "Fetchium it" becomes the instinct for information needs, the default behavior for information retrieval shifts from "find the most engaging result" to "find the best-evidenced result."

This is not a claim about changing human psychology. It is a claim about changing default behavior through better tooling. The tool that is easiest to use is the one that gets used. If the easiest tool is also the most epistemically honest one, the behavior of easy information retrieval changes.

---

### Break Information Asymmetry

Information asymmetry is one of the most powerful forces in economic and political life. The party with better information almost always wins: better pricing intelligence, better competitive awareness, better regulatory understanding, better understanding of customer needs.

Currently, information asymmetry favors incumbents:
- Large companies can afford research teams
- Rich individuals can afford researchers, advisors, and data subscriptions
- Government agencies have internal research infrastructure
- Academic institutions have database access

Fetchium breaks this asymmetry for knowledge retrieval. The entrepreneur competing against a Fortune 500 company can now access the same quality of market research. The NGO competing against a government agency can now access the same quality of policy research. The individual patient can now access the same quality of medical information as the hospital administrator.

This does not equalize all competitive dynamics — funding, network effects, and scale still matter enormously. But information access is one dimension of competitive advantage that can be equalized through open infrastructure, and Fetchium equalizes it.

---

### Every Claim Traceable to Source

We are building toward a world where "I read that somewhere" is not an acceptable epistemological standard. Where "the data says" requires pointing to the data. Where "experts agree" requires naming the experts and their credentials.

This world is achievable through tooling, not through cultural change alone. When the default retrieval tool links every claim to its source, the habit of citing becomes easier than not citing. When the evidence graph makes provenance visible, trust in information becomes earned rather than assumed.

Fetchium is infrastructure for a more epistemically honest information environment. Not because we mandate honesty — we cannot. But because we make honest, cited, verifiable communication the path of least resistance for information retrieval.

---

## 16. Brand Evolution Arc

The brand evolves in four stages, each reflecting where Fetchium is in its journey from tool to infrastructure to platform to institution.

---

### Version 1: "Fetch anything. Verified. Fast."

**Stage:** Launch. First 12 months.

**Audience:** Early adopters, developers, Hacker News readers.

**Message:** This exists. It works. It is fast. It gives you verified answers, not just links.

**Tone:** Technically confident. No-nonsense. Here is what it does, here is how fast it does it, here is the output.

**Marketing channels:** GitHub README, Show HN, r/rust, r/MachineLearning, CLI distribution.

**What success looks like:** 10K GitHub stars. 50K CLI downloads. The developer community knows Fetchium exists and what it does.

**Key assets:**
- An exceptional GitHub README that demonstrates the product in the first scroll
- A compelling `fetchium` CLI that impresses on first use
- Technical blog posts that show the algorithms
- A Show HN post that generates top-100 comments

**Why this message:** At launch, trust is zero. The product has to prove itself. "Fetch anything. Verified. Fast." makes three concrete claims that the product can immediately demonstrate. It does not ask for trust — it invites skepticism and then delivers.

---

### Version 2: "Find. Fetch. Learn."

**Stage:** Product-market fit confirmation. Months 12–24.

**Audience:** Developers, teams, early enterprise explorers.

**Message:** This is a complete intelligence pipeline, not just a search tool. It finds (multi-source discovery), fetches (full content extraction), and learns (persistent intelligence, AI synthesis).

**Tone:** Confident, expanding. The tool works. Now we are explaining how deep it goes.

**Marketing channels:** Developer communities, technical conferences (RustConf, AI Engineer Summit), integration partner announcements (LangChain, LlamaIndex).

**What success looks like:** 50K GitHub stars. 100K active users. 1K paying customers. Team tier launched.

**Key assets:**
- Expanded documentation that walks through each pipeline layer
- Integration guides for major AI frameworks
- Case studies from early adopters
- Conference talks on the algorithm designs

**Why this message:** "Find. Fetch. Learn." mirrors the 5-layer information stack (FIND, FETCH, LEARN, KNOW, ACT) but in accessible language. It signals that Fetchium is not just a point tool — it is a pipeline. It also introduces the learning aspect (PIE), which is the key differentiator from all competitors.

---

### Version 3: "Your internet. Learned."

**Stage:** Mainstream developer adoption. Months 24–42.

**Audience:** Teams, SMBs, enterprise evaluators.

**Message:** Fetchium is not just a retrieval tool. It is your personalized intelligence layer. It learns your research patterns, your trusted sources, your domain. It is not "the internet" — it is *your* internet.

**Tone:** Personal, powerful. The tool has been used long enough that its learning capabilities are a real differentiator. Users who have been using it for 12+ months have a noticeably better experience than new users.

**Marketing channels:** Word of mouth from developer community, LinkedIn for B2B, targeted content for specific verticals (legal, academic, finance).

**What success looks like:** 100K+ GitHub stars. 500K active users. 10K paying customers. Enterprise tier generating meaningful revenue.

**Key assets:**
- Testimonials from power users describing how their Fetchium has gotten smarter over time
- The PIE visual story — showing how source trust and query learning improve results
- Enterprise case studies with ROI data
- Team features prominently featured (shared PIE, collaborative workspaces)

**Why this message:** At this stage, the persistent intelligence features are the primary differentiator. Every competitor can do basic retrieval. No competitor has a retrieval layer that improves over time with use. "Your internet. Learned." claims this territory.

---

### Version 4: "Know more than anyone."

**Stage:** Platform. Months 42–60+.

**Audience:** The broader market — not just developers. Knowledge workers, researchers, professionals, and eventually, anyone who needs information.

**Message:** Fetchium gives you a research and intelligence capability that previously required a team of analysts. You, individually, with Fetchium, can know more about any topic than organizations with 100x your resources used to be able to know.

**Tone:** Ambitious, empowering. The product has matured from "fast, verified retrieval" to "personal intelligence advantage." The brand can now make claims about transformation, not just capability.

**Marketing channels:** Broader distribution — Product Hunt, mainstream tech press, vertical media (legal tech, healthcare IT, financial tech), podcast sponsorships, community programs.

**What success looks like:** The brand is recognized. "Fetchium" is becoming a verb in relevant communities. $100M ARR trajectory is clear. Platform ecosystem is alive.

**Key assets:**
- The narrative of democratization is leading content ("The tool that gives a solo researcher the power of a Wall Street team")
- Community stories of impact — the researcher in Dhaka, the journalist who uncovered the story, the founder who outmaneuvered the incumbent
- Platform ecosystem: partners, plugins, integrations generating third-party content
- The compound interest of open source: 100K+ stars means every AI tutorial references Fetchium

**Why this message:** "Know more than anyone" is the ultimate expression of the product's value proposition. It is not about the features — it is about the outcome. The outcome is information advantage. Applied to individuals, teams, organizations, and communities that currently lack it. This is the message that bridges from developer tool to information infrastructure.

---

### The Through-Line

Each version of the brand message is a layer on top of the previous:

```
v1: "Fetch anything. Verified. Fast."
    → The capability exists and works
        ↓
v2: "Find. Fetch. Learn."
    → It is a complete pipeline, not a point tool
        ↓
v3: "Your internet. Learned."
    → It personalizes and improves over time
        ↓
v4: "Know more than anyone."
    → The outcome is information advantage at any scale
```

The product has to earn each message. The tagline does not evolve because of a marketing decision — it evolves because the product has matured to the point where the new claim is credible.

---

## Appendix: Key Metrics and Milestones

### Technical Milestones

| Milestone | Target Date | Success Metric |
|-----------|------------|---------------|
| 20 algorithms implemented, 941 tests passing | Complete | Cargo test: 0 failures |
| Production deployment (API + web) | Complete | https://api.fetchium.zuhabul.com live |
| Rename Fetchium → Fetchium | Q1 2026 | Brand migration complete |
| LangChain integration | Q1 2026 | PR merged to langchain repo |
| LlamaIndex integration | Q1 2026 | PR merged to llamaindex repo |
| MCP server (Phase 4) | Q2 2026 | `fetchium serve --mode mcp` works |
| Embeddings support (ONNX) | Q2 2026 | `cargo build --features embeddings` |
| Python bindings | Q3 2026 | `pip install fetchium` works |
| Kubernetes Helm chart | Q3 2026 | `helm install fetchium fetchium/fetchium` |
| 1,000 GitHub stars | Q2 2026 | GitHub star count |
| 10,000 GitHub stars | Q3 2026 | GitHub star count |

### Business Milestones

| Milestone | Target Date | Success Metric |
|-----------|------------|---------------|
| First 100 API customers | Q2 2026 | 100 unique API keys with > 10 calls |
| First $10K MRR | Q3 2026 | Stripe dashboard |
| First enterprise customer | Q3 2026 | $50K+ contract signed |
| First $100K MRR | Q4 2026 | Stripe dashboard |
| Seed round close | Q4 2026 | $3–5M at $20–30M valuation |
| 100K active users | Q1 2027 | Monthly active API/CLI users |
| $1M ARR | Q2 2027 | Annualized MRR |
| Series A | Q4 2027 | $15–25M at $100M+ valuation |
| $10M ARR | Q2 2028 | Annualized MRR |
| $100M ARR | Q4 2030 | Annualized MRR |

---

## Final Note

Fetchium is not a feature. It is not a product. It is infrastructure.

The internet's information is not organized. It is not verified. It is not equitably accessible. Fetchium is the engineering response to that fact.

We are building in the open, in Rust, with 20+ algorithms that nobody else has built. We are building for the developer who will embed us in their agent pipeline, the student who will use us to write a better thesis, the journalist who will use us to find the truth faster, and the small business owner who will use us to compete against players ten times their size.

"You ask. Fetchium fetches."

That is the promise. Everything else — the algorithms, the architecture, the business model, the brand — is in service of keeping it.

---

*Fetchium — The Universal Retrieval Layer*
*Document version: 1.0 | February 2026*
*Contact: zuhabul@fetchium.com | https://fetchium.zuhabul.com*
