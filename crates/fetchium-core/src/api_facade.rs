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
///
/// For `key_facts` and `summary` tiers, uses search snippets directly (fast path).
/// For `detailed` and `complete` tiers, fetches URLs in parallel for richer content.
pub async fn search(
    query: &str,
    max_sources: u32,
    tier: &str,
    token_budget: usize,
    config: &HsxConfig,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
    include_content: bool,
) -> Result<Value, HsxError> {
    let start = Instant::now();

    let orch_config = OrchestratorConfig::from_fetchium_config(config, max_sources);
    let orchestrator = SearchOrchestrator::new(http.clone(), orch_config);
    let results = orchestrator.search(query, Some(max_sources)).await?;

    let result_id = Uuid::new_v4().to_string();
    let needs_extraction = include_content || matches!(tier, "detailed" | "complete");
    let max_content_chars = if include_content { token_budget * 4 } else { 800 };
    let results: Vec<_> = results.into_iter().take(max_sources as usize).collect();

    let items: Vec<Value> = if needs_extraction {
        // Parallel URL fetching + CEP extraction
        let mut handles = Vec::with_capacity(results.len());
        for r in &results {
            let http = http.clone();
            let url = r.url.clone();
            let fallback_snippet = r.snippet.clone();
            let max_chars = max_content_chars;
            handles.push(tokio::spawn(async move {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    http.fetch_text(&url),
                )
                .await
                {
                    Ok(Ok(html)) if !html.is_empty() => {
                        let ext = cep_extract(&html, &url);
                        let extracted: String = ext.text.chars().take(max_chars).collect();
                        (fallback_snippet, Some(extracted))
                    }
                    _ => (fallback_snippet, None),
                }
            }));
        }
        let mut items = Vec::with_capacity(results.len());
        for (r, handle) in results.iter().zip(handles) {
            let (snippet, extracted) = handle.await.unwrap_or_else(|_| (r.snippet.clone(), None));
            let mut item = if include_content {
                // Tavily-style: snippet stays as search snippet, content is extracted page text
                json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": snippet,
                    "score": r.score,
                    "content": extracted,
                })
            } else {
                // Legacy detailed/complete: replace snippet with extracted content
                json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": extracted.unwrap_or(snippet),
                    "score": r.score,
                })
            };
            if let Some(ref date) = r.published_date {
                item["published_date"] = json!(date);
            }
            items.push(item);
        }
        items
    } else {
        // Fast path: use search snippets directly (no URL fetching)
        results
            .iter()
            .map(|r| {
                let mut item = json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": r.snippet,
                    "score": r.score,
                });
                if let Some(ref date) = r.published_date {
                    item["published_date"] = json!(date);
                }
                item
            })
            .collect()
    };

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
