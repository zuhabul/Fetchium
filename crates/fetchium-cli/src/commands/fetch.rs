//! `fetchium fetch` / `fetchium view` — URL content extraction (Mode D).
//!
//! Pipeline:
//! 1. Validate URL scheme (http/https only)
//! 2. Check memory cache (unless `--no-cache`)
//! 3. Fetch HTML via HttpClient (retries + rate limiting)
//! 4. Run CEP pipeline (L1 → L2 escalation)
//! 5. Run QATBE if `--query` provided, otherwise fall back to PDS directly
//! 6. Apply PDS tier
//! 7. Format and output (stdout or `--output` file)

use crate::cli::{FetchArgs, Format, Tier};
use anyhow::Context;
use colored::Colorize;
use fetchium_core::cache::{fetch_key, qatbe_key, MemoryCache};
use fetchium_core::config::HsxConfig;
use fetchium_core::extract::pipeline;
use fetchium_core::http::client::HttpClient;
use fetchium_core::output::{format_content_markdown, format_content_text, format_segments_json};
use fetchium_core::token::{pds, qatbe};
use fetchium_core::types::PdsTier;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Instant;
use tracing::{debug, info};

/// Map the CLI `Tier` enum to the core `PdsTier` enum.
fn map_tier(tier: Tier) -> PdsTier {
    match tier {
        Tier::KeyFacts => PdsTier::KeyFacts,
        Tier::Summary => PdsTier::Summary,
        Tier::Detailed => PdsTier::Detailed,
        Tier::Complete => PdsTier::Complete,
    }
}

