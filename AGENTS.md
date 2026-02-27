# AGENTS.md

Guidelines for AI agents (Claude Code, OpenAI Codex, Gemini CLI, etc.) working on the Fetchium codebase.

## Build, Lint, and Test Commands

```bash
# Check compilation (fast, no linking)
cargo check

# Build the hsx binary
cargo build -p hsx-cli

# Build release binary
cargo build -p hsx-cli --release

# Run all tests (currently 563, target: 0 failures)
cargo test

# Run tests for a specific crate
cargo test -p hsx-core

# Run a single test by name
cargo test -p hsx-core segment_type_roundtrip

# Lint (zero warnings policy — enforced in CI)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Run the binary
./target/debug/hsx --help
./target/debug/hsx doctor
./target/debug/hsx provider list
```

## Current Status

- **Tests**: 563 passing, 0 failing, 0 clippy warnings
- **Phases complete**: 0–8 + Multi-provider AI system
- **Binary**: `./target/debug/hsx` with 26 commands

## Architecture

```
crates/
├── hsx-core/     # All algorithms: search, extract, rank, validate, cache, AI, intelligence
├── hsx-cli/      # Binary: clap derive CLI, one file per command in commands/
├── hsx-mcp/      # Manual JSON-RPC 2.0 stdio MCP server (5 tools)
└── hsx-api/      # axum 0.7 REST API server
```

Data flow: `CLI args → HsxConfig → hsx-core pipeline → formatted output`

## Key Module Map (hsx-core/src/)

```
ai/
  credentials.rs      Subscription OAuth detection (Claude Code, Gemini CLI, Codex CLI)
  provider_client.rs  Multi-provider chat client with SSE streaming + fallback chain
  providers.rs        ProviderKind enum, ProvidersConfig, ProviderEntry
  pipeline.rs         run_ai_pipeline() — search → extract → sandwich → provider
  types.rs            AiConfig (providers: ProvidersConfig, fast_model: Option<String>)
  router.rs           select_model(), select_fast_model()
  sandwich.rs         Ms-PoE sandwich layout
  ollama.rs           OllamaClient (local Ollama server)
  prompt.rs           System prompts for synthesis/factual/fallback modes
  setup.rs            DeviceSpec, recommend_models(), format_setup_guide()

search/
  orchestrator.rs     Parallel backend dispatch + BM25 rerank + dedup
  duckduckgo.rs       DDG HTML scraper
  fallback.rs         FallbackChain async executor

extract/
  layer1.rs, layer2.rs  CEP CSS+readability extraction
  pipeline.rs         Speculative parallel extraction
  boilerplate.rs      QADD pre-filter (strips script/style/svg)
  cep_predictor.rs    Decision tree ML predictor for CEP layer selection

token/
  qatbe.rs            BM25 + hybrid embedding ranking, greedy knapsack
  scs.rs              8 segment types, token-efficient JSON
  pds.rs              4-tier progressive streaming
  counter.rs          Heuristic tokenizer + TokenBudget

rank/
  bm25.rs             tantivy-backed Bm25Scorer
  signals.rs          ScoringContext (batch embeddings), HyperFusion 8-signal ranking
  fusion.rs           hyperfusion_rank()

validate/
  cross_source.rs     V4 bigram-Jaccard clustering, negation-aware contradiction
  temporal.rs         V3 exponential decay, intent classification
  authority.rs        V1 domain tiers, SSL/redirect penalties
  rar.rs              5 reflection checkpoints R1-R5

intelligence/
  pie/               STM (source trust), FPM (failure patterns), QPM (query prediction), PKG
  edf.rs             Evidence decay function, domain half-lives
  cce.rs             Confidence calibration (isotonic interpolation)
  acs.rs             Adversarial content shield (shadow/active mode)
  crp.rs             Contradiction resolution protocol
  sgt.rs             Source genealogy tracker (bigram-Jaccard mutation detection)
  totr.rs            Tree-of-Thoughts research

export/
  pandoc.rs          PDF: typst (~1s) → xelatex → default; check_typst() for doctor
  bibtex.rs          Pure Rust BibTeX generator

embeddings/
  engine.rs          fastembed-rs singleton, embed() + embed_batch()
  cache.rs           SHA-256 keyed SQLite cache

qadd/pipeline.rs     5-step DOM pruning, chunked embed_batch (EMBED_BATCH_SIZE=128)
```

## AI Provider System

### Credential Detection (automatic, no config needed)

