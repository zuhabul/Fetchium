use crate::cache::MemoryCache;
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::extract::pipeline::extract as cep_extract;
use crate::http::client::HttpClient;
use crate::rank::fusion::{detect_intent, QueryIntent};
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Truncate a string at a UTF-8 character boundary (safe for multibyte text like Spanish/French/CJK).
/// Always returns a valid `&str` slice, never panics.
#[inline]
fn safe_trunc(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // Walk back from max_bytes to the nearest char boundary
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// Inline cosine similarity (avoids feature-gated embeddings module dependency).
fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let na: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let nb: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

/// Strip markdown code fences and find JSON content in LLM output.
fn extract_json_from_text(text: &str) -> String {
    let text = text.trim();
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    if let Some(pos) = text.find('{').or_else(|| text.find('[')) {
        return text[pos..].to_string();
    }
    text.to_string()
}

/// Clean a search snippet: strip markdown, HTML tags, metadata noise.
fn clean_snippet(raw: &str) -> String {
    let mut s = raw.to_string();
    // Strip bold/italic markdown
    s = s
        .replace("**", "")
        .replace("__", "")
        .replace('*', "")
        .replace('_', " ");
    // Strip HTML tags
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    // Strip [text](url) links → keep text only
    let mut result = String::new();
    let mut chars = out.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '[' {
            let mut link_text = String::new();
            while let Some(&nc) = chars.peek() {
                chars.next();
                if nc == ']' {
                    break;
                }
                link_text.push(nc);
            }
            // Skip (url) part
            if chars.peek() == Some(&'(') {
                chars.next();
                while let Some(&nc) = chars.peek() {
                    chars.next();
                    if nc == ')' {
                        break;
                    }
                }
            }
            result.push_str(&link_text);
        } else {
            result.push(c);
        }
    }
    // Normalize whitespace
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ─── Semantic Reranking ───────────────────────────────────────────────────────

/// Semantically rerank results: blend HyperFusion score (70%) + nomic-embed-text cosine (30%).
/// Falls back to original order if Ollama is unavailable (3s timeout).
async fn semantic_rerank(
    query: &str,
    mut results: Vec<crate::types::ResultItem>,
) -> Vec<crate::types::ResultItem> {
    if results.len() <= 1 {
        return results;
    }

    let snippet_texts: Vec<String> = results
        .iter()
        .map(|r| format!("{} {}", r.title, r.snippet))
        .collect();
    let mut all_texts: Vec<&str> = vec![query];
    all_texts.extend(snippet_texts.iter().map(|s| s.as_str()));

    let ollama_url =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let body = serde_json::json!({ "model": "nomic-embed-text", "input": all_texts });

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return results,
    };

    let resp = match tokio::time::timeout(
        Duration::from_secs(3),
        client
            .post(format!("{ollama_url}/api/embed"))
            .json(&body)
            .send(),
    )
    .await
    {
        Ok(Ok(r)) => r,
        _ => {
            tracing::debug!("Semantic rerank: Ollama unavailable, keeping HyperFusion order");
            return results;
        }
    };

    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(_) => return results,
    };

    let embeddings: Vec<Vec<f32>> = match data["embeddings"].as_array() {
        Some(arr) if arr.len() == all_texts.len() => arr
            .iter()
            .map(|emb| {
                emb.as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect()
            })
            .collect(),
        _ => return results,
    };

    let query_emb = &embeddings[0];
    for (i, result) in results.iter_mut().enumerate() {
        let sem_sim = cosine_sim(query_emb, &embeddings[i + 1]) as f64;
        let fusion_score = result.score.unwrap_or(0.5);
        result.score = Some(0.7 * fusion_score + 0.3 * sem_sim);
    }

    results.sort_by(|a, b| {
        b.score
            .unwrap_or(0.0)
            .partial_cmp(&a.score.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    tracing::debug!("Semantic rerank: reranked {} results", results.len());
    results
}

// ─── Semantic Fact Fusion ────────────────────────────────────────────────────

/// Extract verified facts from top snippets via Qwen + multi-source keyword matching.
/// Returns facts with confidence: 0.6 (1 source), 0.8 (2 sources), 0.95 (3+ sources).
async fn fact_fusion(query: &str, snippets: &[(usize, &str)], http: &HttpClient) -> Vec<Value> {
    if snippets.len() < 2 {
        return Vec::new();
    }

    let combined: String = snippets
        .iter()
        .map(|(i, text)| format!("[Source {}]: {}", i, safe_trunc(text, 400)))
        .collect::<Vec<_>>()
        .join("\n\n");

    let prompt = format!(
        "Extract 3-5 key verifiable facts about '{}' from these snippets.\n\
         Return ONLY a JSON array: [{{'fact': '...', 'keywords': ['word1','word2']}}]\n\n\
         Snippets:\n{}\n\nJSON:",
        query,
        safe_trunc(&combined, 2000)
    );

    let body = serde_json::json!({
        "model": "qwen3.5:2b",
        "messages": [{"role": "user", "content": prompt}],
        "stream": false,
        "options": {"temperature": 0.1, "num_predict": 256}
    });

    let body_str = match serde_json::to_string(&body) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let resp_text = match tokio::time::timeout(
        Duration::from_secs(3),
        http.post_json("http://localhost:11434/api/chat", &body_str),
    )
    .await
    {
        Ok(Ok(t)) => t,
        _ => return Vec::new(),
    };

    let content = match serde_json::from_str::<Value>(&resp_text)
        .ok()
        .and_then(|v| {
            v.get("message")?
                .get("content")?
                .as_str()
                .map(|s| s.to_string())
        }) {
        Some(s) => s,
        None => return Vec::new(),
    };

    let facts: Vec<Value> = match serde_json::from_str(&extract_json_from_text(&content)) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    facts
        .into_iter()
        .filter_map(|fact_obj| {
            let keywords: Vec<String> = fact_obj
                .get("keywords")
                .and_then(|k| k.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_lowercase()))
                        .collect()
                })
                .unwrap_or_default();

            if keywords.is_empty() {
                return None;
            }

            let source_indices: Vec<usize> = snippets
                .iter()
                .filter_map(|(idx, text)| {
                    let text_lower = text.to_lowercase();
                    let hits = keywords
                        .iter()
                        .filter(|kw| text_lower.contains(kw.as_str()))
                        .count();
                    if hits >= 1 {
                        Some(*idx)
                    } else {
                        None
                    }
                })
                .collect();

            let confidence: f64 = match source_indices.len() {
                0 => return None,
                1 => 0.6,
                2 => 0.8,
                _ => 0.95,
            };

            Some(json!({
                "fact": fact_obj.get("fact").unwrap_or(&Value::Null),
                "confidence": confidence,
                "sources": source_indices,
            }))
        })
        .collect()
}

