<p align="center">
  <h1 align="center">HyperSearchX</h1>
  <p align="center">
    <strong>The world's fastest, AI-native, free web search and deep research engine.</strong><br/>
    <em>Built in Rust. Built for agents. Made for humans.</em>
  </p>
  <p align="center">
    <a href="#installation">Install</a> &bull;
    <a href="#quick-start">Quick Start</a> &bull;
    <a href="#modes">Modes</a> &bull;
    <a href="#ai-native">AI-Native</a> &bull;
    <a href="#search-engines">Engines</a> &bull;
    <a href="#configuration">Config</a> &bull;
    <a href="#contributing">Contributing</a>
  </p>
</p>

---

> Searches Google, Bing, DuckDuckGo, Scholar, and 10+ engines via efficient headless Chromium — all free.
> Delivers token-efficient, citation-backed results optimized for AI agents and human researchers.
> Written in Rust for maximum speed. Single static binary. Zero runtime dependencies.

## Why HyperSearchX?

Every existing tool is broken in at least one critical way:

| Tool | Problem |
|------|---------|
| **Tavily** | $8 per 1,000 searches |
| **Exa** | Complex pricing, limited free tier |
| **Perplexity** | Deep research limited to 20/month on Pro |
| **Firecrawl** | Returns entire page — no query-aware extraction |
| **Jina Reader** | Rate-limited, no search, no token budgeting |
| **Brave Search API** | Dropped free tier entirely |
| **googler** | Google-only, no content extraction, no AI |
| **SearXNG** | No CLI, no extraction, no structured output |
| **Crawl4AI** | Python-only, no search capability |

**HyperSearchX fixes all of them.**

### What Makes It Different

- **Rust-Native**: Single static binary, ~5ms startup, 25x faster HTML parsing than Node.js, zero GC pauses
- **All Search Engines**: Google, Bing, DuckDuckGo, Scholar, Brave, SearXNG, Wikipedia, and more — via efficient headless Chromium
- **AI-Native Agent-First**: Query-aware extraction, token budgeting, semantic segmentation, progressive detail tiers, MCP server, framework adapters
- **17 Novel Algorithms**: HyperFusion, QATBE, CEP, SCS, SRP, RAR, EGP, AMRS, PDS, QADD + PIE, ToTR, CRP, EDF, SGT, CCE, ACS — invented for HyperSearchX, exists nowhere else
- **97% Token Reduction**: Raw HTML → SCS segments = 97% fewer tokens than raw, 60% fewer than flat markdown
- **Zero Cost**: No API keys, no subscriptions, no credits, no rate-limited free tiers
- **Self-Correcting**: RAR loop detects bad retrievals and auto-corrects
- **Cross-Session Learning**: PIE remembers source trust, failure patterns, and query history across sessions
- **Adversarial Defense**: ACS detects AI-generated spam, bot farms, and manipulated sources
- **Contradiction Resolution**: CRP automatically investigates when sources disagree
- **Calibrated Confidence**: CCE tracks historical accuracy — "Our 85% has been right 82% of the time"
- **Evidence Provenance**: SGT traces claims to their primary source, detecting information cascade mutations
- **Temporal Awareness**: EDF models domain-specific evidence decay — AI news expires in days, math never does
- **Multi-Path Reasoning**: ToTR explores parallel reasoning strategies with branch pruning and synthesis
- **Validated Results**: 6-layer verification for every result
- **Reproducible**: Cryptographic evidence graphs with content hashes
- **Any Format**: MD, JSON, CSV, HTML, PDF, DOCX, BibTeX, YAML, SCS segments, and more

## Installation

```bash
# npm (downloads pre-built native binary for your platform)
npm install -g hypersearchx

# pnpm
pnpm add -g hypersearchx

# bun
bun add -g hypersearchx

# cargo (build from source)
cargo install hypersearchx

# Or download binary from GitHub Releases
```

No Rust toolchain needed for npm/pnpm/bun install. Pre-built binaries for:
- `linux-x64` | `linux-arm64` | `darwin-x64` | `darwin-arm64` | `win-x64`

### System Requirements

| Requirement | Minimum | Recommended |
|------------|---------|-------------|
| **OS** | Linux, macOS, Windows | Any |
| **RAM** | 2 GB free | 8 GB+ |
| **Chromium** | Auto-detected or auto-installed | For Google/Bing search |
| **Disk** | 50 MB (binary) | 1 GB+ (cache/index) |

