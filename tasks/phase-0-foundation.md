# Phase 0: Project Foundation & Scaffolding

> **Phase:** 0 of 8 | **Priority:** P0 (Critical Path) | **Duration:** Week 1
> **Depends on:** Nothing -- this is the root of the dependency graph
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 11 (CLI Interface), 12 (System Architecture), 13 (Resource Awareness), 43 (Data Model), 44 (Error Handling)
> **Epics:** 3 | **Tasks:** 8

---

## Phase 0 Summary

Phase 0 establishes the entire foundation that every subsequent phase builds upon. It delivers:

1. **Cargo Workspace** -- Multi-crate workspace with shared dependencies, feature flags, and release profiles (PRD SS12)
2. **Core Types** -- All foundational data structures from PRD SS43, fully serializable, with tests
3. **Error System** -- Structured error taxonomy with retry semantics and fallback hints (PRD SS44)
4. **Configuration** -- Layered config loading: defaults -> TOML file -> env vars -> CLI args (PRD SS11)
5. **CI/CD** -- GitHub Actions for build/test/lint on every PR, plus release workflow for multi-platform binaries
6. **npm Wrapper** -- `npm install -g fetchium` installs the native Rust binary via postinstall script (PRD SS12)
7. **CLI Skeleton** -- Full clap derive CLI with all commands stubbed, dispatching to per-command handlers
8. **Doctor Command** -- System health check showing resource tier, available tools, and configuration (PRD SS13)

---

## What Already Exists (Current State Audit)

Before implementing, note what is **already scaffolded** in the repository:

| Component | Status | Notes |
|-----------|--------|-------|
| `Cargo.toml` (workspace root) | **DONE** | 4 crates, workspace deps, release profile configured |
| `rust-toolchain.toml` | **DONE** | Pinned to stable with rustfmt + clippy |
| `crates/hsx-core/Cargo.toml` | **DONE** | Features (headless, embeddings, etc.), all deps declared |
| `crates/hsx-cli/Cargo.toml` | **DONE** | Binary `hsx`, depends on hsx-core |
| `crates/hsx-mcp/Cargo.toml` | **DONE** | Stub package |
| `crates/hsx-api/Cargo.toml` | **DONE** | Stub package |
| `crates/hsx-core/src/lib.rs` | **DONE** | All modules declared, prelude re-exports |
| `crates/hsx-core/src/types.rs` | **DONE** | Full types: `AgentSearchResult`, `SearchMeta`, `Segment`, `SegmentType`, `Finding`, `EvidenceLink`, `Contradiction`, `Source`, `FetchMethod`, `Citation`, `CitationStyle`, `EvidenceGraph`, `AuditEntry`, `SearchMode`, `PdsTier`, `ResourceTier`, `BackendId`, `OutputFormat`, `CepLayer` -- all with serde, tests |
| `crates/hsx-core/src/error.rs` | **DONE** | `ErrorKind` (19 variants), `StructuredError`, `HsxError` (thiserror), `HsxResult`, `is_retryable()`, `to_structured()`, tests |
| `crates/hsx-core/src/config.rs` | **DONE** | `HsxConfig` with `GeneralConfig`, `SearchConfig`, `FetchConfig`, `CacheConfig`, `AiConfig`, `OutputConfig`, all defaults, `load()`, `load_from()`, `data_dir()`, `detect_resource_tier()`, tests |
| `crates/hsx-core/src/http/client.rs` | **DONE** | `HttpClient` with pooled reqwest, `fetch_text()`, test |
| `crates/hsx-core/src/search/mod.rs` | **DONE** | `SearchBackend` trait |
| `crates/hsx-core/src/extract/mod.rs` | **DONE** | `ExtractedContent`, `ContentMetadata` structs |
| `crates/hsx-core/src/resource/mod.rs` | **DONE** | `detect_tier()` wrapper |
| Other `mod.rs` stubs | **DONE** | `cache`, `index`, `output`, `rank`, `token`, `validate`, `citation`, `research`, `ai`, `intelligence`, `plugin`, `privacy`, `collab`, `domain` -- all declared as empty or minimal |
| `crates/hsx-cli/src/main.rs` | **DONE** | `tokio::main`, tracing init, config loading, full command dispatch |
| `crates/hsx-cli/src/cli.rs` | **DONE** | Full clap derive CLI: `Cli`, `Commands` (12 subcommands), all arg structs, value enums |
| `crates/hsx-cli/src/output.rs` | **DONE** | `header()`, `error()`, `warning()`, `info()`, `success()` helpers |
| `crates/hsx-cli/src/commands/*.rs` | **DONE** | All 12 command files exist as stubs (search, fetch, research, ai, deep, agent_search, agent_fetch, agent_research, doctor, config, cache, serve) |
| `crates/hsx-cli/src/commands/doctor.rs` | **DONE** | Full doctor: resource tier, data dir, Chromium detection, Ollama check |
| `.github/workflows/` | **MISSING** | No CI/CD yet |
| `npm/` | **MISSING** | No npm wrapper package yet |

**Conclusion:** The core Rust scaffolding (Epic 1 and Epic 3) is substantially complete. The remaining work focuses on (a) hardening and extending what exists, (b) CI/CD and release infrastructure (Epic 2), and (c) filling gaps identified below.

---

## Prerequisites

None. Phase 0 is the root of the dependency graph.

---

## Epic 0.1: Workspace, Core Types & Configuration

> **PRD Sections:** SS11, SS12, SS13, SS43, SS44
> **Crate:** `hsx-core` -- `src/types.rs`, `src/error.rs`, `src/config.rs`
> **Priority:** P0 | **Tasks:** 3

### P0-E1-T1: Cargo Workspace Setup

**ID:** `P0-E1-T1`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 0.5 day (already complete)

**Description:**
Establish the Cargo workspace with four crates (`hsx-core`, `hsx-cli`, `hsx-mcp`, `hsx-api`), workspace-level shared dependencies, release profile, and `rust-toolchain.toml`.

**What already exists:**
- `/Cargo.toml` -- workspace root with all 4 members, 40+ workspace dependencies, release LTO profile
- `/rust-toolchain.toml` -- stable channel, rustfmt + clippy components
- `/crates/hsx-core/Cargo.toml` -- feature flags (`headless`, `embeddings`, `vector-search`, `mcp`, `llama`), all dependencies
- `/crates/hsx-cli/Cargo.toml` -- binary `hsx`, clap + indicatif + console + colored
- `/crates/hsx-mcp/Cargo.toml` -- MCP stub
- `/crates/hsx-api/Cargo.toml` -- API stub

**What still needs verification:**

1. Ensure `cargo build --workspace` compiles without errors
2. Ensure `cargo clippy --workspace -- -D warnings` passes
3. Ensure `cargo test --workspace` passes

**Files:**
```
Cargo.toml                      # EXISTING -- workspace root
rust-toolchain.toml             # EXISTING -- toolchain pinning
crates/hsx-core/Cargo.toml     # EXISTING -- core library
crates/hsx-cli/Cargo.toml      # EXISTING -- CLI binary
crates/hsx-mcp/Cargo.toml      # EXISTING -- MCP server stub
crates/hsx-api/Cargo.toml      # EXISTING -- REST API stub
```

**Step-by-step verification:**

**Step 1: Verify workspace compiles**

```bash
cargo build --workspace 2>&1
```

If there are compilation errors, fix them by adjusting dependency versions or stub implementations. Common issues include optional dependencies that need feature-gating in code.

