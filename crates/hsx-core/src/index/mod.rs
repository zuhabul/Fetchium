//! Local hybrid search index — Tantivy BM25 + HNSW vector index (Phase 5, PRD §28).
//!
//! ## Architecture
//! - `store.rs` — SQLite document metadata (always available)
//! - `vector.rs` — HNSW vector index via usearch [`vector-search` feature]
//! - `document.rs` — IndexedDocument schema
//!
//! ## Usage
//! ```bash
//! hsx index add <url>          # fetch + extract + embed + store
//! hsx index build              # (re-)compute embeddings for all stored docs
//! hsx index search "query"     # search local index
//! hsx index stats              # show document count, embedding coverage
//! ```

pub mod document;
pub mod store;

#[cfg(feature = "vector-search")]
pub mod vector;

pub use document::{IndexedDocument, IndexStats};
pub use store::DocumentStore;

#[cfg(feature = "vector-search")]
pub use vector::VectorIndex;

/// Reciprocal Rank Fusion score for combining BM25 and vector results.
///
/// `score = sum(1 / (k + rank_i))` where k = 60 (standard constant).
pub fn rrf_score(ranks: &[usize]) -> f64 {
    const K: f64 = 60.0;
    ranks.iter().map(|&r| 1.0 / (K + r as f64 + 1.0)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_single_rank_1() {
        // rank 0 (1st position) gives 1/(60+1) ≈ 0.0164
        let score = rrf_score(&[0]);
        assert!((score - 1.0 / 61.0).abs() < 1e-9);
    }

    #[test]
    fn rrf_two_top_ranks_higher_than_one() {
        let score_two = rrf_score(&[0, 0]);
        let score_one = rrf_score(&[0]);
        assert!(score_two > score_one);
    }
}
