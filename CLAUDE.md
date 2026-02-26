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

---

## Version Control & Release Pipeline

### CRITICAL — Read before making any changes

This project uses **fully automated semantic versioning** via [release-please](https://github.com/googleapis/release-please).

**All version bumping, tagging, changelog generation, and publishing is automatic.**

### Conventional Commits — REQUIRED

Every commit to `main` MUST follow the [Conventional Commits](https://www.conventionalcommits.org/) format. This is enforced by a PR title lint check in CI and a local git hook.

```
<type>(<scope>): <description>

[optional body]

[optional footer: BREAKING CHANGE: ...]
```

**Types and their version impact:**

| Type | Version bump | Example |
|------|-------------|---------|
| `feat` | **minor** (1.0.0 → 1.1.0) | `feat: add cross-lingual query expansion` |
| `fix` | **patch** (1.0.0 → 1.0.1) | `fix: handle empty query gracefully` |
| `feat!` or `BREAKING CHANGE:` | **MAJOR** (1.0.0 → 2.0.0) | `feat!: redesign config file format` |
| `perf` | **patch** | `perf: cache BM25 term frequencies` |
| `docs` | no release | `docs: update rate limit table` |
| `refactor` | no release | `refactor: extract snippet logic` |
| `chore` | no release | `chore: update dependencies` |
| `test` | no release | `test: add fuzzing for URL parser` |
| `ci` | no release | `ci: fix Windows build step` |

### Rules for AI coding agents

1. **NEVER manually edit the `version` field** in `Cargo.toml` — release-please does this automatically.
2. **NEVER manually create git tags** — release-please creates them when the Release PR is merged.
3. **NEVER run `npm publish` manually** — the release workflow does this automatically.
4. **ALWAYS write commit messages in Conventional Commits format** — this is the only way to trigger version bumps.
5. When adding a **new public-facing feature**, use `feat:` — this ensures a minor version bump.
6. When **fixing a bug**, use `fix:` — this ensures a patch version bump.
7. When making a **breaking API change**, use `feat!:` or add `BREAKING CHANGE:` in the footer.
8. The `chore:`, `refactor:`, `docs:`, `test:`, `ci:` types do NOT trigger a release — use them for non-user-facing changes.

### How the pipeline works

```
You commit feat: or fix: → push to main
        ↓
release-please opens/updates a "Release PR"
(title: "chore(main): release 1.2.0")
        ↓
Team merges the Release PR
        ↓
release-please creates:
  - git tag v1.2.0
  - GitHub Release with changelog
        ↓
release.yml workflow fires:
  ├─ Build: Linux x64/arm64, macOS x64/arm64, Windows x64
  ├─ Attach .tar.gz/.zip + SHA256 to GitHub Release
  ├─ Publish npm package (hypersearchx @ 1.2.0)
  ├─ Update Homebrew formula (zuhabul/homebrew-hsx)
  └─ Summary posted to GitHub Actions
```

### Setting up locally (one-time, for humans)

```bash
sh scripts/setup-dev.sh   # installs commit-msg and pre-commit git hooks
```

### Distribution channels (all automated)

| Channel | Install command | Updated automatically |
|---------|----------------|----------------------|
| GitHub Releases | Direct download | ✅ On every release |
| Shell installer | `curl -sSf https://install.hypersearchx.zuhabul.com \| sh` | ✅ Points to latest |
| npm | `npm install -g hypersearchx` | ✅ Via npm publish |
| npx | `npx hypersearchx` | ✅ Via npm publish |
| Homebrew | `brew install zuhabul/tap/hsx` | ✅ Via tap PR |
| cargo-binstall | `cargo binstall hsx` | ✅ Metadata in Cargo.toml |

### Required GitHub Secrets

These must be set in the repository Settings → Secrets → Actions:

| Secret | Purpose |
|--------|---------|
| `NPM_TOKEN` | Publish to npmjs.com — generate at npmjs.com → Access Tokens |
| `HOMEBREW_TAP_TOKEN` | Push to `zuhabul/homebrew-hsx` repo — GitHub PAT with `repo` scope |
| `CARGO_REGISTRY_TOKEN` | Publish to crates.io (optional) — generate at crates.io |

### One-time setup checklist

- [ ] Create GitHub repository `zuhabul/homebrew-hsx` with a `Formula/` directory
- [ ] Add `NPM_TOKEN` secret (npmjs.com → Access Tokens → Granular token for `hypersearchx`)
- [ ] Add `HOMEBREW_TAP_TOKEN` secret (GitHub PAT with `repo` scope on `zuhabul/homebrew-hsx`)
- [ ] Enable GitHub Pages for rustdoc (repo Settings → Pages → Source: GitHub Actions)
- [ ] First release: merge a `Release PR` created by release-please, or push a `v1.0.0` tag manually