// ─── Structured Extraction ────────────────────────────────────────────────────

/// Extract structured data from content using local Qwen model via Ollama.
async fn extract_structured(content: &str, schema: &Value, http: &HttpClient) -> Option<Value> {
    let excerpt: String = content.chars().take(12000).collect();
    let prompt = format!(
        "Extract structured data per this JSON schema.\nReturn ONLY valid JSON.\n\nSchema: {schema}\n\nContent:\n{excerpt}\n\nJSON:"
    );

    let body = serde_json::json!({
        "model": "qwen3.5:2b",
        "messages": [{"role": "user", "content": prompt}],
        "stream": false,
        "options": {"temperature": 0.1, "num_predict": 1024}
    });

    let body_str = serde_json::to_string(&body).ok()?;
    let resp_text = tokio::time::timeout(
        Duration::from_secs(30),
        http.post_json("http://localhost:11434/api/chat", &body_str),
    )
    .await
    .ok()?
    .ok()?;

    let content_str = serde_json::from_str::<Value>(&resp_text)
        .ok()?
        .get("message")?
        .get("content")?
        .as_str()?
        .to_string();

    serde_json::from_str(&extract_json_from_text(&content_str)).ok()
}

// ─── Search Pipeline ──────────────────────────────────────────────────────────

/// Execute a search pipeline: Orchestrator → Semantic Rerank → Fact Fusion → Cache
///
/// Fast path (key_facts/summary): uses search snippets directly.
/// Rich path (detailed/complete/include_content): fetches URLs in parallel.
pub struct SearchRequest<'a> {
    pub query: &'a str,
    pub max_sources: u32,
    pub tier: &'a str,
    pub token_budget: usize,
    pub include_content: bool,
}

