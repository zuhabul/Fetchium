# Architecture Overview

## Workspace Structure

Fetchium is a Cargo workspace with 4 crates:

```
fetchium/
├── crates/
│   ├── fetchium-core/     # All algorithms and business logic
│   ├── fetchium-cli/      # The `fetchium` binary — clap CLI
│   ├── fetchium-mcp/      # MCP stdio server (Model Context Protocol)
│   └── fetchium-api/      # REST API server (axum 0.7)
├── tests/            # Integration + E2E tests
├── benches/          # Criterion benchmarks (in fetchium-core)
├── fuzz/             # cargo-fuzz targets
└── docs/             # This documentation
```

## Data Flow

```
User query
    │
    ▼
fetchium-cli (clap parse + HsxConfig)
    │
    ▼
fetchium-core pipeline:
    │
    ├─ search/ ──────► SearchOrchestrator
    │                   ├─ DuckDuckGo, Google, Bing, Brave
    │                   ├─ Scholar, SearXNG, Reddit, HN
    │                   └─ dedup + BM25 rerank
    │
    ├─ extract/ ─────► CEP (Cascade Extraction Protocol)
    │                   ├─ L1: CSS selectors (scraper, 85% of pages)
    │                   ├─ L2: Streaming rewriter (lol_html)
    │                   ├─ L3: Headless JS (chromiumoxide) [optional]
    │                   ├─ L4: PDF/DOCX (pdftotext/pandoc)
    │                   └─ L5: Screenshot OCR (tesseract)
    │
    ├─ rank/ ────────► HyperFusion (8-signal ranking)
    │                   BM25 + semantic + temporal + authority
    │                   + evidence + diversity + depth + consensus
    │
    ├─ token/ ───────► QATBE + SCS + PDS
    │                   Budget-aware segment selection and streaming
    │
    ├─ validate/ ────► 6-layer validation + RAR self-correction
    │
    ├─ citation/ ────► 7 citation styles + evidence graph
    │
    └─ output/ ──────► markdown / JSON / text / segments
```

## fetchium-core Module Map

```
fetchium-core/src/
├── lib.rs            # Crate root, module declarations
├── types.rs          # All shared data types
├── config.rs         # HsxConfig (TOML + env var loading)
├── error.rs          # HsxError + StructuredError taxonomy
│
├── http/             # HTTP layer
│   ├── client.rs     # HttpClient: pooling, retries, rate limiting
│   ├── sanitize.rs   # HTML sanitization (ammonia)
│   ├── tls.rs        # TLS enforcement
│   └── robots.rs     # robots.txt parser + cache
│
├── search/           # Search backends
│   ├── orchestrator.rs  # Parallel dispatch + dedup
│   ├── duckduckgo.rs
│   ├── google.rs, bing.rs, brave.rs
│   ├── scholar.rs, arxiv.rs
│   ├── reddit.rs, hackernews.rs
│   ├── wikipedia.rs, searxng.rs
│   └── dedup.rs, fallback.rs
│
├── extract/          # CEP extraction pipeline
│   ├── pipeline.rs   # Orchestrates L1-L5
│   ├── layer1.rs     # CSS selector extraction
│   ├── layer2.rs     # lol_html streaming
│   ├── layer3.rs     # Headless browser [headless feature]
│   ├── layer4.rs     # PDF/document
│   ├── layer5.rs     # OCR
│   ├── boilerplate.rs  # QADD pre-filter
│   └── cep_predictor.rs  # ML layer selection
│
├── rank/             # Ranking system
│   ├── bm25.rs       # Tantivy-backed BM25
│   ├── fusion.rs     # HyperFusion 8-signal
│   └── signals.rs    # Individual scoring signals
│
├── token/            # Token management
│   ├── counter.rs    # count_tokens, estimate_tokens_fast
│   ├── qatbe.rs      # Budget-aware segment selection
│   ├── scs.rs        # Structured Content Segmentation
│   └── pds.rs        # Progressive Detail Streaming (4 tiers)
│
├── validate/         # 6-layer validation
├── citation/         # Citation formatting + evidence graph
├── research/         # AMRS + SRP research pipelines
├── ai/               # Ollama client + Ms-PoE sandwich
├── intelligence/     # PIE: persistent learning
├── embeddings/       # ONNX embeddings [embeddings feature]
├── index/            # Local document index
├── monitor/          # URL change monitoring
├── compare/          # Side-by-side comparison
├── export/           # PDF/DOCX/BibTeX export
├── cache/            # moka async LRU cache
├── plugin/           # Plugin system
├── privacy/          # AES-256-GCM encryption
├── proactive/        # Subscriptions, radar, digest
├── evolve/           # Self-tuning (AutoML)
├── multimodal/       # Video, PDF, OCR, charts
└── collab/           # Collaborative workspaces
```

## Key Design Principles

1. **Progressive disclosure** — 4 PDS tiers let callers pay only for what they need
2. **Graceful degradation** — every layer has a fallback; never crash
3. **Token efficiency** — QATBE ensures >97% token reduction vs raw HTML
4. **Agent-first** — all outputs are machine-readable; JSON is a first-class format
5. **Feature flags** — heavy deps (ONNX, Chromium) are optional features
6. **Zero unsafe** — pure safe Rust throughout

## Performance Targets (PRD §40)

| Operation | Cached | Uncached |
|-----------|--------|---------|
| `fetchium agent-fetch` | <300ms | <3s |
| `fetchium search` | <1s | <5s |
| `fetchium fetch` | <200ms | <3s |
| Token estimation 1MB | <100ms | — |
| BM25 rank 100 docs | <10ms | — |
