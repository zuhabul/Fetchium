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

[![Tests](https://img.shields.io/badge/tests-941%20passing-brightgreen?style=flat-square&logo=rust)](https://github.com/zuhabul/Fetchium)
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

# Fetch and extract a single URL (clean text, Markdown, or JSON)
fetchium fetch https://example.com --format markdown

# AI-powered answer with citations
fetchium ai "What causes northern lights?"

# Deep multi-step research report
fetchium research "Impact of LLMs on software engineering jobs"

# Social media intelligence
fetchium social reddit "rust programming tips"
fetchium social twitter "AI news today"

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
| Multi-backend federation | Yes (8+) | No | No | No | No |
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
| **Multi-backend federation** | SearXNG + Brave + DuckDuckGo + Google + Bing + Reddit + HN + GitHub |
| **CEP — 5-layer extraction** | CSS selectors → Readability → Headless JS → PDF → Screenshot OCR |
| **QATBE — token budgeting** | BM25-scored segment ranking + greedy knapsack within token limits |
| **HyperFusion ranking** | 8-signal score: BM25 + semantic + temporal + authority + evidence + diversity + depth + consensus |
| **Evidence graph** | Claim-level source attribution and cross-reference verification |
| **AMRS — agent swarm** | 4 parallel agent types via tokio channels for deep research |
| **PIE — intelligence engine** | Cross-session learning: source trust, failure patterns, query prediction |
| **Social media intelligence** | Native Reddit, Twitter/X, HN, Facebook, TikTok with sentiment analysis |
| **QADD — DOM distillation** | 10-20x token reduction via 5-step DOM pruning |
| **RAR — self-correction** | 5-checkpoint retry-and-refine loop for AI answers |
| **PDS — progressive streaming** | 4 detail tiers: key_facts → summary → detailed → complete |
| **Key pool rotation** | Round-robin + 429-fallback across multiple Gemini / OpenAI keys |
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

### Check configured providers

```bash
fetchium provider list
```

---

## Search Engines

Fetchium federates across these backends and merges results with HyperFusion ranking:

| Backend | Type | Notes |
|---------|------|-------|
| SearXNG (self-hosted) | Meta-search | Covers Google, Bing, DDG and 70+ engines. Preferred Tier 0 |
| Brave Search | Web | Privacy-respecting, good freshness |
| DuckDuckGo | Web | No API key required |
| Google | Web | Via scraping (rate-limited) |
| Bing | Web | Via scraping (rate-limited) |
| Reddit | Social | Native API — threads, comments, scores |
| Hacker News | Social | Algolia search API |
| GitHub | Code | REST API — repos, issues, code search |

Backend selection is automatic via ABS (Adaptive Backend Selector) based on query intent. Code queries route to GitHub; social queries route to Reddit/HN; general queries use SearXNG first.

---

## Social Media Intelligence

```bash
# Reddit — threads + top comments + sentiment
fetchium social reddit "best mechanical keyboards 2025"

# Twitter/X — recent posts via SearXNG (site:x.com)
fetchium social twitter "GPT-5 release"

# Hacker News — stories + discussion
fetchium social hn "Show HN: my new project"

# TikTok — video metadata via tikwm
fetchium social tiktok "rust tutorial"

# Facebook — public posts via SearXNG (site:facebook.com)
fetchium social facebook "local events"
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

Commands:
  search      Federated web search across multiple backends
  fetch       Extract content from a URL (text, markdown, JSON)
  ai          AI-powered answer with source citations
  research    Deep multi-step research report
  social      Social media search and analysis
  serve       Start REST API or MCP server
  provider    Manage AI provider credentials
  setup       Install Chrome for Testing, configure SearXNG
  doctor      Check environment health (backends, AI, Chrome)
  config      Show or edit configuration

Options:
  -o, --output <FORMAT>   Output format: text, json, markdown [default: text]
  -q, --quiet             Suppress progress output
  -v, --verbose           Enable debug logging
      --fast              Skip full-page fetch (snippets only)
      --headless          Force headless Chrome for JS rendering
  -h, --help              Print help
  -V, --version           Print version
```

### Complete Command Reference

#### `fetchium search`

```bash
fetchium search "query" [OPTIONS]

Options:
  -n, --num <N>           Number of results [default: 10]
  -b, --backend <ID>      Force a specific backend
      --no-cache          Skip cache lookup
  -o, --output json       Output as JSON
```

#### `fetchium fetch`

```bash
fetchium fetch <URL> [OPTIONS]

Options:
  -f, --format <FMT>      Output format: text, markdown, json [default: text]
      --headless          Use Chrome for JS-rendered pages
      --pdf               Force PDF extraction mode
      --budget <TOKENS>   Token budget for QATBE extraction [default: 4096]
```

#### `fetchium ai`

```bash
fetchium ai "question" [OPTIONS]

Options:
      --fast              Snippets only, no full-page fetch (~10s)
  -p, --provider <NAME>   Override AI provider: gemini, openai, anthropic, ollama
      --model <MODEL>     Override model name
      --no-citations      Skip source citation output
```

#### `fetchium research`

```bash
fetchium research "topic" [OPTIONS]

Options:
      --depth <1-5>       Research depth (agents, rounds) [default: 3]
      --format <FMT>      report, markdown, json [default: report]
      --time-limit <SEC>  Max wall time [default: 120]
```

#### `fetchium serve`

```bash
fetchium serve [OPTIONS]

Options:
      --mode <MODE>       rest or mcp [default: rest]
      --port <PORT>       Bind port [default: 3050]
      --host <HOST>       Bind address [default: 127.0.0.1]
```

#### `fetchium setup`

```bash
fetchium setup [OPTIONS]

Options:
      --headless          Download Chrome for Testing (~200MB)
      --searxng           Pull and start SearXNG Docker container on port 4040
      --check             Show environment status without installing anything
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

Set `***REMOVED***` env var before starting the server. The server panics on startup if unset.

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
url = "***REMOVED***"

[ai]
default_provider = "gemini"
default_model = "gemini-2.5-flash"

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
FETCHIUM_SEARXNG_URL=***REMOVED***
FETCHIUM_AI_PROVIDER=gemini
***REMOVED***=my-secret
GEMINI_API_KEY=AIza...
GEMINI_API_KEYS=AIza...,AIza...,AIza...   # comma-separated pool
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
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
