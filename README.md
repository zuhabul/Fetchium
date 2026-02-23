<div align="center">

<br/>

```
██╗  ██╗██╗   ██╗██████╗ ███████╗██████╗ ███████╗███████╗ █████╗ ██████╗  ██████╗██╗  ██╗
██║  ██║╚██╗ ██╔╝██╔══██╗██╔════╝██╔══██╗██╔════╝██╔════╝██╔══██╗██╔══██╗██╔════╝██║  ██║
███████║ ╚████╔╝ ██████╔╝█████╗  ██████╔╝███████╗█████╗  ███████║██████╔╝██║     ███████║
██╔══██║  ╚██╔╝  ██╔═══╝ ██╔══╝  ██╔══██╗╚════██║██╔══╝  ██╔══██║██╔══██╗██║     ██╔══██║
██║  ██║   ██║   ██║     ███████╗██║  ██║███████║███████╗██║  ██║██║  ██║╚██████╗██║  ██║
╚═╝  ╚═╝   ╚═╝   ╚═╝     ╚══════╝╚═╝  ╚═╝╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝
```

### **The World's Fastest AI-Native Web Search & Deep Research Engine**
*Built in Rust · Built for Agents · Made for Humans*

<br/>

[![Tests](https://img.shields.io/badge/tests-563%20passing-brightgreen?style=flat-square&logo=rust)](https://github.com/user/hypersearchx)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust)](https://rustup.rs)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Binary](https://img.shields.io/badge/binary-single%20static-purple?style=flat-square)](https://github.com/user/hypersearchx/releases)
[![Startup](https://img.shields.io/badge/startup-~5ms-brightgreen?style=flat-square)]()
[![Algorithms](https://img.shields.io/badge/novel%20algorithms-17-ff6b6b?style=flat-square)]()

<br/>

[**Install**](#-installation) · [**Quick Start**](#-quick-start) · [**AI Setup**](#-ai-provider-setup) · [**Search Engines**](#-search-engines) · [**Intelligence**](#-intelligence-engine) · [**CLI**](#-cli-reference) · [**Config**](#-configuration)

<br/>

</div>

---

<div align="center">

> 🔍 Searches **Google + Bing + DuckDuckGo + Scholar + 10 more** — all free, all parallel
>
> 🤖 **6 AI providers** with subscription OAuth auto-detection — zero API key needed for Claude Code, Gemini CLI, or Codex CLI users
>
> ⚡ **Single Rust binary** — 5ms startup, 97% token reduction, 563 tests, zero runtime dependencies

</div>

---

## 🤔 Why HyperSearchX?

Every existing tool is broken in at least one critical way:

| Tool | The Problem |
|------|-------------|
| **Tavily** | $8 per 1,000 searches — expensive at scale |
| **Exa** | Complex pricing, limited free tier |
| **Perplexity** | Deep research capped at 20/month on Pro |
| **Firecrawl** | Returns entire page — no query-aware extraction |
| **Jina Reader** | Rate-limited, no search, no token budgeting |
| **Brave Search API** | Dropped their free tier entirely |
| **googler** | Google-only, no content extraction, no AI |
| **SearXNG** | No CLI, no extraction, no structured output |
| **Crawl4AI** | Python-only, no search capability |

**HyperSearchX fixes all of them** — free, fast, and designed from the ground up for AI agents.

---

## ✨ What Makes It Different

<table>
<tr>
<td width="50%">

**🦀 Rust-Native Performance**
- ~5ms startup (vs ~200ms Node.js)
- ~2ms HTML parsing (vs ~50ms)
- Single static binary
- Zero GC pauses, zero runtime deps

**🔍 All Search Engines**
- Google, Bing, Scholar via headless Chromium
- DuckDuckGo, SearXNG, Wikipedia, HN
- GitHub, ArXiv, StackOverflow, Reddit
- HyperFusion 8-signal re-ranking

**🤖 Multi-Provider AI**
- Gemini 2.0 Flash (default)
- Claude Max via subscription OAuth
- GPT via Codex CLI OAuth
- Ollama (local, 100% private)
- Auto-detects your existing subscriptions

</td>
<td width="50%">

**📊 17 Novel Algorithms**
- QATBE, SCS, PDS, QADD, HyDE, CEP
- HyperFusion, RAR, AMRS, EGP
- PIE, ToTR, CRP, EDF, SGT, CCE, ACS

**🧠 Cross-Session Intelligence**
- Source trust learning (Bayesian)
- Failure pattern memory
- Query prediction
- Evidence decay by domain type

**🔐 Privacy & Security**
- Standard / Private / Tor / Air-Gap modes
- PII redaction, encrypted cache
- Adversarial content detection
- TLS enforcement, robots.txt respect

</td>
</tr>
</table>

---

## 📦 Installation

```bash
# npm / pnpm / bun — downloads pre-built native binary (no Rust needed)
npm install -g hypersearchx
pnpm add -g hypersearchx
bun add -g hypersearchx

# Build from source
cargo install hypersearchx

# Or grab a binary from GitHub Releases
```

**Platform support:** `linux-x64` · `linux-arm64` · `darwin-x64` · `darwin-arm64` · `win-x64`

### System Requirements

| | Minimum | Recommended |
|--|---------|-------------|
| **OS** | Linux / macOS / Windows | Any |
| **RAM** | 2 GB free | 8 GB+ for local AI |
| **Disk** | 50 MB | 1 GB+ (cache + embeddings) |
| **Chromium** | Optional | For Google/Bing/Scholar |

### Optional Tools

| Tool | Why | Get It |
|------|-----|--------|
| Chromium/Chrome | Headless Google/Bing/Scholar | Usually pre-installed |
| **Gemini CLI** | Free AI via Google subscription | `npm i -g @google/gemini-cli` |
| **OpenAI Codex CLI** | AI via ChatGPT subscription | `npm i -g @openai/codex` |
| Ollama | 100% private local AI | [ollama.com](https://ollama.com) |
| Pandoc | PDF/DOCX export | `brew install pandoc` |
| Typst | Fast PDF (~1s vs 15s LaTeX) | `brew install typst` |
| pdftotext | Local PDF extraction | `brew install poppler` |
| Tesseract | Image/scan OCR | `brew install tesseract` |

---

## 🚀 Quick Start

### For Humans

```bash
# Parallel search across all engines, HyperFusion ranked
hsx search "best rust web framework 2026"

# Multi-source research with APA citations
hsx research "compare bun vs deno vs node.js performance" --citations apa

# Deep 4-agent agentic research (1-10 min)
hsx deep "security implications of WebAssembly in browsers"

# AI answer with streaming (uses your configured provider)
hsx ai "explain the CAP theorem with examples"

# Fetch any webpage as clean markdown
hsx view https://docs.example.com/guide

# Side-by-side comparison table
hsx compare "React vs Vue vs Svelte 2026"

# Monitor a page for changes
hsx monitor add https://github.com/user/repo/releases --interval 1h

# Full terminal UI
hsx tui
```

### For AI Agents

```bash
# Token-budgeted search — 200 tokens of key facts only
hsx agent-search "query" --budget 2000 --tier key_facts --format json

# Query-aware extraction — only pricing content, ≤1500 tokens
hsx agent-fetch https://example.com --query "pricing plans" --budget 1500

# Pre-fetch token estimation — know cost before committing
hsx agent-fetch https://example.com --estimate
# → { "total_tokens": 12340, "relevant_tokens": 1850, "extraction_layer": 1 }

# MCP server for Claude Code and any MCP client
hsx serve --mcp

# REST API for any language
hsx serve --api --port 3000
```

---

## 🤖 AI Provider Setup

HyperSearchX supports **6 AI providers** with **automatic subscription session detection**.
If you have Claude Code, Gemini CLI, or OpenAI Codex CLI installed — **no API key entry needed.**

### Interactive Wizard (Recommended)

```bash
hsx provider setup
```

The wizard auto-detects installed CLI sessions:

```
HyperSearchX AI Provider Setup Wizard
─────────────────────────────────────────────────────────────────
  ★ Subscription auth is auto-detected — no API key needed if you
    have Claude Code, Gemini CLI, or Codex CLI installed!

  1. Google Gemini       fast, generous free tier   [Gemini CLI session detected ✓]
  2. OpenAI              gpt-4o-mini                [Codex CLI session detected ✓]
  3. Anthropic Claude    claude-haiku               [Claude Code max detected ✓]
  4. OpenRouter          100+ models, one key
  5. Ollama              local, 100% private
  6. Gemini CLI          local gemini binary
```

### Supported Providers

| Provider | Auth | Default Model | Zero-Key? |
|----------|------|--------------|-----------|
| **🟦 Google Gemini** | API key **or** `gemini auth login` | `gemini-2.0-flash` | ✅ via Gemini CLI |
| **🟣 Anthropic Claude** | API key **or** Claude Code session | `claude-haiku-4-5-20251001` | ✅ via Claude Code |
| **🟢 OpenAI** | API key **or** Codex CLI session | `gpt-4o-mini` | ✅ via Codex CLI |
| **🟠 OpenRouter** | API key | `gemini-2.0-flash-exp:free` | ❌ key required |
| **⚫ Ollama** | None (localhost) | `gemma3:4b` | ✅ always |
| **🔵 Gemini CLI** | None (local binary) | `gemini-2.0-flash` | ✅ always |

### ✨ Antigravity Auth — Zero API Key Setup

> **Antigravity auth** = using your existing AI subscriptions directly, without any API key.
> HyperSearchX reads credentials from first-party CLI tools automatically.

<details>
<summary><b>🟣 Claude Code (Anthropic Max / Pro subscription)</b></summary>

```bash
# If you have Claude Code installed, HyperSearchX reads your
# macOS Keychain session automatically (service: "Claude Code-credentials")

hsx provider setup anthropic
# → ★ Claude Code max subscription session detected.
# → No API key needed — HyperSearchX will use your existing session.
# Include via subscription session? [Y/n]: Y
# ✓ Anthropic added (subscription auth)
```

</details>

<details>
<summary><b>🟦 Gemini CLI (Google Gemini subscription)</b></summary>

```bash
# Install and authenticate once
npm install -g @google/gemini-cli
gemini auth login   # Opens browser, stores to ~/.gemini/oauth_creds.json

# HyperSearchX reads ~/.gemini/oauth_creds.json and auto-refreshes when expired
hsx provider setup gemini
# → ★ Gemini CLI OAuth session detected (valid).
# → No API key needed — run `gemini auth login` if session expires.
```

</details>

<details>
<summary><b>🟢 OpenAI Codex CLI (ChatGPT subscription)</b></summary>

```bash
# Install and authenticate once
npm install -g @openai/codex
codex auth login    # Stores to ~/.codex/auth.json

hsx provider setup openai
# → ★ OpenAI Codex CLI session detected (ChatGPT subscription).
# → No API key needed.
```

</details>

### Fallback Chain

Providers are tried in order — first success wins:

```bash
hsx provider chain gemini anthropic openai ollama
# ✓ Fallback chain updated.
#   Google Gemini → Anthropic Claude → OpenAI → Ollama
```

```bash
hsx provider list    # View status with chain positions
hsx provider test    # Verify connectivity for all in chain
```

---

## 🔍 Search Engines

HyperSearchX searches **all major engines in parallel** and merges results using HyperFusion:

### Tier 1 — Full Search (headless Chromium)

| Engine | Strength |
|--------|---------|
| **Google** | Best index, freshest results, largest coverage |
| **Google Scholar** | Academic papers, citations, related work |
| **Bing** | Strong for technical and code queries |
| **Brave Search** | Independent index, no Google dependency |

### Tier 2 — Lightweight (HTTP only)

| Engine | Strength |
|--------|---------|
| **DuckDuckGo** | Fast, private, always available |
| **SearXNG** | Aggregates 244+ engines via JSON API |
| **Wikipedia** | Authoritative reference, REST API |
| **Hacker News** | Tech news via Algolia API |
| **ArXiv** | Academic preprints |
| **GitHub** | Code and open-source repos |
| **StackOverflow** | Programming Q&A |
| **Reddit** | Community discussions |

### HyperFusion Ranking

All results merged and re-ranked by **8 signals simultaneously**:

```
BM25 keyword relevance  ×0.25
Semantic embedding similarity  ×0.20
Temporal freshness (EDF)  ×0.15
Authority score  ×0.15
Evidence consensus  ×0.10
Source diversity  ×0.08
Content depth  ×0.04
Cross-source agreement  ×0.03
```

Weights auto-tune via AutoML (perceptron, 50+ feedback events).

```bash
# Configure engines
hsx config set search.backends.scholar true
hsx search "quantum computing breakthroughs" --engines google,bing,scholar,arxiv
```

---

## 🎯 Modes

### 🔎 Search — Instant Results

```bash
hsx search "playwright vs puppeteer memory 2026"
hsx search "rust async runtimes compared" --max-sources 20
hsx search "CVE-2025-xxxx patch" --domain security --engines google,bing
```

< 1s cached · < 3s uncached · HyperFusion ranked · 6-layer validated

### 📋 Research — Structured Reports

```bash
hsx research "GDPR implications for AI training data" --citations apa
hsx research "compare PostgreSQL vs ClickHouse" --output report.md
hsx research "nuclear fusion 2026" --export pdf --citations ieee
```

10-45s · Multi-source evidence map · RAR self-correction · 7 citation styles

### 🕵️ Deep Research — Agentic Investigation

```bash
hsx deep "Compare Puppeteer vs Playwright vs Crawlee at scale"
hsx deep "AI regulation: US vs EU vs China 2026" --max-depth 3
```

**AMRS** (Adaptive Multi-Agent Research Swarm) — 4 agents over tokio channels:
`Search Agent → Extract Agent → Verify Agent → Synthesize Agent`

Multi-hop · Contradiction detection · Evidence graphs · Source genealogy · 1-10 min

### 🤖 AI Synthesis — Multi-Provider Streaming

```bash
hsx ai "explain differential privacy with examples"
hsx ai "what changed in Python 3.14" --model claude-haiku-4-5
hsx ai "summarize AI news this week" --no-stream
```

Search → QATBE extraction → Ms-PoE sandwich layout → provider → streaming → citations

### ⚖️ Compare — Side-by-Side

```bash
hsx compare "React vs Vue vs Svelte 2026"
# Parallel research per item → markdown table with 7 dimensions
```

### 👁️ Monitor — Change Detection

```bash
hsx monitor add https://github.com/user/repo/releases --interval 1h
hsx monitor check https://example.com    # Check now
hsx monitor diff https://example.com     # Show what changed
hsx monitor list                          # All monitored pages
```

SHA-256 snapshots · `similar`-crate diff · SQLite storage · intervals: `30s`/`5m`/`1h`/`7d`

### 📄 Fetch / Export — Web as Files

```bash
hsx fetch https://example.com                     # → clean markdown
hsx fetch https://example.com --format json        # → structured JSON
hsx fetch https://example.com --format pdf         # → PDF (Typst ~1s)
hsx fetch https://example.com --format docx        # → DOCX (Pandoc)
hsx fetch https://example.com --format bibtex      # → BibTeX citation
hsx fetch https://paper.pdf --format md            # → PDF text extraction
hsx fetch https://youtube.com/watch?v=xxx          # → YouTube transcript
```

---

## 📊 Token Efficiency

HyperSearchX achieves **97% token reduction** vs raw HTML:

| Format | Tokens (typical page) | Reduction | How |
|--------|---------------------|-----------|-----|
| Raw HTML | 50,000 | — | Baseline |
| Clean HTML | 12,000 | 76% | Strip tags |
| Flat Markdown *(Firecrawl/Jina)* | 4,000 | 92% | Remove HTML |
| **HyperSearchX SCS** | **1,500** | **97%** | Semantic segments |
| **HyperSearchX key_facts** | **200** | **99.6%** | BM25 + knapsack |

**The pipeline:**
```
DOM (50k tokens)
  → QADD pruning         → 20k  (-60%)  [query-aware node filtering]
  → Boilerplate strip    → 14k  (-30%)  [nav/footer/ads removed]
  → SCS encoding         → 9k   (-35%)  [typed segments, efficient JSON]
  → BM25 filter          → 7k   (-20%)  [only relevant segments]
  → Cross-source dedup   → 6k   (-10%)  [remove duplicates]
  → Greedy knapsack      → 1.5k          [fit within token budget]
```

### SCS Segment Types

Instead of flat markdown, get **8 typed segments** — each encoded most efficiently:

```json
{
  "segments": [
    { "type": "fact",    "claim": "PostgreSQL supports JSONB indexing", "confidence": 0.95 },
    { "type": "table",   "headers": ["Feature","PG","MySQL"], "rows": [["JSON","Native","Basic"]] },
    { "type": "code",    "language": "sql", "code": "CREATE INDEX ON tbl USING gin(col);" },
    { "type": "summary", "text": "PostgreSQL excels for complex analytical queries." },
    { "type": "quote",   "text": "...", "source": "https://..." },
    { "type": "list",    "items": ["Item 1", "Item 2"] },
    { "type": "heading", "level": 2, "text": "Installation" },
    { "type": "meta",    "published": "2025-01-15", "author": "..." }
  ]
}
```

---

## 🧠 Intelligence Engine

HyperSearchX learns and improves across sessions, stored in `~/.hypersearchx/intelligence/`.

```bash
hsx intelligence stats    # View all intelligence metrics
hsx intelligence trust https://reuters.com  # Domain trust score
hsx intelligence suggest  # Predicted follow-up queries
hsx intelligence export   # Export knowledge graph
hsx intelligence reset    # Clear learned data
hsx intelligence totr "Is nuclear fusion viable by 2035?"  # Tree-of-Thoughts
```

### PIE — Persistent Intelligence Engine

**4 memory subsystems** (all SQLite-backed, WAL mode):

| Subsystem | What It Learns |
|-----------|---------------|
| **STM** Source Trust Memory | Bayesian Beta(α,β) trust per domain — reuters.com: 0.94, random-blog.io: 0.31 |
| **FPM** Failure Pattern Memory | Which extraction layer works per domain — site-x needs headless (JS wall) |
| **QPM** Query Prediction | SHA-256 hashed query history → follow-up suggestions |
| **PKG** Personal Knowledge Graph | Entity/relationship graph across all your research |

### EDF — Evidence Decay Function

Domain-specific half-lives (not a fixed 365 days):

```
AI/ML news         → 30 days    (fast-moving field)
Security advisories → 14 days    (patches matter)
Financial data     →  7 days    (market moves)
General tech news  → 90 days
Scientific papers  → 730 days   (stable knowledge)
Reference docs     → ∞          (MDN, RFC, Wikipedia)
Mathematics        → ∞          (proofs don't expire)
```

### CCE — Confidence Calibration Engine

> "Our 85% confidence has been right **82%** of the time."

Tracks historical accuracy per source. Isotonic regression calibration. Activates after 10 samples per bin.

### ACS — Adversarial Content Shield

Detects 3 classes of adversarial content:
- **AI-generated spam**: burstiness analysis + vocabulary variance + sentence length variance
- **Bot farms**: domain pattern heuristics
- **Manipulation**: hedge word density analysis

Shadow mode → transitions to active after 30 days of observation.

### CRP — Contradiction Resolution Protocol

When sources disagree, investigates instead of listing both:

```
Detects: "coffee reduces heart risk" vs "coffee increases anxiety"
Step 1: Date-based resolution (newer study wins?)
Step 2: Authority-based (NIH > personal blog)
Step 3: Context-based (study population? dosage?)
Step 4: Independent investigation
Step 5: Weighted synthesis with full trail
```

### SGT — Source Genealogy Tracker

Traces claims to their primary source:

```
TechBlog post → TheVerge article → Twitter thread → ArXiv paper (PRIMARY)
Detected mutation: "97.3% on MMLU-Pro" → "98% on MMLU" (cascade distortion)
Trust penalty: -15% per hop from primary source
```

### ToTR — Tree-of-Thoughts Research

```bash
hsx intelligence totr "Is nuclear fusion economically viable by 2035?"
# Branch 1: Technical feasibility (ITER, NIF progress)
# Branch 2: Economics (LCOE projections, capital costs)
# Branch 3: Policy & regulation (fusion licensing, grid integration)
# → Prunes low-evidence branches → synthesizes survivors
```

---

## 🔒 Privacy Modes

```bash
hsx search "sensitive query" --privacy private
hsx search "query" --privacy tor         # All traffic via Tor SOCKS5
hsx research "query" --privacy air-gap   # Local index only, zero network

hsx config set privacy.mode private      # Set globally
```

| Mode | Cache | PIE Logging | Network | PII |
|------|-------|-------------|---------|-----|
| `standard` | ✅ Write+Read | ✅ Full | ✅ All | As-is |
| `private` | 🚫 Read-only | 🚫 Disabled | ✅ All | Redacted |
| `tor` | 🚫 Read-only | 🚫 Disabled | 🕵️ Via Tor | Redacted |
| `air-gap` | ✅ Local only | ✅ Local | 🚫 None | As-is |

PII redaction strips: emails, phone numbers, SSNs, credit cards, IP addresses.

---

## 🎨 Proactive Intelligence

```bash
# Subscribe to topics
hsx subscribe add "rust programming" --interval 1d
hsx subscribe add "AI safety research" --interval 12h
hsx subscribe list

# View trending topics from your subscriptions
hsx radar 10

# Generate a digest
hsx digest daily --topics "rust,AI,security"
hsx digest weekly > weekly-report.md
```

---

## 🔌 Plugin System

Extend HyperSearchX with custom backends, extractors, rankers, formatters, validators, and AI providers:

```bash
hsx plugin list
hsx plugin enable my-custom-backend
```

Plugins live in `~/.hypersearchx/plugins/`. Each needs a `plugin.toml`:

```toml
name    = "my-backend"
version = "1.0.0"
type    = "backend"   # backend|extractor|ranker|formatter|validator|ai_provider
runtime = "native"    # native | wasm
```

---

## 🌍 Domain Modes

Activate domain-specific ranking overrides and special extraction features:

```bash
hsx search "quantum entanglement" --domain academic
hsx research "CVE-2025-xxxx" --domain security
hsx search "merger antitrust" --domain legal
```

| Mode | Ranking Priority | Special Features |
|------|----------------|-----------------|
| `academic` | .edu, arXiv, peer-reviewed | BibTeX output, citation tracking |
| `security` | CVE freshness, vendor advisories | Severity extraction, CVSS scoring |
| `legal` | Jurisdiction-aware, formal | Statute citation format |
| `medical` | PubMed, publication date strict | Dosage/risk flag extraction |
| `financial` | Real-time data, verified sources | Number/percentage extraction |
| `code` | GitHub, SO, official docs | Code block preservation |

---

## 🎬 Multimodal

```bash
# YouTube transcript (timedtext API, no key needed)
hsx fetch "https://youtube.com/watch?v=dQw4w9WgXcQ"

# PDF text extraction (pdftotext, page-split)
hsx fetch "https://arxiv.org/pdf/2301.00001.pdf"

# Image OCR (tesseract)
hsx fetch "https://example.com/screenshot.png"

# HTML table extraction → structured JSON
hsx fetch "https://example.com/comparison-table" --format json
```

---

## 📐 Validation Layer

Every result passes through **6-layer validation** + self-correction:

```
V1 Authority    → domain tier, SSL cert, redirect depth, WHOIS age
V2 Content      → relevance score, paywall detection, min length, dedup
V3 Temporal     → EDF decay, published date, cache freshness
V4 Cross-Source → bigram-Jaccard contradiction, negation-aware clustering
V5 Extraction   → truncation check, encoding errors, segment completeness
V6 Output       → citation reachability, content hash drift detection
   RAR Loop     → 5 reflection checkpoints: detect bad result → reformulate → re-search
```

---

## ⚙️ Configuration

Config file: `~/.hypersearchx/config.toml`

```toml
[general]
max_results = 10
verbose = false

[search]
default_budget = 4000
backends = ["duckduckgo", "brave", "searxng", "wikipedia"]

[cache]
enabled = true
ttl_secs = 3600

[ai]
default_model = "gemini-2.0-flash"
fast_model    = "gemini-2.0-flash"   # Used for HyDE + intent classification
ollama_host   = "http://localhost:11434"

[ai.providers]
fallback_chain = ["gemini", "anthropic", "ollama"]

[ai.providers.gemini]
model = "gemini-2.0-flash"
# api_key = "AIza..."  # Omit to auto-detect from Gemini CLI OAuth

[ai.providers.anthropic]
model = "claude-haiku-4-5-20251001"
# api_key = "sk-ant-..."  # Omit to auto-detect from Claude Code session

[ai.providers.openai]
model = "gpt-4o-mini"
# api_key = "sk-..."  # Omit to auto-detect from Codex CLI session

[ai.providers.openrouter]
model = "google/gemini-2.0-flash-exp:free"
api_key = "sk-or-..."   # Required; get at openrouter.ai

[ai.providers.ollama]
model = "gemma3:4b"

[privacy]
mode = "standard"   # standard | private | tor | air-gap

[pie]
enabled = true
```

### Environment Variables

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GEMINI_API_KEY="AIza..."
export OPENROUTER_API_KEY="sk-or-..."
export HSX_AI_PROVIDER="gemini"        # Override provider chain for this session
```

---

## 📟 CLI Reference

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           HyperSearchX Commands                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ HUMAN COMMANDS                                                              │
│  search <query>         Parallel multi-engine search, HyperFusion ranked   │
│  research <query>       Structured report — citations, evidence map, RAR   │
│  deep <query>           Agentic investigation — AMRS 4-agent swarm         │
│  ai <query>             AI synthesis — streaming, multi-provider           │
│  fetch <url>            Fetch + extract (all CEP layers)                   │
│  view <url>             Clean terminal-readable output                     │
│  compare <query>        Side-by-side comparison table                      │
│  monitor <sub>          Change detection — add/remove/check/list/diff      │
├─────────────────────────────────────────────────────────────────────────────┤
│ AGENT COMMANDS                                                              │
│  agent-search <query>   Token-budgeted JSON search (QATBE)                │
│  agent-fetch <url>      Query-aware extraction with budget                 │
│  agent-research <query> Structured research pipeline (JSON)               │
│  serve --mcp            MCP stdio server (5 composite tools)              │
│  serve --api            REST API (axum, POST /api/*)                       │
│  serve --mcp --api      Both simultaneously                                │
├─────────────────────────────────────────────────────────────────────────────┤
│ AI PROVIDER COMMANDS                                                        │
│  provider list          Status + chain position + auth type                │
│  provider setup         Interactive wizard (auto-detects subscriptions)   │
│  provider setup <name>  Setup a specific provider                          │
│  provider set <name>    Set key/model non-interactively                    │
│  provider chain <...>   Set fallback order                                 │
│  provider test [name]   Test connectivity                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│ INTELLIGENCE COMMANDS                                                       │
│  intelligence stats     Source trust, calibration, query topics            │
│  intelligence trust <u> Trust score for a domain                           │
│  intelligence suggest   Predicted follow-up queries                        │
│  intelligence export    Export knowledge graph                             │
│  intelligence reset     Clear all learned data                             │
│  intelligence totr <q>  Tree-of-Thoughts research                         │
├─────────────────────────────────────────────────────────────────────────────┤
│ PROACTIVE COMMANDS                                                          │
│  subscribe add <topic>  Subscribe for monitoring                           │
│  subscribe list/remove  Manage subscriptions                               │
│  radar [limit]          Trending topics from subscriptions                 │
│  digest <period>        Generate digest (daily/weekly/monthly)             │
├─────────────────────────────────────────────────────────────────────────────┤
│ WORKSPACE & PLUGIN                                                          │
│  workspace create/fork/merge/sync   Collaborative research sessions        │
│  plugin list/enable/disable         Plugin management                      │
├─────────────────────────────────────────────────────────────────────────────┤
│ UTILITY                                                                     │
│  doctor                 System health check (providers + resources)        │
│  config get/set         Configuration management                           │
│  cache clear/stats      Cache management                                   │
│  index add/search/stats Local document index (HNSW + SQLite)              │
│  completions <shell>    Shell completions (bash/zsh/fish/powershell)       │
│  tui                    Full terminal UI                                   │
└─────────────────────────────────────────────────────────────────────────────┘

KEY FLAGS
  --budget <n>            Token budget (default: 4000)
  --tier <level>          key_facts | summary | detailed | complete
  --engines <list>        google,bing,ddg,scholar,searxng,arxiv,...
  --format <fmt>          md | json | segments | csv | html | yaml | bibtex | pdf | docx
  --citations <style>     inline | footnote | apa | mla | chicago | ieee | bibtex
  --validate <mode>       strict | standard | fast | off
  --privacy <mode>        standard | private | tor | air-gap
  --domain <mode>         academic | security | legal | medical | financial | code
  --model <m>             Override AI model for this query
  --max-sources <n>       Maximum sources
  --no-stream             Disable streaming output
  --estimate              Pre-fetch token estimation
  --evidence-graph        Output evidence graph JSON
  -v / -q                 Verbose / quiet
  --no-cache              Bypass cache for this query
```

---

## 📊 How HyperSearchX Compares

| Feature | **HyperSearchX** | Tavily | Perplexity | Firecrawl | Crawl4AI | googler |
|---------|:-----------:|:------:|:----------:|:---------:|:--------:|:-------:|
| **Language** | 🦀 Rust | Python | N/A | Python | Python | Python |
| **Cost** | **Free** | $8/1K | $20/mo | $16+/mo | Free | Free |
| **Engines** | **12+** | 1 | Proprietary | 0 | 0 | 1 |
| **Startup** | **~5ms** | ~500ms | N/A | ~2s | ~1s | ~200ms |
| **Binary** | **Single static** | pip | SaaS | Docker | pip | pip |
| **Query-Aware Extract** | ✅ QATBE | ❌ | N/A | ❌ | ❌ | ❌ |
| **Token Budgeting** | ✅ PDS 4 tiers | ⚠️ basic | N/A | ❌ | ❌ | ❌ |
| **Semantic Segments** | ✅ SCS 8 types | ❌ | N/A | ❌ | ❌ | ❌ |
| **Self-Correction** | ✅ RAR 5-step | ❌ | ❌ | ❌ | ❌ | ❌ |
| **MCP Server** | ✅ Composite | ⚠️ basic | ❌ | ⚠️ basic | Community | ❌ |
| **Validation** | ✅ 6-layer | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Evidence Graph** | ✅ SHA-256 | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Deep Research** | ✅ Unlimited | ❌ | 20/month | ❌ | ❌ | ❌ |
| **Cross-Session Learning** | ✅ PIE | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Contradiction Detection** | ✅ CRP | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Source Genealogy** | ✅ SGT | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Adversarial Shield** | ✅ ACS | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Confidence Calibration** | ✅ CCE | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Multi-Path Reasoning** | ✅ ToTR | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Evidence Decay Model** | ✅ EDF | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Privacy Modes** | ✅ 4 (incl. Tor) | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Plugin System** | ✅ 6 types | ❌ | ❌ | ❌ | ❌ | ❌ |
| **Multimodal** | ✅ YouTube/PDF/OCR | ❌ | ❌ | ⚠️ | ❌ | ❌ |
| **Self-Evolving** | ✅ AutoML + A/B | ❌ | ❌ | ❌ | ❌ | ❌ |
| **AI Synthesis** | ✅ 6 providers | ❌ | ✅ paid | ❌ | ❌ | ❌ |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        HyperSearchX                             │
├──────────────┬──────────────┬────────────────┬─────────────────┤
│   hsx-cli    │   hsx-core   │    hsx-mcp     │    hsx-api      │
│  (binary)    │  (library)   │  (MCP stdio)   │  (REST axum)    │
└──────┬───────┴──────┬───────┴────────────────┴─────────────────┘
       │              │
       │        ┌─────▼──────────────────────────────────────┐
       │        │                hsx-core                     │
       │        ├──────────┬──────────┬──────────┬───────────┤
       └───────►│  search  │ extract  │   rank   │  token    │
                │ (12 eng) │ (CEP 5L) │(HyFusion)│(QATBE/SCS)│
                ├──────────┼──────────┼──────────┼───────────┤
                │ validate │ research │    ai    │intelligence│
                │ (6-layer)│ (AMRS)   │(6 prov.) │(PIE/ToTR) │
                ├──────────┼──────────┼──────────┼───────────┤
                │ citation │embeddings│  export  │  privacy  │
                │ (7 styles│(fastembed│(typst/pd)│(4 modes)  │
                ├──────────┼──────────┼──────────┼───────────┤
                │  plugin  │  collab  │proactive │  evolve   │
                │ (6 types)│workspace │(radar/  )│(automl/ab)│
                └──────────┴──────────┴──────────┴───────────┘
```

### Key Files

```
crates/hsx-core/src/
├── ai/
│   ├── credentials.rs      ← OAuth detection (Claude Code · Gemini CLI · Codex CLI)
│   ├── provider_client.rs  ← Fallback chain, SSE streaming, token refresh
│   ├── providers.rs        ← ProviderKind, ProvidersConfig, ProviderEntry
│   ├── pipeline.rs         ← run_ai_pipeline() (search→extract→sandwich→provider)
│   ├── router.rs           ← select_model() + select_fast_model() for HyDE
│   ├── sandwich.rs         ← Ms-PoE sandwich layout (lost-in-middle mitigation)
│   └── ollama.rs           ← Local Ollama client with streaming
├── intelligence/
│   ├── pie/                ← STM + FPM + QPM + PKG subsystems
│   ├── edf.rs              ← Evidence decay, 10 domain half-life categories
│   ├── cce.rs              ← Confidence calibration (isotonic regression)
│   ├── acs.rs              ← Adversarial content shield (shadow → active)
│   ├── crp.rs              ← Contradiction resolution (5-step pipeline)
│   ├── sgt.rs              ← Source genealogy tracker (bigram-Jaccard)
│   └── totr.rs             ← Tree-of-Thoughts research
├── rank/
│   └── signals.rs          ← ScoringContext with batch embeddings (7.5× speedup)
├── token/
│   └── qatbe.rs            ← BM25 + hybrid embedding ranking + greedy knapsack
├── export/
│   └── pandoc.rs           ← PDF: typst (~1s) → xelatex → pandoc default
└── query/
    └── hyde.rs             ← Hypothetical Document Embedding
```

---

## 🤝 Contributing

```bash
# Prerequisites: Rust 1.75+, Chromium (optional, for headless tests)

# Clone and build
git clone https://github.com/user/hypersearchx.git
cd hypersearchx
cargo build -p hsx-cli

# Run (26 commands)
./target/debug/hsx --help
./target/debug/hsx doctor

# Tests (563 passing, 0 failures)
cargo test

# Lint — zero warnings policy
cargo clippy -- -D warnings

# Format
cargo fmt

# Benchmark
cargo bench

# Build with optional features
cargo build -p hsx-core --features embeddings
cargo build -p hsx-core --features "embeddings,vector-search,headless"
```

### Feature Flags

| Flag | Dependency | Adds |
|------|-----------|------|
| `embeddings` | `fastembed` | Sentence embeddings (384-dim, all-MiniLM-L6-v2) for semantic search |
| `vector-search` | `usearch` | HNSW vector index for `hsx index` |
| `headless` | `chromiumoxide` | Headless Chromium (Google/Bing/Scholar/Scholar) |
| `mcp` | `rmcp` | MCP server protocol support |
| `llama` | `llama-cpp-2` | Direct llama.cpp integration for local inference |

---

## 📄 License

MIT © HyperSearchX Contributors

---

<div align="center">

**HyperSearchX** — AI-native. Rust-powered. Agent-first. Human-friendly.

*The fastest path from question to knowledge. Free. Open-source. Yours.*

<br/>

[![Star on GitHub](https://img.shields.io/github/stars/user/hypersearchx?style=social)](https://github.com/user/hypersearchx)

</div>
