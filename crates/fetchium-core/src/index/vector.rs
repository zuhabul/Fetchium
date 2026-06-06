//! HNSW vector index wrapper — usearch v2 (Phase 5, PRD §28).
//!
//! Feature-gated behind `vector-search`. Enable with:
//! ```bash
//! cargo build -p fetchium-core --features vector-search
//! ```
//!
//! Uses SIMD-accelerated approximate nearest neighbour search with cosine distance.
//! Index parameters follow PRD §21: M=16, ef_construction=128, ef_search=64.

use crate::error::FetchiumError;
use std::path::Path;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

/// HNSW vector index backed by usearch.
pub struct VectorIndex {
    index: Index,
    dimension: usize,
}

impl VectorIndex {
    /// Create a new empty index for vectors of `dimension` dimensions.
    pub fn new(dimension: usize) -> Result<Self, FetchiumError> {
        let options = IndexOptions {
            dimensions: dimension,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            connectivity: 16,     // M — graph edges per node
            expansion_add: 128,   // ef_construction
            expansion_search: 64, // ef_search
            multi: false,
        };
        let index = Index::new(&options)
            .map_err(|e| FetchiumError::Internal(format!("Failed to create HNSW index: {e}")))?;
        Ok(Self { index, dimension })
    }

    /// Load index from disk, or create a new empty one if the file doesn't exist.
    pub fn load_or_new(path: &Path, dimension: usize) -> Result<Self, FetchiumError> {
        let idx = Self::new(dimension)?;
        if path.exists() {
            let path_str = path
                .to_str()
                .ok_or_else(|| FetchiumError::Internal("Invalid path string".into()))?;
            idx.index
                .load(path_str)
                .map_err(|e| FetchiumError::Internal(format!("Failed to load HNSW index: {e}")))?;
        }
        Ok(idx)
    }

    /// Persist the index to disk.
    pub fn save(&self, path: &Path) -> Result<(), FetchiumError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let path_str = path
            .to_str()
            .ok_or_else(|| FetchiumError::Internal("Invalid path string".into()))?;
        self.index
            .save(path_str)
            .map_err(|e| FetchiumError::Internal(format!("Failed to save HNSW index: {e}")))
    }

    /// Reserve capacity for `n` additional vectors.
    pub fn reserve(&self, n: usize) -> Result<(), FetchiumError> {
        self.index
            .reserve(n)
            .map_err(|e| FetchiumError::Internal(format!("Failed to reserve index capacity: {e}")))
    }

    /// Add a document vector with the given numeric `id`.
    pub fn add(&self, id: u64, vector: &[f32]) -> Result<(), FetchiumError> {
        assert_eq!(
            vector.len(),
            self.dimension,
            "vector dimension mismatch: expected {}, got {}",
            self.dimension,
            vector.len()
        );
        self.index
            .add(id, vector)
            .map_err(|e| FetchiumError::Internal(format!("Failed to add vector to index: {e}")))
    }

    /// Search for the `k` nearest neighbours. Returns `(id, distance)` pairs.
    pub fn search(&self, query_vector: &[f32], k: usize) -> Result<Vec<(u64, f32)>, FetchiumError> {
        let results = self
            .index
            .search(query_vector, k)
            .map_err(|e| FetchiumError::Internal(format!("HNSW search failed: {e}")))?;
        Ok(results.keys.into_iter().zip(results.distances).collect())
    }

    /// Number of vectors currently stored.
    pub fn len(&self) -> usize {
        self.index.size()
    }

    /// Returns `true` if the index contains no vectors.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_index_is_empty() {
        let idx = VectorIndex::new(4).unwrap();
        assert!(idx.is_empty());
    }

    #[test]
    fn add_and_search() {
        let idx = VectorIndex::new(4).unwrap();
        idx.reserve(10).unwrap();
        idx.add(1, &[1.0, 0.0, 0.0, 0.0]).unwrap();
        idx.add(2, &[0.0, 1.0, 0.0, 0.0]).unwrap();
        idx.add(3, &[0.0, 0.0, 1.0, 0.0]).unwrap();
        assert_eq!(idx.len(), 3);

        // Query vector similar to id=1
        let results = idx.search(&[0.9, 0.1, 0.0, 0.0], 1).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 1);
    }
}
