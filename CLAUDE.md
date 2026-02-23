# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Check all crates compile (fast, no linking)
cargo check

# Build the hsx binary
cargo build -p hsx-cli

# Build optimized release binary
cargo build -p hsx-cli --release

# Run the binary
./target/debug/hsx --help
./target/debug/hsx doctor

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p hsx-core
cargo test -p hsx-cli

# Run a single test by name
cargo test -p hsx-core extract::layer1::tests::extract_simple_page

# Lint (zero warnings policy — treat warnings as errors in CI)
cargo clippy -- -D warnings

# Format
cargo fmt

# Generate docs
cargo doc --open
```

## macOS SDK Note

Xcode on this machine sometimes defaults to the iPhoneOS SDK, which breaks C dependency compilation (`zstd-sys`, `rusqlite`, `ring`). This is fixed in `.cargo/config.toml` which forces `SDKROOT` to the macOS SDK. If you see `"architecture not supported"` errors, run:

```bash
export SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
```

## Architecture Overview

HyperSearchX is a Cargo workspace with 4 crates:

| Crate | Role |
|-------|------|
| `hsx-core` | All algorithms: search, extract, rank, validate, cache, AI, intelligence |
| `hsx-cli` | The `hsx` binary — clap derive CLI, delegates everything to hsx-core |
| `hsx-mcp` | MCP server (Phase 4) — exposes hsx-core as Model Context Protocol tools |
| `hsx-api` | REST API server via axum (Phase 4) |

**Data flow:** `CLI command → HsxConfig → hsx-core pipeline → formatted output`

The CLI (`hsx-cli/src/main.rs`) parses args, loads config, then dispatches to one file per command in `crates/hsx-cli/src/commands/`. Each command calls into `hsx-core` modules.

## hsx-core Module Map

Most modules are currently stubs awaiting Phase 1+ implementation. Implemented:

- `types.rs` — All shared data types (PRD §43): `AgentSearchResult`, `SearchResult`, `ResultItem`, `Segment`, `Finding`, `Source`, `EvidenceGraph`, `CepLayer`, `PdsTier`, `ResourceTier`, `BackendId`, etc.
- `error.rs` — `HsxError`, `StructuredError`, `ErrorKind` (19 variants), `HsxResult<T>`
- `config.rs` — `HsxConfig` loaded from `~/.hypersearchx/config.toml` with env var overrides; includes `detect_resource_tier()` and `data_dir()`
- `http/client.rs` — `HttpClient` stub (reqwest with pooling/retries)
- `resource/mod.rs` — `detect_tier()` delegating to `HsxConfig::detect_resource_tier()`

Planned modules follow this pipeline order:
```
search/ → extract/ → rank/ → token/ → validate/ → citation/ → output/
                                                              ↑
                                             research/ ──────┘
```
Advanced: `ai/`, `intelligence/`, `cache/`, `index/`, `plugin/`, `privacy/`, `collab/`, `domain/`

## Optional Feature Flags (hsx-core)

Heavy optional dependencies are gated behind features:

| Feature | Crate | Phase |
|---------|-------|-------|
| `headless` | `chromiumoxide` | Phase 2 |
| `embeddings` | `ort` (ONNX Runtime) | Phase 5 |
| `vector-search` | `usearch` | Phase 5 |
| `mcp` | `rmcp` | Phase 4 |
| `llama` | `llama-cpp-2` | Phase 4 |

Build with a feature: `cargo build -p hsx-core --features headless`

## Task Planning System

Implementation is tracked across 9 phases:

- **`TASKS.md`** — Master index: phase overview, dependency graph, task ID format (`P{phase}-E{epic}-T{task}`), parallelization matrix
- **`tasks/phase-N-*.md`** — Detailed per-phase files with step-by-step Rust code, file paths, and acceptance criteria checklists
- **`prd.md`** — Product Requirements Document (source of truth for all algorithms and behaviors)

**Current status:** Phase 0 scaffold complete. Phases 1–7 are stubs. Start implementation at `tasks/phase-1-mvp-core.md`.

Rules from `TASKS.md` to follow: run `cargo build && cargo test && cargo clippy` after every task; keep files under 500 lines; use workspace dependencies (never add directly to a crate's Cargo.toml if it can be shared); all public APIs get `///` doc comments.

## Key PRD Algorithms

The PRD (§8) defines 17 novel algorithms that don't exist in other tools:

- **CEP** (Content Extraction Protocol) — 5-layer cascade: CSS selectors → readability → headless JS → PDF → screenshot OCR
- **QATBE** (Query-Aware Token-Budgeted Extraction) — BM25-scored segment ranking + greedy knapsack packing within token budget
- **SCS** (Semantic Content Segmentation) — 8 segment types with type-aware token efficiency
- **PDS** (Progressive Detail Streaming) — 4 tiers: key_facts (~200 tok), summary (~1000), detailed (~5000), complete
- **HyperFusion** — 8-signal ranking: BM25 + semantic + temporal + authority + evidence + diversity + depth + consensus
- **QADD** (Query-Aware DOM Distillation) — 5-step DOM pruning for 10-20x token reduction
- **AMRS** (Adaptive Multi-Agent Research Swarm) — 4 agent types via tokio channels
- **PIE** (Persistent Intelligence Engine) — Cross-session learning via SQLite (source trust, failure patterns, query prediction)
- **RAR** (Retry-and-Refine) — 5-checkpoint self-correction loop

## Adding New Dependencies

All shared dependencies go in the **workspace** `Cargo.toml` under `[workspace.dependencies]`, then reference them with `.workspace = true` in each crate's `Cargo.toml`. Never add a version number directly in a crate's `Cargo.toml` for anything already in the workspace.
