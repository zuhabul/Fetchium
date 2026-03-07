use crate::cache::MemoryCache;
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::extract::pipeline::extract as cep_extract;
use crate::http::client::HttpClient;
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use serde_json::{json, Value};
use std::time::Instant;
use uuid::Uuid;

/// Execute a search pipeline: Orchestrator -> Extraction -> Format -> Cache
pub async fn search(
    query: &str,
    max_sources: u32,
    tier: &str,
    token_budget: usize,
    config: &HsxConfig,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Result<Value, HsxError> {
    let start = Instant::now();

    let orch_config = OrchestratorConfig::from_fetchium_config(config, max_sources);
    let orchestrator = SearchOrchestrator::new(http.clone(), orch_config);
    let results = orchestrator.search(query, Some(max_sources)).await?;

    let mut items = Vec::new();
    let result_id = Uuid::new_v4().to_string();

    for r in results.into_iter().take(max_sources as usize) {
        // Run QATBE Extraction
        let html = http.fetch_text(&r.url).await.unwrap_or_default();
        let snippet = if html.is_empty() {
            r.snippet.clone()
        } else {
            let ext = cep_extract(&html, &r.url);
            let snippet = ext.text.chars().take(400).collect::<String>();
            snippet
        };

        items.push(json!({
            "title": r.title,
            "url": r.url,
            "snippet": snippet,
            "score": r.score,
        }));
    }

    let duration_ms = start.elapsed().as_millis() as u64;

    let response = json!({
        "meta": {
            "query": query,
            "tier": tier,
            "tokens_used": token_budget,
            "sources_count": items.len(),
            "duration_ms": duration_ms,
            "result_id": result_id.clone(),
        },
        "results": items,
    });

    if let Some(c) = cache {
        c.set(&format!("expand:{}", result_id), &response).await;
    }

    Ok(response)
}

/// Execute a fetch pipeline: Fetch -> Extract -> Cache
pub async fn fetch(
    url: &str,
    token_budget: usize,
    format: &str,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Result<Value, HsxError> {
    let html = http.fetch_text(url).await?;
    let ext = cep_extract(&html, url);

    let max_chars = token_budget * 4;
    let content = if ext.text.len() > max_chars {
        ext.text[..max_chars].to_string()
    } else {
        ext.text
    };

    let tokens = crate::extract::layer1::estimate_tokens(&content) as usize;
    let result_id = Uuid::new_v4().to_string();

    let response = json!({
        "url": url,
        "title": if ext.title.is_empty() { None } else { Some(ext.title) },
        "content": content,
        "tokens": tokens,
        "format": format,
        "result_id": result_id.clone(),
    });

    if let Some(c) = cache {
        c.set(&format!("expand:{}", result_id), &response).await;
    }

    Ok(response)
}

/// Expand a previous result from the session cache
pub async fn expand(
    result_id: &str,
    tier: &str,
    cache: Option<&MemoryCache>,
) -> Result<Value, HsxError> {
    if let Some(c) = cache {
        if let Some(cached_data) = c.get::<Value>(&format!("expand:{}", result_id)).await {
            // Provide expanded details from cache based on tier
            let mut expanded_data = cached_data;
            if let Some(meta) = expanded_data.get_mut("meta") {
                meta["tier"] = json!(tier);
            }
            return Ok(expanded_data);
        }
    }
    Err(HsxError::Internal(
        "Cache miss or cache not configured for session".into(),
    ))
}
