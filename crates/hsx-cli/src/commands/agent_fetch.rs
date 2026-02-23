//! `hsx agent-fetch` — agent-optimized URL fetch (JSON segments).
//!
//! Fetches a URL, runs the CEP extraction pipeline, applies QATBE and PDS,
//! and returns a structured JSON payload for AI agent consumption.

use crate::cli::{AgentFetchArgs, Tier};
use hsx_core::{
    cache::{fetch_key, qatbe_key, MemoryCache},
    config::HsxConfig,
    extract::pipeline,
    http::client::HttpClient,
    token::{pds, qatbe},
    types::PdsTier,
};
use sha2::{Digest, Sha256};
use std::time::Instant;
use tracing::info;

/// Map CLI Tier enum to core PdsTier.
fn to_pds_tier(tier: Tier) -> PdsTier {
    match tier {
        Tier::KeyFacts => PdsTier::KeyFacts,
        Tier::Summary => PdsTier::Summary,
        Tier::Detailed => PdsTier::Detailed,
        Tier::Complete => PdsTier::Complete,
    }
}

pub async fn run(args: AgentFetchArgs, config: &HsxConfig) -> anyhow::Result<()> {
    let start = Instant::now();

    // Step 1: Validate URL scheme
    let url = args.url.trim();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        let err = serde_json::json!({
            "error": true,
            "code": "invalid_url",
            "message": format!("URL must begin with http:// or https://, got: {url}"),
            "url": url,
        });
        println!("{}", serde_json::to_string_pretty(&err)?);
        return Ok(());
    }

    let pds_tier = to_pds_tier(args.tier);
    let query = args.query.clone().unwrap_or_default();

    // Step 2: Initialise cache and HTTP client
    let cache = MemoryCache::from_config(&config.cache);
    let http_client = match HttpClient::new(config) {
        Ok(c) => c,
        Err(e) => {
            let err = serde_json::json!({
                "error": true,
                "code": "http_client_init_failed",
                "message": e.to_string(),
                "url": url,
            });
            println!("{}", serde_json::to_string_pretty(&err)?);
            return Ok(());
        }
    };

    // Step 3: Fetch HTML (with cache)
    let fkey = fetch_key(url);
    let html: Option<String> = cache.get(&fkey).await;
    let html = match html {
        Some(h) => {
            info!("Cache HIT for {url}");
            h
        }
        None => match http_client.fetch_text(url).await {
            Ok(h) => {
                cache.set(&fkey, &h).await;
                h
            }
            Err(e) => {
                let err = serde_json::json!({
                    "error": true,
                    "code": "fetch_failed",
                    "message": e.to_string(),
                    "url": url,
                });
                println!("{}", serde_json::to_string_pretty(&err)?);
                return Ok(());
            }
        },
    };

    // Step 4: CEP extraction pipeline (L1 → L2)
    let extracted = pipeline::extract(&html, url);
    info!(
        "CEP extracted {} tokens via {:?}",
        extracted.tokens, extracted.layer_used
    );

    // Step 5: QATBE with query context (empty query falls back gracefully)
    let qkey = qatbe_key(url, &query, args.budget);
    let qatbe_result: Option<qatbe::QatbeResult> = cache.get(&qkey).await;
    let qatbe_result = match qatbe_result {
        Some(r) => r,
        None => {
            let r = qatbe::extract_with_budget(&extracted, &query, args.budget);
            cache.set(&qkey, &r).await;
            r
        }
    };

    let relevance_coverage = qatbe_result.relevance_coverage;

    // Step 6: Apply PDS tier to produce final segments and content
    let pds_result = pds::apply_tier(&extracted.text, &qatbe_result.segments, pds_tier);

    let duration_ms = start.elapsed().as_millis() as u64;
    info!(
        "agent-fetch: {} tokens used, {}ms",
        pds_result.tokens_used, duration_ms
    );

    // Step 6b: Compute SHA-256 content hash of the extracted text
    let content_hash = format!("{:x}", Sha256::digest(extracted.text.as_bytes()));

    // Step 7: Build JSON response
    let segments_json: Vec<serde_json::Value> = pds_result
        .segments
        .iter()
        .map(|seg| {
            serde_json::json!({
                "type": seg.seg_type,
                "tokens": seg.tokens,
                "relevance": (seg.relevance * 100.0).round() / 100.0,
                "content": seg.content,
            })
        })
        .collect();

    let response = serde_json::json!({
        "url": url,
        "title": extracted.title,
        "tier": format!("{pds_tier:?}").to_lowercase(),
        "tokens_used": pds_result.tokens_used,
        "tokens_budget": args.budget,
        "relevance_coverage": (relevance_coverage * 100.0).round() / 100.0,
        "truncated": pds_result.truncated,
        "layer_used": format!("{:?}", extracted.layer_used),
        "duration_ms": duration_ms,
        "content_hash": content_hash,
        "metadata": {
            "description": extracted.metadata.description,
            "author": extracted.metadata.author,
            "published_date": extracted.metadata.published_date,
            "language": extracted.metadata.language,
        },
        "segments": segments_json,
    });

    println!("{}", serde_json::to_string_pretty(&response)?);
    Ok(())
}
