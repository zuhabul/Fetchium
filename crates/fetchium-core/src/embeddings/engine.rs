//! fastembed-rs embedding engine — all-MiniLM-L6-v2 inference (Phase 5, PRD §21).
//!
//! Uses a `OnceCell`-backed singleton to initialise the ONNX session exactly once.
//! On first call, fastembed downloads the model (~90 MB) to:
//!   `~/.cache/fastembed_cache/` (default) or `$FETCHIUM_MODEL_DIR` if set.
//!
//! All output vectors are L2-normalised to unit length by fastembed.

use crate::error::HsxError;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use once_cell::sync::OnceCell;
use std::path::PathBuf;
use tracing::{debug, info};

/// Cached embedding engine singleton (initialised once per process).
static ENGINE: OnceCell<TextEmbedding> = OnceCell::new();

/// Directory where fastembed caches downloaded models.
fn model_cache_dir() -> PathBuf {
    std::env::var("FETCHIUM_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("fastembed_cache")
        })
}

/// Return (or lazily initialise) the embedding engine singleton.
fn get_engine() -> Result<&'static TextEmbedding, HsxError> {
    ENGINE.get_or_try_init(|| {
        info!("Initialising all-MiniLM-L6-v2 embedding model (first call)…");
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                .with_show_download_progress(true)
                .with_cache_dir(model_cache_dir()),
        )
        .map_err(|e| HsxError::Internal(format!("Failed to load embedding model: {e}")))?;
        info!("Embedding model loaded successfully.");
        Ok(model)
    })
}

/// Embed a single text into a 384-dimensional unit-norm f32 vector.
pub fn embed(text: &str) -> Result<Vec<f32>, HsxError> {
    let results = embed_batch(&[text])?;
    Ok(results
        .into_iter()
        .next()
        .unwrap_or_else(|| vec![0.0_f32; super::EMBEDDING_DIM]))
}

/// Embed a batch of texts. Returns one `Vec<f32>` per input.
///
/// Batching amortises ONNX session overhead — prefer this for multiple texts.
pub fn embed_batch(texts: &[&str]) -> Result<Vec<Vec<f32>>, HsxError> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    let engine = get_engine()?;
    debug!("Embedding batch of {} texts", texts.len());
    engine
        .embed(texts.to_vec(), None)
        .map_err(|e| HsxError::Internal(format!("Embedding inference failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// These tests require the model to be downloaded (internet + ~90MB).
    /// Marked `#[ignore]` so they don't run in standard CI.

    #[test]
    #[ignore = "requires model download"]
    fn embed_returns_correct_dimension() {
        let v = embed("hello world").unwrap();
        assert_eq!(v.len(), super::super::EMBEDDING_DIM);
    }

    #[test]
    #[ignore = "requires model download"]
    fn embed_batch_correct_count() {
        let texts = ["rust programming", "python scripting", "machine learning"];
        let results = embed_batch(&texts).unwrap();
        assert_eq!(results.len(), 3);
        for v in &results {
            assert_eq!(v.len(), super::super::EMBEDDING_DIM);
        }
    }

    #[test]
    #[ignore = "requires model download"]
    fn embeddings_are_unit_norm() {
        let v = embed("test").unwrap();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "norm={norm}");
    }
}