**Step 2: Verify clippy passes**

```bash
cargo clippy --workspace -- -D warnings 2>&1
```

**Step 3: Verify tests pass**

```bash
cargo test --workspace 2>&1
```

**Step 4 (if needed): Add missing directory scaffolding**

The following directories should exist per PRD SS12 workspace layout:

```
tests/
  fixtures/                  # HTML test fixtures (Phase 1+)
  integration/               # Integration tests (Phase 1+)
  e2e/                       # End-to-end CLI tests (Phase 1+)
benches/                     # Criterion benchmarks (Phase 8)
docs/                        # Documentation (Phase 8)
```

Create them as empty directories with `.gitkeep` if not present:

```bash
mkdir -p tests/fixtures tests/integration tests/e2e benches docs
touch tests/fixtures/.gitkeep tests/integration/.gitkeep tests/e2e/.gitkeep benches/.gitkeep docs/.gitkeep
```

**Acceptance criteria:**

- [ ] `cargo build --workspace` succeeds with zero errors
- [ ] `cargo clippy --workspace -- -D warnings` produces zero warnings
- [ ] `cargo test --workspace` passes all tests
- [ ] Workspace has 4 crate members: `hsx-core`, `hsx-cli`, `hsx-mcp`, `hsx-api`
- [ ] Release profile has `lto = "thin"`, `codegen-units = 1`, `strip = true`
- [ ] `rust-toolchain.toml` pins stable with rustfmt + clippy
- [ ] Directory structure matches PRD SS12 layout

---

### P0-E1-T2: Core Types

**ID:** `P0-E1-T2`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 0 days (already complete, needs review only)
**Dependencies:** `P0-E1-T1`

**Description:**
Define all foundational data types from PRD SS43 in `crates/hsx-core/src/types.rs`. These types flow through the entire pipeline: search -> extract -> rank -> validate -> output.

**What already exists:**
The file `crates/hsx-core/src/types.rs` is **fully implemented** with all types from PRD SS43:

| Type | Status | PRD Ref |
|------|--------|---------|
| `AgentSearchResult` | DONE | SS43 top-level agent output |
| `SearchResult` | DONE | Human-facing output |
| `SearchMeta` | DONE | Metadata with typed enums (not strings) |
| `ResultItem` | DONE | Search result with backend ID |
| `Segment` + `SegmentType` (14 variants) | DONE | SS18 SCS |
| `Finding` | DONE | Research findings |
| `EvidenceLink` + `EvidenceType` | DONE | SS24 EGP |
| `Contradiction` + `ContradictionSeverity` | DONE | Contradiction detection |
| `Source` + `FetchMethod` | DONE | Source tracking |
| `Citation` + `CitationStyle` (6 styles) | DONE | SS24 Citations |
| `EvidenceGraph`, `EvidenceNode`, `EvidenceEdge`, `EvidenceNodeType` | DONE | SS24 EGP |
| `AuditEntry` | DONE | Operation audit trail |
| `SearchMode` (7 variants) | DONE | SS10 modes |
| `PdsTier` (4 tiers) | DONE | SS27 progressive disclosure |
| `ResourceTier` (4 tiers) | DONE | SS13 resource awareness |
| `BackendId` (12 backends + Custom) | DONE | SS15 search backends |
| `OutputFormat` (7 formats) | DONE | SS26 output |
| `CepLayer` (5 layers, ordered) | DONE | SS16 extraction cascade |
| Display impl for `BackendId` | DONE | -- |
| Tests (4 tests) | DONE | Roundtrip, display, serialization, ordering |

**What could be improved (optional, non-blocking):**

The following enhancements are NOT required for Phase 0 completion but may be useful in later phases. Document them here for future reference:

1. **`ResourceTier` naming:** PRD SS13 uses 5 tiers (Minimal/Light/Standard/Power/Ultra) but the code uses 4 (Minimal/Standard/Performance/Server). Consider aligning in a later phase when resource monitoring is fully built (Phase 1 P1-E3 or Phase 5).

2. **`SearchMode` missing `Index` variant:** PRD SS10 Mode H (Index Mode) is not yet in `SearchMode`. Add when implementing the index subcommand in Phase 7.

3. **`OutputFormat` missing `Pdf` and `Docx`:** These are Phase 5 features (P5-E6). No action needed now.

**Step-by-step verification:**

**Step 1: Review type coverage**

Read through `crates/hsx-core/src/types.rs` and cross-reference against PRD SS43 to ensure all entities exist.

**Step 2: Run existing tests**

```bash
cargo test -p hsx-core types::tests 2>&1
```

Expected: 4 tests pass (segment_type_roundtrip, backend_id_display, search_meta_serialization, cep_layer_ordering).

**Step 3: Verify serde roundtrip for all major types**

If you want to add additional coverage, add these tests to `crates/hsx-core/src/types.rs`:

```rust
#[test]
fn agent_search_result_roundtrip() {
    let result = AgentSearchResult {
        meta: SearchMeta {
            query: "test query".into(),
            mode: SearchMode::Search,
            tier: PdsTier::Summary,
            tokens_used: 500,
            tokens_budget: 4000,
            sources_fetched: 3,
            sources_validated: 3,
            validation_pass_rate: 1.0,
            duration_ms: 2100,
            resource_tier: ResourceTier::Standard,
            timestamp: "2026-02-23T12:00:00Z".into(),
            result_id: "r-001".into(),
            content_hashes: HashMap::new(),
        },
        segments: vec![],
        findings: vec![],
        evidence: vec![],
        contradictions: vec![],
        sources: vec![],
        evidence_graph: None,
        audit_trail: vec![],
    };
    let json = serde_json::to_string_pretty(&result).unwrap();
    let back: AgentSearchResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.meta.query, "test query");
    assert_eq!(back.meta.tokens_budget, 4000);
}

#[test]
fn evidence_type_all_variants() {
    for variant in [EvidenceType::Supports, EvidenceType::Contradicts, EvidenceType::PartiallySupports] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: EvidenceType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn output_format_default_is_markdown() {
    assert_eq!(OutputFormat::default(), OutputFormat::Markdown);
}

#[test]
fn pds_tier_default_is_summary() {
    assert_eq!(PdsTier::default(), PdsTier::Summary);
}
```

**Acceptance criteria:**

- [ ] All PRD SS43 entities have corresponding Rust types in `types.rs`
- [ ] All types derive `Debug, Clone, Serialize, Deserialize`
- [ ] All enums use `#[serde(rename_all = "snake_case")]` for JSON compatibility
- [ ] `BackendId` implements `Display`
- [ ] `CepLayer` implements `PartialOrd, Ord` for comparison
- [ ] All types have appropriate `#[serde(skip_serializing_if)]` annotations
- [ ] `SearchMode`, `PdsTier`, `ResourceTier`, `OutputFormat` have `Default` impls
- [ ] At least 4 unit tests pass covering serialization roundtrips

---

### P0-E1-T3: Configuration System

**ID:** `P0-E1-T3`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 0 days (already complete, needs review and minor hardening)
**Dependencies:** `P0-E1-T2`

**Description:**
Build the layered configuration system per PRD SS11: defaults -> `~/.fetchium/config.toml` -> environment variables -> CLI args. The config file covers search, fetch, cache, AI, and output settings.

**What already exists:**
The file `crates/hsx-core/src/config.rs` is **fully implemented** with:

