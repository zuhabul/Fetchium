# AGENTS.md

Guidelines for AI agents working on the HyperSearchX codebase.

## Build, Lint, and Test Commands

```bash
# Check compilation (fast, no linking)
cargo check

# Build the hsx binary
cargo build -p hsx-cli

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p hsx-core

# Run a single test by name
cargo test -p hsx-core segment_type_roundtrip

# Run a specific test module
cargo test -p hsx-core types::tests

# Lint (zero warnings policy)
cargo clippy -- -D warnings

# Format code
cargo fmt
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

```rust
// lib.rs pattern
pub mod config;
pub mod error;
pub mod types;

pub mod prelude {
    pub use crate::config::HsxConfig;
    pub use crate::error::{HsxError, HsxResult};
    pub use crate::types::*;
}
```

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

## Architecture

```
crates/
├── hsx-core/     # All algorithms: search, extract, rank, validate, cache
├── hsx-cli/      # Binary: clap derive CLI, one file per command
├── hsx-mcp/      # MCP server
└── hsx-api/      # REST API server
```

Data flow: `CLI args → HsxConfig → hsx-core pipeline → formatted output`

## Task Workflow

1. Run `cargo check` after writing code
2. Run `cargo test` after completing a feature
3. Run `cargo clippy -- -D warnings` before committing
4. All public APIs must have `///` doc comments
5. Reference PRD sections in comments: `(PRD §16)`
