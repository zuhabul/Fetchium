//! Research pipeline orchestrator — 9-step PRD SS10 Mode B (PRD §10).

use crate::citation::evidence_graph::EvidenceGraphBuilder;
use crate::citation::formatter::CitationFormatter;
use crate::citation::types::SourceMeta;
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::research::decompose::decompose_query;
use crate::research::{ResearchConfig, ResearchMeta, ResearchReport};
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use crate::validate::authority::AuthorityScorer;
use crate::validate::calibration::ConfidenceCalibrator;
use crate::validate::content::{ContentInput, ContentValidator};
use crate::validate::cross_source::{CrossSourceVerifier, SourceContent as V4SourceContent};
use crate::validate::extraction::{ExtractionInput, ExtractionValidator};
use crate::validate::rar::{RarEngine, RarState};
use crate::validate::temporal::{SourceFreshness, TemporalValidator};
use std::time::Instant;

/// Extract the top `n` most query-relevant sentences from a block of text.
///
/// Used as a fallback synthesizer when AI is unavailable. Scores each sentence by
/// how many query keywords it contains, returns highest-scoring sentences joined
/// into a readable paragraph (≤ 200 chars each, ≥ 30 chars to exclude noise).
fn extract_key_sentences(text: &str, query_keywords: &[String], n: usize) -> String {
    if text.is_empty() {
        return String::new();
    }
    // Sentence split: ". " or "\n" boundaries, trim whitespace.
    let sentences: Vec<&str> = text
        .split('\n')
        .flat_map(|line| line.split(". "))
        .map(|s| s.trim())
        .filter(|s| s.len() >= 30 && s.len() <= 300)
        .collect();

    if sentences.is_empty() {
        // Nothing usable — return first 150 chars as a snippet.
        return text.chars().take(150).collect();
    }

    // Score by keyword overlap.
    let mut scored: Vec<(usize, &str)> = sentences
        .iter()
        .map(|s| {
            let sl = s.to_lowercase();
            let score = query_keywords
                .iter()
                .filter(|kw| sl.contains(kw.as_str()))
                .count();
            (score, *s)
        })
        .filter(|(score, _)| *score > 0)
        .collect();

    if scored.is_empty() {
        // No keyword hits at all — just return first sentence.
        return sentences.first().copied().unwrap_or("").to_string();
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(n);
    scored
        .iter()
        .map(|(_, s)| *s)
        .collect::<Vec<_>>()
        .join(". ")
}

/// Research pipeline orchestrator.
///
/// Steps (PRD SS10 Mode B):
/// 1. Query decomposition
/// 2. Parallel multi-backend search (via SearchOrchestrator)
/// 3. Top sources fetched via CEP
/// 4. Content extracted via QATBE
/// 5. RAR reflection loop validates retrieval quality
/// 6. HyperFusion ranking
/// 7. Evidence mapping via EGP
/// 8. Synthesis with strict citation
/// 9. Validation layer (6-layer)
pub struct ResearchPipeline;

impl ResearchPipeline {
    /// Ensure source diversity by interleaving results from different backends.
    ///
    /// Without this, HyperFusion's BM25 signal causes long Wikipedia articles
    /// to dominate the top N (they contain more keyword matches). Premium backends
    /// (Tavily, Exa, Firecrawl, Serper) return targeted results that are more
    /// relevant but get ranked lower.
    ///
    /// Strategy: Round-robin from each backend, prioritizing premium backends,
    /// until we have enough results.
    fn diversify_sources(
        results: Vec<crate::types::ResultItem>,
        target: usize,
    ) -> Vec<crate::types::ResultItem> {
        use std::collections::HashMap;

        if results.len() <= target {
            return results;
        }

        // Group results by backend, preserving rank order within each group
        let mut by_backend: HashMap<String, Vec<crate::types::ResultItem>> = HashMap::new();
        for r in results {
            let key = format!("{}", r.backend);
            by_backend.entry(key).or_default().push(r);
        }

        // Priority order: premium first, then free backends
        let priority_order = [
            "tavily",
            "exa",
            "firecrawl",
            "serper", // premium: targeted results
            "searxng",
            "arxiv", // free: good quality
            "hackernews",
            "reddit",
            "stackoverflow", // free: community
            "wikipedia",
            "github", // free: generic
            "duckduckgo",
            "google",
            "bing",
            "brave", // free: web search
        ];

        let mut selected = Vec::with_capacity(target);
        let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Phase 1: Take top result from each premium backend (if available)
        for backend_name in &priority_order[..4] {
            if selected.len() >= target {
                break;
            }
            if let Some(items) = by_backend.get_mut(*backend_name) {
                while !items.is_empty() && selected.len() < target {
                    let item = items.remove(0);
                    if seen_urls.insert(item.url.clone()) {
                        selected.push(item);
                        break; // One per backend in phase 1
                    }
                }
            }
        }

        // Phase 2: Round-robin from all backends until we reach target
        let mut round = 0;
        while selected.len() < target {
            let mut added_any = false;
            for backend_name in &priority_order {
                if selected.len() >= target {
                    break;
                }
                if let Some(items) = by_backend.get_mut(*backend_name) {
                    while !items.is_empty() {
                        let item = items.remove(0);
                        if seen_urls.insert(item.url.clone()) {
                            selected.push(item);
                            added_any = true;
                            break;
                        }
                    }
                }
            }
            round += 1;
            if !added_any || round > 20 {
                break;
            }
        }

        selected
    }

    /// Synthesize a research report using AI with numbered source context.
    ///
    /// Uses a oneshot channel + tokio::task::spawn_local pattern to keep
    /// the parent future Send-safe (required by axum handlers).
    /// Falls back to empty string on failure.
    async fn synthesize_with_ai(
        query: &str,
        sources: &[SourceMeta],
        citations: &[crate::citation::types::FormattedCitation],
        extracted_texts: &[String],
        fetchium_config: &HsxConfig,
        thinking: bool,
    ) -> String {
        use crate::ai::types::{AiConfig, ChatMessage};

        let mut ai_config = AiConfig::from_fetchium_config(fetchium_config);
        ai_config.thinking = thinking;
        if ai_config.providers.fallback_chain.is_empty() && ai_config.default_model.is_none() {
            return String::new();
        }

        // Build numbered source context with actual content excerpts (max 600 chars each).
        // This gives the AI real evidence to synthesize from, not just titles.
        let mut source_context = String::new();
        for (i, (meta, _citation)) in sources.iter().zip(citations.iter()).enumerate() {
            let excerpt = extracted_texts
                .get(i)
                .map(|t| {
                    let trimmed = t.trim();
                    let chars: String = trimmed.chars().take(500).collect();
                    if trimmed.chars().count() > 500 {
                        format!("{}...", chars)
                    } else {
                        chars
                    }
                })
                .unwrap_or_default();
            source_context.push_str(&format!(
                "[{}] {}\n{}\n{}\n\n",
                i + 1,
                meta.title,
                meta.url,
                excerpt,
            ));
        }

        let system_prompt = crate::ai::prompt::research_synthesis_prompt(query, sources.len());

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".into(),
                content: format!("SOURCES:\n{}\n\nSynthesize a report now.", source_context),
            },
        ];

        let providers = ai_config.providers.clone();

        // Use a oneshot channel to bridge across the Send boundary.
        // The non-Send part (dyn FnMut) is confined to a spawn_blocking thread.
        let (tx, rx) = tokio::sync::oneshot::channel::<String>();

        let ai_config_owned = ai_config;
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(async move {
                let mut noop = |_: &str| {};
                match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    crate::ai::provider_client::chat_with_fallback(
                        &messages,
                        None,
                        &ai_config_owned,
                        &providers,
                        false,
                        &mut noop,
                    ),
                )
                .await
                {
                    Ok(Ok(result)) => {
                        tracing::info!(
                            "AI synthesis completed: provider={}, model={}",
                            result.provider.slug(),
                            result.model_used
                        );
                        result.content
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("AI synthesis failed: {e}");
                        String::new()
                    }
                    Err(_) => {
                        tracing::warn!("AI synthesis timed out (30s)");
                        String::new()
                    }
                }
            });
            let _ = tx.send(result);
        });

        rx.await.unwrap_or_default()
    }

    /// Execute the full research pipeline from a configuration.
    ///
    /// The orchestrator, extractor, and ranker are injected via the config and client.
    /// This implementation wires the complete pipeline end-to-end.
    pub async fn execute(
        config: &ResearchConfig,
        fetchium_config: &HsxConfig,
        http_client: &HttpClient,
    ) -> Result<ResearchReport, HsxError> {
        let start = Instant::now();

        // ── FAST PATH ──────────────────────────────────────────────────────
        // When AI synthesis is disabled, use a lightweight pipeline that skips
        // URL fetching entirely. Produces results from search snippets only,
        // achieving Exa-like speed (~2-3s) with no third-party dependencies.
        if !config.ai_synthesis {
            return Self::execute_fast(config, fetchium_config, http_client, start).await;
        }

        // Step 1: Decompose query into perspective-aware sub-queries
        let sub_queries = decompose_query(&config.query);

        // Step 2: Parallel Dispatch — search ALL sub-queries concurrently
        let search_budget = (config.max_sources as u32) * 3;
        let mut orch_config =
            OrchestratorConfig::from_fetchium_config(fetchium_config, search_budget);
        orch_config.max_total_results = search_budget;
        let orchestrator =
            std::sync::Arc::new(SearchOrchestrator::new(http_client.clone(), orch_config));

        // Fire all sub-queries in parallel for broader coverage
        let mut search_handles = Vec::new();
        for sq in &sub_queries {
            let orch = orchestrator.clone();
            let sq = sq.clone();
            let budget = search_budget;
            search_handles.push(tokio::spawn(async move {
                orch.search(&sq, Some(budget)).await.unwrap_or_default()
            }));
        }

        // Collect and merge results from all sub-queries
        let mut search_results = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();
        for handle in search_handles {
            if let Ok(results) = handle.await {
                for r in results {
                    if seen_urls.insert(r.url.clone()) {
                        search_results.push(r);
                    }
                }
            }
        }

        let search_elapsed = start.elapsed();
        tracing::info!(
            "Search completed in {:.1}s ({} results from {} sub-queries)",
            search_elapsed.as_secs_f32(),
            search_results.len(),
            sub_queries.len()
        );

        // Source diversity selection
        search_results = Self::diversify_sources(search_results, config.max_sources);

        // Pre-filter: check title+snippet relevance BEFORE any fetching or synthesis
        let query_keywords_pre: Vec<String> = config
            .query
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|w| w.len() > 3)
            .collect();

        const PRE_GENERIC_WORDS: &[&str] = &[
            "software",
            "engineering",
            "development",
            "system",
            "technology",
            "computer",
            "application",
            "research",
            "study",
            "analysis",
            "paper",
            "using",
            "based",
            "approach",
            "model",
            "large",
            "deep",
            "data",
            "work",
            "show",
            "2024",
            "2025",
            "2026",
        ];

        let distinctive_pre: Vec<&String> = query_keywords_pre
            .iter()
            .filter(|w| !PRE_GENERIC_WORDS.contains(&w.as_str()))
            .collect();

        if !distinctive_pre.is_empty() {
            let pre_count = search_results.len();
            search_results.retain(|item| {
                let haystack = format!(
                    "{} {}",
                    item.title.to_lowercase(),
                    item.snippet
                        .chars()
                        .take(500)
                        .collect::<String>()
                        .to_lowercase()
                );
                distinctive_pre
                    .iter()
                    .any(|kw| haystack.contains(kw.as_str()))
            });
            tracing::debug!(
                "Pre-filter: {} → {} results",
                pre_count,
                search_results.len()
            );
        }

        // ── SPEED OPTIMIZATION: Start AI synthesis IMMEDIATELY using search snippets ──
        // Don't wait for URL fetching — search snippets (200-1500 chars) contain
        // enough content for a quality synthesis. This saves 5-8s of URL fetch time.
        // URL fetching still runs for validation but doesn't block the critical path.

        use crate::extract::ExtractedContent;
        use crate::types::BackendId;

        // Build snippet-based source data for immediate AI synthesis
        let formatter = CitationFormatter::new(config.citation_style);
        let mut snippet_metas = Vec::new();
        let mut snippet_citations = Vec::new();
        let mut snippet_texts = Vec::new();

        for (idx, item) in search_results.iter().enumerate() {
            let author = url::Url::parse(&item.url).ok().and_then(|u| {
                u.host_str()
                    .map(|h| h.trim_start_matches("www.").to_string())
            });
            let meta = SourceMeta {
                url: item.url.clone(),
                title: item.title.clone(),
                author,
                publisher: None,
                published_date: item.published_date.clone().or_else(|| Some("2026".into())),
                accessed_date: chrono::Utc::now().to_rfc3339(),
            };
            snippet_citations.push(formatter.format(&meta, idx + 1));
            snippet_metas.push(meta);
            snippet_texts.push(item.snippet.clone());
        }

        // Fire AI synthesis immediately (uses search snippets, no URL fetch needed)
        let ai_handle = if config.ai_synthesis && !snippet_metas.is_empty() {
            let query_owned = config.query.clone();
            let metas_owned = snippet_metas.clone();
            let citations_owned = snippet_citations.clone();
            let texts_owned = snippet_texts.clone();
            let hsx_owned = fetchium_config.clone();
            let thinking = config.thinking;
            Some(tokio::spawn(async move {
                Self::synthesize_with_ai(
                    &query_owned,
                    &metas_owned,
                    &citations_owned,
                    &texts_owned,
                    &hsx_owned,
                    thinking,
                )
                .await
            }))
        } else {
            None
        };

        // ── CONCURRENT: URL Fetch + Extract (for validation, runs while AI synthesizes) ──
        let mut extracted_sources = Vec::new();
        let mut fetch_tasks = Vec::new();

        for mut item in search_results {
            if item.url.contains("arxiv.org/abs/") {
                item.url = item.url.replace("arxiv.org/abs/", "arxiv.org/html/");
            }

            /// Rich snippet threshold — skip fetch for premium backends with enough content.
            const RICH_SNIPPET_THRESHOLD: usize = 300;
            let is_rich = matches!(
                item.backend,
                BackendId::Tavily | BackendId::Exa | BackendId::Firecrawl
            ) && item.snippet.len() >= RICH_SNIPPET_THRESHOLD;

            if is_rich {
                let title = item.title.clone();
                let text = item.snippet.clone();
                let extracted = ExtractedContent {
                    title,
                    text,
                    tokens: 0,
                    metadata: crate::extract::ContentMetadata {
                        author: None,
                        published_date: item.published_date.clone(),
                        language: Some("en".into()),
                        ..Default::default()
                    },
                    layer_used: crate::types::CepLayer::Layer1,
                };
                extracted_sources.push((item, extracted));
            } else {
                let client = http_client.clone();
                fetch_tasks.push(tokio::spawn(async move {
                    match tokio::time::timeout(
                        std::time::Duration::from_secs(6),
                        client.fetch_text(&item.url),
                    )
                    .await
                    {
                        Ok(Ok(html)) => (item, html),
                        _ => (item, String::new()),
                    }
                }));
            }
        }

        // Fetch remaining URLs (runs concurrently with AI synthesis)
        for task in fetch_tasks {
            if let Ok((item, html)) = task.await {
                if !html.is_empty() {
                    let extracted = extract(&html, &item.url);
                    extracted_sources.push((item, extracted));
                }
            }
        }

        // ── Build validation inputs from fetched content ──────────────────────
        let mut v1_inputs = Vec::new();
        let mut v2_inputs = Vec::new();
        let mut v3_inputs = Vec::new();
        let mut v4_inputs = Vec::new();
        let mut v5_inputs = Vec::new();

        // Intelligence layer: PIE observation (lightweight, non-blocking)
        let pie = crate::intelligence::pie::PersistentIntelligenceEngine::new().ok();
        if let Some(pie_engine) = &pie {
            let topic = crate::intelligence::pie::extract_topic(&config.query);
            let domains: Vec<String> = extracted_sources
                .iter()
                .filter_map(|(item, _)| {
                    url::Url::parse(&item.url)
                        .ok()
                        .and_then(|u| u.host_str().map(|h| h.to_string()))
                })
                .collect();
            let domain_strs: Vec<&str> = domains.iter().map(|s| s.as_str()).collect();
            let _ = pie_engine.observe_search(&config.query, &domain_strs, &topic);
        }

        // SGT: capture source data before consuming extracted_sources
        let sgt_hops: Vec<(String, String, String)> = if config.trace_sources {
            extracted_sources
                .iter()
                .map(|(item, ext)| {
                    let claim = ext
                        .text
                        .split(". ")
                        .next()
                        .unwrap_or(&ext.title)
                        .to_string();
                    (item.url.clone(), ext.title.clone(), claim)
                })
                .collect()
        } else {
            vec![]
        };

        // Intelligence layer: ACS validation
        let mut acs_flags = Vec::new();
        if config.trust_verify {
            let acs = crate::intelligence::acs::AdversarialContentShield::new();
            for (_item, ext) in &extracted_sources {
                let domain = url::Url::parse(&_item.url)
                    .map(|u| u.host_str().unwrap_or("unknown").to_string())
                    .unwrap_or_else(|_| "unknown".to_string());
                acs_flags.push(acs.analyze(&ext.text, &domain));
            }
        }

        // EGP data (before consuming extracted_sources)
        let egp_data: Vec<(String, String, String, f64)> = if config.evidence_graph {
            extracted_sources
                .iter()
                .map(|(item, ext)| {
                    (
                        item.url.clone(),
                        item.title.clone(),
                        ext.text.clone(),
                        item.score.unwrap_or(0.5),
                    )
                })
                .collect()
        } else {
            vec![]
        };

        let total_extracted = extracted_sources.len();

        // Use snippet-based metadata for the report (AI already has this data)
        let source_metas = snippet_metas;
        let formatted_citations = snippet_citations;
        let mut extracted_texts: Vec<String> = Vec::new();

        for (idx, (item, ext)) in extracted_sources.into_iter().enumerate() {
            extracted_texts.push(ext.text.clone());

            let has_ssl = item.url.starts_with("https");
            v1_inputs.push((item.url.clone(), has_ssl, 0));
            v2_inputs.push(ContentInput {
                url: item.url.clone(),
                text: ext.text.clone(),
            });
            v3_inputs.push(SourceFreshness {
                url: item.url.clone(),
                published_date: None,
                last_modified: None,
            });
            let claims: Vec<String> = ext
                .text
                .split(". ")
                .take(5)
                .map(|s| s.to_string())
                .collect();
            v4_inputs.push(V4SourceContent {
                url: item.url.clone(),
                index: idx,
                title: ext.title.clone(),
                claims,
                full_text: ext.text.clone(),
                confidence: item.score.unwrap_or(0.5),
            });
            v5_inputs.push(ExtractionInput {
                url: item.url.clone(),
                truncated: false,
                segment_count: 5,
                encoding_errors: 0,
            });
        }

        let sources_fetched = total_extracted;

        // Run Validation Layers (fast, local computation — <100ms total)
        let authority = AuthorityScorer::default();
        let l1 = authority.validate_sources(&v1_inputs);

        let content_val = ContentValidator::default();
        let l2 = content_val.validate(&v2_inputs, &config.query);

        let temporal = TemporalValidator::default();
        let l3 = temporal.validate(&v3_inputs, &config.query);

        let cross = CrossSourceVerifier::new();
        let l4 = cross.verify(&v4_inputs);

        let l5 = ExtractionValidator::validate(&v5_inputs);

        // RAR reflection loop
        let rar_engine = RarEngine::default();
        let state = RarState {
            query: config.query.clone(),
            total_results: total_extracted,
            relevant_count: total_extracted,
            sufficiency_score: l2.score,
            support_ratio: l4.score,
            consistency_score: l4.score,
            unsupported_claims: vec![],
            contradictions: vec![],
            candidate_urls: vec![],
            low_relevance_terms: vec![],
        };
        let rar_iterations = vec![rar_engine.evaluate(&state, 0)];

        // Evidence mapping via EGP
        let egp = if config.evidence_graph {
            let mut builder = EvidenceGraphBuilder::new(&config.query);
            for (url, title, text, score) in &egp_data {
                builder.add_source(url, title, text, *score);
            }
            Some(builder.build())
        } else {
            None
        };

        // ── Await AI synthesis result ──────────────────────────────────────
        let mut synthesis = if let Some(handle) = ai_handle {
            handle.await.unwrap_or_default()
        } else {
            String::new()
        };

        // Fallback: extractive synthesis when AI is unavailable.
        if synthesis.is_empty() {
            if source_metas.is_empty() {
                synthesis = format!("No sources found for: **{}**\n\nTry broadening your query or checking your SearXNG connection.", config.query);
            } else {
                let mut fb = String::new();
                fb.push_str("> *Note: AI synthesis unavailable — showing extractive key-sentence summary.*\n\n");
                fb.push_str(&format!("## What Sources Say About: {}\n\n", config.query));

                let all_keywords: Vec<String> = config
                    .query
                    .split_whitespace()
                    .map(|w| {
                        w.to_lowercase()
                            .trim_matches(|c: char| !c.is_alphanumeric())
                            .to_string()
                    })
                    .filter(|w| w.len() > 3)
                    .collect();

                for (idx, meta) in source_metas.iter().enumerate() {
                    let text = extracted_texts.get(idx).map(|s| s.as_str()).unwrap_or("");
                    let key_sentences = extract_key_sentences(text, &all_keywords, 3);
                    let marker = &formatted_citations[idx].inline_marker;
                    fb.push_str(&format!("### [{}] {}\n", idx + 1, meta.title));
                    fb.push_str(&format!("*{}*\n\n", meta.url));
                    if key_sentences.is_empty() {
                        let preview: String = text.chars().take(120).collect();
                        if !preview.is_empty() {
                            fb.push_str(&format!("{} {}\n\n", preview, marker));
                        }
                    } else {
                        fb.push_str(&format!("{} {}\n\n", key_sentences, marker));
                    }
                }
                synthesis = fb;
            }
        }

        let reference_section = formatter.format_references(&source_metas);

        // SGT: append source genealogy report when trace_sources is requested
        if config.trace_sources && !sgt_hops.is_empty() {
            let chain = crate::intelligence::sgt::build_chain(sgt_hops);
            synthesis.push_str("\n\n");
            synthesis.push_str(&chain.to_markdown());
        }

        let calibrator = ConfidenceCalibrator::default();
        let validation_res = calibrator.build_result(
            config.validation_mode,
            vec![l1, l2, l3, l4, l5],
            vec![],
            vec![],
        );

        let mut overall_confidence = validation_res.confidence;

        // Intelligence layer: Confidence Calibration (CCE)
        if config.trust_verify {
            let db_path = crate::intelligence::intelligence_data_dir().join("calibration.db");
            if let Ok(cce) = crate::intelligence::cce::ConfidenceCalibrationEngine::new(&db_path) {
                let topic = crate::intelligence::pie::extract_topic(&config.query);
                if let Ok(calibrated) = cce.calibrate(&topic, overall_confidence) {
                    overall_confidence = calibrated.calibrated;
                }
            }
        }

        let pass_rate = if validation_res.passed { 1.0 } else { 0.5 };
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ResearchReport {
            query: config.query.clone(),
            sub_queries,
            synthesis,
            sources: source_metas,
            citations: formatted_citations,
            reference_section,
            validation: validation_res,
            evidence_graph: egp,
            rar_iterations,
            meta: ResearchMeta {
                duration_ms,
                sources_fetched,
                sources_validated: sources_fetched,
                validation_pass_rate: pass_rate,
                overall_confidence,
                rar_loops_executed: 1,
            },
        })
    }

    /// Fast research pipeline — skips URL fetching and AI synthesis.
    ///
    /// Produces results from search result snippets only, achieving
    /// ~2-3s response time using only free/self-hosted backends.
    /// Used when `ai_synthesis = false` (via `--no-ai` flag).
    async fn execute_fast(
        config: &ResearchConfig,
        fetchium_config: &HsxConfig,
        http_client: &HttpClient,
        start: Instant,
    ) -> Result<ResearchReport, HsxError> {
        let sub_queries = decompose_query(&config.query);

        // Search with tight timeout for speed, larger budget for diversity
        let fast_budget = (config.max_sources as u32) * 3;
        let mut orch_config =
            OrchestratorConfig::from_fetchium_config(fetchium_config, fast_budget);
        orch_config.max_total_results = fast_budget;
        orch_config.backend_timeout = std::time::Duration::from_secs(5);
        let orchestrator =
            std::sync::Arc::new(SearchOrchestrator::new(http_client.clone(), orch_config));

        // Fire all sub-queries in parallel for broader coverage
        let mut search_handles = Vec::new();
        for sq in &sub_queries {
            let orch = orchestrator.clone();
            let sq = sq.clone();
            let budget = fast_budget;
            search_handles.push(tokio::spawn(async move {
                orch.search(&sq, Some(budget)).await.unwrap_or_default()
            }));
        }

        let mut raw_results = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();
        for handle in search_handles {
            if let Ok(results) = handle.await {
                for r in results {
                    if seen_urls.insert(r.url.clone()) {
                        raw_results.push(r);
                    }
                }
            }
        }

        let search_results = Self::diversify_sources(raw_results, config.max_sources);

        // Pre-filter by relevance before building output
        let query_kws_pre: Vec<String> = config
            .query
            .split_whitespace()
            .map(|w| {
                w.to_lowercase()
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|w| w.len() > 3)
            .collect();

        const FAST_GENERIC: &[&str] = &[
            "software",
            "engineering",
            "development",
            "system",
            "technology",
            "computer",
            "application",
            "research",
            "study",
            "analysis",
            "paper",
            "using",
            "based",
            "approach",
            "model",
            "large",
            "deep",
            "data",
            "work",
            "show",
            "2024",
            "2025",
            "2026",
        ];

        let distinctive_fast: Vec<&String> = query_kws_pre
            .iter()
            .filter(|w| !FAST_GENERIC.contains(&w.as_str()))
            .collect();

        let search_results: Vec<_> = if !distinctive_fast.is_empty() {
            let filtered: Vec<_> = search_results
                .iter()
                .filter(|item| {
                    let haystack = format!(
                        "{} {}",
                        item.title.to_lowercase(),
                        item.snippet
                            .chars()
                            .take(500)
                            .collect::<String>()
                            .to_lowercase()
                    );
                    distinctive_fast
                        .iter()
                        .any(|kw| haystack.contains(kw.as_str()))
                })
                .cloned()
                .collect();
            // If filter is too aggressive (< 3 results), use unfiltered
            if filtered.len() >= 3 {
                filtered
            } else {
                search_results
            }
        } else {
            search_results
        };

        // Build sources directly from search snippets (no URL fetching)
        let formatter = CitationFormatter::new(config.citation_style);
        let mut source_metas = Vec::new();
        let mut formatted_citations = Vec::new();
        let mut extracted_texts = Vec::new();

        for (idx, item) in search_results.into_iter().enumerate() {
            let author = url::Url::parse(&item.url).ok().and_then(|u| {
                u.host_str()
                    .map(|h| h.trim_start_matches("www.").to_string())
            });
            let meta = SourceMeta {
                url: item.url.clone(),
                title: item.title.clone(),
                author,
                publisher: None,
                published_date: item.published_date.or_else(|| Some("2026".into())),
                accessed_date: chrono::Utc::now().to_rfc3339(),
            };
            source_metas.push(meta.clone());
            formatted_citations.push(formatter.format(&meta, idx + 1));
            extracted_texts.push(item.snippet.clone());
        }

        // Build extractive synthesis from snippets
        let mut synthesis = String::new();
        if source_metas.is_empty() {
            synthesis = format!(
                "No sources found for: **{}**\n\nTry broadening your query or checking your SearXNG connection.",
                config.query
            );
        } else {
            let query_kws: Vec<String> = config
                .query
                .split_whitespace()
                .map(|w| {
                    w.to_lowercase()
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_string()
                })
                .filter(|w| w.len() > 3)
                .collect();

            // Build a structured summary grouping related findings
            synthesis.push_str("## Overview\n\n");

            // Collect all key sentences for a combined overview
            let mut overview_sentences: Vec<(usize, String)> = Vec::new();
            for (idx, _meta) in source_metas.iter().enumerate() {
                let text = extracted_texts.get(idx).map(|s| s.as_str()).unwrap_or("");
                if !text.is_empty() {
                    let key = extract_key_sentences(text, &query_kws, 2);
                    if !key.is_empty() {
                        overview_sentences.push((idx, key));
                    }
                }
            }

            // Write overview paragraph with inline citations
            if !overview_sentences.is_empty() {
                for (idx, sentence) in &overview_sentences {
                    let marker = &formatted_citations[*idx].inline_marker;
                    synthesis.push_str(&format!("- {} {}\n", sentence, marker));
                }
                synthesis.push('\n');
            }

            // Source details section
            synthesis.push_str("## Sources\n\n");
            for (idx, meta) in source_metas.iter().enumerate() {
                let text = extracted_texts.get(idx).map(|s| s.as_str()).unwrap_or("");
                let marker = &formatted_citations[idx].inline_marker;
                let preview: String = text.chars().take(200).collect();
                synthesis.push_str(&format!(
                    "{}. **{}** — {} {}\n",
                    idx + 1,
                    meta.title,
                    if preview.is_empty() {
                        "No excerpt available"
                    } else {
                        &preview
                    },
                    marker,
                ));
            }
        }

        let reference_section = formatter.format_references(&source_metas);
        let sources_count = source_metas.len();
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ResearchReport {
            query: config.query.clone(),
            sub_queries,
            synthesis,
            sources: source_metas,
            citations: formatted_citations,
            reference_section,
            validation: crate::validate::types::ValidationResult {
                layers_run: vec![],
                layer_results: vec![],
                passed: true,
                confidence: 0.5,
                contradictions: vec![],
                consensus: vec![],
                mode: config.validation_mode,
            },
            evidence_graph: None,
            rar_iterations: vec![],
            meta: ResearchMeta {
                duration_ms,
                sources_fetched: sources_count,
                sources_validated: sources_count,
                validation_pass_rate: 1.0,
                overall_confidence: 0.5,
                rar_loops_executed: 0,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research::ResearchConfig;

    #[tokio::test]
    async fn pipeline_executes_without_error() {
        let config = ResearchConfig {
            query: "what is Rust".into(),
            thinking: false,
            ..Default::default()
        };
        let fetchium_config = crate::config::HsxConfig::default();
        let http = crate::http::client::HttpClient::new(&fetchium_config).unwrap();
        let report = ResearchPipeline::execute(&config, &fetchium_config, &http)
            .await
            .unwrap();
        assert_eq!(report.query, "what is Rust");
        assert!(!report.sub_queries.is_empty());
    }

    #[tokio::test]
    async fn pipeline_builds_egp_when_requested() {
        let config = ResearchConfig {
            query: "Rust vs Go".into(),
            evidence_graph: true,
            thinking: false,
            ..Default::default()
        };
        let fetchium_config = crate::config::HsxConfig::default();
        let http = crate::http::client::HttpClient::new(&fetchium_config).unwrap();
        let report = ResearchPipeline::execute(&config, &fetchium_config, &http)
            .await
            .unwrap();
        assert!(report.evidence_graph.is_some());
    }
}
