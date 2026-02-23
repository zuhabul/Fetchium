# Phase 5: Semantic Search & Advanced Features

> **Duration:** Weeks 19-26 | **Priority:** P2
> **Depends On:** Phase 4 complete (AI Engine, Deep Research, MCP)
> **PRD Sections:** 8.1, 8.2, 8.10, 17, 21, 22, 26, 27, 28, 40, 48

---

## Epic 5.1: Embeddings & Semantic Search

### P5-E1-T1: Local ONNX Embedding Generation

| Field | Value |
|-------|-------|
| **ID** | `P5-E1-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Integrate local ONNX-based embedding generation using the `ort` crate with the `all-MiniLM-L6-v2` model. Provide an `embed()` function returning `Vec<f32>`, batch embedding support, an embedding cache backed by SQLite, and feature-gate the entire subsystem behind the `embeddings` Cargo feature flag. |
| **PRD Ref** | 8.1 (HyperFusion semantic signal), 21 (Semantic Search & Hybrid Ranking), 48 (ort crate) |
| **Depends On** | `P4-E1` (AI engine infra), `P1-E6` (SQLite disk cache) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/Cargo.toml` | Add `ort` dependency under `[features] embeddings` |
| `crates/hsx-core/src/embeddings/mod.rs` | New module root |
| `crates/hsx-core/src/embeddings/model.rs` | ONNX model loader and session manager |
| `crates/hsx-core/src/embeddings/embed.rs` | `embed()` and `embed_batch()` functions |
| `crates/hsx-core/src/embeddings/cache.rs` | SQLite-backed embedding cache |
| `crates/hsx-core/src/embeddings/download.rs` | Auto-download model on first use |
| `crates/hsx-core/src/lib.rs` | Register `embeddings` module behind feature gate |
| `models/` | Directory for downloaded ONNX model files |

#### Step-by-Step Implementation Guide

**Step 1: Add dependencies and feature gate**

```toml
# crates/hsx-core/Cargo.toml
[features]
default = []
embeddings = ["ort", "tokenizers"]

[dependencies]
ort = { version = "2", optional = true, features = ["download-binaries"] }
tokenizers = { version = "0.20", optional = true }
```

**Step 2: Create the embedding model manager**

```rust
// crates/hsx-core/src/embeddings/model.rs
#[cfg(feature = "embeddings")]
use ort::{Session, SessionBuilder, GraphOptimizationLevel};
use std::path::PathBuf;
use std::sync::OnceLock;

static SESSION: OnceLock<Session> = OnceLock::new();

/// Path where the ONNX model is stored locally.
fn model_path() -> PathBuf {
    let base = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hypersearchx")
        .join("models");
    base.join("all-MiniLM-L6-v2.onnx")
}

/// Initialize or return the cached ONNX session.
/// Downloads the model on first use if not present.
pub fn get_session() -> Result<&'static Session, crate::Error> {
    SESSION.get_or_try_init(|| {
        let path = model_path();
        if !path.exists() {
            super::download::download_model(&path)?;
        }
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(4)?
            .commit_from_file(&path)?;
        Ok(session)
    })
}
```

**Step 3: Implement embed() and embed_batch()**

```rust
// crates/hsx-core/src/embeddings/embed.rs
use ndarray::{Array2, Axis};
use tokenizers::Tokenizer;
use std::sync::OnceLock;

static TOKENIZER: OnceLock<Tokenizer> = OnceLock::new();

const MAX_SEQ_LEN: usize = 256;
const EMBEDDING_DIM: usize = 384; // MiniLM-L6-v2 dimension

fn get_tokenizer() -> Result<&'static Tokenizer, crate::Error> {
    TOKENIZER.get_or_try_init(|| {
        let path = super::model::tokenizer_path();
        Tokenizer::from_file(&path)
            .map_err(|e| crate::Error::Embedding(format!("Tokenizer load failed: {e}")))
    })
}

/// Embed a single text string into a 384-dimensional f32 vector.
pub fn embed(text: &str) -> Result<Vec<f32>, crate::Error> {
    let results = embed_batch(&[text])?;
    Ok(results.into_iter().next().unwrap())
}

/// Embed a batch of text strings. Returns one Vec<f32> per input.
/// Batching amortizes the ONNX session overhead.
pub fn embed_batch(texts: &[&str]) -> Result<Vec<Vec<f32>>, crate::Error> {
    let session = super::model::get_session()?;
    let tokenizer = get_tokenizer()?;

    // Tokenize all inputs
    let encodings = tokenizer
        .encode_batch(texts.to_vec(), true)
        .map_err(|e| crate::Error::Embedding(format!("Tokenize failed: {e}")))?;

    let batch_size = encodings.len();

    // Pad to uniform length
    let max_len = encodings.iter()
        .map(|e| e.get_ids().len().min(MAX_SEQ_LEN))
        .max()
        .unwrap_or(0);

    // Build input tensors: input_ids, attention_mask, token_type_ids
    let mut input_ids = Array2::<i64>::zeros((batch_size, max_len));
    let mut attention_mask = Array2::<i64>::zeros((batch_size, max_len));
    let mut token_type_ids = Array2::<i64>::zeros((batch_size, max_len));

    for (i, encoding) in encodings.iter().enumerate() {
        let ids = encoding.get_ids();
        let mask = encoding.get_attention_mask();
        let len = ids.len().min(max_len);
        for j in 0..len {
            input_ids[[i, j]] = ids[j] as i64;
            attention_mask[[i, j]] = mask[j] as i64;
            // token_type_ids stays 0 for single-sequence inputs
        }
    }

    // Run inference
    let outputs = session.run(ort::inputs![
        "input_ids" => input_ids.view(),
        "attention_mask" => attention_mask.view(),
        "token_type_ids" => token_type_ids.view(),
    ]?)?;

    // Extract embeddings: output shape [batch, seq_len, 384]
    // Apply mean pooling over the sequence dimension using attention mask
    let token_embeddings = outputs[0]
        .try_extract_tensor::<f32>()?;

    let mut results = Vec::with_capacity(batch_size);
    for i in 0..batch_size {
        let mut pooled = vec![0.0f32; EMBEDDING_DIM];
        let mut count = 0.0f32;
        for j in 0..max_len {
            if attention_mask[[i, j]] == 1 {
                for k in 0..EMBEDDING_DIM {
                    pooled[k] += token_embeddings[[i, j, k]];
                }
                count += 1.0;
            }
        }
        // Mean pool
        if count > 0.0 {
            for k in 0..EMBEDDING_DIM {
                pooled[k] /= count;
            }
        }
        // L2 normalize
        let norm: f32 = pooled.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for k in 0..EMBEDDING_DIM {
                pooled[k] /= norm;
            }
        }
        results.push(pooled);
    }

    Ok(results)
}
```

**Step 4: Implement the embedding cache**

