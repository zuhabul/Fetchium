//! `fetchium search` — web search (Mode A).
//!
//! Pipeline:
//! 1. Build OrchestratorConfig (respects --backends override)
//! 2. Check memory cache
//! 3. Run search via SearchOrchestrator (parallel backends)
//! 4. Wrap into SearchResult with SearchMeta
//! 5. Format and output (stdout or --output file)

use crate::cli::{Format, SearchArgs};
use anyhow::Context;
use colored::Colorize;
use fetchium_core::cache::{search_key, MemoryCache};
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::output::{format_search_json, format_search_markdown};
use fetchium_core::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use fetchium_core::types::{PdsTier, ResourceTier, SearchMeta, SearchMode, SearchResult};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

/// Run the `hsx search "<query>"` command.
pub async fn run(
    args: SearchArgs,
    config: &HsxConfig,
    format: Format,
    quiet: bool,
) -> anyhow::Result<()> {
    let query = args.query.trim();
    let max_results = args.max_results;

    // ── Build orchestrator config ─────────────────────────────────────────
    let mut orch_config = OrchestratorConfig::from_hsx_config(config, max_results);

    // Override backends if the user specified --backends
    if !args.backends.is_empty() {
        let parsed: Vec<fetchium_core::types::BackendId> = args
            .backends
            .iter()
            .filter_map(|s| fetchium_core::search::orchestrator::parse_backend_id(s))
            .collect();
        if !parsed.is_empty() {
            orch_config.enabled_backends = parsed;
        } else {
            eprintln!("warn: none of the requested backends are recognised; using defaults");
        }
    }

    // ── Cache check ───────────────────────────────────────────────────────
    let cache = MemoryCache::from_config(&config.cache);
    let backends_label = orch_config
        .enabled_backends
        .iter()
        .map(|b| format!("{b:?}").to_lowercase())
        .collect::<Vec<_>>()
        .join("+");
    let cache_key = search_key(query, &backends_label, max_results);

    if let Some(cached) = cache.get::<SearchResult>(&cache_key).await {
        info!("Cache HIT for search: {query:?}");
        let formatted = format_result(&cached, format);
        write_output(&formatted, args.output.as_deref())?;
        if !quiet {
            eprintln!("  {} results (cached)", cached.items.len());
        }
        return Ok(());
    }

    // ── Execute search ────────────────────────────────────────────────────
    let start = Instant::now();
    let http = HttpClient::new(config).context("Failed to build HTTP client")?;

    // Use headless Chrome browser pool when compiled with --features headless.
    // This avoids CAPTCHA for Google/Bing by using a real browser fingerprint.
    #[cfg(feature = "headless")]
    let orchestrator = {
        use fetchium_core::browser::pool::{BrowserPool, BrowserTier};
        use std::sync::Arc;
        let tier = match fetchium_core::config::HsxConfig::detect_resource_tier() {
            fetchium_core::types::ResourceTier::Minimal => BrowserTier::Minimal,
            fetchium_core::types::ResourceTier::Standard => BrowserTier::Standard,
            _ => BrowserTier::Performance,
        };
        let pool = Arc::new(BrowserPool::new(tier));
        // Best-effort browser init — fall back to HTTP if Chrome isn't available
        if pool.init().await.is_err() {
            tracing::warn!("Headless Chrome init failed — falling back to HTTP scrapers");
            SearchOrchestrator::new(http, orch_config.clone())
        } else {
            SearchOrchestrator::with_pool(http, orch_config.clone(), pool)
        }
    };

    #[cfg(not(feature = "headless"))]
    let orchestrator = SearchOrchestrator::new(http, orch_config.clone());

    // Show spinner while searching (only when not quiet and not JSON output,
    // so we don't pollute machine-readable output)
    let pb = if !quiet && format != Format::Json {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));
        spinner.set_message(format!("Searching for {}...", query.bold()));
        Some(spinner)
    } else {
        None
    };

    let items = orchestrator
        .search(query, Some(max_results))
        .await
        .with_context(|| format!("Search failed for query: {query:?}"))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    info!(
        "Search complete: {} results in {}ms",
        items.len(),
        duration_ms
    );

    // Stop spinner before printing output
    if let Some(ref spinner) = pb {
        spinner.finish_and_clear();
    }

    // ── Build SearchResult ────────────────────────────────────────────────
    let result = SearchResult {
        meta: SearchMeta {
            query: query.to_string(),
            mode: SearchMode::Search,
            tier: PdsTier::Summary,
            tokens_used: 0,
            tokens_budget: config.search.default_budget,
            sources_fetched: items.len() as u32,
            sources_validated: items.len() as u32,
            validation_pass_rate: 1.0,
            duration_ms,
            resource_tier: ResourceTier::Standard,
            timestamp: chrono::Utc::now().to_rfc3339(),
            result_id: uuid::Uuid::new_v4().to_string(),
            content_hashes: HashMap::new(),
        },
        items,
    };

    // ── Cache the result ──────────────────────────────────────────────────
    cache.set(&cache_key, &result).await;

    // ── ACS: --trust-verify analysis ─────────────────────────────────────
    if args.trust_verify && !quiet {
        let acs = fetchium_core::intelligence::acs::AdversarialContentShield::new();
        eprintln!("\n{}", "── Trust Verification (ACS) ──────────".dimmed());
        for item in &result.items {
            let domain = url::Url::parse(&item.url)
                .map(|u| u.host_str().unwrap_or("unknown").to_string())
                .unwrap_or_else(|_| "unknown".to_string());
            let snippet_text = format!("{} {}", item.title, item.snippet);
            let acs_res = acs.analyze(&snippet_text, &domain);
            let trust_pct = (acs_res.trust_score * 100.0).round() as u64;
            let indicator = if acs_res.trust_score > 0.8 {
                "✓".green()
            } else if acs_res.trust_score > 0.5 {
                "?".yellow()
            } else {
                "✗".red()
            };
            eprintln!("  {indicator} {domain} — trust {trust_pct}%");
        }
        eprintln!();
    }

    // ── Format and output ─────────────────────────────────────────────────
    let formatted = match format {
        Format::Json => format_result(&result, format),
        // Markdown and other human-facing formats get colored terminal output
        _ => {
            if !quiet {
                // Print colored result count header
                let header = format!(
                    "\n  {}\n",
                    format!(
                        "Found {} result(s) in {}ms",
                        result.items.len(),
                        duration_ms
                    )
                    .green()
                    .bold()
                );
                eprint!("{header}");
            }
            format_result(&result, format)
        }
    };

    write_output(&formatted, args.output.as_deref())?;

    if !quiet {
        eprintln!(
            "  Found {} result(s) in {}ms",
            result.items.len(),
            duration_ms
        );
    }

    Ok(())
}