/// Run the `hsx fetch <url>` / `hsx view <url>` command.
pub async fn run(args: FetchArgs, config: &HsxConfig, format: Format) -> anyhow::Result<()> {
    // ── Step 1: Validate URL ──────────────────────────────────────────────
    let url = args.url.trim();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        eprintln!("error: URL must begin with http:// or https://  (got: {url})");
        std::process::exit(1);
    }

    let pds_tier = map_tier(args.tier);
    let budget = args.budget;
    let query = args.query.clone().unwrap_or_default();

    // ── Step 2: Initialise memory cache ──────────────────────────────────
    let cache = MemoryCache::from_config(&config.cache);

    // Try cache for the formatted string (keyed by url + query + budget + tier).
    // We cache the final PDS result content to avoid re-running the full pipeline.
    let effective_key = if query.is_empty() {
        fetch_key(url)
    } else {
        qatbe_key(url, &query, budget)
    };
    debug!("Fetch cache key: {effective_key}");

    if let Some(cached_body) = cache.get::<String>(&effective_key).await {
        info!("Cache HIT for {url}");
        write_output(&cached_body, args.output.as_deref())?;
        return Ok(());
    }

    // ── Step 3: Fetch HTML ────────────────────────────────────────────────
    let start = Instant::now();

    // Show spinner while fetching
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb.set_message(format!("Fetching {}...", url));

    let http = HttpClient::new(config).context("Failed to build HTTP client")?;

    info!("Fetching {url}");
    let fetch_result = http
        .fetch(url)
        .await
        .with_context(|| format!("Failed to fetch URL: {url}"))?;
    let elapsed_fetch = start.elapsed().as_millis();

    debug!(
        "Fetched {} bytes in {}ms (status {})",
        fetch_result.body.len(),
        elapsed_fetch,
        fetch_result.status
    );

    // Use the final (possibly redirected) URL for extraction context.
    let effective_url = &fetch_result.url;

    // ── Step 4: CEP extraction pipeline (L1 → L2) ────────────────────────
    pb.set_message("Extracting content...");

    let extracted = pipeline::extract(&fetch_result.body, effective_url);

    info!(
        "CEP extracted {} tokens via {:?} for {}",
        extracted.tokens, extracted.layer_used, effective_url
    );

    // ── Step 5: QATBE + PDS or direct PDS ────────────────────────────────
    let (content, tokens_used, segments) = if !query.is_empty() {
        // Query provided → run QATBE then PDS
        let qatbe_result = qatbe::extract_with_budget(&extracted, &query, budget);
        let pds_result = pds::apply_tier(&extracted.text, &qatbe_result.segments, pds_tier);
        (
            pds_result.content,
            pds_result.tokens_used,
            pds_result.segments,
        )
    } else {
        // No query → run QATBE with empty query for segmentation, then PDS
        let qatbe_result = qatbe::extract_with_budget(&extracted, "", budget);
        let pds_result = pds::apply_tier(&extracted.text, &qatbe_result.segments, pds_tier);
        (
            pds_result.content,
            pds_result.tokens_used,
            pds_result.segments,
        )
    };

    let total_elapsed = start.elapsed().as_millis();
    info!(
        "Fetch complete: {} tokens, {}ms total",
        tokens_used, total_elapsed
    );

    // Stop spinner before printing output
    pb.finish_and_clear();

    // ── ACS: --check-ai analysis ──────────────────────────────────────────
    if args.check_ai {
        let domain = url::Url::parse(effective_url)
            .map(|u| u.host_str().unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let acs = fetchium_core::intelligence::acs::AdversarialContentShield::new();
        let result = acs.analyze(&extracted.text, &domain);
        eprintln!("\n{}", "── ACS Analysis ──────────────────────".dimmed());
        eprintln!(
            "  AI-generated probability: {:.0}%",
            result.ai_generated_probability * 100.0
        );
        eprintln!(
            "  Bot-farm probability:     {:.0}%",
            result.bot_farm_probability * 100.0
        );
        eprintln!(
            "  Manipulation probability: {:.0}%",
            result.manipulation_probability * 100.0
        );
        eprintln!(
            "  Trust score:              {:.0}% ({})",
            result.trust_score * 100.0,
            if result.is_shadow {
                "shadow mode"
            } else {
                "active mode"
            }
        );
        if !result.flags.is_empty() {
            eprintln!("  Flags: {:?}", result.flags);
        }
    }

    // ── Step 6: Format output ─────────────────────────────────────────────
    let metadata_str = {
        let mut parts = Vec::new();
        if let Some(author) = &extracted.metadata.author {
            parts.push(format!("Author: {author}"));
        }
        if let Some(date) = &extracted.metadata.published_date {
            parts.push(format!("Published: {date}"));
        }
        if let Some(lang) = &extracted.metadata.language {
            parts.push(format!("Lang: {lang}"));
        }
        parts.join(" | ")
    };

    let formatted = match format {
        Format::Json => {
            // Return a JSON object with all relevant fields.
            let obj = serde_json::json!({
                "url": effective_url,
                "title": extracted.title,
                "tokens_used": tokens_used,
                "tier": format!("{pds_tier:?}"),
                "layer_used": format!("{:?}", extracted.layer_used),
                "elapsed_ms": total_elapsed,
                "metadata": {
                    "description": extracted.metadata.description,
                    "author": extracted.metadata.author,
                    "published_date": extracted.metadata.published_date,
                    "language": extracted.metadata.language,
                    "content_type": extracted.metadata.content_type,
                },
                "content": content,
            });
            serde_json::to_string_pretty(&obj).unwrap_or_else(|_| content.clone())
        }
        Format::Segments => format_segments_json(&segments),
        Format::Markdown => {
            // Build the base formatted content
            let base = format_content_markdown(
                &extracted.title,
                effective_url,
                &content,
                tokens_used,
                if metadata_str.is_empty() {
                    None
                } else {
                    Some(&metadata_str)
                },
            );
            // Prepend colored header: bold title + separator in dimmed style
            let header = format!(
                "\n{}\n{}\n",
                extracted.title.bold(),
                "=".repeat(60).dimmed()
            );
            // Append dimmed metadata line if present
            let meta_line = if !metadata_str.is_empty() {
                format!("{}\n\n", metadata_str.dimmed())
            } else {
                String::new()
            };
            format!("{}{}{}", header, meta_line, base)
        }
        // For Csv, Yaml, Html — fall back to plain text until Phase 5 formatters exist.
        _ => format_content_text(&extracted.title, effective_url, &content),
    };

    // ── Step 7: Cache and output ──────────────────────────────────────────
    cache.set(&effective_key, &formatted).await;
    write_output(&formatted, args.output.as_deref())?;

    Ok(())
}

/// Write `content` to a file path or stdout.
fn write_output(content: &str, path: Option<&str>) -> anyhow::Result<()> {
    match path {
        Some(p) => {
            std::fs::write(p, content)
                .with_context(|| format!("Failed to write output to file: {p}"))?;
            eprintln!("Output written to {p}");
        }
        None => print!("{content}"),
    }
    Ok(())
}