| Component | Status |
|-----------|--------|
| `HsxConfig` (top-level) | DONE |
| `GeneralConfig` (max_results, format, verbose, data_dir) | DONE |
| `SearchConfig` (backends, budget, tier, concurrency, timeout, searxng_url) | DONE |
| `FetchConfig` (user_agent, robots, page size, timeout, redirects) | DONE |
| `CacheConfig` (enabled, memory entries, disk MB, TTL) | DONE |
| `AiConfig` (ollama host, model, max tokens) | DONE |
| `OutputConfig` (format, include_sources, include_confidence) | DONE |
| All `Default` impls | DONE |
| `load()` / `load_from()` | DONE |
| `data_dir()` | DONE |
| `detect_resource_tier()` | DONE |
| Tests (3 tests) | DONE |

**What could be improved (recommended hardening for Phase 0):**

The following additions make the config system more robust. These are recommended but the existing implementation is functional.

**Step-by-step hardening:**

**Step 1: Add environment variable override support**

The PRD SS11 specifies a layered config: defaults -> file -> **env vars** -> CLI args. Currently env var support is missing. Add it to `crates/hsx-core/src/config.rs`:

```rust
impl HsxConfig {
    /// Apply environment variable overrides.
    /// Convention: `HSX_SECTION_KEY` (uppercase, underscore-separated).
    /// Examples: HSX_SEARCH_DEFAULT_BUDGET=8000, HSX_CACHE_ENABLED=false
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("HSX_SEARCH_DEFAULT_BUDGET") {
            if let Ok(budget) = val.parse::<u32>() {
                self.search.default_budget = budget;
            }
        }
        if let Ok(val) = std::env::var("HSX_SEARCH_MAX_CONCURRENT") {
            if let Ok(n) = val.parse::<u32>() {
                self.search.max_concurrent = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_SEARCH_TIMEOUT_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.search.timeout_secs = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_CACHE_ENABLED") {
            if let Ok(b) = val.parse::<bool>() {
                self.cache.enabled = b;
            }
        }
        if let Ok(val) = std::env::var("HSX_CACHE_TTL_SECS") {
            if let Ok(n) = val.parse::<u64>() {
                self.cache.ttl_secs = n;
            }
        }
        if let Ok(val) = std::env::var("HSX_AI_OLLAMA_HOST") {
            self.ai.ollama_host = val;
        }
        if let Ok(val) = std::env::var("HSX_AI_DEFAULT_MODEL") {
            self.ai.default_model = val;
        }
        if let Ok(val) = std::env::var("HSX_GENERAL_VERBOSE") {
            if let Ok(b) = val.parse::<bool>() {
                self.general.verbose = b;
            }
        }
        if let Ok(val) = std::env::var("HSX_FETCH_USER_AGENT") {
            self.fetch.user_agent = val;
        }
        if let Ok(val) = std::env::var("HSX_FETCH_RESPECT_ROBOTS") {
            if let Ok(b) = val.parse::<bool>() {
                self.fetch.respect_robots = b;
            }
        }
    }
}
```

**Step 2: Integrate env overrides into `load()` and `load_from()`**

Modify the existing `load_from` method in `crates/hsx-core/src/config.rs`:

```rust
pub fn load_from(path: Option<&std::path::Path>) -> Self {
    let config_path = path.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".fetchium")
            .join("config.toml")
    });

    let mut config = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    tracing::warn!("Failed to parse config at {}: {e}", config_path.display());
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("Failed to read config at {}: {e}", config_path.display());
                Self::default()
            }
        }
    } else {
        Self::default()
    };

    // Layer 3: environment variable overrides
    config.apply_env_overrides();

    config
}
```

**Step 3: Add `ensure_data_dir()` method**

```rust
impl HsxConfig {
    /// Ensure the data directory exists, creating it if necessary.
    pub fn ensure_data_dir(&self) -> std::io::Result<PathBuf> {
        let dir = self.data_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Get the path to the config file.
    pub fn config_file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".fetchium")
            .join("config.toml")
    }

    /// Write the current config to the config file (for `hsx config set`).
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, toml_str)
    }
}
```

**Step 4: Add tests for env overrides and save/load**

```rust
#[test]
fn env_override_budget() {
    std::env::set_var("HSX_SEARCH_DEFAULT_BUDGET", "8000");
    let mut config = HsxConfig::default();
    config.apply_env_overrides();
    assert_eq!(config.search.default_budget, 8000);
    std::env::remove_var("HSX_SEARCH_DEFAULT_BUDGET");
}

#[test]
fn env_override_cache_disabled() {
    std::env::set_var("HSX_CACHE_ENABLED", "false");
    let mut config = HsxConfig::default();
    config.apply_env_overrides();
    assert!(!config.cache.enabled);
    std::env::remove_var("HSX_CACHE_ENABLED");
}

#[test]
fn save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");

    let mut config = HsxConfig::default();
    config.search.default_budget = 9999;

    // Save
    let toml_str = toml::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, &toml_str).unwrap();

    // Reload
    let loaded = HsxConfig::load_from(Some(&config_path));
    assert_eq!(loaded.search.default_budget, 9999);
}

#[test]
fn config_file_path_contains_fetchium() {
    let path = HsxConfig::config_file_path();
    assert!(path.to_string_lossy().contains(".fetchium"));
    assert!(path.to_string_lossy().ends_with("config.toml"));
}
```

**Files modified:**
```
crates/hsx-core/src/config.rs   # Add env overrides, ensure_data_dir, save, config_file_path
```

**Acceptance criteria:**

- [ ] Config loads from `~/.fetchium/config.toml` when file exists
- [ ] Config falls back to defaults when file is missing
- [ ] Config applies environment variable overrides (HSX_SEARCH_DEFAULT_BUDGET, etc.)
- [ ] `data_dir()` returns `~/.fetchium` by default
- [ ] `ensure_data_dir()` creates the directory if missing
- [ ] `save()` writes current config to TOML file
- [ ] `detect_resource_tier()` returns appropriate tier based on system RAM/CPU
- [ ] TOML roundtrip: serialize -> deserialize preserves all fields
- [ ] At least 7 tests pass (3 existing + 4 new)

**Testing:**

```bash
cargo test -p hsx-core config::tests
```

---

## Epic 0.2: CI/CD & npm Wrapper Package

> **PRD Sections:** SS12 (npm/pnpm/bun Compatibility), SS45 (Testing Strategy)
> **Files:** `.github/workflows/`, `npm/`
> **Priority:** P0 | **Tasks:** 3

### P0-E2-T1: GitHub Actions CI

**ID:** `P0-E2-T1`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 1 day
**Dependencies:** `P0-E1-T1`

**Description:**
Create a GitHub Actions CI workflow that runs on every push and pull request. It must build the workspace, run tests, run clippy, check formatting, and verify the workspace compiles on all three target platforms (Linux, macOS, Windows).

**PRD References:**
- SS45 "Testing Strategy" -- cargo test, clippy, criterion
- SS12 "Technology Stack" -- Rust, cross-platform builds

**Files to create:**
```
.github/workflows/ci.yml       # CI workflow
```

**Step-by-step implementation:**