/// Format a `SearchResult` according to the chosen output format.
fn format_result(result: &SearchResult, format: Format) -> String {
    match format {
        Format::Json => format_search_json(result),
        Format::Csv => format_csv(result),
        Format::Yaml => {
            // Phase 5 will add native YAML; fall back to JSON for now.
            format_search_json(result)
        }
        Format::Segments | Format::Html => {
            // Segments/HTML not meaningful for raw search results; use markdown.
            format_search_markdown(result)
        }
        // Markdown is the default human-facing format — apply colored output.
        Format::Markdown => format_search_colored(result),
    }
}

/// Render search results with colored terminal output.
///
/// Format per result:
/// ```
///   1.  <bold/white title>
///      <blue+underline url>
///      <dimmed snippet>
/// ```
fn format_search_colored(result: &SearchResult) -> String {
    let mut out = String::new();
    for item in &result.items {
        // Rank number in dimmed style
        let rank_str = format!("  {}.", item.rank).dimmed().to_string();
        // Title in bold white
        let title_str = format!("  {}", item.title.bold().white());
        // URL in blue underline
        let url_str = format!("     {}", item.url.blue().underline());
        // Snippet in dimmed
        let snippet_str = format!("     {}", item.snippet.dimmed());

        out.push_str(&format!(
            "{rank_str}\n{title_str}\n{url_str}\n{snippet_str}\n\n"
        ));
    }
    out
}

/// Minimal CSV export: rank, title, url, snippet.
fn format_csv(result: &SearchResult) -> String {
    let mut out = String::from("rank,title,url,snippet\n");
    for item in &result.items {
        let title = item.title.replace('"', "\"\"");
        let url = item.url.replace('"', "\"\"");
        let snippet = item.snippet.replace('"', "\"\"");
        out.push_str(&format!(
            "{},\"{title}\",\"{url}\",\"{snippet}\"\n",
            item.rank
        ));
    }
    out
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