```rust
// crates/hsx-core/src/embeddings/cache.rs
use rusqlite::Connection;
use sha2::{Sha256, Digest};
use std::sync::Mutex;

pub struct EmbeddingCache {
    conn: Mutex<Connection>,
}

impl EmbeddingCache {
    pub fn new(db_path: &std::path::Path) -> Result<Self, crate::Error> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS embedding_cache (
                text_hash TEXT PRIMARY KEY,
                embedding BLOB NOT NULL,
                created_at INTEGER DEFAULT (strftime('%s', 'now'))
            );
            CREATE INDEX IF NOT EXISTS idx_cache_created ON embedding_cache(created_at);"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn hash_text(text: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn get(&self, text: &str) -> Result<Option<Vec<f32>>, crate::Error> {
        let hash = Self::hash_text(text);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare_cached(
            "SELECT embedding FROM embedding_cache WHERE text_hash = ?1"
        )?;
        let result = stmt.query_row([&hash], |row| {
            let blob: Vec<u8> = row.get(0)?;
            Ok(blob)
        }).optional()?;

        match result {
            Some(blob) => {
                // Deserialize f32 vector from bytes
                let floats: Vec<f32> = blob
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
                    .collect();
                Ok(Some(floats))
            }
            None => Ok(None),
        }
    }

    pub fn put(&self, text: &str, embedding: &[f32]) -> Result<(), crate::Error> {
        let hash = Self::hash_text(text);
        let blob: Vec<u8> = embedding.iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO embedding_cache (text_hash, embedding) VALUES (?1, ?2)",
            rusqlite::params![hash, blob],
        )?;
        Ok(())
    }

    /// Evict entries older than `max_age_secs`.
    pub fn evict_older_than(&self, max_age_secs: u64) -> Result<usize, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM embedding_cache WHERE created_at < strftime('%s', 'now') - ?1",
            [max_age_secs],
        )?;
        Ok(deleted)
    }
}
```

**Step 5: Model auto-download**

```rust
// crates/hsx-core/src/embeddings/download.rs
use std::path::Path;

const MODEL_URL: &str =
    "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx";
const TOKENIZER_URL: &str =
    "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json";

pub fn download_model(model_path: &Path) -> Result<(), crate::Error> {
    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    tracing::info!("Downloading all-MiniLM-L6-v2 ONNX model (first run only)...");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    // Download model
    let resp = client.get(MODEL_URL).send()?;
    std::fs::write(model_path, resp.bytes()?)?;

    // Download tokenizer
    let tokenizer_path = model_path.with_file_name("tokenizer.json");
    let resp = client.get(TOKENIZER_URL).send()?;
    std::fs::write(&tokenizer_path, resp.bytes()?)?;

    tracing::info!("Model downloaded successfully.");
    Ok(())
}
```

#### Acceptance Criteria

- [ ] `embed("hello world")` returns a `Vec<f32>` of length 384
- [ ] `embed_batch(&["a", "b", "c"])` returns 3 vectors, each of length 384
- [ ] Cosine similarity between `embed("king")` and `embed("queen")` is > 0.7
- [ ] Cosine similarity between `embed("king")` and `embed("banana")` is < 0.4
- [ ] Embedding cache hits return identical vectors without running ONNX inference
- [ ] Model auto-downloads on first invocation if not present
- [ ] `cargo build` succeeds with and without `--features embeddings`
- [ ] Feature gate: code compiles to no-op stubs when `embeddings` is disabled
- [ ] Batch of 100 texts completes in < 2 seconds on Apple M-series
- [ ] L2 normalization is applied: all output vectors have unit norm (within f32 epsilon)

#### Pitfalls

- **ONNX Runtime version conflicts**: Pin the `ort` crate version and test across platforms. macOS ARM may need the `coreml` execution provider for best performance.
- **Tokenizer mismatch**: The tokenizer file MUST match the model. Always download both from the same HuggingFace revision.
- **Memory overhead**: The ONNX session is ~90MB in RAM. Use `OnceLock` to ensure exactly one session exists per process.
- **Thread safety**: `ort::Session` is `Send + Sync` but inference is not parallelizable within a single session. For true parallel embedding, consider a pool of sessions or use `tokio::task::spawn_blocking`.
- **Max sequence length**: MiniLM has a 512-token limit. Truncate inputs to 256 tokens for safety margin and speed.
- **SQLite lock contention on cache**: Use WAL mode and `Mutex<Connection>` to avoid `SQLITE_BUSY` under concurrent access.

---

### P5-E1-T2: HyperFusion Semantic Signal Upgrade

| Field | Value |
|-------|-------|
| **ID** | `P5-E1-T2` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Replace the placeholder/stub semantic similarity signal in the HyperFusion ranking algorithm with real cosine similarity using the ONNX embedding engine from P5-E1-T1. When the `embeddings` feature is disabled, fall back to BM25-only scoring. |
| **PRD Ref** | 8.1 (HyperFusion formula), 21 (Semantic Search & Hybrid Ranking) |
| **Depends On** | `P5-E1-T1`, `P2-E4` (HyperFusion ranking engine) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/ranking/hyperfusion.rs` | Replace stub semantic signal |
| `crates/hsx-core/src/ranking/signals/semantic.rs` | New: real cosine similarity computation |
| `crates/hsx-core/src/ranking/signals/mod.rs` | Register semantic signal |

#### Step-by-Step Implementation Guide

**Step 1: Create the semantic signal module**

```rust
// crates/hsx-core/src/ranking/signals/semantic.rs

/// Compute cosine similarity between two pre-normalized vectors.
/// Since embed() returns L2-normalized vectors, cosine = dot product.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "Vectors must have same dimension");
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Score a result against a query using semantic similarity.
/// Requires pre-computed embeddings for both query and result.
pub struct SemanticSignal {
    query_embedding: Vec<f32>,
}

impl SemanticSignal {
    #[cfg(feature = "embeddings")]
    pub fn new(query: &str) -> Result<Self, crate::Error> {
        let cache = crate::embeddings::cache::global_cache()?;
        let query_embedding = match cache.get(query)? {
            Some(emb) => emb,
            None => {
                let emb = crate::embeddings::embed::embed(query)?;
                cache.put(query, &emb)?;
                emb
            }
        };
        Ok(Self { query_embedding })
    }

    #[cfg(not(feature = "embeddings"))]
    pub fn new(_query: &str) -> Result<Self, crate::Error> {
        Ok(Self { query_embedding: vec![] })
    }

    /// Returns 0.0..1.0 semantic similarity score.
    /// When embeddings feature is disabled, always returns 0.0.
    pub fn score(&self, result_text: &str) -> f32 {
        #[cfg(feature = "embeddings")]
        {
            let cache = crate::embeddings::cache::global_cache().ok();
            let result_emb = cache
                .and_then(|c| c.get(result_text).ok().flatten())
                .or_else(|| crate::embeddings::embed::embed(result_text).ok());

            match result_emb {
                Some(emb) => cosine_similarity(&self.query_embedding, &emb).max(0.0),
                None => 0.0,
            }
        }
        #[cfg(not(feature = "embeddings"))]
        {
            let _ = result_text;
            0.0
        }
    }
}
```

**Step 2: Integrate into HyperFusion**

```rust
// In crates/hsx-core/src/ranking/hyperfusion.rs
// Replace the stub semantic_score computation:

pub fn rank_results(
    query: &str,
    results: &mut [SearchResult],
    intent: &QueryIntent,
) -> Result<(), crate::Error> {
    let semantic = SemanticSignal::new(query)?;
    let weights = intent.signal_weights();

    for result in results.iter_mut() {
        let bm25_score = result.bm25_score;
        let semantic_score = semantic.score(&result.snippet);
        let temporal_score = temporal_decay(result.published_date, intent.freshness_need);
        let authority_score = authority_chain(&result.domain, &result.citations);
        let evidence_score = evidence_density(&result.content);
        let diversity_score = diversity_bonus(&result.domain, &seen_domains);
        let depth_score = content_depth(result.word_count, &result.structure);
        let consensus_score = consensus(&result.claims, &all_claims);

        result.fusion_score =
            weights.bm25 * bm25_score
            + weights.semantic * semantic_score
            + weights.temporal * temporal_score
            + weights.authority * authority_score
            + weights.evidence * evidence_score
            + weights.diversity * diversity_score
            + weights.depth * depth_score
            + weights.consensus * consensus_score
            - result.duplicate_penalty;
    }

    results.sort_by(|a, b| b.fusion_score.partial_cmp(&a.fusion_score).unwrap());
    Ok(())
}
```

#### Acceptance Criteria

- [ ] HyperFusion uses real cosine similarity when `embeddings` feature is enabled
- [ ] HyperFusion gracefully falls back to 0.0 for semantic signal when feature is disabled
- [ ] Semantically similar results (e.g., "Rust web framework" and "Actix web server") rank higher than semantically unrelated ones
- [ ] Query embedding is computed once per search, not per-result
- [ ] Result snippet embeddings are cached in the embedding cache
- [ ] No performance regression: ranking 50 results completes in < 500ms (embedding included)
- [ ] Unit test: ranking with semantic signal enabled produces different ordering than BM25-only

#### Pitfalls

- **Embedding the wrong text**: Embed the result `snippet` or `title + snippet` concatenated, not the full HTML. Full content is too long for the 256-token window.
- **Cold start latency**: The first search after boot will be slow due to ONNX session initialization. Consider lazy loading with a spinner.
- **Score range mismatch**: BM25 scores are unbounded, cosine is 0..1. Normalize BM25 to 0..1 (e.g., min-max across the result set) before combining with cosine in HyperFusion.

---

### P5-E1-T3: HNSW Vector Index for Local Search

| Field | Value |
|-------|-------|
| **ID** | `P5-E1-T3` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Build a local HNSW vector index for storing and searching document embeddings. Expose via `hsx index add`, `hsx index search`, and `hsx index build` CLI commands. Uses the `usearch` or `hnswlib-rs` crate for SIMD-accelerated nearest neighbor search. |
| **PRD Ref** | 28 (Caching & Local Index), 21 (Cascade Retrieval), 48 (hnswlib-rs / usearch) |
| **Depends On** | `P5-E1-T1` (embedding generation), `P1-E5` (BM25 tantivy index), `P1-E6` (SQLite cache) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/Cargo.toml` | Add `usearch` or `hnsw_rs` under `vector-search` feature |
| `crates/hsx-core/src/index/mod.rs` | New module root |
| `crates/hsx-core/src/index/vector.rs` | HNSW index wrapper |
| `crates/hsx-core/src/index/hybrid.rs` | Hybrid BM25 + vector search |
| `crates/hsx-core/src/index/document.rs` | Indexed document schema |
| `crates/hsx-cli/src/commands/index.rs` | CLI subcommands: add, search, build, stats |

#### Step-by-Step Implementation Guide

**Step 1: Define the indexed document schema**

```rust
// crates/hsx-core/src/index/document.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDocument {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub content: String,           // full extracted text
    pub domain: String,
    pub fetched_at: chrono::DateTime<chrono::Utc>,
    pub content_hash: String,      // SHA-256
    pub embedding: Option<Vec<f32>>,
}
```

**Step 2: HNSW vector index wrapper**

```rust
// crates/hsx-core/src/index/vector.rs
use std::path::Path;

pub struct VectorIndex {
    // Using usearch for SIMD-accelerated HNSW
    index: usearch::Index,
    dimension: usize,
}

impl VectorIndex {
    pub fn new(dimension: usize) -> Result<Self, crate::Error> {
        let options = usearch::IndexOptions {
            dimensions: dimension,
            metric: usearch::MetricKind::Cos,
            quantization: usearch::ScalarKind::F32,
            connectivity: 16,      // M parameter
            expansion_add: 128,    // ef_construction
            expansion_search: 64,  // ef_search
        };
        let index = usearch::Index::new(&options)?;
        Ok(Self { index, dimension })
    }

    pub fn load(path: &Path, dimension: usize) -> Result<Self, crate::Error> {
        let mut idx = Self::new(dimension)?;
        if path.exists() {
            idx.index.load(path)?;
        }
        Ok(idx)
    }

    pub fn save(&self, path: &Path) -> Result<(), crate::Error> {
        self.index.save(path)?;
        Ok(())
    }

    pub fn add(&self, id: u64, vector: &[f32]) -> Result<(), crate::Error> {
        assert_eq!(vector.len(), self.dimension);
        self.index.add(id, vector)?;
        Ok(())
    }

    /// Search for k nearest neighbors. Returns (id, distance) pairs.
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<(u64, f32)>, crate::Error> {
        let results = self.index.search(query_vector, k)?;
        Ok(results.keys.into_iter()
            .zip(results.distances.into_iter())
            .collect())
    }

    pub fn len(&self) -> usize {
        self.index.size()
    }
}
```

**Step 3: Hybrid search combining BM25 + vector**

```rust
// crates/hsx-core/src/index/hybrid.rs
use std::collections::HashMap;

pub struct HybridIndex {
    vector: super::vector::VectorIndex,
    // tantivy index handle from P1-E5
    bm25: tantivy::Index,
    // SQLite metadata store
    meta_db: rusqlite::Connection,
}

impl HybridIndex {
    /// Hybrid search: BM25 sparse retrieval + HNSW dense retrieval + RRF fusion.
    pub fn search(
        &self,
        query: &str,
        k: usize,
    ) -> Result<Vec<super::document::IndexedDocument>, crate::Error> {
        // Stage 1: BM25 retrieval (top 100)
        let bm25_results = self.bm25_search(query, 100)?;

        // Stage 2: Vector retrieval (top 100)
        #[cfg(feature = "embeddings")]
        let vector_results = {
            let query_emb = crate::embeddings::embed::embed(query)?;
            self.vector.search(&query_emb, 100)?
        };
        #[cfg(not(feature = "embeddings"))]
        let vector_results: Vec<(u64, f32)> = vec![];

        // Stage 3: Reciprocal Rank Fusion
        let mut rrf_scores: HashMap<u64, f64> = HashMap::new();
        let rrf_k = 60.0; // standard RRF constant

        for (rank, (doc_id, _score)) in bm25_results.iter().enumerate() {
            *rrf_scores.entry(*doc_id).or_default() += 1.0 / (rrf_k + rank as f64 + 1.0);
        }
        for (rank, (doc_id, _dist)) in vector_results.iter().enumerate() {
            *rrf_scores.entry(*doc_id).or_default() += 1.0 / (rrf_k + rank as f64 + 1.0);
        }

        // Sort by RRF score descending, take top k
        let mut fused: Vec<(u64, f64)> = rrf_scores.into_iter().collect();
        fused.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        fused.truncate(k);

        // Load full documents from metadata DB
        let doc_ids: Vec<u64> = fused.iter().map(|(id, _)| *id).collect();
        self.load_documents(&doc_ids)
    }
}
```

**Step 4: CLI commands**