**Step 1: Create the CI workflow (`.github/workflows/ci.yml`)**

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  check:
    name: Check (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry & build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-

      - name: Build workspace
        run: cargo build --workspace --all-features

      - name: Run tests
        run: cargo test --workspace --all-features

  lint:
    name: Lint & Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo registry & build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-lint-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-lint-

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run clippy
        run: cargo clippy --workspace --all-features -- -D warnings

  doc:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry & build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-doc-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-doc-

      - name: Build documentation
        run: cargo doc --workspace --no-deps --all-features
        env:
          RUSTDOCFLAGS: "-D warnings"

  # Minimal feature builds to catch feature-gating issues
  minimal:
    name: Minimal Features
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry & build
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-minimal-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-minimal-

      - name: Build with default features only
        run: cargo build --workspace

      - name: Test with default features only
        run: cargo test --workspace
```

**Step 2: Create the `.github/workflows/` directory**

```bash
mkdir -p .github/workflows
```

**Step 3: Verify locally**

Before pushing, verify the CI steps pass locally:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo build --workspace
cargo test --workspace
cargo doc --workspace --no-deps
```

**Acceptance criteria:**

- [ ] `.github/workflows/ci.yml` exists and is valid YAML
- [ ] CI runs on push to `main`/`develop` and on all PRs
- [ ] CI tests on Linux, macOS, and Windows
- [ ] Clippy runs with `-D warnings` (zero warnings policy)
- [ ] Formatting check runs via `cargo fmt --check`
- [ ] Documentation builds without warnings
- [ ] Cargo caching is enabled for faster builds
- [ ] Minimal feature build is tested separately
- [ ] All local verification commands pass

**Testing:**

```bash
# Validate YAML syntax (requires yq or python)
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"

# Run CI steps locally
cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

---

### P0-E2-T2: Release Workflow

**ID:** `P0-E2-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1.5 days
**Dependencies:** `P0-E2-T1`

**Description:**
Create a GitHub Actions release workflow that builds optimized binaries for 5 target platforms, creates a GitHub Release with all artifacts, and publishes to crates.io. Triggered by git tags matching `v*`.

**PRD References:**
- SS12 "npm/pnpm/bun Compatibility" -- platform-specific binaries: `linux-x64`, `linux-arm64`, `darwin-x64`, `darwin-arm64`, `win-x64`
- SS12 "Release profile" -- LTO, strip, codegen-units=1

**Files to create:**
```
.github/workflows/release.yml   # Release workflow
```

**Step-by-step implementation:**

**Step 1: Create the release workflow (`.github/workflows/release.yml`)**

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            artifact: hsx-linux-x64
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            artifact: hsx-linux-arm64
            cross: true
          - target: x86_64-apple-darwin
            os: macos-latest
            artifact: hsx-darwin-x64
          - target: aarch64-apple-darwin
            os: macos-latest
            artifact: hsx-darwin-arm64
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            artifact: hsx-win-x64

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (for cross-compilation)
        if: matrix.cross
        run: cargo install cross --locked

      - name: Build release binary
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }} -p hsx-cli
          else
            cargo build --release --target ${{ matrix.target }} -p hsx-cli
          fi
        shell: bash

      - name: Package binary (Unix)
        if: runner.os != 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          tar czf ../../../${{ matrix.artifact }}.tar.gz hsx
          cd ../../..
          sha256sum ${{ matrix.artifact }}.tar.gz > ${{ matrix.artifact }}.tar.gz.sha256

      - name: Package binary (Windows)
        if: runner.os == 'Windows'
        run: |
          cd target/${{ matrix.target }}/release
          7z a ../../../${{ matrix.artifact }}.zip hsx.exe
          cd ../../..
          certutil -hashfile ${{ matrix.artifact }}.zip SHA256 > ${{ matrix.artifact }}.zip.sha256

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: |
            ${{ matrix.artifact }}.tar.gz
            ${{ matrix.artifact }}.tar.gz.sha256
            ${{ matrix.artifact }}.zip
            ${{ matrix.artifact }}.zip.sha256
          if-no-files-found: ignore

  release:
    name: Create GitHub Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true

      - name: Create release
        uses: softprops/action-gh-release@v2
        with:
          generate_release_notes: true
          files: |
            artifacts/*.tar.gz
            artifacts/*.zip
            artifacts/*.sha256
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-crate:
    name: Publish to crates.io
    needs: release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Publish hsx-core
        run: cargo publish -p hsx-core --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
        continue-on-error: true

      - name: Publish hsx-cli
        run: cargo publish -p hsx-cli --token ${{ secrets.CARGO_REGISTRY_TOKEN }}
        continue-on-error: true
```

**Step 2: Verify the release profile is optimal**

Check that `Cargo.toml` has the release profile (it already does):

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = true
opt-level = 3
```

**Step 3: Test a local release build**

```bash
cargo build --release -p hsx-cli
ls -lh target/release/hsx
# Should produce a small, stripped binary
```

**Acceptance criteria:**

- [ ] `.github/workflows/release.yml` exists and is valid YAML
- [ ] Release is triggered by pushing a tag matching `v*`
- [ ] Builds for 5 targets: linux-x64, linux-arm64, darwin-x64, darwin-arm64, win-x64
- [ ] Each artifact is a compressed archive (tar.gz for Unix, zip for Windows)
- [ ] SHA256 checksums are generated for each artifact
- [ ] GitHub Release is created with all artifacts attached
- [ ] Release notes are auto-generated from git history
- [ ] crates.io publishing is attempted (with `continue-on-error` for safety)
- [ ] Release binary uses the optimized profile (LTO, strip, opt-level 3)

**Testing:**

```bash
# Test local release build
cargo build --release -p hsx-cli
file target/release/hsx
# Should show: "Mach-O 64-bit executable arm64" (on Apple Silicon) or equivalent

# Test binary size
ls -lh target/release/hsx
# Should be <15MB thanks to strip + LTO

# Validate YAML
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"
```

---

### P0-E2-T3: npm Wrapper Package

**ID:** `P0-E2-T3`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1.5 days
**Dependencies:** `P0-E2-T2`

**Description:**
Create the npm wrapper package that allows `npm install -g fetchium` (or pnpm/bun). The package contains a `postinstall` script that downloads the platform-specific binary from GitHub Releases, and bin stubs (`hsx`, `hyper`) that invoke the native binary.

**PRD References:**
- SS12 "npm/pnpm/bun Compatibility": `npm install -g fetchium`, `pnpm add -g fetchium`, `bun add -g fetchium`
- SS11 "Binary Names": Primary `hsx`, Alias `hyper`

**Files to create:**
```
npm/
  package.json              # npm package manifest
  scripts/install-binary.js # postinstall script to download platform binary
  bin/hsx.js                # bin stub for 'hsx' command
  bin/hyper.js              # bin stub for 'hyper' alias
  README.md                 # npm package readme (minimal)
```

**Step-by-step implementation:**

**Step 1: Create `npm/package.json`**

```json
{
  "name": "fetchium",
  "version": "0.1.0",
  "description": "AI-native search engine for humans and agents — blazing fast, free, zero API keys",
  "keywords": [
    "search",
    "ai",
    "research",
    "web-search",
    "mcp",
    "agent",
    "rust",
    "cli"
  ],
  "license": "MIT OR Apache-2.0",
  "repository": {
    "type": "git",
    "url": "https://github.com/fetchium/fetchium"
  },
  "homepage": "https://github.com/fetchium/fetchium",
  "bin": {
    "hsx": "bin/hsx.js",
    "hyper": "bin/hyper.js"
  },
  "scripts": {
    "postinstall": "node scripts/install-binary.js"
  },
  "os": [
    "darwin",
    "linux",
    "win32"
  ],
  "cpu": [
    "x64",
    "arm64"
  ],
  "engines": {
    "node": ">=18.0.0"
  },
  "files": [
    "bin/",
    "scripts/",
    "README.md"
  ]
}
```

**Step 2: Create `npm/scripts/install-binary.js`**

```javascript
#!/usr/bin/env node

"use strict";

const { execSync } = require("child_process");
const fs = require("fs");
const https = require("https");
const os = require("os");
const path = require("path");
const { createWriteStream, mkdirSync } = require("fs");
const { pipeline } = require("stream/promises");
const zlib = require("zlib");
const { createGunzip } = require("zlib");

const REPO = "fetchium/fetchium";
const BINARY_NAME = "hsx";

/**
 * Map Node.js platform/arch to our release artifact names.
 */
