//! MCP request handlers — one per composite tool (PRD §30).

use crate::tools::{EstimateInput, ExpandInput, FetchInput, ResearchInput, SearchInput};
use fetchium_core::cache::MemoryCache;
use fetchium_core::citation::types::CitationStyle;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::research::pipeline::ResearchPipeline;
use fetchium_core::research::ResearchConfig;
use fetchium_core::validate::types::ValidationMode;
use serde_json::{json, Value};

pub async fn handle_search(
    input: SearchInput,
    config: &HsxConfig,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Value {
    let max_sources = input.max_sources.unwrap_or(10) as u32;
    let tier = input.tier.as_deref().unwrap_or("summary");
    let token_budget = input.token_budget.unwrap_or(2000);

    match fetchium_core::api_facade::search(
        &input.query,
        max_sources,
        tier,
        token_budget,
        config,
        http,
        cache,
    )
    .await
    {
        Ok(v) => v,
        Err(e) => json!({ "error": e.to_string() }),
    }
}

pub async fn handle_fetch(
    input: FetchInput,
    _config: &HsxConfig,
    http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Value {
    let budget = input.token_budget.unwrap_or(3000);
    let format = input.format.as_deref().unwrap_or("markdown");

    match fetchium_core::api_facade::fetch(&input.url, budget, format, http, cache).await {
        Ok(v) => v,
        Err(e) => json!({ "error": e.to_string() }),
    }
}

pub async fn handle_research(
    input: ResearchInput,
    config: &HsxConfig,
    http: &HttpClient,
    _cache: Option<&MemoryCache>,
) -> Value {
    let citation_style = match input.citation_style.as_deref() {
        Some("apa") => CitationStyle::Apa,
        Some("ieee") => CitationStyle::Ieee,
        Some("mla") => CitationStyle::Mla,
        Some("chicago") => CitationStyle::Chicago,
        Some("bibtex") => CitationStyle::Bibtex,
        _ => CitationStyle::Inline,
    };

    let rc = ResearchConfig {
        query: input.query.clone(),
        max_sources: input
            .max_sources
            .unwrap_or(config.general.max_results as usize),
        token_budget: input.token_budget,
        citation_style,
        validation_mode: ValidationMode::Standard,
        strict_evidence: input.strict_evidence.unwrap_or(false),
        evidence_graph: false,
        trace_sources: false,
        trust_verify: false,
        max_rar_loops: 2,
    };

    match ResearchPipeline::execute(&rc, config, http).await {
        Ok(report) => {
            let sources: Vec<Value> = report
                .sources
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    json!({
                        "index": i + 1,
                        "title": s.title,
                        "url": s.url,
                    })
                })
                .collect();

            json!({
                "meta": {
                    "query": report.query,
                    "sources_count": report.meta.sources_fetched,
                    "confidence": report.meta.overall_confidence,
                    "result_id": uuid::Uuid::new_v4().to_string(),
                },
                "report": report.synthesis,
                "reference_section": report.reference_section,
                "sources": sources,
                "confidence": report.meta.overall_confidence,
            })
        }
        Err(e) => json!({ "error": e.to_string() }),
    }
}

pub async fn handle_estimate(
    input: EstimateInput,
    _config: &HsxConfig,
    http: &HttpClient,
    _cache: Option<&MemoryCache>,
) -> Value {
    // Use HEAD request to check content-length, fall back to heuristic
    let estimated_tokens = match http.fetch_text(&input.url).await {
        Ok(html) => {
            // Quick estimate from raw HTML size (before extraction)
            let raw_tokens = fetchium_core::extract::layer1::estimate_tokens(&html) as usize;
            // Extraction typically reduces by 60–80%
            raw_tokens / 4
        }
        Err(_) => 1000, // fallback estimate
    };

    json!({
        "url": input.url,
        "estimated_tokens": estimated_tokens,
        "estimated_relevant_tokens": estimated_tokens / 2,
        "extraction_layer": 1,
        "content_type": "text/html",
    })
}

pub async fn handle_expand(
    input: ExpandInput,
    _config: &HsxConfig,
    _http: &HttpClient,
    cache: Option<&MemoryCache>,
) -> Value {
    match fetchium_core::api_facade::expand(&input.result_id, &input.tier, cache).await {
        Ok(v) => v,
        Err(e) => json!({ "error": e.to_string() }),
    }
}
