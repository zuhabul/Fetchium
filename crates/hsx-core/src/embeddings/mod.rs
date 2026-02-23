//! Semantic embeddings — all-MiniLM-L6-v2 via fastembed-rs (Phase 5, PRD §21).
//!
//! Feature-gated behind `embeddings`. Enable with:
//! ```bash
//! cargo build -p hsx-core --features embeddings
//! ```
//!
//! ## Pipeline
//! `Text → fastembed (ort v2 + tokenizers) → Vec<f32> (384-dim, L2-normalised) → SQLite cache`

/// Embedding dimension for all-MiniLM-L6-v2.
pub const EMBEDDING_DIM: usize = 384;

/// Cosine similarity between two equal-length embedding vectors.
/// Returns a value in `[-1.0, 1.0]`.
/// For L2-normalised vectors (as produced by fastembed), this equals the dot product.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    debug_assert_eq!(a.len(), b.len(), "embedding dimension mismatch");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

// ── Feature-gated subsystems ─────────────────────────────────────

#[cfg(feature = "embeddings")]
pub mod engine;

#[cfg(feature = "embeddings")]
pub mod cache;

#[cfg(feature = "embeddings")]
pub use engine::{embed, embed_batch};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0_f32, 0.0, 0.0];
        assert!((cosine_similarity(&v, &v) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0_f32, 0.0, 0.0];
        let b = vec![0.0_f32, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 1e-6);
    }

    #[test]
    fn cosine_zero_vector_returns_zero() {
        let a = vec![0.0_f32, 0.0, 0.0];
        let b = vec![1.0_f32, 2.0, 3.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn cosine_normalized_vectors_dot_product() {
        // For L2-normalised vectors, cosine = dot product
        let a = vec![1.0_f32 / 2f32.sqrt(), 1.0_f32 / 2f32.sqrt()];
        let b = vec![1.0_f32 / 2f32.sqrt(), 1.0_f32 / 2f32.sqrt()];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-5);
    }
}
