//! # Fetchium Core Library
//!
//! AI-native search engine for humans and agents.
//!
//! `fetchium-core` contains all algorithms, pipelines, and data structures that power the
//! `fetchium` CLI and MCP/REST API servers. Everything in this crate is designed for both
//! interactive use (humans) and programmatic use (AI agents).
//!
//! ## Architecture
//!
//! The core pipeline runs in this order:
//!
//! ```text
//! Search → Extract (CEP) → Rank (HyperFusion) → Token (QATBE/SCS) →
//! Validate (6 layers) → Citation → Output (markdown/JSON/segments)
//!                                    ↑
//!                  Research (AMRS) ──┘
//! ```
//!
//! ## Key Algorithms (PRD §8)
//!
//! | Algorithm | Description | PRD |
//! |-----------|-------------|-----|
//! | **CEP** | Content Extraction Protocol — 5-layer cascade | §16 |
//! | **QATBE** | Query-Aware Token-Budgeted Extraction — BM25 + semantic | §17 |
//! | **SCS** | Semantic Content Segmentation — 8 segment types | §18 |
//! | **PDS** | Progressive Detail Streaming — 4 tiers | §19 |
//! | **HyperFusion** | 8-signal ranking: BM25 + semantic + temporal + authority | §21 |
//! | **QADD** | Query-Aware DOM Distillation — 5-step DOM pruning | §15 |
//! | **AMRS** | Adaptive Multi-Agent Research Swarm | §24 |
//! | **PIE** | Persistent Intelligence Engine — cross-session learning | §32 |
//! | **RAR** | Retry-and-Refine — 5-checkpoint self-correction | §44 |
//!
//! ## Quick Example
//!
//! ```rust
//! use fetchium_core::rank::rerank;
//! use fetchium_core::types::{BackendId, ResultItem};
//!
//! // Build some search results
//! let mut results = vec![
//!     ResultItem {
//!         title: "Rust Programming Language".into(),
//!         url: "https://rust-lang.org".into(),
//!         snippet: "A language empowering everyone to build reliable software.".into(),
//!         rank: 0,
//!         backend: BackendId::DuckDuckGo,
//!         score: None,
//!         published_date: None,
//!     },
//!     ResultItem {
//!         title: "Python is great".into(),
//!         url: "https://python.org".into(),
//!         snippet: "Python is a versatile scripting language.".into(),
//!         rank: 1,
//!         backend: BackendId::DuckDuckGo,
//!         score: None,
//!         published_date: None,
//!     },
//! ];
//!
//! // Re-rank with BM25 — the Rust result should score higher for "rust"
//! let ranked = rerank(results, "rust programming language");
//! assert_eq!(ranked[0].url, "https://rust-lang.org");
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Dependency | What it enables |
//! |---------|-----------|-----------------|
//! | `embeddings` | `fastembed` | Semantic scoring, hybrid QATBE, batch embedding |
//! | `vector-search` | `usearch` | HNSW approximate nearest-neighbor index |
//! | `headless` | `chromiumoxide` | CEP layers 3+5, JS rendering, screenshot OCR |
//! | `mcp` | `rmcp` | MCP protocol support |
//!
//! ## Prelude
//!
//! Import the most commonly used items in one line:
//!
//! ```rust
//! use fetchium_core::prelude::*;
//! ```

pub mod api_facade;
pub mod config;
pub mod error;
pub mod types;

// Core pipeline modules
pub mod cache;
pub mod extract;
pub mod http;
pub mod index;
pub mod output;
pub mod rank;
pub mod resource;
pub mod search;
pub mod token;

// Phase 2: QADD and browser pool
pub mod browser;
pub mod qadd;

// Phase 3+
pub mod citation;
pub mod validate;

// Phase 4+
pub mod ai;
pub mod research;

// Phase 5+ (feature-gated optional subsystems)
#[cfg(feature = "embeddings")]
pub mod embeddings;

// Always-available Phase 5 modules
pub mod compare;
pub mod export;
pub mod monitor;
pub mod query;

// Summarization pipeline
pub mod summarize;

// Phase 6+
pub mod intelligence;

// Resilience layer — circuit breakers, adaptive rate limiting, bulkhead isolation
pub mod resilience;

// Telemetry — pipeline observability, metrics, health monitoring
pub mod telemetry;

// Phase 7+
pub mod collab;
pub mod domain;
pub mod evolve;
pub mod multimodal;
pub mod plugin;
pub mod privacy;
pub mod proactive;
pub mod proxy;

// Environment setup utilities (Chromium download, path resolution, checks)
pub mod setup;

// YouTube Intelligence System
pub mod youtube;

// Social Media Intelligence System (Twitter/X, Reddit, TikTok, HackerNews + unified)
pub mod social;

#[cfg(test)]
pub mod test_utils;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::config::HsxConfig;
    pub use crate::error::{HsxError, HsxResult};
    pub use crate::rank::{detect_intent, hyperfusion_rank, QueryIntent};
    pub use crate::types::*;
}