function getArtifactName() {
  const platform = os.platform();
  const arch = os.arch();

  const map = {
    "darwin-x64": "hsx-darwin-x64",
    "darwin-arm64": "hsx-darwin-arm64",
    "linux-x64": "hsx-linux-x64",
    "linux-arm64": "hsx-linux-arm64",
    "win32-x64": "hsx-win-x64",
  };

  const key = `${platform}-${arch}`;
  const artifact = map[key];

  if (!artifact) {
    console.error(
      `Unsupported platform: ${platform}-${arch}. ` +
        `Supported: ${Object.keys(map).join(", ")}`
    );
    console.error(
      "You can build from source: cargo install fetchium"
    );
    process.exit(1);
  }

  return artifact;
}

/**
 * Get the version from package.json.
 */
function getVersion() {
  const pkg = JSON.parse(
    fs.readFileSync(path.join(__dirname, "..", "package.json"), "utf8")
  );
  return pkg.version;
}

/**
 * Download a file from a URL, following redirects.
 */
function download(url) {
  return new Promise((resolve, reject) => {
    https
      .get(url, { headers: { "User-Agent": "fetchium-npm" } }, (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          // Follow redirect
          return download(res.headers.location).then(resolve).catch(reject);
        }
        if (res.statusCode !== 200) {
          reject(new Error(`Download failed: HTTP ${res.statusCode} from ${url}`));
          return;
        }
        resolve(res);
      })
      .on("error", reject);
  });
}

/**
 * Extract a tar.gz archive to a destination directory.
 */
async function extractTarGz(archivePath, destDir) {
  // Use tar command (available on macOS, Linux, and Git Bash on Windows)
  try {
    execSync(`tar xzf "${archivePath}" -C "${destDir}"`, { stdio: "pipe" });
  } catch (e) {
    throw new Error(`Failed to extract archive: ${e.message}`);
  }
}

/**
 * Main installation logic.
 */
async function install() {
  const artifact = getArtifactName();
  const version = getVersion();
  const tag = `v${version}`;
  const isWindows = os.platform() === "win32";
  const ext = isWindows ? "zip" : "tar.gz";

  const downloadUrl = `https://github.com/${REPO}/releases/download/${tag}/${artifact}.${ext}`;
  const binDir = path.join(__dirname, "..", "bin");
  const tmpDir = path.join(os.tmpdir(), `hsx-install-${Date.now()}`);

  console.log(`Fetchium: Downloading ${artifact} (${tag})...`);
  console.log(`  URL: ${downloadUrl}`);

  mkdirSync(tmpDir, { recursive: true });
  mkdirSync(binDir, { recursive: true });

  const archivePath = path.join(tmpDir, `${artifact}.${ext}`);

  try {
    // Download the archive
    const res = await download(downloadUrl);
    const fileStream = createWriteStream(archivePath);
    await pipeline(res, fileStream);

    // Extract
    if (isWindows) {
      execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}'"`, {
        stdio: "pipe",
      });
    } else {
      await extractTarGz(archivePath, tmpDir);
    }

    // Copy binary to bin/
    const binaryExt = isWindows ? ".exe" : "";
    const srcBinary = path.join(tmpDir, `${BINARY_NAME}${binaryExt}`);
    const destBinary = path.join(binDir, `${BINARY_NAME}${binaryExt}`);

    if (!fs.existsSync(srcBinary)) {
      throw new Error(`Binary not found in archive at: ${srcBinary}`);
    }

    fs.copyFileSync(srcBinary, destBinary);
    if (!isWindows) {
      fs.chmodSync(destBinary, 0o755);
    }

    console.log(`Fetchium: Installed ${BINARY_NAME} to ${destBinary}`);
  } catch (err) {
    console.error(`Fetchium: Failed to install binary.`);
    console.error(`  Error: ${err.message}`);
    console.error(``);
    console.error(`  You can install manually:`);
    console.error(`    cargo install fetchium`);
    console.error(`  Or download from:`);
    console.error(`    https://github.com/${REPO}/releases`);

    // Don't fail the npm install -- allow graceful degradation
    // The bin stubs will show a helpful error if the binary is missing
    process.exit(0);
  } finally {
    // Clean up temp directory
    try {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    } catch (_) {
      // ignore cleanup errors
    }
  }
}

install();
```

**Step 3: Create `npm/bin/hsx.js`**

```javascript
#!/usr/bin/env node

"use strict";

const { execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const binaryExt = os.platform() === "win32" ? ".exe" : "";
const binaryPath = path.join(__dirname, `hsx${binaryExt}`);

if (!fs.existsSync(binaryPath)) {
  console.error("Error: Fetchium binary not found.");
  console.error("");
  console.error("The native binary was not installed during `npm install`.");
  console.error("Try reinstalling:");
  console.error("  npm install -g fetchium");
  console.error("");
  console.error("Or install directly via Cargo:");
  console.error("  cargo install fetchium");
  console.error("");
  console.error("Or download from GitHub Releases:");
  console.error("  https://github.com/fetchium/fetchium/releases");
  process.exit(1);
}

// Forward all arguments to the native binary
try {
  const result = execFileSync(binaryPath, process.argv.slice(2), {
    stdio: "inherit",
    env: process.env,
  });
} catch (err) {
  // execFileSync throws on non-zero exit codes
  // The child process has already printed its output
  process.exit(err.status || 1);
}
```

**Step 4: Create `npm/bin/hyper.js`**

```javascript
#!/usr/bin/env node

"use strict";

// 'hyper' is an alias for 'hsx' per PRD SS11
require("./hsx.js");
```

**Step 5: Create minimal `npm/README.md`**

```markdown
# Fetchium

AI-native search engine for humans and agents.

## Install

```bash
npm install -g fetchium
```

## Usage

```bash
hsx search "your query"
hsx fetch https://example.com
hsx doctor
```

See [GitHub](https://github.com/fetchium/fetchium) for full documentation.
```

**Step 6: Test the npm package structure locally**

```bash
cd npm
node -e "const pkg = require('./package.json'); console.log(pkg.name, pkg.version, pkg.bin);"
# Should output: fetchium 0.1.0 { hsx: 'bin/hsx.js', hyper: 'bin/hyper.js' }

# Verify postinstall script syntax
node -c scripts/install-binary.js
# Should output nothing (no syntax errors)

# Verify bin stubs syntax
node -c bin/hsx.js
node -c bin/hyper.js
```

**Acceptance criteria:**

- [ ] `npm/package.json` exists with name `fetchium`, correct version, bin stubs
- [ ] `npm/scripts/install-binary.js` downloads platform-specific binary from GitHub Releases
- [ ] Supports 5 platforms: darwin-x64, darwin-arm64, linux-x64, linux-arm64, win32-x64
- [ ] `npm/bin/hsx.js` forwards all args to the native binary
- [ ] `npm/bin/hyper.js` is an alias for `hsx.js`
- [ ] Graceful error message if binary download fails (suggests `cargo install`)
- [ ] Graceful error message if binary is not found at runtime
- [ ] `postinstall` script does not fail npm install on download errors (exits 0)
- [ ] All JS files pass `node -c` syntax check
- [ ] Package specifies `engines.node >= 18.0.0`

**Testing:**

```bash
# Verify package structure
cd npm && npm pack --dry-run
# Should list: package.json, scripts/install-binary.js, bin/hsx.js, bin/hyper.js, README.md

# Syntax checks
node -c npm/scripts/install-binary.js
node -c npm/bin/hsx.js
node -c npm/bin/hyper.js
```

---

## Epic 0.3: CLI Skeleton

> **PRD Sections:** SS10 (Modes of Operation), SS11 (CLI Interface Design), SS13 (Resource Awareness)
> **Crate:** `hsx-cli` -- `src/main.rs`, `src/cli.rs`, `src/commands/`
> **Priority:** P0 | **Tasks:** 2

### P0-E3-T1: clap Derive CLI

**ID:** `P0-E3-T1`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 0 days (already complete, needs review and minor additions)
**Dependencies:** `P0-E1-T2`

**Description:**
Build the full CLI skeleton using clap derive with all commands from PRD SS11, global flags, per-command args, and value enums. Every command is stubbed to print a placeholder message, ready for Phase 1+ implementation.

**What already exists:**
The CLI is **fully implemented** in `crates/hsx-cli/src/`:

| File | Status | Contents |
|------|--------|----------|
| `main.rs` | DONE | tokio::main, tracing init, config load, full 12-command dispatch |
| `cli.rs` | DONE | `Cli` struct, `Commands` enum (12 variants), all arg structs, value enums (Format, Tier, CitationStyle, ServerMode) |
| `output.rs` | DONE | 5 terminal formatting helpers (header, error, warning, info, success) |
| `commands/mod.rs` | DONE | All 12 command modules declared |
| `commands/search.rs` | DONE | Stub: prints query |
| `commands/fetch.rs` | DONE | Stub |
| `commands/research.rs` | DONE | Stub |
| `commands/ai.rs` | DONE | Stub |
| `commands/deep.rs` | DONE | Stub |
| `commands/agent_search.rs` | DONE | Stub |
| `commands/agent_fetch.rs` | DONE | Stub |
| `commands/agent_research.rs` | DONE | Stub |
| `commands/doctor.rs` | DONE | Full implementation (resource tier, data dir, Chromium, Ollama) |
| `commands/config.rs` | DONE | Stub |
| `commands/cache.rs` | DONE | Stub |
| `commands/serve.rs` | DONE | Stub |

**What could be improved (recommended hardening):**

**Step 1: Add `--no-cache` and `--quiet` global flags**

PRD SS11 specifies these global flags. Add them to `crates/hsx-cli/src/cli.rs`:

```rust
/// Fetchium -- AI-native search engine for humans and agents.
#[derive(Debug, Parser)]
#[command(name = "hsx", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format
    #[arg(short, long, global = true, default_value = "markdown")]
    pub format: Format,

    /// Verbose output (show debug logs)
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Quiet mode (suppress all non-essential output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Bypass cache for this request
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Config file path override
    #[arg(long, global = true)]
    pub config: Option<String>,
}
```

**Step 2: Add `--output` flag to SearchArgs and FetchArgs**

PRD SS11 specifies `--output <file>` for writing results to a file:

```rust
#[derive(Debug, Parser)]
pub struct SearchArgs {
    /// Search query
    pub query: String,

    /// Maximum number of results
    #[arg(short = 'n', long, default_value = "10")]
    pub max_results: u32,

    /// Search backends to use
    #[arg(short, long)]
    pub backends: Vec<String>,

    /// Write output to a file
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Debug, Parser)]
pub struct FetchArgs {
    /// URL to fetch
    pub url: String,

    /// Token budget
    #[arg(short, long, default_value = "4000")]
    pub budget: u32,

    /// PDS tier
    #[arg(short, long, default_value = "summary")]
    pub tier: Tier,

    /// Extract with query context (QATBE)
    #[arg(short, long)]
    pub query: Option<String>,

    /// Write output to a file
    #[arg(short, long)]
    pub output: Option<String>,
}
```

**Step 3: Update `main.rs` to pass `--quiet` and `--no-cache` to config**

In `crates/hsx-cli/src/main.rs`, after loading config, apply CLI overrides:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose {
        "hsx=debug,hsx_core=debug"
    } else if cli.quiet {
        "hsx=error,hsx_core=error"
    } else {
        "hsx=info,hsx_core=warn"
    };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with_target(false)
        .init();

    // Load config (file -> env -> CLI overrides)
    let mut config = match &cli.config {
        Some(path) => hsx_core::config::HsxConfig::load_from(Some(std::path::Path::new(path))),
        None => hsx_core::config::HsxConfig::load(),
    };

    // CLI flag overrides
    if cli.no_cache {
        config.cache.enabled = false;
    }
    if cli.verbose {
        config.general.verbose = true;
    }

    // Dispatch command
    match cli.command {
        Commands::Search(args) => commands::search::run(args, &config).await,
        Commands::Fetch(args) | Commands::View(args) => {
            commands::fetch::run(args, &config).await
        }
        Commands::Research(args) => commands::research::run(args, &config).await,
        Commands::Ai(args) => commands::ai::run(args, &config).await,
        Commands::Deep(args) => commands::deep::run(args, &config).await,
        Commands::AgentSearch(args) => commands::agent_search::run(args, &config).await,
        Commands::AgentFetch(args) => commands::agent_fetch::run(args, &config).await,
        Commands::AgentResearch(args) => commands::agent_research::run(args, &config).await,
        Commands::Doctor => commands::doctor::run(&config).await,
        Commands::Config(args) => commands::config::run(args, &config).await,
        Commands::Cache(args) => commands::cache::run(args, &config).await,
        Commands::Serve(args) => commands::serve::run(args, &config).await,
    }
}
```

**Step 4: Add basic E2E test for CLI**

Create a test in `tests/e2e/` (or inline in `hsx-cli`). Add to `crates/hsx-cli/tests/cli_smoke.rs`:

This file needs to be created at `crates/hsx-cli/tests/cli_smoke.rs`:

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_version() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hsx"));
}

#[test]
fn cli_help() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-native search engine"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_search_help() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search query"));
}

#[test]
fn cli_unknown_command() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn cli_doctor_runs() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fetchium Doctor"));
}
```

**Files modified/created:**
```
crates/hsx-cli/src/cli.rs            # Add --quiet, --no-cache flags, --output on SearchArgs/FetchArgs
crates/hsx-cli/src/main.rs           # Apply --quiet, --no-cache to config
crates/hsx-cli/tests/cli_smoke.rs    # NEW: E2E smoke tests (5 tests)
```

**Acceptance criteria:**

- [ ] `hsx --version` prints the version
- [ ] `hsx --help` lists all 12 commands with descriptions
- [ ] `hsx search --help` shows all search flags (query, max-results, backends, output)
- [ ] `hsx fetch --help` shows all fetch flags (url, budget, tier, query, output)
- [ ] `hsx doctor` runs and prints system diagnostics
- [ ] `hsx nonexistent` exits with error code
- [ ] All 12 commands dispatch correctly (even though most are stubs)
- [ ] Global flags (`--verbose`, `--quiet`, `--format`, `--no-cache`, `--config`) work
- [ ] `--no-cache` disables cache in config
- [ ] `--verbose` enables debug logging
- [ ] `--quiet` suppresses non-essential output
- [ ] At least 5 E2E tests pass via `assert_cmd`

**Testing:**

```bash
# Build and test CLI
cargo build -p hsx-cli
cargo test -p hsx-cli

