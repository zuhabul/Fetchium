<div align="center">

<br/>

```
███████╗███████╗████████╗ ██████╗██╗  ██╗██╗██╗   ██╗███╗   ███╗
██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║██║██║   ██║████╗ ████║
█████╗  █████╗     ██║   ██║     ███████║██║██║   ██║██╔████╔██║
██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║██║██║   ██║██║╚██╔╝██║
██║     ███████╗   ██║   ╚██████╗██║  ██║██║╚██████╔╝██║ ╚═╝ ██║
╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝     ╚═╝
```

### **The Universal Retrieval Layer for the Internet**
*Open-source · Rust-native · Built for Humans and AI Agents*

<br/>

[![Tests](https://img.shields.io/badge/tests-974%20passing-brightgreen?style=flat-square&logo=rust)](https://github.com/zuhabul/Fetchium)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust)](https://rustup.rs)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=flat-square)](LICENSE)
[![Binary](https://img.shields.io/badge/binary-single%20static-purple?style=flat-square)](https://github.com/zuhabul/Fetchium/releases)
[![Startup](https://img.shields.io/badge/startup-~5ms-brightgreen?style=flat-square)]()
[![Algorithms](https://img.shields.io/badge/novel%20algorithms-20%2B-ff6b6b?style=flat-square)]()

<br/>

[**Install**](#installation) · [**Quick Start**](#quick-start) · [**Fetch Modes**](#7-fetch-modes) · [**AI Setup**](#ai-provider-setup) · [**API**](#api) · [**Config**](#configuration) · [**Docs**](https://docs.fetchium.dev) · [**Discord**](https://discord.gg/fetchium)

<br/>

</div>

---

Fetchium is an open-source, Rust-native AI search engine that **finds, fetches, understands, and verifies** information from any source on the internet. It ships as a single static binary with 20+ novel retrieval algorithms, multi-backend search federation, evidence-based citations, and a built-in REST + MCP API — ready for both humans at the terminal and AI agents over the network.

---

## Installation

### npm (recommended — works everywhere Node is installed)

```bash
npm install -g fetchium
fetchium --version
```

### npx (no install required)

```bash
npx fetchium search "quantum computing breakthroughs 2025"
```

### Shell installer (Linux / macOS)

```bash
curl -sSf https://install.fetchium.dev | sh
```

### cargo-binstall

```bash
cargo binstall fetchium
```

### Homebrew

```bash
brew install zuhabul/tap/fetchium
```

### Build from source

```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build -p fetchium-cli --release
./target/release/fetchium --version
```

---

## Quick Start

```bash
# Web search — federated across multiple backends
fetchium search "best Rust async runtimes 2025"

# Fetch and extract a URL (8000 tokens, detailed tier)
fetchium fetch https://example.com

# AI-powered answer with citations
fetchium ai "What causes northern lights?"

# Deep multi-step research report
fetchium research "Impact of LLMs on software engineering jobs"

# Summarize any URL or text
fetchium summarize https://docs.rs/tokio/latest/tokio/
fetchium summarize "long article text here..."

# Compare anything — auto-generates AI-powered comparison tables
fetchium compare "Rust vs Go vs Python"

# Transcribe YouTube / any audio URL
fetchium transcribe https://www.youtube.com/watch?v=dQw4w9WgXcQ

# Agentic multi-step search with reasoning
fetchium agent-search "latest developments in quantum computing"

# Social media intelligence — platform commands
fetchium x search "GPT-5 release"                # X (Twitter)
fetchium reddit search "rust programming tips"    # Reddit
fetchium hn top                                   # Hacker News top stories
fetchium youtube transcript https://youtube.com/watch?v=...

# Social shorthand (platform as first arg)
fetchium social twitter "AI news today"
fetchium social reddit "mechanical keyboards"

# Check environment health
fetchium doctor

# Run the REST API server
fetchium serve --mode rest --port 3050

# Run the MCP server (for AI agents)
fetchium serve --mode mcp
```

---

## Why Fetchium?

| Capability | Fetchium | Perplexity | Tavily | Exa | Firecrawl |
|------------|----------|------------|--------|-----|-----------|
| Open source | Yes | No | No | No | Partial |
| Self-hostable | Yes | No | No | No | Yes |
| Rust-native binary | Yes | No | No | No | No |
| Multi-backend federation | Yes (15+) | No | No | No | No |
| Social media search | Yes | Partial | No | No | No |
| AI answer with citations | Yes | Yes | Yes | Partial | No |
| Deep research mode | Yes | Yes | Partial | No | No |
| MCP server built-in | Yes | No | No | No | No |
| REST API built-in | Yes | Yes (SaaS) | Yes (SaaS) | Yes (SaaS) | Yes |
| Evidence graph | Yes | No | No | No | No |
| Offline / local LLM | Yes | No | No | No | No |
| 20+ novel algorithms | Yes | No | No | No | No |
| Free tier / no API key needed | Yes | No | No | No | No |

---

## Feature Highlights

| Feature | Description |
|---------|-------------|
| **Multi-backend federation** | SearXNG + Brave + DuckDuckGo + Google + Bing + Reddit + HN + GitHub + Tavily + Serper + Exa + Firecrawl |
| **CEP — 5-layer extraction** | CSS selectors → Readability → Headless JS → PDF → Screenshot OCR |
| **QATBE — token budgeting** | BM25-scored segment ranking + greedy knapsack within token limits |
| **HyperFusion ranking** | 8-signal score: BM25 + semantic + temporal + authority + evidence + diversity + depth + consensus |
| **Evidence graph** | Claim-level source attribution and cross-reference verification |
| **AMRS — agent swarm** | 4 parallel agent types via tokio channels for deep research |
| **PIE — intelligence engine** | Cross-session learning: source trust, failure patterns, query prediction |
| **Social media intelligence** | Native Reddit, Twitter/X, HN, YouTube, Facebook, TikTok with sentiment analysis |
| **QADD — DOM distillation** | 10-20x token reduction via 5-step DOM pruning |
| **RAR — self-correction** | 5-checkpoint retry-and-refine loop for AI answers |
| **PDS — progressive streaming** | 4 detail tiers: key_facts → summary → detailed → complete |
| **Key pool rotation** | Round-robin + 429-cooldown across multiple Gemini / OpenAI / premium API keys |
| **Premium backend integration** | Tavily, Serper, Exa, Firecrawl — optional, auto-detected from env vars |
| **YouTube-aware summarization** | Extracts transcript instead of HTML for YouTube URLs — far better summaries |
| **Perspective-aware decomposition** | Generates parallel sub-queries for broader coverage (freshness + depth) |
| **Unified AI comparison** | Single AI call fills entire comparison table with specific, sourced data |
| **MCP protocol** | Native Model Context Protocol server for Claude, GPT, and any MCP client |
| **Single static binary** | Zero runtime deps, ~5ms startup, cross-platform |

---

## 7 Fetch Modes

Fetchium adapts its retrieval strategy based on the content type and your needs:

| Mode | Command flag | When to use |
|------|-------------|-------------|
| **Fast** | `--fast` | Snippets only — no full-page fetch. ~6s, ideal for quick AI answers |
| **Standard** | *(default)* | Full-page fetch + extraction. ~15s, best general-purpose quality |
| **Headless** | `--headless` | JavaScript-rendered pages, SPAs, paywalls. Uses Chrome for Testing |
| **PDF** | `--pdf` | Native PDF text extraction via pdfium or pdftotext |
| **OCR** | `--ocr` | Screenshot + Tesseract OCR for image-heavy or locked content |
| **Research** | `--research` | Multi-step AMRS swarm. 60–120s, comprehensive report with citations |
| **Social** | `--social <platform>` | Platform-specific fetch: Reddit threads, Twitter/X, HN, TikTok |

---

## AI Provider Setup

Fetchium works with multiple AI providers. Configure once, use everywhere.

### Gemini (default — free tier available)

```bash
# Set a single key
fetchium provider set gemini --key AIza...

# Add multiple keys (auto-rotated on 429)
fetchium provider set gemini --add-key AIza...
fetchium provider set gemini --add-key AIza...

# Switch model
fetchium provider set gemini --model flash3   # gemini-3-flash
```

### OpenAI

```bash
fetchium provider set openai --key sk-...
fetchium provider set openai --model gpt-4o
```

### Anthropic

```bash
fetchium provider set anthropic --key sk-ant-...
```

### Local / Ollama

```bash
fetchium provider set ollama --url http://localhost:11434 --model llama3
```

### Premium Search Backends (optional)

```bash
# Tavily — AI-optimized search
export TAVILY_API_KEY=tvly-...

# Serper — Google SERP API
export SERPER_API_KEY=...

# Exa — Neural search
export EXA_API_KEY=...

# Firecrawl — AI-ready web scraping
export FIRECRAWL_API_KEY=...

# Or configure in ~/.fetchium/config.toml:
# [search]
# tavily_api_key = "tvly-..."
# serper_api_key = "..."
# exa_api_key = "..."
# firecrawl_api_key = "..."
```

### Check configured providers

```bash
fetchium provider list
```

---

## Search Engines

Fetchium federates across these backends and merges results with HyperFusion ranking:

| Backend | Type | API Key | Notes |
|---------|------|---------|-------|
| SearXNG (self-hosted) | Meta-search | No | Covers Google, Bing, DDG and 70+ engines. Preferred Tier 0 |
| Brave Search | Web | No | Privacy-respecting, good freshness |
| DuckDuckGo | Web | No | Always available, zero config |
| Google | Web | No | Via scraping (rate-limited) |
| Bing | Web | No | Via scraping (rate-limited) |
| Wikipedia | Knowledge | No | Factual queries, always reliable |
| Reddit | Social | No | Native API — threads, comments, scores |
| Hacker News | Social | No | Algolia search API |
| GitHub | Code | No | REST API — repos, issues, code search |
| ArXiv | Academic | No | Academic papers, scientific research |
| Google Scholar | Academic | No | Scholarly articles and citations |
| **Tavily** | Premium | Yes | AI-optimized search with pre-scored results |
| **Serper** | Premium | Yes | Google SERP API with structured data |
| **Exa** | Premium | Yes | Neural search with semantic understanding |
| **Firecrawl** | Premium | Yes | Web scraping with AI-ready extraction |

Backend selection is automatic via **ABS (Adaptive Backend Selector)** — a UCB1 multi-armed bandit that routes queries to optimal backends based on query intent, historical success rates, and backend health. Academic queries prefer ArXiv + Scholar; code queries prefer GitHub + StackOverflow; SearXNG is always included as a meta-search aggregator.

**Premium backends are optional** — Fetchium works great standalone with 11 free backends. Premium backends add extra coverage when API keys are configured.

---

## Social Media Intelligence

Each platform has both a dedicated top-level command and a shorthand via `fetchium social`:

```bash
# Per-platform commands (recommended):
fetchium x search "GPT-5 release"
fetchium reddit search "best mechanical keyboards 2025"
fetchium hn search "Show HN: my new project"
fetchium tiktok search "rust tutorial"
fetchium facebook search "local tech events"
fetchium youtube search "rust tutorial"

# Social shorthand (platform as first arg):
fetchium social twitter "GPT-5 release"
fetchium social reddit "best mechanical keyboards 2025"
fetchium social hn "Show HN: my new project"

# Social unified (all platforms at once):
fetchium social "AI tools 2025" --unified
fetchium social "AI tools 2025" --reddit --hackernews
```

Output includes: posts, scores/engagement, sentiment (positive/negative/neutral), topics, and a unified `SocialInsight` summary.

---

## Intelligence Engine (PIE)

Fetchium learns across sessions using a local SQLite database:

- **Source trust scores** — tracks which domains give accurate, high-quality answers
- **Failure pattern memory** — avoids repeating backend failures
- **Query prediction** — pre-fetches likely follow-up results
- **Cache** — deduplicates identical fetches within a session window

Data stored at `~/.fetchium/intelligence.db`. Fully local, never transmitted.

---

## CLI Reference

```
fetchium [OPTIONS] <COMMAND>

Core Commands:
  search        Federated web search across multiple backends
  fetch         Extract content from a URL (text, markdown, JSON)
  view          Alias for fetch
  ai            AI-powered answer with source citations
  research      Deep multi-step research report with citations
  deep          Deep multi-agent research swarm (AMRS)
  compare       Side-by-side comparison ("Rust vs Go") — auto-uses AI when configured
  summarize     AI-powered summarization of any URL or text (YouTube-aware)
  transcribe    Transcribe audio/video from any URL
  agent-search  Agentic multi-step search with reasoning

Platform Commands:
  x / twitter   X (Twitter) intelligence — search, trends, sentiment, monitor
  reddit        Reddit intelligence — search, hot, top, fetch
  hackernews    Hacker News intelligence — search, top, new, fetch  (alias: hn)
  facebook      Facebook intelligence — search, fetch               (alias: fb)
  tiktok        TikTok intelligence — search, trends, fetch
  youtube       YouTube intelligence — search, analyze, transcript, research
  social        Unified multi-platform social search

System:
  serve         Start REST API or MCP server
  provider      Manage AI provider credentials
  setup         Install Chrome for Testing, configure SearXNG
  doctor        Check environment health (backends, AI, Chrome)
  config        Show or edit configuration
  cache         Manage the result cache

Global Options:
  -f, --format <FMT>    Output format: markdown, json, text [default: markdown]
  -q, --quiet           Suppress progress output
  -v, --verbose         Enable debug logging
      --no-cache        Skip cache for this request
  -h, --help            Print help
  -V, --version         Print version
```

### Core Commands

#### `fetchium search`

```bash
fetchium search "best Rust async runtimes 2025"
fetchium search "query" -n 20                    # 20 results
fetchium search "query" -f json                  # JSON output
fetchium search "query" --trust-verify           # trust/adversarial scoring
```

#### `fetchium fetch` / `fetchium view`

```bash
fetchium fetch https://example.com               # extract full page (8000 tokens, detailed)
fetchium fetch https://example.com --budget 4000 # custom token budget
fetchium fetch https://example.com --query "AI"  # QATBE query-aware extraction
fetchium fetch https://example.com -f json       # JSON segments output
```

#### `fetchium ai`

```bash
fetchium ai "What causes northern lights?"
fetchium ai "query" --model flash3               # override model
fetchium ai "query" --fast                       # snippet-only context (faster)
fetchium ai "query" --no-stream                  # wait for full response
```

#### `fetchium research`

```bash
fetchium research "Impact of LLMs on software engineering"
fetchium research "query" --max-sources 15       # fetch more sources
fetchium research "query" --no-ai                # heuristic listing (no AI)
fetchium research "query" --thinking             # enable thinking/reasoning mode
fetchium research "query" -o report.md           # save to file
```

#### `fetchium deep`

```bash
fetchium deep "Rust web frameworks 2026"         # multi-agent research swarm
fetchium deep "query" --timeout 60               # 60-second timeout
```

#### `fetchium compare`

```bash
fetchium compare "Rust vs Go"                    # auto-uses AI when provider configured
fetchium compare "React vs Vue vs Svelte"        # multi-item comparison
fetchium compare "Python vs JavaScript vs Go"    # context-aware search per item
fetchium compare "query" --ai                    # explicitly request AI comparison
fetchium compare "query" -f json                 # JSON output
```

#### `fetchium agent-search`

```bash
fetchium agent-search "latest Rust async frameworks comparison"
fetchium agent-search "query" --max-steps 5      # limit reasoning steps
fetchium agent-search "query" --model flash3     # use specific model
```

#### `fetchium summarize`

```bash
fetchium summarize https://docs.rs/tokio/latest/tokio/
fetchium summarize "long text to summarize here..."
fetchium summarize <URL> --length short          # ~100 words
fetchium summarize <URL> --length long           # ~700 words
fetchium summarize https://youtube.com/watch?v=...  # YouTube-aware (uses transcript)
```

#### `fetchium transcribe`

```bash
fetchium transcribe https://www.youtube.com/watch?v=...    # YouTube transcript
fetchium transcribe https://any-url.com/podcast.mp3        # generic URL
fetchium transcribe <youtube-url> --chapters               # align to chapters
```

---

### Platform Commands

#### `fetchium x` / `fetchium twitter`

```bash
# Aliases: fetchium x, fetchium twitter, fetchium tw
fetchium x search "GPT-5 release"               # search tweets
fetchium x search "query" -n 30                 # 30 tweets
fetchium x trends                                # trending in US
fetchium x trends --country uk                  # trending in UK
fetchium x sentiment "AI tools"                  # sentiment analysis
fetchium x fetch https://x.com/user/status/...  # fetch single tweet (oEmbed)
fetchium x monitor "AI news" --interval 120     # realtime monitor (every 2 min)
fetchium x profile elonmusk                      # user profile + recent tweets
fetchium x research "query"                      # deep research via X
```

#### `fetchium reddit`

```bash
fetchium reddit search "best mechanical keyboards 2025"
fetchium reddit search "query" --subreddits r/rust,r/webdev
fetchium reddit hot rust                         # hot posts in r/rust
fetchium reddit top rust --period week           # top posts this week
fetchium reddit top rust --period month -n 50
fetchium reddit fetch https://reddit.com/r/...   # fetch post + comments
fetchium reddit research "query"                 # deep research via Reddit
```

#### `fetchium hackernews` / `fetchium hn`

```bash
fetchium hn search "llm benchmarks 2025"
fetchium hn top                                  # top stories right now
fetchium hn top -n 30                            # top 30 stories
fetchium hn new                                  # newest stories
fetchium hn fetch https://news.ycombinator.com/item?id=12345
fetchium hn research "startup funding trends"
```

#### `fetchium youtube`

```bash
fetchium youtube search "rust tutorial"
fetchium youtube search "query" -n 20
fetchium youtube transcript https://www.youtube.com/watch?v=...
fetchium youtube transcript <url> --chapters     # with chapter alignment
fetchium youtube analyze https://www.youtube.com/watch?v=...
fetchium youtube analyze <url> --transcript --comments
fetchium youtube research "machine learning 2025"
fetchium youtube research "query" --max-videos 10 --fact-check
fetchium youtube compare <url1> <url2>           # compare 2 videos
```

#### `fetchium facebook` / `fetchium fb`

```bash
fetchium facebook search "tech events Bangladesh"
fetchium facebook fetch https://facebook.com/...
```

#### `fetchium tiktok`

```bash
fetchium tiktok search "coding tips"
fetchium tiktok trends                           # trending hashtags/sounds
fetchium tiktok fetch https://tiktok.com/...
```

#### `fetchium social` (unified multi-platform)

```bash
# Platform shorthand (NEW — first arg can be platform name):
fetchium social twitter "GPT-5 release"
fetchium social reddit "best mechanical keyboards"
fetchium social hn "Show HN: my new project"
fetchium social facebook "local tech events"
fetchium social tiktok "coding tips"
fetchium social youtube "rust tutorial"

# Flag style (also works):
fetchium social "AI tools" --twitter
fetchium social "AI tools" --reddit
fetchium social "AI tools" --hackernews
fetchium social "AI tools" --reddit --tiktok        # multi-platform
fetchium social "AI tools" --unified --ideas         # all platforms + content ideas
fetchium social "query" --reddit --subreddits r/ML,r/AI

# Options:
#  -n, --max <N>     posts per platform [default: 50]
#  --trends          also fetch Twitter trending topics
#  --deep            deep analysis mode
#  --ideas           generate 20 viral content ideas
```

### System Commands

#### `fetchium serve`

```bash
fetchium serve --mode rest --port 3050           # REST API
fetchium serve --mode mcp                        # MCP server for AI agents
```

#### `fetchium provider`

```bash
fetchium provider list                           # show configured providers
fetchium provider setup                          # interactive setup wizard
fetchium provider set gemini --key AIza...
fetchium provider set gemini --model flash3      # use gemini-3-flash-preview
fetchium provider set openai --key sk-...
fetchium provider set anthropic --key sk-ant-...
fetchium provider set ollama --model qwen3:8b
```

#### `fetchium setup`

```bash
fetchium setup                                   # check + download anything missing
fetchium setup --headless                        # download Chrome for Testing (~200MB)
fetchium setup --check                           # check only, no downloads
```

---

## API

### REST API

Start the server:

```bash
fetchium serve --mode rest --port 3050
```

#### Endpoints

```
POST /search          — Federated search
POST /fetch           — URL content extraction
POST /ai              — AI answer with citations
POST /research        — Deep research report
POST /social/:platform — Social media search
GET  /health          — Health check (503 if SearXNG down)
```

#### Example — search

```bash
curl -X POST http://localhost:3050/search \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer hsx_YOUR_API_KEY" \
  -d '{"query": "rust async runtimes", "num": 5}'
```

```json
{
  "meta": { "backend": "searxng", "elapsed_ms": 312, "total": 5 },
  "results": [
    {
      "title": "Tokio — An async Rust runtime",
      "url": "https://tokio.rs",
      "snippet": "Tokio is an asynchronous runtime for the Rust programming language...",
      "score": 0.94
    }
  ]
}
```

#### Example — AI answer

```bash
curl -X POST http://localhost:3050/ai \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer hsx_YOUR_API_KEY" \
  -d '{"query": "What is quantum entanglement?"}'
```

#### Admin API

```bash
# Create an API key
curl -X POST http://localhost:3050/admin/keys \
  -H "X-Admin-Secret: YOUR_ADMIN_SECRET" \
  -d '{"label": "my-app"}'

# List keys
curl http://localhost:3050/admin/keys \
  -H "X-Admin-Secret: YOUR_ADMIN_SECRET"
```

Set `FETCHIUM_ADMIN_SECRET` env var before starting the server. The server panics on startup if unset.

### MCP Server

Fetchium implements the Model Context Protocol, making it usable as a tool server for Claude, GPT-4o, and any MCP-compatible agent.

```bash
fetchium serve --mode mcp
```

Configure in your MCP client:

```json
{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["serve", "--mode", "mcp"]
    }
  }
}
```

Available MCP tools: `search`, `fetch`, `ai_answer`, `research`, `social_search`

---

## Configuration

Config file: `~/.fetchium/config.toml`

```toml
[search]
default_backends = ["searxng", "brave"]
num_results = 10
timeout_secs = 30

[searxng]
url = "http://localhost:4040"

[ai]
default_provider = "gemini"
default_model = "gemini-3-flash-preview"

[headless]
chrome_path = ""   # leave empty to use fetchium-managed Chrome

[cache]
enabled = true
ttl_secs = 3600

[token]
default_budget = 4096
```

### Environment Variables

All config values can be overridden with environment variables. The `HSX_` prefix is supported for backward compatibility; `FETCHIUM_` is preferred.

```bash
FETCHIUM_SEARXNG_URL=http://localhost:4040
FETCHIUM_AI_PROVIDER=gemini
FETCHIUM_ADMIN_SECRET=my-secret
GEMINI_API_KEY=AIza...
GEMINI_API_KEYS=AIza...,AIza...,AIza...   # comma-separated pool (round-robin + 429 cooldown)
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...

# Premium search backends (optional — enhances results when available)
TAVILY_API_KEY=tvly-...
SERPER_API_KEY=...
EXA_API_KEY=...
FIRECRAWL_API_KEY=...
```

### First-time setup

```bash
# Auto-install Chrome for Testing + SearXNG Docker container
fetchium setup

# Verify everything is working
fetchium doctor
```

---

## Development

### Prerequisites

- Rust 1.75+ (`rustup update stable`)
- Docker (optional, for SearXNG)
- Chrome / Chromium (optional, for headless mode — or run `fetchium setup --headless`)

### Build

```bash
# Check all crates compile (fast, no linking)
cargo check

# Build the fetchium binary
cargo build -p fetchium-cli

# Optimized release build
cargo build -p fetchium-cli --release

# Run the binary
./target/debug/fetchium --help
./target/debug/fetchium doctor
```

### Test

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p fetchium-core
cargo test -p fetchium-cli

# Run a single test by name
cargo test -p fetchium-core extract::layer1::tests::extract_simple_page

# Skip slow network-dependent tests
cargo test -- --skip research::pipeline
```

### Lint & Format

```bash
# Lint (zero warnings policy)
cargo clippy -- -D warnings

# Format
cargo fmt

# Generate docs
cargo doc --open
```

### Crate Structure

```
fetchium-core    — All algorithms: search, extract, rank, validate, cache, AI, intelligence
fetchium-cli     — The fetchium binary (clap derive CLI, delegates to fetchium-core)
fetchium-mcp     — MCP server (Phase 4) — exposes fetchium-core as MCP tools
fetchium-api     — REST API server via axum (Phase 4)
```

### Adding Dependencies

All shared dependencies go in the **workspace** `Cargo.toml` under `[workspace.dependencies]`, then reference them with `.workspace = true` in each crate's `Cargo.toml`. Never add a version number directly in a crate's `Cargo.toml` for anything already in the workspace.

---

## Algorithms

The PRD defines 20+ novel algorithms that don't exist in other tools:

| Algorithm | Full Name | What it does |
|-----------|-----------|-------------|
| **CEP** | Content Extraction Protocol | 5-layer cascade: CSS → Readability → Headless JS → PDF → OCR |
| **QATBE** | Query-Aware Token-Budgeted Extraction | BM25 segment scoring + greedy knapsack within token budget |
| **SCS** | Semantic Content Segmentation | 8 segment types with type-aware token efficiency |
| **PDS** | Progressive Detail Streaming | 4 tiers: key_facts → summary → detailed → complete |
| **HyperFusion** | — | 8-signal ranking: BM25 + semantic + temporal + authority + evidence + diversity + depth + consensus |
| **QADD** | Query-Aware DOM Distillation | 5-step DOM pruning for 10-20x token reduction |
| **AMRS** | Adaptive Multi-Agent Research Swarm | 4 agent types via tokio channels |
| **PIE** | Persistent Intelligence Engine | Cross-session learning via SQLite |
| **RAR** | Retry-and-Refine | 5-checkpoint self-correction loop |
| **SPRE** | Speculative Pre-Ranking | Pre-rank results before full extraction |
| **QFD** | Query Fingerprinting | Deduplicate semantically equivalent queries |
| **ABS** | Adaptive Backend Selector | Route queries to optimal backend by intent |
| **TDR** | Temporal Decay Ranking | Boost fresh results for time-sensitive queries |
| **STP** | Source Trust Persistence | Learned domain authority scores across sessions |
| **RDO** | Result Diversity Optimization | MMR-based result diversification |
| **ATB** | Adaptive Token Budget | Dynamic budget allocation by query complexity |
| **CLQB** | Cross-Lingual Query Bridge | Translate + federate across language boundaries |
| **AXE** | Answer Extraction | Direct answer span detection from snippets |
| **EGB** | Evidence Graph Builder | Claim-level cross-source citation graph |
| **LP** | Latency Predictor | Backend latency prediction for parallel fetch ordering |

---

## Contributing

Contributions are welcome. Please read `TASKS.md` for the implementation roadmap and task ID format before opening a PR.

1. Fork the repository
2. Create a branch: `git checkout -b feat/my-feature`
3. Write code + tests
4. Run `cargo test && cargo clippy -- -D warnings && cargo fmt`
5. Commit using [Conventional Commits](https://www.conventionalcommits.org/): `feat: add X`
6. Open a pull request

All commits to `main` must follow Conventional Commits — this drives automated versioning via release-please.

---

## License

Licensed under either of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT))
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))

at your option.

---

<div align="center">

**[Docs](https://docs.fetchium.dev)** · **[Discord](https://discord.gg/fetchium)** · **[Twitter / X](https://x.com/fetchiumdev)** · **[GitHub](https://github.com/zuhabul/Fetchium)**

Built with Rust. No tracking. No telemetry. Your queries stay on your machine.

</div>