pub async fn search(
    request: SearchRequest<'_>,
    config: &HsxConfig,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Result<Value, HsxError> {
    let SearchRequest {
        query,
        max_sources,
        tier,
        token_budget,
        include_content,
    } = request;
    let start = Instant::now();

    let orch_config = OrchestratorConfig::from_fetchium_config(config, max_sources);
    let orchestrator = SearchOrchestrator::new(http.clone(), orch_config);
    let results = orchestrator.search(query, Some(max_sources)).await?;

    let result_id = Uuid::new_v4().to_string();
    let needs_extraction = include_content || matches!(tier, "detailed" | "complete");
    let max_content_chars = if include_content {
        token_budget * 4
    } else {
        800
    };

    // Limit + semantic reranking (blend HyperFusion 70% + nomic-embed-text 30%)
    let results: Vec<_> = results.into_iter().take(max_sources as usize).collect();
    let results = semantic_rerank(query, results).await;

    // Fact fusion: run concurrently with item-building (no latency penalty)
    let top_snippets: Vec<(usize, String)> = results
        .iter()
        .take(5)
        .enumerate()
        .map(|(i, r)| (i + 1, r.snippet.clone()))
        .collect();
    let http_ff = http.clone();
    let query_owned = query.to_string();
    let fact_handle = tokio::spawn(async move {
        let refs: Vec<(usize, &str)> = top_snippets.iter().map(|(i, s)| (*i, s.as_str())).collect();
        fact_fusion(&query_owned, &refs, &http_ff).await
    });

    let items: Vec<Value> = if needs_extraction {
        // Parallel URL fetching + CEP extraction
        let mut handles = Vec::with_capacity(results.len());
        for r in &results {
            let http2 = http.clone();
            let url = r.url.clone();
            let fallback = r.snippet.clone();
            handles.push(tokio::spawn(async move {
                match tokio::time::timeout(Duration::from_secs(5), http2.fetch_text(&url)).await {
                    Ok(Ok(html)) if !html.is_empty() => {
                        let ext = cep_extract(&html, &url);
                        let extracted: String = ext.text.chars().take(max_content_chars).collect();
                        (fallback, Some(extracted))
                    }
                    _ => (fallback, None),
                }
            }));
        }

        let mut items = Vec::with_capacity(results.len());
        for (idx, (r, handle)) in results.iter().zip(handles).enumerate() {
            let (snippet, extracted) = handle.await.unwrap_or_else(|_| (r.snippet.clone(), None));
            let clean = clean_snippet(&snippet);
            let mut item = if include_content {
                json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": clean,
                    "score": r.score,
                    "content": extracted,
                    "source_index": idx + 1,
                })
            } else {
                json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": extracted.unwrap_or(clean),
                    "score": r.score,
                    "source_index": idx + 1,
                })
            };
            if let Some(ref date) = r.published_date {
                item["published_date"] = json!(date);
            }
            items.push(item);
        }
        items
    } else {
        // Fast path: snippets only
        results
            .iter()
            .enumerate()
            .map(|(idx, r)| {
                let mut item = json!({
                    "title": r.title,
                    "url": r.url,
                    "snippet": clean_snippet(&r.snippet),
                    "score": r.score,
                    "source_index": idx + 1,
                });
                if let Some(ref date) = r.published_date {
                    item["published_date"] = json!(date);
                }
                item
            })
            .collect()
    };

    // Predictive pre-fetch: fire-and-forget background fetch of top-3 URLs
    if !needs_extraction {
        let top3: Vec<String> = results.iter().take(3).map(|r| r.url.clone()).collect();
        if let Some(c) = cache {
            for url in top3 {
                let http2 = http.clone();
                let c2 = c.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    if let Ok(Ok(html)) =
                        tokio::time::timeout(Duration::from_secs(8), http2.fetch_text(&url)).await
                    {
                        if html.len() > 500 {
                            let ext = cep_extract(&html, &url);
                            let data = json!({
                                "url": &url,
                                "title": ext.title,
                                "content": safe_trunc(&ext.text, 8000),
                                "prefetched": true,
                            });
                            c2.set(&format!("prefetch:{url}"), &data).await;
                        }
                    }
                });
            }
        }
    }

    // Collect fact fusion result (was running concurrently with item-building)
    let verified_facts = fact_handle.await.unwrap_or_default();

    // Citations list for every result
    let citations: Vec<Value> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            json!({
                "index": i + 1,
                "title": item.get("title").and_then(|v| v.as_str()).unwrap_or(""),
                "url":   item.get("url").and_then(|v| v.as_str()).unwrap_or(""),
            })
        })
        .collect();

    let duration_ms = start.elapsed().as_millis() as u64;
    let intent = detect_intent(query);
    let intent_str = format!("{:?}", intent).to_lowercase();

    let response = json!({
        "meta": {
            "query": query,
            "tier": tier,
            "intent": intent_str,
            "tokens_used": token_budget,
            "sources_count": items.len(),
            "duration_ms": duration_ms,
            "result_id": result_id,
            "credits_used": 1,
            "citations": citations,
            "verified_facts": verified_facts,
        },
        "results": items,
    });

    if let Some(c) = cache {
        // Semantic TTL: cache duration based on query intent
        let _cache_ttl = match intent {
            QueryIntent::CurrentEvents => Duration::from_secs(60),
            QueryIntent::Factual => Duration::from_secs(86400 * 7),
            QueryIntent::HowTo => Duration::from_secs(86400 * 30),
            QueryIntent::Academic => Duration::from_secs(86400 * 14),
            QueryIntent::Code => Duration::from_secs(86400 * 7),
            QueryIntent::Opinion => Duration::from_secs(3600 * 6),
            QueryIntent::Casual => Duration::from_secs(3600 * 12),
            _ => Duration::from_secs(3600 * 4),
        };
        c.set(&format!("expand:{result_id}"), &response).await;
    }

    Ok(response)
}

