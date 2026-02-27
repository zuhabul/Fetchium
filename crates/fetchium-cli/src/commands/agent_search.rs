//! `hsx agent-search` — agent-optimized search (JSON segments).
//!
//! Searches the web, extracts and token-budgets content from each result,
//! and outputs a structured JSON payload for AI agent consumption.

use crate::cli::{AgentSearchArgs, Tier};
use fetchium_core::{
    cache::{fetch_key, qatbe_key, search_key, MemoryCache},
    config::HsxConfig,
    extract::pipeline,
    http::client::HttpClient,
    output::format_agent_json,
    search::orchestrator::{OrchestratorConfig, SearchOrchestrator},
    token::{pds, qatbe},
    types::{
        AgentSearchResult, AuditEntry, FetchMethod, PdsTier, SearchMeta, SearchMode, Segment,
        Source,
    },
};
use std::time::Instant;
use tokio::task::JoinSet;
use tracing::warn;
use url::Url;

/// Map CLI Tier enum to core PdsTier.
fn to_pds_tier(tier: Tier) -> PdsTier {
    match tier {
        Tier::KeyFacts => PdsTier::KeyFacts,
        Tier::Summary => PdsTier::Summary,
        Tier::Detailed => PdsTier::Detailed,
        Tier::Complete => PdsTier::Complete,
    }
}

/// Extract domain from a URL string, returning "unknown" on failure.
fn extract_domain(url_str: &str) -> String {
    Url::parse(url_str)
        .map(|u| u.host_str().unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

pub async fn run(args: AgentSearchArgs, config: &HsxConfig) -> anyhow::Result<()> {
    let start = Instant::now();
    let pds_tier = to_pds_tier(args.tier);

    // Set up HTTP client and cache
    let http_client = match HttpClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            let err = serde_json::json!({
                "error": true,
                "code": "http_client_init_failed",
                "message": e.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&err)?);
            return Ok(());
        }
    };
    let cache = MemoryCache::from_config(&config.cache);

    // Step 1: Search
    let orch_config = OrchestratorConfig::from_hsx_config(config, args.max_results);
    let orchestrator = SearchOrchestrator::new(http_client.clone(), orch_config);

    let cache_key = search_key(&args.query, "all", args.max_results);
    let search_results = if let Some(cached) = cache
        .get::<Vec<fetchium_core::types::ResultItem>>(&cache_key)
        .await
    {
        cached
    } else {
        match orchestrator
            .search(&args.query, Some(args.max_results))
            .await
        {
            Ok(results) => {
                cache.set(&cache_key, &results).await;
                results
            }
            Err(e) => {
                let err = serde_json::json!({
                    "error": true,
                    "code": "search_failed",
                    "query": args.query,
                    "message": e.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&err)?);
                return Ok(());
            }
        }
    };

    let sources_fetched = search_results.len() as u32;

    // Step 2: Parallel fetch + extract + QATBE; budget distributed per page
    let per_page_budget = args.budget / args.max_results.max(1);

    let mut join_set: JoinSet<Option<(Vec<Segment>, Source)>> = JoinSet::new();

    for (idx, item) in search_results.iter().enumerate() {
        let url = item.url.clone();
        let title = item.title.clone();
        let query = args.query.clone();
        let client = http_client.clone();
        let cache_clone = cache.clone();
        let source_id = idx as u32;

        join_set.spawn(async move {
            // Check fetch cache
            let fkey = fetch_key(&url);
            let html: Option<String> = cache_clone.get(&fkey).await;
            let html = match html {
                Some(h) => h,
                None => match client.fetch_text(&url).await {
                    Ok(h) => {
                        cache_clone.set(&fkey, &h).await;
                        h
                    }
                    Err(e) => {
                        warn!("agent-search: fetch failed for {url}: {e}");
                        return None;
                    }
                },
            };

            // CEP extraction
            let extracted = pipeline::extract(&html, &url);

            // QATBE with per-page budget
            let qkey = qatbe_key(&url, &query, per_page_budget);
            let qatbe_result: Option<qatbe::QatbeResult> = cache_clone.get(&qkey).await;
            let qatbe_result = match qatbe_result {
                Some(r) => r,
                None => {
                    let r = qatbe::extract_with_budget(&extracted, &query, per_page_budget);
                    cache_clone.set(&qkey, &r).await;
                    r
                }
            };

            let domain = extract_domain(&url);
            let source = Source {
                id: source_id,
                url: url.clone(),
                title,
                domain,
                fetch_method: FetchMethod::Http,
                content_type: "text/html".to_string(),
                tokens: qatbe_result.tokens_used,
                published_date: None,
                trust_score: 0.7,
                citation: None,
            };

            // Tag each segment with the source id
            let mut segments = qatbe_result.segments;
            for seg in &mut segments {
                seg.source_ref = Some(source_id);
            }

            Some((segments, source))
        });
    }

    // Collect results
    let mut all_segments: Vec<Segment> = Vec::new();
    let mut sources: Vec<Source> = Vec::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Some((segs, src))) => {
                all_segments.extend(segs);
                sources.push(src);
            }
            Ok(None) => {}
            Err(e) => warn!("agent-search: task panicked: {e}"),
        }
    }

    // Sort segments by relevance desc before applying PDS tier
    all_segments.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Step 3: Apply PDS tier to trim to overall budget
    let pds_result = pds::apply_tier("", &all_segments, pds_tier);
    let tokens_used = pds_result.tokens_used;
    let duration_ms = start.elapsed().as_millis() as u64;

    // Step 4: Build AgentSearchResult
    let meta = SearchMeta {
        query: args.query.clone(),
        mode: SearchMode::Search,
        tier: pds_tier,
        tokens_used,
        tokens_budget: args.budget,
        sources_fetched,
        sources_validated: sources.len() as u32,
        validation_pass_rate: if sources_fetched > 0 {
            sources.len() as f64 / sources_fetched as f64
        } else {
            0.0
        },
        duration_ms,
        resource_tier: HsxConfig::detect_resource_tier(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        result_id: format!("as-{}", uuid::Uuid::new_v4()),
        content_hashes: std::collections::HashMap::new(),
    };

    let audit = vec![AuditEntry {
        step: "agent_search".to_string(),
        duration_ms,
        detail: format!(
            "query={:?} results={} segments={} tokens={}",
            args.query,
            sources_fetched,
            pds_result.segments.len(),
            tokens_used
        ),
    }];

    let agent_result = AgentSearchResult {
        meta,
        segments: pds_result.segments,
        findings: Vec::new(),
        evidence: Vec::new(),
        contradictions: Vec::new(),
        sources,
        evidence_graph: None,
        audit_trail: audit,
    };

    println!("{}", format_agent_json(&agent_result));
    Ok(())
}