# Manual smoke tests
cargo run -p hsx-cli -- --version
cargo run -p hsx-cli -- --help
cargo run -p hsx-cli -- search --help
cargo run -p hsx-cli -- doctor
```

---

### P0-E3-T2: Doctor Command

**ID:** `P0-E3-T2`
**Status:** `DONE`
**Priority:** P0
**Estimated effort:** 0 days (already complete, needs review and hardening)
**Dependencies:** `P0-E3-T1`, `P0-E1-T3`

**Description:**
Implement `hsx doctor` -- a system health check that detects CPU, RAM, GPU, network, available browsers, Ollama status, config validity, and recommends an execution tier per PRD SS13.

**What already exists:**
The file `crates/hsx-cli/src/commands/doctor.rs` is **implemented** with:

| Check | Status |
|-------|--------|
| Rust toolchain | DONE (always true since binary is compiled) |
| Resource tier detection | DONE (via `HsxConfig::detect_resource_tier()`) |
| Data directory existence | DONE |
| Chromium/Chrome detection | DONE (checks 4 paths) |
| Ollama availability | DONE (HTTP check to localhost:11434) |

**What should be added for production quality (recommended hardening):**

**Step 1: Add detailed system info display**

Enhance `crates/hsx-cli/src/commands/doctor.rs` with more detailed system info using `sysinfo`:

```rust
//! `hsx doctor` -- system health check (PRD SS13).

use colored::Colorize;
use hsx_core::config::HsxConfig;
use sysinfo::System;

pub async fn run(config: &HsxConfig) -> anyhow::Result<()> {
    println!("{}", "Fetchium Doctor".bold().cyan());
    println!("{}", "=".repeat(50));
    println!();

    // ---- System Information ----
    println!("{}", "System Information".bold());
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_name = sys.cpus().first().map(|c| c.brand().to_string()).unwrap_or("unknown".into());
    let cpu_count = sys.cpus().len();
    let total_ram_mb = sys.total_memory() / (1024 * 1024);
    let free_ram_mb = sys.available_memory() / (1024 * 1024);
    let used_pct = if total_ram_mb > 0 {
        ((total_ram_mb - free_ram_mb) as f64 / total_ram_mb as f64 * 100.0) as u64
    } else {
        0
    };

    check("CPU", true, &format!("{cpu_name} ({cpu_count} cores)"));
    check("RAM", true, &format!(
        "{total_ram_mb} MB total, {free_ram_mb} MB free ({used_pct}% used)"
    ));

    // ---- Resource Tier ----
    let tier = HsxConfig::detect_resource_tier();
    let tier_detail = match tier {
        hsx_core::types::ResourceTier::Minimal => "Minimal (< 4 GB RAM)",
        hsx_core::types::ResourceTier::Standard => "Standard (4-16 GB RAM)",
        hsx_core::types::ResourceTier::Performance => "Performance (16-32 GB RAM)",
        hsx_core::types::ResourceTier::Server => "Server (32+ GB RAM, 8+ cores)",
    };
    check("Resource Tier", true, tier_detail);

    println!();

    // ---- Configuration ----
    println!("{}", "Configuration".bold());
    let data_dir = config.data_dir();
    let dir_exists = data_dir.exists();
    check(
        "Data directory",
        dir_exists,
        &format!("{} {}", data_dir.display(), if dir_exists { "(exists)" } else { "(will be created)" }),
    );

    let config_path = HsxConfig::config_file_path();
    let config_exists = config_path.exists();
    check(
        "Config file",
        config_exists,
        &format!("{} {}", config_path.display(), if config_exists { "(loaded)" } else { "(using defaults)" }),
    );

    check("Default budget", true, &format!("{} tokens", config.search.default_budget));
    check("Cache", config.cache.enabled, if config.cache.enabled { "enabled" } else { "disabled" });

    println!();

    // ---- External Tools ----
    println!("{}", "External Tools".bold());

    // Chromium / Chrome
    let chromium = which_chromium();
    check(
        "Chromium/Chrome",
        chromium.is_some(),
        chromium.as_deref().unwrap_or("not found (headless features unavailable)"),
    );

    // Ollama
    let ollama_status = check_ollama(&config.ai.ollama_host).await;
    check(
        "Ollama",
        ollama_status.is_some(),
        ollama_status.as_deref().unwrap_or("not running (AI features unavailable)"),
    );

    println!();

    // ---- Summary ----
    println!("{}", "Summary".bold());
    let parallel = match tier {
        hsx_core::types::ResourceTier::Minimal => "2-4",
        hsx_core::types::ResourceTier::Standard => "8-16",
        hsx_core::types::ResourceTier::Performance => "16-32",
        hsx_core::types::ResourceTier::Server => "32-50",
    };
    let browsers = match tier {
        hsx_core::types::ResourceTier::Minimal => "0-1",
        hsx_core::types::ResourceTier::Standard => "2-4",
        hsx_core::types::ResourceTier::Performance => "4-6",
        hsx_core::types::ResourceTier::Server => "6-8",
    };
    println!("  Recommended parallel fetches: {}", parallel.green());
    println!("  Recommended browser pool:     {}", browsers.green());
    if chromium.is_none() {
        println!("  {}", "Install Chrome/Chromium for headless search (Google, Bing, Scholar)".yellow());
    }
    if ollama_status.is_none() {
        println!("  {}", "Install Ollama for AI features: https://ollama.ai".yellow());
    }

    println!();
    Ok(())
}

fn check(name: &str, ok: bool, detail: &str) {
    let icon = if ok {
        "OK".green().bold()
    } else {
        "WARN".yellow().bold()
    };
    println!("  [{icon}] {name}: {detail}");
}

fn which_chromium() -> Option<String> {
    let candidates = if cfg!(target_os = "macos") {
        vec![
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ]
    } else {
        vec![
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ]
    };

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    None
}

