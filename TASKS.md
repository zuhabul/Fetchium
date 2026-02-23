# HyperSearchX — Master Implementation Task Plan

> **Version:** 1.0.0 | **Generated:** 2026-02-23
> **Total Phases:** 9 | **Total Epics:** 42 | **Total Tasks:** 200+
> **Target:** Full production-ready deployment
> **PRD Reference:** `prd.md` v4.0.0 (49 sections, 17 novel algorithms, 350+ features)

---

## How to Use This Task Plan

### For AI Agents
1. **Read this file first** — it's the master index with dependency graph and overview
2. **Open the phase file** for your assigned phase (e.g., `tasks/phase-0-foundation.md`)
3. **Check dependencies** — do NOT start a task until all deps are `DONE`
4. **Follow implementation guidance** — each task has step-by-step "how to build" instructions
5. **Verify acceptance criteria** — a task is ONLY done when ALL criteria are met
6. **Update status** — mark tasks as `IN_PROGRESS` → `DONE` when complete

### Task ID Format
`P{phase}-E{epic}-T{task}` — Example: `P1-E2-T3` = Phase 1, Epic 2, Task 3

### Status Legend
- `TODO` — Not started
- `IN_PROGRESS` — Being worked on
- `DONE` — Complete and verified
- `BLOCKED` — Waiting on dependency

### Priority Legend
- `P0` — Critical path, blocks everything
- `P1` — Core functionality
- `P2` — Important features
- `P3` — Advanced features
- `P4` — Polish and optimization

### Rules for Agents
1. **Never skip dependencies** — build on solid foundations
2. **Run `cargo build` after every task** — ensure it compiles
3. **Run `cargo test` after every task** — ensure nothing is broken
4. **Run `cargo clippy` after every task** — no warnings
5. **Write tests alongside code** — no separate "write tests" step
6. **Document all public APIs** with `///` doc comments
7. **Use the PRD as the source of truth** — when in doubt, check `prd.md`
8. **Keep files under 500 lines** — split into submodules when exceeding
9. **Use workspace dependencies** — never add a dep directly to a crate's Cargo.toml if it can be shared

---

## Phase Overview

