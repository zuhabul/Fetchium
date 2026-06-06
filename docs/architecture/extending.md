# Extending Fetchium

## Adding a New Search Backend

1. Create `crates/fetchium-core/src/search/myengine.rs`:

```rust
//! MyEngine search backend.

use crate::error::FetchiumResult;
use crate::http::HttpClient;
use crate::types::ResultItem;

pub struct MyEngineBackend {
    client: HttpClient,
    base_url: String,
}

impl MyEngineBackend {
    pub fn new(client: HttpClient) -> Self {
        Self {
            client,
            base_url: "https://api.myengine.com".to_string(),
        }
    }

    pub async fn search(&self, query: &str, max_results: usize) -> FetchiumResult<Vec<ResultItem>> {
        let url = format!("{}/search?q={}&limit={}", self.base_url,
            urlencoding::encode(query), max_results);
        let response = self.client.fetch(&url).await?;

        // Parse response into ResultItem vec
        let items = parse_results(&response.body)?;
        Ok(items)
    }
}
```

2. Register in `crates/fetchium-core/src/search/orchestrator.rs`:

```rust
use crate::search::myengine::MyEngineBackend;

// In SearchOrchestrator::search():
let myengine = MyEngineBackend::new(self.client.clone());
handles.push(tokio::spawn(async move {
    myengine.search(&query, max).await
}));
```

3. Add `BackendId::MyEngine` to `crates/fetchium-core/src/types.rs`.

## Adding a Plugin

Implement the `Plugin` trait from `crates/fetchium-core/src/plugin/traits.rs`:

```rust
use fetchium_core::plugin::traits::{Plugin, PluginContext, PluginResult};

pub struct MyPlugin;

#[async_trait::async_trait]
impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "Does something useful" }

    async fn execute(
        &self,
        input: serde_json::Value,
        ctx: &PluginContext,
    ) -> PluginResult {
        // Your logic here
        PluginResult::success(serde_json::json!({"result": "done"}))
    }
}
```

Build as a dynamic library (`.so` / `.dylib` / `.dll`) and place in
`~/.fetchium/plugins/`.

## Adding a New CLI Command

1. Create `crates/fetchium-cli/src/commands/mycmd.rs`:

```rust
//! `fetchium mycmd` — description.

pub fn run(args: &MyArgs) -> anyhow::Result<()> {
    // Your implementation
    Ok(())
}
```

2. Add to `crates/fetchium-cli/src/commands/mod.rs`:
```rust
pub mod mycmd;
```

3. Add to `crates/fetchium-cli/src/cli.rs` (Commands enum):
```rust
/// My command description
MyCMD(MyArgs),
```

4. Dispatch in `crates/fetchium-cli/src/main.rs`:
```rust
Commands::MyCMD(args) => commands::mycmd::run(&args)?,
```

## Feature Flags

Optional heavy dependencies use Cargo features:

| Feature | Crate | What it enables |
|---------|-------|-----------------|
| `headless` | `chromiumoxide` | CEP L3 (JavaScript rendering) |
| `embeddings` | `fastembed` | Semantic search, hybrid ranking |
| `vector-search` | `usearch` | HNSW vector index |
| `mcp` | `rmcp` | MCP server protocol |

Build with a feature:
```bash
cargo build -p fetchium-core --features headless,embeddings
```