async fn check_ollama(host: &str) -> Option<String> {
    let url = format!("{host}/api/tags");
    match reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
    {
        Ok(resp) => {
            if resp.status().is_success() {
                // Try to parse model count
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let model_count = body
                        .get("models")
                        .and_then(|m| m.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0);
                    Some(format!("running ({model_count} models available)"))
                } else {
                    Some("running".into())
                }
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
```

**Step 2: Add test for doctor output**

Add to `crates/hsx-cli/tests/cli_smoke.rs` (or the existing test file):

```rust
#[test]
fn doctor_shows_system_info() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("System Information"))
        .stdout(predicate::str::contains("CPU"))
        .stdout(predicate::str::contains("RAM"))
        .stdout(predicate::str::contains("Resource Tier"))
        .stdout(predicate::str::contains("Configuration"))
        .stdout(predicate::str::contains("External Tools"));
}
```

**Files modified/created:**
```
crates/hsx-cli/src/commands/doctor.rs   # Enhanced doctor with full system info
crates/hsx-cli/tests/cli_smoke.rs       # Add doctor output test
```

**Acceptance criteria:**

- [ ] `hsx doctor` displays CPU model and core count
- [ ] `hsx doctor` displays total RAM, free RAM, and usage percentage
- [ ] `hsx doctor` displays detected resource tier with description
- [ ] `hsx doctor` checks data directory existence
- [ ] `hsx doctor` checks config file existence
- [ ] `hsx doctor` shows default budget and cache status from config
- [ ] `hsx doctor` detects Chrome/Chromium on macOS, Linux, and Windows
- [ ] `hsx doctor` checks Ollama availability and reports model count
- [ ] `hsx doctor` shows recommended parallel fetches and browser pool size
- [ ] `hsx doctor` suggests installing missing tools (Chrome, Ollama)
- [ ] Output is color-coded: green OK, yellow WARN
- [ ] Doctor command completes in < 5 seconds
- [ ] E2E test verifies all major sections appear in output

**Testing:**

```bash
# Run doctor
cargo run -p hsx-cli -- doctor

# Expected output structure:
# Fetchium Doctor
# ==================================================
#
# System Information
#   [OK] CPU: Apple M2 Pro (12 cores)
#   [OK] RAM: 32768 MB total, 18432 MB free (44% used)
#   [OK] Resource Tier: Server (32+ GB RAM, 8+ cores)
#
# Configuration
#   [OK] Data directory: /Users/user/.fetchium (exists)
#   [WARN] Config file: /Users/user/.fetchium/config.toml (using defaults)
#   [OK] Default budget: 4000 tokens
#   [OK] Cache: enabled
#
# External Tools
#   [OK] Chromium/Chrome: /Applications/Google Chrome.app/Contents/MacOS/Google Chrome
#   [WARN] Ollama: not running (AI features unavailable)
#
# Summary
#   Recommended parallel fetches: 32-50
#   Recommended browser pool:     6-8
#   Install Ollama for AI features: https://ollama.ai

# Run E2E tests
cargo test -p hsx-cli
```

---

## Task Dependency Graph (Phase 0)

```
P0-E1-T1 (workspace) ────────────────────┬──── P0-E2-T1 (CI)
    │                                      │         │
    ▼                                      │         ▼
P0-E1-T2 (types) ──── DONE                │    P0-E2-T2 (release)
    │                                      │         │
    ▼                                      │         ▼
P0-E1-T3 (config) ─── needs hardening     │    P0-E2-T3 (npm wrapper)
    │                                      │
    ▼                                      │
P0-E3-T1 (CLI) ────── needs minor adds    │
    │                                      │
    ▼                                      │
P0-E3-T2 (doctor) ─── needs hardening ────┘
```

**Parallelization:**
- **Agent A** can work on E1 (workspace + types + config) sequentially
- **Agent B** can work on E2 (CI/CD + npm) in parallel, since it only depends on `P0-E1-T1` which is already DONE
- E3 (CLI + doctor) depends on E1 being done, which it already is

---

## Phase 0 Completion Checklist

When all tasks are done, verify the following holistic acceptance criteria:

- [ ] `cargo build --workspace` -- zero errors
- [ ] `cargo test --workspace` -- all tests pass (target: 15+ tests)
- [ ] `cargo clippy --workspace -- -D warnings` -- zero warnings
- [ ] `cargo fmt --all -- --check` -- formatting is consistent
- [ ] `cargo doc --workspace --no-deps` -- documentation builds
- [ ] `cargo run -p hsx-cli -- --version` -- prints version
- [ ] `cargo run -p hsx-cli -- --help` -- shows all commands
- [ ] `cargo run -p hsx-cli -- doctor` -- shows full system diagnostics
- [ ] `cargo run -p hsx-cli -- search "test"` -- runs without crash (stub OK)
- [ ] `.github/workflows/ci.yml` -- valid, tests on 3 platforms
- [ ] `.github/workflows/release.yml` -- valid, builds for 5 targets
- [ ] `npm/package.json` -- valid, `npm pack --dry-run` lists correct files
- [ ] Config loads from file, env vars, and CLI overrides
- [ ] All types serialize/deserialize correctly (JSON, TOML)
- [ ] Error types have retry semantics and structured fallback info
- [ ] Project is ready for Phase 1 (MVP Core) implementation

---

## Effort Summary

| Task | Status | Estimated Effort | Notes |
|------|--------|-----------------|-------|
| P0-E1-T1: Workspace Setup | DONE | 0 days | Already complete, verify only |
| P0-E1-T2: Core Types | DONE | 0 days | Already complete, optional extra tests |
| P0-E1-T3: Config System | DONE (needs hardening) | 0.5 day | Add env overrides, save(), ensure_data_dir() |
| P0-E2-T1: CI Workflow | TODO | 1 day | New: `.github/workflows/ci.yml` |
| P0-E2-T2: Release Workflow | TODO | 1.5 days | New: `.github/workflows/release.yml` |
| P0-E2-T3: npm Wrapper | TODO | 1.5 days | New: `npm/` directory (5 files) |
| P0-E3-T1: CLI Skeleton | DONE (needs minor adds) | 0.5 day | Add --quiet, --no-cache, E2E tests |
| P0-E3-T2: Doctor Command | DONE (needs hardening) | 0.5 day | Enhanced system info, per-platform Chromium paths |
| **Total** | | **~5.5 days** | Mostly CI/CD and npm wrapper (new work) |

---

## Files Created/Modified in Phase 0

### New Files
```
.github/workflows/ci.yml                   # P0-E2-T1
.github/workflows/release.yml              # P0-E2-T2
npm/package.json                            # P0-E2-T3
npm/scripts/install-binary.js               # P0-E2-T3
npm/bin/hsx.js                              # P0-E2-T3
npm/bin/hyper.js                            # P0-E2-T3
npm/README.md                               # P0-E2-T3
crates/hsx-cli/tests/cli_smoke.rs           # P0-E3-T1
tests/fixtures/.gitkeep                     # P0-E1-T1
tests/integration/.gitkeep                  # P0-E1-T1
tests/e2e/.gitkeep                          # P0-E1-T1
benches/.gitkeep                            # P0-E1-T1
docs/.gitkeep                               # P0-E1-T1
```

### Modified Files
```
crates/hsx-core/src/config.rs               # P0-E1-T3 (env overrides, save, ensure_data_dir)
crates/hsx-core/src/types.rs                # P0-E1-T2 (optional: extra tests)
crates/hsx-cli/src/cli.rs                   # P0-E3-T1 (--quiet, --no-cache, --output flags)
crates/hsx-cli/src/main.rs                  # P0-E3-T1 (quiet/no-cache handling)
crates/hsx-cli/src/commands/doctor.rs       # P0-E3-T2 (enhanced system info)
```

### Already Complete (no changes needed)
```
Cargo.toml                                  # P0-E1-T1
rust-toolchain.toml                         # P0-E1-T1
crates/hsx-core/Cargo.toml                 # P0-E1-T1
crates/hsx-cli/Cargo.toml                  # P0-E1-T1
crates/hsx-mcp/Cargo.toml                  # P0-E1-T1
crates/hsx-api/Cargo.toml                  # P0-E1-T1
crates/hsx-core/src/lib.rs                 # P0-E1-T1
crates/hsx-core/src/error.rs               # P0-E1-T2
crates/hsx-core/src/http/mod.rs            # Existing
crates/hsx-core/src/http/client.rs         # Existing
crates/hsx-core/src/search/mod.rs          # Existing
crates/hsx-core/src/extract/mod.rs         # Existing
crates/hsx-core/src/resource/mod.rs        # Existing
crates/hsx-cli/src/output.rs              # P0-E3-T1
crates/hsx-cli/src/commands/mod.rs        # P0-E3-T1
crates/hsx-cli/src/commands/*.rs           # All command stubs
```