```rust
// crates/hsx-cli/src/commands/index.rs
use clap::Subcommand;

#[derive(Subcommand)]
pub enum IndexCommand {
    /// Add a URL or sitemap to the local index
    Add {
        /// URL or sitemap URL to index
        url: String,
        /// Re-index even if content hash unchanged
        #[arg(long)]
        force: bool,
    },
    /// Build/rebuild the vector index from stored documents
    Build {
        /// Number of parallel embedding workers
        #[arg(long, default_value = "4")]
        workers: usize,
    },
    /// Search the local index
    Search {
        /// Search query
        query: String,
        /// Max results
        #[arg(short = 'k', default_value = "10")]
        limit: usize,
        /// Search mode: bm25, vector, or hybrid
        #[arg(long, default_value = "hybrid")]
        mode: String,
    },
    /// Show index statistics
    Stats,
}
```

#### Acceptance Criteria

- [ ] `hsx index add <url>` fetches, extracts, embeds, and stores a document
- [ ] `hsx index add <sitemap.xml>` crawls and indexes all URLs from the sitemap
- [ ] `hsx index build` (re-)computes embeddings for all stored documents and builds the HNSW index
- [ ] `hsx index search "query"` returns ranked results from the local index
- [ ] `hsx index search "query" --mode hybrid` fuses BM25 + vector results via RRF
- [ ] `hsx index stats` shows document count, index size, embedding coverage
- [ ] Vector search of 10,000 documents returns in < 20ms
- [ ] Index persists across sessions (stored at `~/.hypersearchx/index/`)
- [ ] Duplicate URLs detected by content hash and skipped

#### Pitfalls

- **HNSW index size**: Each 384-dim f32 vector is 1.5KB. 100K documents = ~150MB index file. Monitor disk usage and warn users.
- **Re-indexing**: When `hsx index build` runs, existing documents without embeddings need to be embedded. Use batching (Step 3 of embed.rs) to avoid OOM.
- **Stale index**: If the HNSW index is built but new documents are added later, the index is stale. Either rebuild incrementally or mark it as stale and auto-rebuild.
- **Platform-specific SIMD**: `usearch` uses SIMD intrinsics that may not be available on all platforms. Test on x86_64 and aarch64.

---

### P5-E1-T4: HyDE (Hypothetical Document Embeddings)

| Field | Value |
|-------|-------|
| **ID** | `P5-E1-T4` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Implement Hypothetical Document Embeddings (HyDE) for ambiguous queries. When the query intent classifier detects ambiguity (low confidence), generate a hypothetical answer via the local LLM, embed it, and use that embedding for vector search instead of the raw query embedding. This dramatically improves retrieval for vague or underspecified queries. |
| **PRD Ref** | 21 (Query Understanding - HyDE), 22 (HyDE reference) |
| **Depends On** | `P5-E1-T1` (embeddings), `P4-E1` (Ollama / local LLM), `P2-E4` (intent classifier) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/query/hyde.rs` | HyDE implementation |
| `crates/hsx-core/src/query/mod.rs` | Register HyDE module |
| `crates/hsx-core/src/query/intent.rs` | Add ambiguity detection threshold |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/query/hyde.rs

const HYDE_PROMPT_TEMPLATE: &str = r#"Write a short, factual paragraph that would be a perfect answer to this question. Do not include any preamble or meta-commentary. Just write the answer directly.

Question: {query}

Answer:"#;

/// Generate a hypothetical document for the query using local LLM,
/// then embed it for improved retrieval.
pub async fn hyde_embed(
    query: &str,
    ai_client: &crate::ai::AiClient,
) -> Result<Vec<f32>, crate::Error> {
    // Step 1: Generate hypothetical answer
    let prompt = HYDE_PROMPT_TEMPLATE.replace("{query}", query);
    let hypothetical = ai_client
        .complete(&prompt, crate::ai::CompletionParams {
            max_tokens: 200,
            temperature: 0.7,  // slightly creative for diverse hypotheticals
            ..Default::default()
        })
        .await?;

    // Step 2: Embed the hypothetical document (not the query)
    let embedding = crate::embeddings::embed::embed(&hypothetical.text)?;

    tracing::debug!(
        query = query,
        hypothetical_len = hypothetical.text.len(),
        "HyDE generated hypothetical document"
    );

    Ok(embedding)
}

/// Determine whether to use HyDE based on query ambiguity.
/// Returns the best embedding (HyDE or direct query embedding).
pub async fn smart_embed(
    query: &str,
    intent: &crate::query::QueryIntent,
    ai_client: Option<&crate::ai::AiClient>,
) -> Result<Vec<f32>, crate::Error> {
    // Use HyDE when:
    // 1. Intent confidence is low (ambiguous query)
    // 2. AI client is available
    // 3. Query is short (< 6 words — likely underspecified)
    let word_count = query.split_whitespace().count();
    let should_hyde = intent.confidence < 0.6
        && ai_client.is_some()
        && word_count < 6;

    if should_hyde {
        tracing::info!(query = query, "Using HyDE for ambiguous query");
        hyde_embed(query, ai_client.unwrap()).await
    } else {
        crate::embeddings::embed::embed(query).map_err(Into::into)
    }
}
```

#### Acceptance Criteria

- [ ] Ambiguous queries (e.g., "Apple") produce better search results with HyDE than without
- [ ] HyDE is only triggered when intent confidence < 0.6 AND an LLM is available
- [ ] Without an LLM, falls back to direct query embedding silently
- [ ] Hypothetical document generation completes in < 2s with a local 7B model
- [ ] Generated embedding is cached to avoid regeneration on identical queries
- [ ] Unit test: `smart_embed` returns HyDE embedding for short ambiguous queries
- [ ] Unit test: `smart_embed` returns direct embedding for clear specific queries

#### Pitfalls

- **LLM hallucination in HyDE**: The hypothetical answer may contain incorrect facts. This is acceptable because we only use the *embedding* (semantic direction), not the text.
- **Latency**: HyDE adds ~1-2s of LLM inference. Only use it when the query is genuinely ambiguous — never for clear factual queries.
- **Model dependency**: HyDE requires a running LLM. Make this entirely optional and gracefully fall back.

---

## Epic 5.2: QATBE/QADD Semantic Upgrade

### P5-E2-T1: QATBE Semantic Scoring Upgrade

| Field | Value |
|-------|-------|
| **ID** | `P5-E2-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Upgrade the QATBE (Query-Aware Token-Budgeted Extraction) Stage 3 ranking to use the formula from PRD 8.2: `relevance_score = BM25(segment, query) * 0.6 + CosineSim(embed(segment), embed(query)) * 0.4`. Previously QATBE used BM25-only. |
| **PRD Ref** | 8.2 (QATBE Stage 3: RANK), 17 |
| **Depends On** | `P5-E1-T1` (embeddings), `P1-E3` (QATBE basic), `P1-E5` (BM25) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/extraction/qatbe.rs` | Upgrade Stage 3 scoring |

#### Step-by-Step Implementation Guide

```rust
// In crates/hsx-core/src/extraction/qatbe.rs — upgrade the rank_segments function

use crate::ranking::signals::semantic::SemanticSignal;

/// Stage 3 of QATBE: Rank segments by query relevance.
/// Uses hybrid scoring: 0.6 * BM25 + 0.4 * cosine_similarity
pub fn rank_segments(
    segments: &mut [ContentSegment],
    query: &str,
) -> Result<(), crate::Error> {
    // Compute BM25 scores for all segments
    let bm25_scores = compute_bm25_scores(segments, query);

    // Normalize BM25 to 0..1 range
    let bm25_max = bm25_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bm25_min = bm25_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let bm25_range = (bm25_max - bm25_min).max(1e-10);

    // Compute semantic scores (if embeddings available)
    let semantic = SemanticSignal::new(query)?;

    for (i, segment) in segments.iter_mut().enumerate() {
        let bm25_norm = (bm25_scores[i] - bm25_min) / bm25_range;
        let cosine = semantic.score(&segment.text) as f64;

        // PRD formula: 0.6 * BM25 + 0.4 * cosine
        segment.relevance = 0.6 * bm25_norm + 0.4 * cosine;
    }

    // Sort by relevance descending
    segments.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance).unwrap());
    Ok(())
}
```