| Provider | Credential Location | Auth Type |
|----------|--------------------|-----------| 
| **Anthropic** | macOS Keychain `"Claude Code-credentials"` | OAuth Bearer (`sk-ant-oat01-…`) |
| **Gemini** | `~/.gemini/oauth_creds.json` | Google OAuth Bearer, auto-refreshed |
| **OpenAI** | `~/.codex/auth.json` | JWT Bearer from ChatGPT OAuth |
| **OpenRouter** | `~/.openrouter/config.json` OR env | API key |
| **Ollama** | None (localhost:11434) | Local |
| **GeminiCli** | `gemini` binary in PATH | Local subprocess |

### Priority order (per provider)
1. Config file `api_key` field
2. Environment variable (`ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `OPENAI_API_KEY`, etc.)
3. Subscription CLI session (auto-detected)

### Key types
- `ProviderKind` — enum: Ollama, OpenAi, Anthropic, Gemini, GeminiCli, OpenRouter
- `ProvidersConfig.fallback_chain: Vec<String>` — ordered slugs, tried in sequence
- `chat_with_fallback()` — tries each provider, returns first success
- `check_provider()` — availability check without LLM call (used by `hsx doctor` and `hsx provider list`)

### Anthropic OAuth vs API key
```rust
// API key: x-api-key header
// OAuth (Claude Code subscription): Authorization: Bearer {sk-ant-oat01-...}
call_anthropic(token, use_oauth=false, ...)  // API key
call_anthropic(token, use_oauth=true, ...)   // OAuth
```

### Gemini OAuth vs API key
```rust
// API key: URL ?key={api_key} parameter, no auth header
call_gemini_api_key(api_key, model, ...)

// OAuth: Authorization: Bearer header, no ?key= parameter
call_gemini_oauth(access_token, model, ...)
```

## Code Style Guidelines

### Imports

```rust
// Order: std → external crates → internal crates → current crate modules
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::warn;
use crate::error::HsxResult;
use crate::types::{PdsTier, ResourceTier};
```

### Formatting

- Use `cargo fmt` before committing
- Max line width: 100 characters (default)
- Zero warnings: `cargo clippy -- -D warnings` must pass

### Types and Serialization

```rust
// All public types must derive Debug, Clone, Serialize, Deserialize
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub some_field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<String>,
    #[serde(default)]
    pub new_field: bool,
}

// Enums use snake_case in JSON
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status { Active, Pending, Completed }
```

### Naming Conventions

| Type | Convention | Example |
|------|------------|---------|
| Structs | PascalCase | `SearchMeta`, `ResultItem` |
| Enums | PascalCase | `PdsTier`, `BackendId` |
| Functions | snake_case | `fetch_text` |
| Variables | snake_case | `max_results` |
| Constants | SCREAMING_SNAKE | `MAX_REDIRECTS` |
| Crate prefix | hsx- | `hsx-core`, `hsx-cli` |

### Error Handling

```rust
// Use HsxResult<T> for fallible operations in hsx-core
use crate::error::{HsxError, HsxResult};

pub fn do_something() -> HsxResult<String> {
    let data = fetch_data()?;
    if data.is_empty() {
        return Err(HsxError::Extraction("No data found".into()));
    }
    Ok(data)
}

// CLI commands use anyhow::Result
pub async fn run(config: &HsxConfig) -> anyhow::Result<()> { }
```

### Documentation

```rust
/// Brief description on first line.
/// Reference PRD sections where applicable: (PRD §43)
pub fn function_name() {}
```

### Module Organization

- Keep files under 500 lines — split into submodules when exceeding
- One file per CLI command in `crates/hsx-cli/src/commands/`
- Tests go in the same file with `#[cfg(test)] mod tests { }`

### Async Patterns

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> { }

#[async_trait]
pub trait SearchBackend: Send + Sync {
    async fn search(&self, query: &str) -> HsxResult<Vec<ResultItem>>;
}
```

## Dependency Management

All shared dependencies go in the workspace `Cargo.toml`:

```toml
# In workspace Cargo.toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }

# In crates/hsx-core/Cargo.toml
[dependencies]
tokio.workspace = true
```

**Never add version numbers directly in crate Cargo.toml** for dependencies already in workspace.

## Task Workflow

1. Run `cargo check` after writing code
2. Run `cargo test` after completing a feature
3. Run `cargo clippy -- -D warnings` before committing
4. All public APIs must have `///` doc comments
5. Reference PRD sections in comments: `(PRD §16)`
6. After any change: verify `cargo test` shows 563+ tests, 0 failures
