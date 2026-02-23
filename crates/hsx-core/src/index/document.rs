//! Indexed document schema for the local hybrid search index (Phase 5, PRD §28).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A document stored in the local index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDocument {
    /// Unique numeric ID (used as key in the HNSW vector index).
    pub id: u64,
    /// Source URL.
    pub url: String,
    /// Page title.
    pub title: String,
    /// Full extracted text content.
    pub content: String,
    /// Registered domain (for dedup and authority scoring).
    pub domain: String,
    /// When this document was indexed.
    pub fetched_at: DateTime<Utc>,
    /// SHA-256 of the extracted content (change-detection).
    pub content_hash: String,
    /// Pre-computed embedding, present after `hsx index build`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl IndexedDocument {
    /// Check if this document has an embedding.
    pub fn is_embedded(&self) -> bool {
        self.embedding.is_some()
    }
}

/// Summary statistics for the local index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    /// Total number of indexed documents.
    pub document_count: usize,
    /// Number of documents with embeddings.
    pub embedded_count: usize,
    /// Approximate size of the index on disk in bytes.
    pub index_size_bytes: u64,
    /// Whether the HNSW vector index is built and ready.
    pub vector_index_ready: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_embedded_returns_correct_values() {
        let mut doc = IndexedDocument {
            id: 1,
            url: "https://example.com".into(),
            title: "Test".into(),
            content: "content".into(),
            domain: "example.com".into(),
            fetched_at: Utc::now(),
            content_hash: "abc".into(),
            embedding: None,
        };
        assert!(!doc.is_embedded());
        doc.embedding = Some(vec![0.0; 384]);
        assert!(doc.is_embedded());
    }
}