#### Acceptance Criteria

- [ ] QATBE segments are scored with `0.6 * BM25_normalized + 0.4 * cosine_similarity`
- [ ] When embeddings feature is disabled, scoring degrades to 1.0 * BM25 (no crash)
- [ ] Segment ordering changes compared to BM25-only for semantically relevant but lexically different content
- [ ] Token budget packing (Stage 4) still works correctly with the new scores
- [ ] Benchmark: QATBE with semantic scoring processes a page in < 200ms (excluding model init)

#### Pitfalls

- **BM25 normalization**: BM25 scores can be negative (especially with no matching terms). Clamp to 0 before normalization.
- **Short segments**: Very short segments (< 10 tokens) have noisy embeddings. Consider a minimum segment length threshold.

---

### P5-E2-T2: QADD Semantic Embedding Pruning

| Field | Value |
|-------|-------|
| **ID** | `P5-E2-T2` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Upgrade QADD (Query-Aware DOM Distillation) Step 3 to use real semantic embedding checks. After BM25 text-node scoring, embed remaining DOM nodes and remove those with cosine similarity below threshold. This is the second stage of QADD's pruning pipeline per PRD 8.10. |
| **PRD Ref** | 8.10 (QADD Step 3: Semantic embedding check), 16 (QADD) |
| **Depends On** | `P5-E1-T1` (embeddings), `P2-E5` (QADD basic) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/extraction/qadd.rs` | Add Step 3 semantic pruning |

#### Step-by-Step Implementation Guide

```rust
// In crates/hsx-core/src/extraction/qadd.rs

/// Step 3: Semantic embedding pruning.
/// Embed each surviving DOM node text and the query.
/// Remove nodes with cosine similarity below threshold.
fn semantic_pruning(
    nodes: &mut Vec<DomNode>,
    query: &str,
    threshold: f32,
) -> Result<(), crate::Error> {
    #[cfg(feature = "embeddings")]
    {
        let query_emb = crate::embeddings::embed::embed(query)?;

        // Batch embed all node texts for efficiency
        let texts: Vec<&str> = nodes.iter().map(|n| n.text.as_str()).collect();
        let embeddings = crate::embeddings::embed::embed_batch(&texts)?;

        // Compute cosine similarities and mark for removal
        let mut keep_flags: Vec<bool> = Vec::with_capacity(nodes.len());
        for emb in &embeddings {
            let sim = crate::ranking::signals::semantic::cosine_similarity(&query_emb, emb);
            keep_flags.push(sim >= threshold);
        }

        // Remove nodes below threshold
        let mut i = 0;
        nodes.retain(|_| {
            let keep = keep_flags[i];
            i += 1;
            keep
        });

        tracing::debug!(
            retained = nodes.len(),
            threshold = threshold,
            "QADD semantic pruning complete"
        );
    }

    #[cfg(not(feature = "embeddings"))]
    {
        let _ = (query, threshold);
        tracing::debug!("QADD semantic pruning skipped: embeddings feature disabled");
    }

    Ok(())
}

// In the main QADD pipeline:
pub fn distill(
    dom: &Dom,
    query: &str,
    token_budget: usize,
) -> Result<Vec<DomNode>, crate::Error> {
    let mut nodes = structural_pruning(dom)?;                // Step 1
    bm25_text_node_scoring(&mut nodes, query, 0.1)?;       // Step 2
    semantic_pruning(&mut nodes, query, 0.3)?;              // Step 3 (NEW)
    restore_context(&mut nodes, dom)?;                       // Step 4
    budget_pack(&mut nodes, token_budget)?;                  // Step 5
    Ok(nodes)
}
```

#### Acceptance Criteria

- [ ] QADD Step 3 removes DOM nodes with cosine similarity < 0.3 (configurable threshold)
- [ ] Batch embedding is used for efficiency (not per-node embedding)
- [ ] Feature-gated: when `embeddings` is disabled, Step 3 is skipped and pipeline continues
- [ ] Net token reduction: QADD with semantic pruning achieves ~70% reduction vs BM25-only pruning
- [ ] Context preservation (Step 4) correctly restores parent headings/table headers of surviving nodes
- [ ] End-to-end test: `agent-fetch <url> --query "X" --budget 1000` returns semantically relevant content

#### Pitfalls

- **Too aggressive pruning**: If the threshold is too high, relevant content gets removed. Start at 0.3 and make configurable.
- **Batching overhead**: If a page has 500+ text nodes, batch embedding may use significant memory. Cap batch sizes to 128 and process in chunks.
- **Context loss**: After semantic pruning, surviving nodes may lack context (e.g., a table cell without its header). Step 4 (context preservation) is critical.

---

## Epic 5.3: CEP ML Predictor

### P5-E3-T1: ML Model Predicting Extraction Layer from URL Features

| Field | Value |
|-------|-------|
| **ID** | `P5-E3-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Build an ML classifier that predicts the required CEP extraction layer for a URL *before* attempting extraction. Uses URL features (domain, path patterns, content-type header, known SPA domain list) to predict Layer 1-5. Trains on historical extraction success/failure data stored by PIE. |
| **PRD Ref** | 8.3 (CEP ML Method Predictor), 16 |
| **Depends On** | `P2-E5` (CEP layers 3-5), `P1-E1` (CEP layers 1-2) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/extraction/cep_predictor.rs` | ML predictor implementation |
| `crates/hsx-core/src/extraction/cep_features.rs` | Feature extraction from URLs |
| `crates/hsx-core/src/extraction/cep.rs` | Integrate predictor into CEP cascade |
| `data/spa_domains.txt` | Known SPA domain list |
| `data/cep_training_data.csv` | Initial training data (manual labels) |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/extraction/cep_features.rs

/// Features extracted from a URL for CEP layer prediction.
#[derive(Debug, Clone)]
pub struct CepFeatures {
    pub domain_is_known_spa: bool,       // React/Vue/Angular app
    pub domain_age_known: bool,          // in our SPA list?
    pub path_has_hash_routing: bool,     // /#/ or /#! patterns
    pub content_type_is_html: bool,      // from HEAD request
    pub has_noscript_tag: bool,          // from initial HTML peek
    pub script_tag_count: u32,           // from initial HTML peek
    pub html_size_bytes: u64,
    pub text_to_html_ratio: f32,         // stripped text / raw HTML
    pub has_framework_markers: bool,     // __next, __nuxt, ng-app, etc.
    pub historical_layer: Option<u8>,    // what worked last time (from PIE)
    pub historical_success_rate: f32,    // per domain per layer
}

impl CepFeatures {
    /// Extract features from a URL using a HEAD request + initial HTML peek.
    pub async fn extract(
        url: &str,
        client: &reqwest::Client,
    ) -> Result<Self, crate::Error> {
        // HEAD request for content-type and size
        let head = client.head(url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?;

        let content_type = head.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // Peek first 10KB of HTML for structural analysis
        let peek = client.get(url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?
            .text()
            .await?;
        let peek = &peek[..peek.len().min(10240)];

        let domain = url::Url::parse(url)?
            .domain()
            .unwrap_or("")
            .to_string();

        Ok(Self {
            domain_is_known_spa: SPA_DOMAINS.contains(&domain.as_str()),
            domain_age_known: true,
            path_has_hash_routing: url.contains("/#/") || url.contains("/#!"),
            content_type_is_html: content_type.contains("text/html"),
            has_noscript_tag: peek.contains("<noscript"),
            script_tag_count: peek.matches("<script").count() as u32,
            html_size_bytes: peek.len() as u64,
            text_to_html_ratio: compute_text_ratio(peek),
            has_framework_markers: check_framework_markers(peek),
            historical_layer: None, // filled from PIE when available
            historical_success_rate: 1.0,
        })
    }
}

/// Simple decision tree predictor (no ML framework needed).
/// Can be replaced with ONNX model later.
pub fn predict_layer(features: &CepFeatures) -> u8 {
    // Priority 1: Historical success
    if let Some(layer) = features.historical_layer {
        if features.historical_success_rate > 0.8 {
            return layer;
        }
    }

    // Priority 2: Known SPA domain or framework markers
    if features.domain_is_known_spa || features.has_framework_markers {
        if features.path_has_hash_routing {
            return 4; // Full SPA, needs Playwright + wait
        }
        return 3; // Headless shell usually sufficient
    }

    // Priority 3: Script density heuristic
    if features.script_tag_count > 20 {
        return 3; // Heavy JS, needs headless
    }

    // Priority 4: Text ratio
    if features.text_to_html_ratio > 0.3 {
        return 1; // Mostly static, HTTP + Cheerio
    }

    if features.text_to_html_ratio > 0.15 {
        return 2; // Article page, HTTP + Readability
    }

    // Default: try HTTP + Readability
    2
}
```