| Phase | Name | Duration | Priority | Tasks | Detail File |
|-------|------|----------|----------|-------|-------------|
| 0 | [Project Foundation & Scaffolding](#phase-0) | Week 1 | P0 | 8 | [`tasks/phase-0-foundation.md`](tasks/phase-0-foundation.md) |
| 1 | [MVP Core — Search & Fetch](#phase-1) | Weeks 2-4 | P1 | 17 | [`tasks/phase-1-mvp-core.md`](tasks/phase-1-mvp-core.md) |
| 2 | [Multi-Engine Search & Headless](#phase-2) | Weeks 5-8 | P1 | 12 | [`tasks/phase-2-search-engines.md`](tasks/phase-2-search-engines.md) |
| 3 | [Validation, Research & Citations](#phase-3) | Weeks 9-12 | P1 | 7 | [`tasks/phase-3-research.md`](tasks/phase-3-research.md) |
| 4 | [AI Engine, Deep Research & MCP](#phase-4) | Weeks 13-18 | P1-P2 | 8 | [`tasks/phase-4-ai-deep-mcp.md`](tasks/phase-4-ai-deep-mcp.md) |
| 5 | [Semantic Search & Advanced Features](#phase-5) | Weeks 19-26 | P2 | 12 | [`tasks/phase-5-semantics.md`](tasks/phase-5-semantics.md) |
| 6 | [Intelligence Algorithms](#phase-6) | Weeks 27-36 | P2-P3 | 7 | [`tasks/phase-6-intelligence.md`](tasks/phase-6-intelligence.md) |
| 7 | [Advanced Features & Polish](#phase-7) | Weeks 37-48 | P3 | 12 | [`tasks/phase-7-advanced.md`](tasks/phase-7-advanced.md) |
| 8 | [Testing, Benchmarks & Production](#phase-8) | Ongoing | P0-P1 | 9 | [`tasks/phase-8-production.md`](tasks/phase-8-production.md) |

---

## Dependency Graph

```
Phase 0 (Foundation)
├── P0-E1: Cargo workspace + types + config
├── P0-E2: CI/CD + npm wrapper
└── P0-E3: CLI skeleton + doctor
         │
         ▼
Phase 1 (MVP) ← depends on Phase 0
├── P1-E1: HTTP client + extraction (CEP L1-2) + fetch/view
├── P1-E2: DDG search + orchestrator + search command
├── P1-E3: Token counter + QATBE + SCS + PDS
├── P1-E4: Agent commands (agent-search, agent-fetch)
├── P1-E5: BM25 ranking + dedup
├── P1-E6: Memory cache + disk cache
└── P1-E7: Output formatters (md, json, csv, yaml, html, segments)
         │
         ▼
Phase 2 (Multi-Engine) ← depends on Phase 1
├── P2-E1: Headless Chromium pool + Google + Bing + Scholar
├── P2-E2: SearXNG + Wikipedia + Brave + HN + ArXiv + GitHub + Reddit + SO
├── P2-E3: Full parallel search orchestrator
├── P2-E4: HyperFusion 8-signal ranking
└── P2-E5: CEP layers 3-5 (headless) + QADD
         │
         ▼
Phase 3 (Research & Validation) ← depends on Phase 2
├── P3-E1: 6-layer validation + RAR self-correction
├── P3-E2: Citation system (6 styles) + Evidence Graph Protocol (EGP)
└── P3-E3: Research mode + agent-research
         │
         ▼
Phase 4 (AI & Deep Research) ← depends on Phase 3
├── P4-E1: Ollama integration + AI command
├── P4-E2: AMRS multi-agent swarm + deep command
├── P4-E3: SRP speculative pipelining + streaming
├── P4-E4: MCP server (5 composite tools)
└── P4-E5: REST API server (axum)
         │
         ▼
Phase 5 (Semantics & Polish) ← depends on Phase 4
├── P5-E1: ONNX embeddings + vector index + HyDE
├── P5-E2: QATBE/QADD semantic upgrade
├── P5-E3: CEP ML predictor
├── P5-E4: Full PDS (all 4 tiers)
├── P5-E5: Compare + monitor commands
└── P5-E6: PDF/DOCX export
         │
         ▼
Phase 6 (Intelligence) ← depends on Phase 5
├── P6-E1: PIE — Persistent Intelligence Engine
├── P6-E2: ToTR — Tree-of-Thoughts Research
├── P6-E3: CRP — Contradiction Resolution Protocol
├── P6-E4: EDF — Evidence Decay Function
├── P6-E5: SGT — Source Genealogy Tracker
├── P6-E6: CCE — Confidence Calibration Engine
└── P6-E7: ACS — Adversarial Content Shield
         │
         ▼
Phase 7 (Advanced) ← depends on Phase 6
├── P7-E1: Plugin system (dynamic/WASM)
├── P7-E2: Privacy modes (private, tor, air-gap)
├── P7-E3: Collaborative research protocol
├── P7-E4: Domain-specific modes (academic, code, legal, medical, security, financial)
├── P7-E5: Proactive intelligence (subscriptions, radar, prefetch)
├── P7-E6: Multimodal content (images, video, PDF, charts)
├── P7-E7: Self-evolving architecture (AutoML, A/B testing)
├── P7-E8: Interactive TUI (ratatui)
├── P7-E9: Framework adapters (LangChain, CrewAI)
└── P7-E10: Shell completions (bash, zsh, fish)

Phase 8 (Production) ← parallel with ALL phases
├── P8-E1: Test suite (unit, integration, E2E, benchmark, fuzz)
├── P8-E2: Documentation (API docs, user guide)
└── P8-E3: Production hardening (security, perf, errors, release automation)
```

---

## Critical Path (MVP)

The minimum path to a **working product** (`hsx search` + `hsx fetch` for agents):

```
P0-E1-T1 (workspace) → P0-E1-T2 (types) → P0-E1-T3 (config) → P0-E3-T1 (CLI) →
P1-E1-T1 (HTTP client) → P1-E1-T2 (extraction) → P1-E1-T3 (fetch cmd) →
P1-E2-T1 (DDG backend) → P1-E2-T2 (orchestrator) → P1-E2-T3 (search cmd) →
P1-E3-T1 (tokens) → P1-E3-T2 (QATBE) → P1-E3-T3 (SCS) → P1-E3-T4 (PDS) →
P1-E4-T1 (agent-search) → P1-E4-T2 (agent-fetch) →
P1-E5-T1 (BM25) → P1-E6-T1 (cache) → P1-E7-T1 (formatters)
```

---

## Parallelization Matrix

These task groups can run **simultaneously** by different agents:

### Phase 0
| Agent A | Agent B |
|---------|---------|
| P0-E1 (workspace + types + config) | P0-E2 (CI/CD + npm wrapper) |

### Phase 1 (after P0-E1 done)
| Agent A | Agent B | Agent C | Agent D |
|---------|---------|---------|---------|
| P1-E1 (HTTP + extract) | P1-E2 (DDG search) | P1-E3 (token system) | P1-E5 (ranking) |
| → P1-E1-T3 (fetch cmd) | → P1-E2-T3 (search cmd) | → P1-E3-T2/T3/T4 | P1-E6 (cache) |
| | | → P1-E4 (agent cmds) | P1-E7 (output) |

### Phase 2
| Agent A | Agent B | Agent C |
|---------|---------|---------|
| P2-E1 (headless + Google + Bing + Scholar) | P2-E2 (SearXNG + Wiki + other HTTP backends) | P2-E4 (HyperFusion ranking) |
| → P2-E5 (CEP L3-5 + QADD) | → P2-E3 (full orchestrator after A+B) | |

### Phase 6
| Agent A | Agent B | Agent C | Agent D |
|---------|---------|---------|---------|
| P6-E1 (PIE) | P6-E7 (ACS) | P6-E2 (ToTR) | P6-E3 (CRP) |
| P6-E6 (CCE) | | | P6-E4 (EDF) + P6-E5 (SGT) |

---

## Technology Stack Quick Reference

| Component | Crate | Purpose |
|-----------|-------|---------|
| Async runtime | `tokio` | All async operations |
| HTTP client | `reqwest` | Web fetching with retries |
| HTML parsing (CSS) | `scraper` | CSS selector extraction |
| HTML streaming | `lol_html` | Streaming HTML rewriting |
| Headless browser | `chromiumoxide` | Google/Bing/Scholar search |
| CLI | `clap` (derive) | Command-line parsing |
| Progress | `indicatif` | Spinners and progress bars |
| Terminal | `console` + `colored` | Colors and formatting |
| SQLite | `rusqlite` | Disk cache, PIE, indexes |
| Full-text search | `tantivy` | BM25 ranking |
| Serialization | `serde` + `serde_json` + `simd-json` | JSON/YAML/TOML |
| Config | `config` + `dirs` | Configuration loading |
| Embeddings | `ort` | ONNX Runtime for local embeddings |
| Vector store | `hnswlib-rs` or `usearch` | HNSW vector index |
| AI (Ollama) | `reqwest` | HTTP to localhost:11434 |
| AI (llama.cpp) | `llama-cpp-rs` | Embedded GGUF inference |
| MCP server | `rmcp` | Model Context Protocol |
| REST API | `axum` | HTTP API server |
| TUI | `ratatui` | Terminal UI |
| System info | `sysinfo` | CPU/RAM/GPU detection |
| Error handling | `thiserror` + `anyhow` | Error types |
| Logging | `tracing` + `tracing-subscriber` | Structured logging |
| Testing | `cargo test` + `assert_cmd` + `insta` | Unit/E2E/snapshot tests |
| Benchmarks | `criterion` | Performance benchmarks |
| Fuzzing | `cargo-fuzz` | Fuzz testing |

---

## Cargo Workspace Structure

```
hypersearchx/
├── Cargo.toml                    # Workspace root
├── rust-toolchain.toml           # Pin Rust version
├── .github/workflows/            # CI/CD
│   ├── ci.yml
│   └── release.yml
├── crates/
│   ├── hsx-core/                 # Core library (all algorithms, search, extract, rank, etc.)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs          # Core data types (§43)
│   │       ├── error.rs          # Error types (§44)
│   │       ├── config.rs         # Configuration (§11)
│   │       ├── http/             # HTTP client
│   │       ├── search/           # Search backends + orchestrator (§15)
│   │       ├── extract/          # Content extraction + CEP + QADD (§16)
│   │       ├── rank/             # Ranking (HyperFusion, BM25, semantic) (§21)
│   │       ├── token/            # QATBE + SCS + PDS + budget (§17-18, §27)
│   │       ├── validate/         # 6-layer validation + RAR + ACS (§19, §35)
│   │       ├── citation/         # Citations + EGP (§24)
│   │       ├── research/         # Research pipeline + AMRS + SRP (§10)
│   │       ├── ai/               # Ollama + llama.cpp + routing (§23)
│   │       ├── intelligence/     # PIE + ToTR + CRP + EDF + SGT + CCE (§31-39)
│   │       ├── cache/            # Memory LRU + SQLite disk (§28)
│   │       ├── index/            # Tantivy BM25 + HNSW vectors (§28)
│   │       ├── resource/         # System profiling + monitoring (§13)
│   │       ├── output/           # Formatters (md, json, csv, etc.) (§26)
│   │       ├── plugin/           # Plugin system (§29)
│   │       ├── privacy/          # Privacy modes (§36)
│   │       ├── collab/           # Collaborative research (§37)
│   │       └── domain/           # Domain-specific modes (§38)
│   ├── hsx-cli/                  # CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── cli.rs            # clap derive definitions
│   │       ├── commands/         # One file per command
│   │       ├── output.rs         # Terminal formatting
│   │       └── tui/              # Interactive TUI (ratatui)
│   ├── hsx-mcp/                  # MCP server (§30)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tools.rs
│   │       └── handlers.rs
│   └── hsx-api/                  # REST API server (§9)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── routes.rs
│           ├── handlers.rs
│           └── middleware.rs
├── npm/                          # npm wrapper package
│   ├── package.json
│   ├── scripts/install-binary.js
│   └── bin/
├── adapters/                     # Framework adapters
│   ├── langchain/                # Python LangChain adapter
│   └── crewai/                   # Python CrewAI adapter
├── tests/
│   ├── fixtures/                 # HTML test fixtures
│   ├── integration/              # Integration tests
│   └── e2e/                      # End-to-end CLI tests
├── benches/                      # Criterion benchmarks
├── docs/                         # Documentation
├── tasks/                        # Task plan detail files
├── prd.md                        # Product Requirements Document
└── README.md
```

---

*For detailed implementation guidance, open the phase file for your assigned work.*
*Each phase file contains: step-by-step instructions, code patterns, Rust snippets, crate usage examples, and exact acceptance criteria.*
