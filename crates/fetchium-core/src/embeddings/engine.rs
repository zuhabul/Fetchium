//! External embedding engine via Ollama HTTP API (nomic-embed-text).
//!
//! Zero RAM overhead in Fetchium — delegates to Ollama running as a separate service.
//!
//! Model: nomic-embed-text (768-dim, L2-normalised)
//! Latency: ~76ms warm for 5 texts, ~1.4s cold start

use crate::error::HsxError;
use once_cell::sync::Lazy;
use tracing::{debug, warn};

/// Ollama endpoint (configurable via env var).
static OLLAMA_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string())
});

/// Embedding model to use.
const EMBED_MODEL: &str = "nomic-embed-text";

/// Shared async HTTP client (connection pooling).
static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .pool_max_idle_per_host(4)
        .build()
        .expect("Failed to create HTTP client")
});

/// Embed a single text (sync — uses blocking HTTP, safe from any context).
pub fn embed(text: &str) -> Result<Vec<f32>, HsxError> {
    let results = embed_batch(&[text])?;
    Ok(results
        .into_iter()
        .next()
        .unwrap_or_else(|| vec![0.0_f32; super::EMBEDDING_DIM]))
}

/// Embed a batch of texts (sync — blocking HTTP, safe from any context).
pub fn embed_batch(texts: &[&str]) -> Result<Vec<Vec<f32>>, HsxError> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    debug!("Embedding batch of {} texts via Ollama (blocking)", texts.len());
    let url = format!("{}/api/embed", &*OLLAMA_URL);
    let body = serde_json::json!({
        "model": EMBED_MODEL,
        "input": texts,
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| HsxError::Internal(format!("HTTP client error: {e}")))?;

    let resp = client.post(&url).json(&body).send().map_err(|e| {
        warn!("Ollama embedding request failed: {e}");
        HsxError::Internal(format!("Ollama embed request failed: {e}"))
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(HsxError::Internal(format!("Ollama embed returned {status}: {body}")));
    }

    let data: serde_json::Value = resp
        .json()
        .map_err(|e| HsxError::Internal(format!("Ollama response parse error: {e}")))?;
    parse_ollama_response(&data)
}

/// Async embed a single text.
pub async fn embed_async(text: &str) -> Result<Vec<f32>, HsxError> {
    let results = embed_batch_async(&[text]).await?;
    Ok(results
        .into_iter()
        .next()
        .unwrap_or_else(|| vec![0.0_f32; super::EMBEDDING_DIM]))
}

/// Async embed a batch of texts via Ollama's /api/embed endpoint.
pub async fn embed_batch_async(texts: &[&str]) -> Result<Vec<Vec<f32>>, HsxError> {
    if texts.is_empty() {
        return Ok(Vec::new());
    }
    debug!("Embedding batch of {} texts via Ollama (async)", texts.len());
    let url = format!("{}/api/embed", &*OLLAMA_URL);
    let body = serde_json::json!({
        "model": EMBED_MODEL,
        "input": texts,
    });

    let resp = HTTP_CLIENT.post(&url).json(&body).send().await.map_err(|e| {
        warn!("Ollama embedding request failed: {e}");
        HsxError::Internal(format!("Ollama embed request failed: {e}"))
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(HsxError::Internal(format!("Ollama embed returned {status}: {body}")));
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| HsxError::Internal(format!("Ollama response parse error: {e}")))?;
    parse_ollama_response(&data)
}

/// Parse Ollama /api/embed JSON response.
fn parse_ollama_response(data: &serde_json::Value) -> Result<Vec<Vec<f32>>, HsxError> {
    let embeddings = data["embeddings"]
        .as_array()
        .ok_or_else(|| HsxError::Internal("Ollama response missing 'embeddings' array".into()))?;

    let result: Vec<Vec<f32>> = embeddings
        .iter()
        .map(|emb| {
            emb.as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect()
        })
        .collect();

    debug!(
        "Ollama returned {} embeddings of dim {}",
        result.len(),
        result.first().map(|v| v.len()).unwrap_or(0)
    );
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires running Ollama with nomic-embed-text"]
    fn embed_returns_correct_dimension() {
        let v = embed("hello world").unwrap();
        assert_eq!(v.len(), crate::embeddings::EMBEDDING_DIM);
    }

    #[test]
    #[ignore = "requires running Ollama with nomic-embed-text"]
    fn embed_batch_correct_count() {
        let texts = ["rust programming", "python scripting", "machine learning"];
        let results = embed_batch(&texts).unwrap();
        assert_eq!(results.len(), 3);
    }
}