#### Acceptance Criteria

- [ ] Predictor correctly routes static HTML pages (Wikipedia, blogs) to Layer 1-2
- [ ] Predictor correctly routes SPAs (React/Angular apps) to Layer 3-4
- [ ] Predictor uses historical data from PIE (when available) for domain-specific routing
- [ ] Prediction latency < 100ms (HEAD request + feature extraction)
- [ ] `>90%` prediction accuracy on a test set of 100 manually labeled URLs
- [ ] CEP skips lower layers when predictor suggests a higher layer, saving time
- [ ] Prediction results are logged for later training data collection
- [ ] Known SPA domain list includes top 200 JS-heavy sites

#### Pitfalls

- **HEAD request failures**: Some servers reject HEAD requests. Fall back to a truncated GET with a range header.
- **Text ratio on empty pages**: SPAs may return a shell with `<div id="root"></div>` resulting in near-zero text ratio. This is actually a strong signal for Layer 3+.
- **Over-reliance on historical data**: If a site redesigns, the historical layer may be wrong. Always fall back to feature-based prediction when historical success rate drops below threshold.

---

## Epic 5.4: Full PDS Tiers

### P5-E4-T1: Detailed + Complete PDS Tiers and Tier Expansion API

| Field | Value |
|-------|-------|
| **ID** | `P5-E4-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Complete the PDS (Progressive Detail Streaming) system with all 4 tiers: `key_facts` (~200 tokens), `summary` (~1000 tokens), `detailed` (~5000 tokens), and `complete` (all tokens). MVP built tiers 0-1. This task adds tiers 2-3 and the tier expansion API that lets agents request more detail without re-fetching. |
| **PRD Ref** | 8.9 (PDS), 27 (Progressive Detail Streaming), 28 (PDS tier cache) |
| **Depends On** | `P1-E3` (PDS tiers 0-1), `P1-E6` (SQLite cache), `P5-E1-T1` (embeddings for relevance filtering) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/pds/tiers.rs` | Add detailed and complete tier generation |
| `crates/hsx-core/src/pds/expansion.rs` | Tier expansion API |
| `crates/hsx-core/src/pds/cache.rs` | SQLite-backed tier cache |
| `crates/hsx-cli/src/commands/expand.rs` | CLI `hsx expand <result_id> --tier detailed` |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/pds/tiers.rs

/// Generate all 4 PDS tiers from extracted content.
/// Called once at extraction time; all tiers cached.
pub fn generate_all_tiers(
    segments: &[ContentSegment],
    query: &str,
) -> Result<PdsTiers, crate::Error> {
    let complete = generate_complete(segments);
    let detailed = generate_detailed(segments, query, 5000)?;
    let summary = generate_summary(segments, query, 1000)?;
    let key_facts = generate_key_facts(segments, query, 200)?;

    Ok(PdsTiers {
        key_facts,
        summary,
        detailed,
        complete,
    })
}

/// Tier 3: COMPLETE — all extracted content, nothing omitted.
fn generate_complete(segments: &[ContentSegment]) -> TierContent {
    TierContent {
        tier: Tier::Complete,
        tokens: segments.iter().map(|s| s.tokens).sum(),
        segments: segments.to_vec(),
    }
}

/// Tier 2: DETAILED — top segments by relevance up to ~5000 tokens.
/// Uses semantic + BM25 ranking to select the most relevant content.
fn generate_detailed(
    segments: &[ContentSegment],
    query: &str,
    budget: usize,
) -> Result<TierContent, crate::Error> {
    let mut ranked = segments.to_vec();
    // Re-use QATBE Stage 3 ranking
    super::super::extraction::qatbe::rank_segments(&mut ranked, query)?;
    // Greedy knapsack packing
    let packed = budget_pack(&ranked, budget);
    Ok(TierContent {
        tier: Tier::Detailed,
        tokens: packed.iter().map(|s| s.tokens).sum(),
        segments: packed,
    })
}

/// Tier 1: SUMMARY — executive summary with key findings, ~1000 tokens.
/// Compresses by selecting top-relevance segments and truncating.
fn generate_summary(
    segments: &[ContentSegment],
    query: &str,
    budget: usize,
) -> Result<TierContent, crate::Error> {
    let mut ranked = segments.to_vec();
    super::super::extraction::qatbe::rank_segments(&mut ranked, query)?;
    // Take only the highest-relevance segments
    let packed = budget_pack(&ranked, budget);
    Ok(TierContent {
        tier: Tier::Summary,
        tokens: packed.iter().map(|s| s.tokens).sum(),
        segments: packed,
    })
}

/// Tier 0: KEY_FACTS — top 5 factual claims, one sentence each.
fn generate_key_facts(
    segments: &[ContentSegment],
    query: &str,
    budget: usize,
) -> Result<TierContent, crate::Error> {
    // Filter to fact-type segments only, then top 5
    let mut facts: Vec<ContentSegment> = segments.iter()
        .filter(|s| matches!(s.seg_type, SegmentType::Fact | SegmentType::Data | SegmentType::Paragraph))
        .cloned()
        .collect();
    super::super::extraction::qatbe::rank_segments(&mut facts, query)?;
    facts.truncate(5);
    let packed = budget_pack(&facts, budget);
    Ok(TierContent {
        tier: Tier::KeyFacts,
        tokens: packed.iter().map(|s| s.tokens).sum(),
        segments: packed,
    })
}
```

**Tier Expansion API:**

```rust
// crates/hsx-core/src/pds/expansion.rs