// ─── Fetch Pipeline ───────────────────────────────────────────────────────────

/// Execute a fetch pipeline: Fetch → CEP Extract → (optional Qwen schema extraction) → Cache
pub async fn fetch(
    url: &str,
    token_budget: usize,
    format: &str,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
    schema: Option<&Value>,
) -> Result<Value, HsxError> {
    // Check prefetch cache first
    if let Some(c) = cache {
        if let Some(cached) = c.get::<Value>(&format!("prefetch:{url}")).await {
            if let Some(content) = cached.get("content").and_then(|v| v.as_str()) {
                let tokens = crate::extract::layer1::estimate_tokens(content) as usize;
                let result_id = Uuid::new_v4().to_string();
                return Ok(json!({
                    "url": url,
                    "title": cached.get("title"),
                    "content": content,
                    "tokens": tokens,
                    "format": format,
                    "result_id": result_id,
                    "from_cache": true,
                }));
            }
        }
    }

    let html = http.fetch_text(url).await?;
    let html = if html.is_empty() {
        return Err(HsxError::Internal("Empty response from URL".into()));
    } else {
        html
    };

    let ext = cep_extract(&html, url);
    let max_chars = token_budget * 4;
    let content = if ext.text.len() > max_chars {
        ext.text[..max_chars].to_string()
    } else {
        ext.text
    };

    let tokens = crate::extract::layer1::estimate_tokens(&content) as usize;
    let result_id = Uuid::new_v4().to_string();

    let structured = if let Some(s) = schema {
        extract_structured(&content, s, http).await
    } else {
        None
    };

    let response = json!({
        "url": url,
        "title": if ext.title.is_empty() { Value::Null } else { json!(ext.title) },
        "content": content,
        "tokens": tokens,
        "format": format,
        "result_id": result_id,
        "structured": structured,
    });

    if let Some(c) = cache {
        c.set(&format!("expand:{result_id}"), &response).await;
    }

    Ok(response)
}

// ─── Expand ───────────────────────────────────────────────────────────────────

/// Expand a previous result from the session cache.
pub async fn expand(
    result_id: &str,
    tier: &str,
    cache: Option<&MemoryCache>,
) -> Result<Value, HsxError> {
    if let Some(c) = cache {
        if let Some(cached_data) = c.get::<Value>(&format!("expand:{result_id}")).await {
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