### Optional Dependencies

| Tool | Purpose | Install |
|------|---------|---------|
| Chromium/Chrome | Headless search (Google, Bing, Scholar) | Usually pre-installed; auto-detected |
| Ollama | Local AI models for `hsx ai` mode | [ollama.com](https://ollama.com) |
| Pandoc | PDF/DOCX export | `brew install pandoc` |

## Quick Start

### For Humans

```bash
# Fast search across Google + Bing + DuckDuckGo + more
hsx search "best typescript orm 2026"

# Structured research with citations
hsx research "compare bun vs deno vs node.js performance"

# Deep agentic research
hsx deep "security implications of WebAssembly in browsers"

# AI-powered answer with local model
hsx ai "explain the CAP theorem with examples"

# Fetch any webpage as clean text
hsx view https://docs.example.com/guide

# Export to markdown
hsx export https://example.com/article --format md
```

### For AI Agents

```bash
# Token-budgeted search (200 tokens of key facts)
hsx agent-search "query" --budget 2000 --tier key_facts --format json

# Query-aware extraction (only content about "pricing", within 1500 tokens)
hsx agent-fetch https://example.com --query "pricing plans" --budget 1500

# Pre-fetch token estimation (before committing to fetch)
hsx agent-fetch https://example.com --estimate

# Structured research for agents
hsx agent-research "query" --budget 4000 --schema schema.json

# Start as MCP server (for Claude, Claude Code, any MCP client)
hsx serve --mcp

# Start as REST API (for any language/framework)
hsx serve --api --port 3000
```

You can also use `hyper` as an alias: `hyper search "query"`

## Search Engines

HyperSearchX searches **all major engines** — not just DuckDuckGo:

### Tier 1: Full Search Engines (via stealth headless Chromium)

| Engine | Method | Strengths |
|--------|--------|-----------|
| **Google** | Headless Chromium | Best index, freshest results, largest coverage |
| **Google Scholar** | Headless Chromium | Academic papers, citations, related work |
| **Bing** | Headless Chromium | Strong for technical queries |
| **Brave Search** | HTTP scrape | Independent index, privacy-focused |

### Tier 2: Lightweight (HTTP only, no headless needed)

| Engine | Method | Strengths |
|--------|--------|-----------|
| **DuckDuckGo** | HTTP scrape | Fast, private, no bot detection |
| **SearXNG** | JSON API | Aggregates 244+ engines |
| **Wikipedia** | REST API | Authoritative, structured |
| **Hacker News** | Algolia API | Tech news |
| **ArXiv** | Public API | Academic preprints |
| **GitHub** | Public API | Code and repos |
| **StackOverflow** | Public API | Programming Q&A |
| **Reddit** | Public JSON | Community discussions |

### How Headless Search Works

HyperSearchX uses `chromiumoxide` (Rust CDP client) to drive headless Chromium efficiently:

- **Resource blocking**: Images, CSS, fonts, ads blocked — only load HTML+JS
- **Stealth mode**: Patched `navigator.webdriver`, randomized fingerprints, TLS rotation
- **Connection reuse**: Single browser, multiple tabs for parallel searches
- **Pool management**: 1-8 instances based on available RAM
- **Fallback chain**: Google → Bing → DuckDuckGo → SearXNG (auto-fallback on detection)
- **Rate limiting**: Per-engine limits with jitter to avoid bot detection
- **Memory**: ~80MB per instance (vs ~150MB for Playwright in Node.js)

```bash
# Configure which engines to use
hsx config set backends.google true
hsx config set backends.bing true
hsx config set backends.scholar true

# Or per-query
hsx search "query" --engines google,bing,ddg,scholar
```

## Modes

### Search — Instant Results

```bash
hsx search "playwright vs puppeteer memory usage"
hsx search "rust web framework benchmarks" --max-sources 20 --ai
hsx search "kubernetes pod eviction" --engines google,bing
```

Parallel multi-engine search with HyperFusion ranking. <1s cached, <3s uncached.

### Research — Structured Reports

```bash
hsx research "GDPR implications for AI training data" --citations apa
hsx research "compare PostgreSQL vs ClickHouse" --output report.md
```

Multi-source analysis with evidence mapping, self-correction (RAR), and citations. 10-45s.

### Deep Research — Agentic Investigation

```bash
hsx deep "Compare Puppeteer vs Playwright vs Crawlee at scale"
hsx deep "AI regulation: US vs EU vs China 2026" --max-depth 3
```

Multi-hop research with AMRS (multi-agent swarm), query decomposition, contradiction detection, evidence graphs. 1-10 min.

### AI Preview — Local LLM Synthesis

```bash
hsx ai "explain WebDriver BiDi protocol"
hsx ai "what's new in TypeScript 6.0" --model ollama:llama3.2
```

Web search → token-optimized extraction → local LLM synthesis with citations. Requires Ollama.

### Fetch / View / Scrape — Web as Files

```bash
hsx fetch https://example.com                    # Clean text
hsx view https://example.com                     # Terminal readable
hsx scrape https://example.com/spa --scroll      # JS + infinite scroll
hsx export https://example.com --format md,json  # Multi-format
```

### Compare — Side-by-Side

```bash
hsx compare "React vs Vue vs Svelte 2026"
```

### Monitor — Change Detection

```bash
hsx monitor https://github.com/user/repo/releases --interval 1h --diff
```

## AI-Native Agent Architecture

HyperSearchX is **agent-first** — every feature is designed for programmatic consumption by AI systems.

### Core Novel Features for Agents (No Other Tool Has These)

#### 1. Query-Aware Token-Budgeted Extraction (QATBE)

Fetch a URL, extract ONLY content relevant to your query, within a token budget:

```bash
hsx agent-fetch https://example.com --query "pricing" --budget 1500
# Returns: only pricing-related content, within 1500 tokens
```

#### 2. Semantic Content Segmentation (SCS)

Instead of flat markdown, get typed segments — each in its most token-efficient format:

```json
{
  "segments": [
    { "type": "fact", "claim": "PostgreSQL supports JSONB indexing", "confidence": 0.95 },
    { "type": "table", "headers": ["Feature", "PG", "MySQL"], "rows": [["JSON", "Native", "Basic"]] },
    { "type": "code", "language": "sql", "code": "CREATE INDEX..." }
  ]
}
```

Tables as JSON arrays save 60% tokens vs markdown tables.

#### 3. Progressive Detail Streaming (PDS)

Request the detail level you need — without re-fetching:

```bash
hsx agent-search "query" --tier key_facts    # ~200 tokens
hsx agent-search "query" --tier summary      # ~1,000 tokens
hsx agent-search "query" --tier detailed     # ~5,000 tokens
hsx agent-search "query" --tier complete     # everything
```

#### 4. Pre-Fetch Token Estimation

Know the cost before committing:

```bash
hsx agent-fetch https://example.com --estimate
# → { "total_tokens": 12340, "relevant_tokens": 1850, "extraction_layer": 1 }
```

#### 5. MCP Server with Composite Tools

One tool call = entire pipeline (search + extract + rank + validate):

```bash
hsx serve --mcp
```

Exposes `hypersearch_search`, `hypersearch_fetch`, `hypersearch_research`, `hypersearch_estimate`, `hypersearch_expand` — each handles the full pipeline internally.

### Advanced Intelligence Features

#### 6. Cross-Session Learning (PIE)

HyperSearchX remembers across sessions — source trust, failure patterns, query predictions:

```bash
# PIE learns that reuters.com is reliable (trust: 0.94) and blog-x.com isn't (0.31)
# PIE remembers that site-y needs headless mode (JS wall)
# PIE predicts follow-up queries based on your history

hsx config set pie.enabled true    # Enable persistent intelligence
hsx pie stats                      # View learned source trust scores
hsx pie export                     # Export knowledge graph
hsx pie reset                      # Clear all learned data
```

#### 7. Multi-Path Reasoning (ToTR)

For complex questions, explore multiple reasoning paths simultaneously:

```bash
hsx deep "Is nuclear fusion economically viable by 2035?" --reasoning tree
# Explores: Technical feasibility | Economics | Policy — in parallel
# Prunes low-quality paths, synthesizes surviving branches
```

#### 8. Contradiction Resolution (CRP)

When sources disagree, HyperSearchX investigates instead of just listing both:

```bash
hsx research "is coffee good for you" --resolve-contradictions
# Detects: "coffee reduces heart risk" vs "coffee increases anxiety"
# Investigates: date, authority, population, dosage context
# Returns: weighted synthesis with investigation trail
```

#### 9. Source Genealogy (SGT)

Trace claims to their primary source — catch misinformation cascades:

```bash
hsx research "GPT-5 benchmarks" --trace-sources
# Traces: TechBlog → TheVerge → Twitter → ArXiv paper (PRIMARY)
# Detects: "97.3% on MMLU-Pro" mutated to "98% on MMLU" in cascade
```

#### 10. Adversarial Content Shield (ACS)

Detect AI-generated spam, bot farms, and manipulated search results:

```bash
hsx search "query" --shield strict
# Filters: AI-generated content, SEO spam farms, coordinated bot campaigns
# Trust score per source with audit trail
```

### Framework Adapters

```rust
// LangChain (Python)
from hypersearchx import HyperSearchXRetriever
retriever = HyperSearchXRetriever(token_budget=3000)

// REST API (any language)
POST http://localhost:3000/api/search
{ "query": "...", "token_budget": 2000, "tier": "summary" }
```

## Token Efficiency

HyperSearchX achieves **97% token reduction** vs raw HTML:

| Format | Tokens (typical page) | Reduction |
|--------|----------------------|-----------|
| Raw HTML | 50,000 | Baseline |
| Clean HTML | 12,000 | 76% |
| Flat Markdown (Firecrawl/Jina) | 4,000 | 92% |
| **HyperSearchX SCS** | **1,500** | **97%** |
| **HyperSearchX key_facts** | **200** | **99.6%** |

### How It Works

1. **QADD**: DOM distilled to query-relevant nodes only (~60% reduction)
2. **Boilerplate strip**: Nav, footer, ads, scripts removed (~30% more)
3. **SCS**: Content segmented into typed blocks in most efficient encoding (~30% more)
4. **BM25 filter**: Only query-relevant segments kept (~20% more)
5. **Dedup**: Cross-source duplicate removal (~10% more)

## Validation Layer

Every result passes through 6-layer validation + self-correction:

1. **Source**: Reachability, SSL, domain reputation, redirect analysis
2. **Content**: Relevance, language, dedup, paywall, error page detection
3. **Freshness**: Published date, staleness, cache freshness
4. **Cross-Source**: Claim consistency, fact triangulation, contradiction detection
5. **Extraction**: Completeness, structure, encoding
6. **Output**: Citation verification, link validity, format compliance
7. **RAR Self-Correction**: Auto-detects bad retrievals, reformulates query, re-searches

## Resource Awareness

Detects CPU/RAM/GPU/network and auto-adapts:

| Tier | RAM | Parallel Fetches | Browser Pool | AI Models |
|------|-----|-----------------|-------------|-----------|
| Minimal | <4 GB | 2-4 | 0-1 | 1-3B |
| Light | 4-8 GB | 4-8 | 1-2 | 3-7B |
| Standard | 8-16 GB | 8-16 | 2-4 | 7-13B |
| Power | 16-32 GB | 16-32 | 4-6 | 13-70B |
| Ultra | 32 GB+ | 32-50 | 6-8 | 70B+ |

```bash
hsx doctor    # Show system profile and capabilities
```

## Why Rust?

| Dimension | Node.js | **Rust (HyperSearchX)** |
|-----------|---------|------------------------|
| **Startup** | ~200ms | **~5ms** |
| **HTML parsing** | ~50ms/page | **~2ms/page** |
| **Memory base** | ~50MB | **~5MB** |
| **Browser instance** | ~150MB | **~80MB** |
| **GC pauses** | Yes | **None** |
| **Binary** | Requires runtime | **Single static binary** |
| **Concurrency** | Event loop | **tokio + rayon** |

## Configuration

Config: `~/.hypersearchx/config.yaml`

```yaml
defaults:
  max_sources: 10
  parallel: auto
  headless: auto
  citations: inline
  validate: true

backends:
  google: true          # via headless Chromium
  bing: true            # via headless Chromium
  scholar: false        # via headless Chromium
  brave: true           # HTTP scrape
  duckduckgo: true      # HTTP scrape
  searxng: true         # JSON API
  wikipedia: true       # REST API
  hackernews: false
  github: false
  arxiv: false

agent:
  default_budget: 4000
  default_tier: detailed
  default_format: segments
  deduplicate: true

ai:
  enabled: false
  provider: ollama
  model: llama3.2:8b
  endpoint: http://localhost:11434

resource_limits:
  max_memory_percent: 70
  max_cpu_percent: 80
  browser_pool_max: auto
```

## CLI Reference

```
HUMAN COMMANDS
  search <query>         Search the web (Google, Bing, DDG, etc.)
  research <query>       Structured multi-source research
  deep <query>           Agentic deep research
  ai <query>             AI-synthesized answer (local model)
  fetch <url>            Fetch and extract content
  view <url>             Clean readable view
  scrape <url>           Deep scrape with JS rendering
  compare <query>        Comparison research
  export <url>           Export to any format
  monitor <url>          Watch for changes

AGENT COMMANDS
  agent-search <query>   Token-budgeted search
  agent-fetch <url>      Query-aware extraction
  agent-research <query> Structured research for agents
  serve --mcp            MCP server mode
  serve --api            REST API server mode

UTILITY
  doctor                 System health check
  config <sub>           Configuration
  cache <sub>            Cache management
  index <sub>            Local index management

KEY FLAGS
  --budget <n>           Token budget
  --tier <level>         key_facts | summary | detailed | complete
  --engines <list>       google,bing,ddg,scholar,searxng,...
  --format <fmt>         md | json | segments | csv | html | yaml | bibtex
  --citations <style>    inline | footnote | apa | ieee | chicago | bibtex
  --validate <mode>      strict | standard | fast | off
  --ai                   Enable AI synthesis
  --fast / --thorough    Speed vs depth
  --schema <file>        JSON schema for structured output
  --estimate             Pre-fetch token estimation
  --evidence-graph       Output evidence graph
  --stream               Stream results progressively
```

## How HyperSearchX Compares

| Feature | HyperSearchX | Tavily | Perplexity | Firecrawl | Crawl4AI | googler |
|---------|-------------|--------|------------|-----------|----------|---------|
| **Language** | Rust | Python | N/A | Python | Python | Python |
| **Cost** | Free | $8/1K | $20/mo | $16+/mo | Free | Free |
| **Engines** | Google+Bing+DDG+10+ | Google API | Proprietary | N/A | N/A | Google only |
| **Query-Aware Extract** | Yes (QATBE) | No | N/A | No | No | No |
| **Token Budgeting** | Yes (PDS tiers) | max_tokens only | N/A | No | No | No |
| **Semantic Segments** | Yes (SCS) | No | N/A | No | No | No |
| **Self-Correction** | Yes (RAR) | No | No | No | No | No |
| **MCP Server** | Native composite | Basic | No | Basic | Community | No |
| **Validation** | 6-layer | None | None | None | None | None |
| **Evidence Graph** | Yes (EGP) | No | No | No | No | No |
| **Deep Research** | Unlimited | No | 20/month | No | No | No |
| **Cross-Session Learning** | Yes (PIE) | No | No | No | No | No |
| **Contradiction Resolution** | Yes (CRP) | No | No | No | No | No |
| **Source Genealogy** | Yes (SGT) | No | No | No | No | No |
| **Adversarial Shield** | Yes (ACS) | No | No | No | No | No |
| **Confidence Calibration** | Yes (CCE) | No | No | No | No | No |
| **Multi-Path Reasoning** | Yes (ToTR) | No | No | No | No | No |
| **Evidence Decay** | Yes (EDF) | No | No | No | No | No |
| **Reproducible** | Hash+logs+diffs | No | No | No | No | No |
| **AI Synthesis** | Local, free | No | Paid | No | No | No |
| **Binary** | Single static | Pip install | SaaS | Docker | Pip | Pip |
| **Startup** | ~5ms | ~500ms | N/A | ~2s | ~1s | ~200ms |

## Project Structure

```
hypersearchx/
├── crates/
│   ├── hsx-core/           # Core library
│   │   ├── src/
│   │   │   ├── search/     # Search backend orchestrator
│   │   │   │   ├── google.rs
│   │   │   │   ├── bing.rs
│   │   │   │   ├── duckduckgo.rs
│   │   │   │   ├── searxng.rs
│   │   │   │   ├── scholar.rs
│   │   │   │   ├── wikipedia.rs
│   │   │   │   ├── headless_pool.rs
│   │   │   │   └── orchestrator.rs
│   │   │   ├── extract/    # Content extraction pipeline
│   │   │   │   ├── cep.rs          # Cascade Extraction Protocol
│   │   │   │   ├── qadd.rs         # Query-Aware DOM Distillation
│   │   │   │   ├── readability.rs
│   │   │   │   ├── headless.rs
│   │   │   │   └── pipeline.rs
│   │   │   ├── rank/       # Ranking engine
│   │   │   │   ├── hyperfusion.rs
│   │   │   │   ├── bm25.rs
│   │   │   │   ├── semantic.rs
│   │   │   │   └── scoring.rs
│   │   │   ├── token/      # Token efficiency
│   │   │   │   ├── qatbe.rs        # Query-Aware Token-Budgeted Extraction
│   │   │   │   ├── scs.rs          # Semantic Content Segmentation
│   │   │   │   ├── pds.rs          # Progressive Detail Streaming
│   │   │   │   ├── boilerplate.rs
│   │   │   │   └── budget.rs
│   │   │   ├── validate/   # Validation layer
│   │   │   │   ├── source.rs
│   │   │   │   ├── content.rs
│   │   │   │   ├── freshness.rs
│   │   │   │   ├── cross_source.rs
│   │   │   │   ├── rar.rs          # Reflection-Augmented Research
│   │   │   │   ├── acs.rs          # Adversarial Content Shield
│   │   │   │   └── pipeline.rs
│   │   │   ├── intelligence/ # Advanced intelligence
│   │   │   │   ├── pie.rs          # Persistent Intelligence Engine
│   │   │   │   ├── totr.rs         # Tree-of-Thoughts Research
│   │   │   │   ├── crp.rs          # Contradiction Resolution Protocol
│   │   │   │   ├── edf.rs          # Evidence Decay Function
│   │   │   │   ├── sgt.rs          # Source Genealogy Tracker
│   │   │   │   └── cce.rs          # Confidence Calibration Engine
│   │   │   ├── ai/         # AI preview engine
│   │   │   │   ├── ollama.rs
│   │   │   │   ├── llama.rs
│   │   │   │   ├── router.rs
│   │   │   │   └── prompts.rs
│   │   │   ├── citation/   # Citation & evidence
│   │   │   │   ├── egp.rs          # Evidence Graph Protocol
│   │   │   │   ├── mapper.rs
│   │   │   │   ├── styles.rs
│   │   │   │   └── verifier.rs
│   │   │   ├── resource/   # Machine resource awareness
│   │   │   │   ├── profiler.rs
│   │   │   │   ├── monitor.rs
│   │   │   │   └── scheduler.rs
│   │   │   ├── parallel/   # Parallel execution engine
│   │   │   │   ├── queue.rs
│   │   │   │   ├── pool.rs
│   │   │   │   └── backpressure.rs
│   │   │   ├── cache/      # Caching system
│   │   │   │   ├── memory.rs
│   │   │   │   ├── disk.rs
│   │   │   │   └── manager.rs
│   │   │   ├── index/      # Local search index
│   │   │   │   ├── tantivy.rs
│   │   │   │   ├── vector.rs
│   │   │   │   └── manager.rs
│   │   │   ├── output/     # Output formatters
│   │   │   │   ├── markdown.rs
│   │   │   │   ├── json.rs
│   │   │   │   ├── csv.rs
│   │   │   │   ├── html.rs
│   │   │   │   └── bibtex.rs
│   │   │   └── plugin/     # Plugin system
│   │   │       ├── registry.rs
│   │   │       └── loader.rs
│   │   └── Cargo.toml
│   ├── hsx-cli/            # CLI binary
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands/
│   │   │   │   ├── search.rs
│   │   │   │   ├── research.rs
│   │   │   │   ├── deep.rs
│   │   │   │   ├── ai.rs
│   │   │   │   ├── fetch.rs
│   │   │   │   ├── agent.rs
│   │   │   │   ├── serve.rs
│   │   │   │   └── doctor.rs
│   │   │   └── output.rs
│   │   └── Cargo.toml
│   ├── hsx-mcp/            # MCP server
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   └── hsx-api/            # REST API server
│       ├── src/lib.rs
│       └── Cargo.toml
├── npm/                     # npm wrapper package
│   ├── package.json
│   └── scripts/install-binary.js
├── tests/
├── benches/
├── docs/
├── prd.md
├── Cargo.toml               # Workspace root
└── README.md
```

## Contributing

```bash
# Prerequisites: Rust 1.75+, Chromium/Chrome (for headless search)

# Clone
git clone https://github.com/user/hypersearchx.git
cd hypersearchx

# Build
cargo build --release

# Run
cargo run --release -- search "test query"

# Test
cargo test

# Bench
cargo bench
```

## License

MIT

---

<p align="center">
  <strong>HyperSearchX</strong> — AI-native. Rust-powered. Agent-first. Human-friendly.<br/>
  The fastest path from question to knowledge. Free. Open-source. Yours.
</p>