/// Expand a previous result to a higher detail tier without re-fetching.
/// Uses the cached tier data keyed by result_id.
pub async fn expand_tier(
    result_id: &str,
    target_tier: Tier,
    cache: &PdsCache,
) -> Result<TierContent, crate::Error> {
    let tiers = cache.get_tiers(result_id)?
        .ok_or_else(|| crate::Error::NotFound(format!(
            "Result {result_id} not found in PDS cache. It may have expired."
        )))?;

    match target_tier {
        Tier::KeyFacts => Ok(tiers.key_facts),
        Tier::Summary => Ok(tiers.summary),
        Tier::Detailed => Ok(tiers.detailed),
        Tier::Complete => Ok(tiers.complete),
    }
}
```

#### Acceptance Criteria

- [ ] All 4 tiers generated at extraction time and cached in SQLite
- [ ] `hsx agent-search "query" --tier key_facts` returns ~200 tokens
- [ ] `hsx agent-search "query" --tier detailed` returns ~5000 tokens
- [ ] `hsx agent-search "query" --tier complete` returns all extracted content
- [ ] Tier expansion via `expand_tier(result_id, "detailed")` returns cached tier instantly (< 5ms)
- [ ] Result IDs are UUIDs persisted across requests within cache TTL
- [ ] PDS cache eviction respects configured max_size
- [ ] Each tier's token count is accurate (within 5% of target)

#### Pitfalls

- **Token counting accuracy**: Use the same tokenizer (tiktoken-rs or cl100k_base approximation) consistently across all tier generation. Off-by-one errors compound.
- **Cache key collisions**: Use `SHA256(query + url + extraction_params)` as the cache key, not just the URL.
- **Tier staleness**: If the source page changes, cached tiers are stale. Include a `fetched_at` timestamp and honor cache TTL.

---

## Epic 5.5: Compare + Monitor Commands

### P5-E5-T1: Comparison Research Command

| Field | Value |
|-------|-------|
| **ID** | `P5-E5-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Implement `hsx compare "A vs B vs C"` that researches each item in parallel, extracts comparison dimensions, and generates a structured comparison table with per-cell citations. |
| **PRD Ref** | 10 (Mode F: Compare Mode) |
| **Depends On** | `P3-E3` (research mode), `P2-E3` (parallel orchestrator) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/compare/mod.rs` | Comparison engine |
| `crates/hsx-core/src/compare/parser.rs` | Parse "A vs B vs C" queries |
| `crates/hsx-core/src/compare/dimension.rs` | Extract comparison dimensions |
| `crates/hsx-core/src/compare/table.rs` | Generate comparison table |
| `crates/hsx-cli/src/commands/compare.rs` | CLI command |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/compare/parser.rs

/// Parse a comparison query into individual items.
pub fn parse_comparison_query(query: &str) -> ComparisonQuery {
    // Patterns: "A vs B", "A versus B", "A or B", "A compared to B", "compare A B C"
    let separators = [" vs ", " versus ", " or ", " compared to ", " vs. "];
    let mut items: Vec<String> = vec![query.to_string()];

    for sep in &separators {
        if query.contains(sep) {
            items = query.split(sep)
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            break;
        }
    }

    ComparisonQuery {
        items,
        raw_query: query.to_string(),
    }
}

// crates/hsx-core/src/compare/mod.rs

/// Run comparison research for multiple items.
pub async fn compare(
    query: &str,
    config: &CompareConfig,
) -> Result<ComparisonResult, crate::Error> {
    let parsed = parser::parse_comparison_query(query);

    // Research each item in parallel
    let mut handles = Vec::new();
    for item in &parsed.items {
        let search_query = format!("{} features capabilities", item);
        let handle = tokio::spawn(async move {
            crate::research::run_research(&search_query, &Default::default()).await
        });
        handles.push((item.clone(), handle));
    }

    let mut item_research = Vec::new();
    for (item, handle) in handles {
        let result = handle.await??;
        item_research.push((item, result));
    }

    // Extract comparison dimensions from all research
    let dimensions = dimension::extract_dimensions(&item_research)?;

    // Build comparison table
    let table = table::build_comparison_table(&parsed.items, &dimensions, &item_research)?;

    Ok(ComparisonResult { query: parsed, table, sources: collect_sources(&item_research) })
}
```

#### Acceptance Criteria

- [ ] `hsx compare "React vs Vue vs Svelte"` produces a comparison table
- [ ] Each cell in the table has a citation reference
- [ ] Items are researched in parallel (visible in `--profile` output)
- [ ] Output formats: markdown table, JSON array of dimension objects, segments
- [ ] At least 5 comparison dimensions auto-extracted (performance, learning curve, ecosystem, etc.)
- [ ] `--format json` produces structured comparison data

#### Pitfalls

- **Dimension alignment**: Different sources describe the same dimension differently ("speed" vs "performance" vs "benchmarks"). Use semantic similarity to cluster dimension names.
- **Missing data**: Not all items will have data for all dimensions. Output "N/A" with explanation rather than leaving cells empty.

---

### P5-E5-T2: Change Detection / Monitor Command

| Field | Value |
|-------|-------|
| **ID** | `P5-E5-T2` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Implement `hsx monitor <url>` for content-hash-based change detection with diff output and optional notifications. Supports interval-based polling and query-based monitoring. |
| **PRD Ref** | 10 (Mode G: Monitor Mode) |
| **Depends On** | `P1-E1` (HTTP fetch), `P1-E6` (SQLite cache for snapshots) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/monitor/mod.rs` | Monitor engine |
| `crates/hsx-core/src/monitor/snapshot.rs` | Content snapshot + hash storage |
| `crates/hsx-core/src/monitor/diff.rs` | Content diff computation |
| `crates/hsx-core/src/monitor/scheduler.rs` | Polling scheduler |
| `crates/hsx-cli/src/commands/monitor.rs` | CLI command |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/monitor/snapshot.rs

pub struct SnapshotStore {
    conn: rusqlite::Connection,
}

impl SnapshotStore {
    pub fn new() -> Result<Self, crate::Error> {
        let path = crate::config::data_dir().join("monitor.db");
        let conn = rusqlite::Connection::open(&path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                content TEXT NOT NULL,
                fetched_at TEXT DEFAULT (datetime('now')),
                UNIQUE(url, content_hash)
            );
            CREATE TABLE IF NOT EXISTS monitors (
                url TEXT PRIMARY KEY,
                interval_secs INTEGER NOT NULL,
                last_checked TEXT,
                notify_method TEXT
            );"
        )?;
        Ok(Self { conn })
    }

    pub fn save_snapshot(&self, url: &str, content: &str) -> Result<bool, crate::Error> {
        let hash = sha256_hex(content);
        // Check if content has changed
        let last_hash: Option<String> = self.conn.query_row(
            "SELECT content_hash FROM snapshots WHERE url = ?1 ORDER BY id DESC LIMIT 1",
            [url],
            |row| row.get(0),
        ).optional()?;

        let changed = last_hash.as_deref() != Some(&hash);
        if changed {
            self.conn.execute(
                "INSERT INTO snapshots (url, content_hash, content) VALUES (?1, ?2, ?3)",
                rusqlite::params![url, hash, content],
            )?;
        }
        Ok(changed)
    }
}

// crates/hsx-core/src/monitor/diff.rs

/// Compute a human-readable diff between two content versions.
pub fn compute_diff(old: &str, new: &str) -> ContentDiff {
    let diff = similar::TextDiff::from_lines(old, new);
    let mut additions = 0;
    let mut deletions = 0;
    let mut changes = Vec::new();

    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Insert => {
                additions += 1;
                changes.push(DiffLine::Added(change.to_string()));
            }
            similar::ChangeTag::Delete => {
                deletions += 1;
                changes.push(DiffLine::Removed(change.to_string()));
            }
            similar::ChangeTag::Equal => {}
        }
    }

    ContentDiff {
        additions,
        deletions,
        similarity: diff.ratio(),
        changes,
    }
}
```

#### Acceptance Criteria

- [ ] `hsx monitor <url> --interval 1h` registers a URL for periodic checking
- [ ] `hsx monitor <url> --diff` shows changes since last snapshot
- [ ] Content hash based change detection (SHA-256)
- [ ] Human-readable diff output with additions/deletions highlighted
- [ ] `hsx monitor list` shows all monitored URLs with last-check time
- [ ] `hsx monitor remove <url>` stops monitoring
- [ ] Snapshots persist in SQLite across sessions
- [ ] `--notify webhook` sends POST request on change detection

#### Pitfalls

- **Dynamic content**: Pages with timestamps, ads, or session tokens will always "change." Strip volatile content (dates, random IDs) before hashing.
- **Storage growth**: Each snapshot stores full content. Implement retention limits (e.g., keep last 10 snapshots per URL).
- **Background polling**: The scheduler must run as a background process or daemon. Use `tokio::time::interval` for in-process polling.

---

## Epic 5.6: PDF/DOCX Export

### P5-E6-T1: PDF and DOCX Export via Pandoc

| Field | Value |
|-------|-------|
| **ID** | `P5-E6-T1` |
| **Status** | `TODO` |
| **Priority** | P2 |
| **Description** | Add PDF and DOCX export formats by invoking Pandoc as an external process. Also add BibTeX citation export for academic use cases. Pandoc is an optional external dependency; provide clear error message if not installed. |
| **PRD Ref** | 26 (Output & Export System), 48 (Pandoc external dependency) |
| **Depends On** | `P1-E7` (output formatters), `P3-E2` (citation system) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/export/pandoc.rs` | Pandoc subprocess invocation |
| `crates/hsx-core/src/export/bibtex.rs` | BibTeX citation export |
| `crates/hsx-core/src/export/mod.rs` | Register new export formats |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/export/pandoc.rs
use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

/// Check if Pandoc is available on the system.
pub fn check_pandoc() -> Result<String, crate::Error> {
    let output = Command::new("pandoc")
        .arg("--version")
        .output()
        .map_err(|_| crate::Error::ExternalTool(
            "Pandoc not found. Install it from https://pandoc.org/installing.html".into()
        ))?;
    let version = String::from_utf8_lossy(&output.stdout);
    let first_line = version.lines().next().unwrap_or("unknown");
    Ok(first_line.to_string())
}

/// Export markdown content to PDF via Pandoc.
pub fn export_pdf(
    markdown: &str,
    output_path: &std::path::Path,
    title: Option<&str>,
) -> Result<(), crate::Error> {
    check_pandoc()?;

    let mut tmp = NamedTempFile::with_suffix(".md")?;
    // Add YAML frontmatter for Pandoc metadata
    if let Some(title) = title {
        writeln!(tmp, "---")?;
        writeln!(tmp, "title: \"{}\"", title)?;
        writeln!(tmp, "date: \"{}\"", chrono::Utc::now().format("%Y-%m-%d"))?;
        writeln!(tmp, "geometry: margin=1in")?;
        writeln!(tmp, "---\n")?;
    }
    write!(tmp, "{}", markdown)?;
    tmp.flush()?;

    let status = Command::new("pandoc")
        .arg(tmp.path())
        .args(["-o", &output_path.to_string_lossy()])
        .args(["--pdf-engine", "xelatex"]) // fallback: tectonic, pdflatex
        .arg("--standalone")
        .arg("--toc") // table of contents
        .status()?;

    if !status.success() {
        // Try without xelatex (fall back to default engine)
        let status = Command::new("pandoc")
            .arg(tmp.path())
            .args(["-o", &output_path.to_string_lossy()])
            .arg("--standalone")
            .status()?;

        if !status.success() {
            return Err(crate::Error::ExternalTool("Pandoc PDF generation failed".into()));
        }
    }

    Ok(())
}

/// Export markdown content to DOCX via Pandoc.
pub fn export_docx(
    markdown: &str,
    output_path: &std::path::Path,
) -> Result<(), crate::Error> {
    check_pandoc()?;

    let mut tmp = NamedTempFile::with_suffix(".md")?;
    write!(tmp, "{}", markdown)?;
    tmp.flush()?;

    let status = Command::new("pandoc")
        .arg(tmp.path())
        .args(["-o", &output_path.to_string_lossy()])
        .arg("--standalone")
        .status()?;

    if !status.success() {
        return Err(crate::Error::ExternalTool("Pandoc DOCX generation failed".into()));
    }

    Ok(())
}

// crates/hsx-core/src/export/bibtex.rs

/// Generate BibTeX entries from sources.
pub fn generate_bibtex(sources: &[crate::types::Source]) -> String {
    let mut bib = String::new();
    for (i, source) in sources.iter().enumerate() {
        let key = format!("source{}", i + 1);
        let entry_type = if source.url.contains("arxiv") {
            "article"
        } else {
            "misc"
        };
        bib.push_str(&format!("@{}{{{},\n", entry_type, key));
        bib.push_str(&format!("  title = {{{}}},\n", source.title));
        bib.push_str(&format!("  url = {{{}}},\n", source.url));
        if let Some(author) = &source.author {
            bib.push_str(&format!("  author = {{{}}},\n", author));
        }
        if let Some(date) = &source.published_date {
            bib.push_str(&format!("  year = {{{}}},\n", date.format("%Y")));
        }
        bib.push_str(&format!("  note = {{Accessed: {}}},\n", chrono::Utc::now().format("%Y-%m-%d")));
        bib.push_str("}\n\n");
    }
    bib
}
```

#### Acceptance Criteria

- [ ] `hsx research "topic" --format pdf --output report.pdf` generates a PDF with title, TOC, and citations
- [ ] `hsx research "topic" --format docx --output report.docx` generates a Word document
- [ ] `hsx research "topic" --format bibtex --output refs.bib` generates valid BibTeX entries
- [ ] Clear error message when Pandoc is not installed (with install link)
- [ ] PDF includes table of contents when research report has headings
- [ ] BibTeX entries have proper keys, URLs, titles, and access dates
- [ ] Works on macOS, Linux, and Windows (Pandoc is cross-platform)

#### Pitfalls

- **PDF engine availability**: `xelatex` is not always installed. Fall back to `pdflatex` or `tectonic`. Pandoc's `--pdf-engine` flag controls this.
- **Unicode in PDF**: Standard pdflatex may fail on Unicode characters. xelatex handles Unicode natively. Recommend xelatex.
- **Large reports**: Very long reports may take 10+ seconds to render as PDF. Show a progress indicator.
- **BibTeX special characters**: Escape `&`, `%`, `$`, `#`, `_`, `{`, `}` in BibTeX fields.
