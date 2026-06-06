//! Speculative Research Pipelining — stream findings as they arrive (PRD §8.5).
//!
//! The SRP pipeline:
//! 1. Launches parallel search across all backends
//! 2. As results arrive, extracts top sources immediately
//! 3. Emits `Initial` chunk once `min_initial_sources` are processed
//! 4. Continues processing remaining sources
//! 5. Emits `Update` chunks for confirmations, `Correction` for contradictions
//! 6. Emits `Final` chunk when all sources are processed

use crate::config::FetchiumConfig;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::research::srp_types::{SrpChunk, SrpConfig, SrpEvent};
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use std::time::Instant;
use tokio::sync::mpsc;

/// Launch the SRP pipeline and return a channel receiver for streaming chunks.
///
/// The caller can consume chunks as they arrive and display them incrementally.
pub fn run_srp_pipeline(
    query: String,
    config: SrpConfig,
    fetchium_config: FetchiumConfig,
    http_client: HttpClient,
) -> mpsc::Receiver<SrpChunk> {
    let (tx, rx) = mpsc::channel::<SrpChunk>(64);

    tokio::spawn(async move {
        let start = Instant::now();

        let orch_config = OrchestratorConfig::from_fetchium_config(&fetchium_config, 10);
        let orchestrator = SearchOrchestrator::new(http_client.clone(), orch_config);

        // Search
        let results = orchestrator
            .search(&query, Some(10))
            .await
            .unwrap_or_default();

        let mut processed_content: Vec<String> = Vec::new();
        let mut initial_emitted = false;
        let batch_size = 3usize;

        for (batch_idx, batch) in results.chunks(batch_size).enumerate() {
            // Fetch and extract this batch
            let mut handles = Vec::new();
            for item in batch {
                let client = http_client.clone();
                let url = item.url.clone();
                handles.push(tokio::spawn(async move {
                    let html = client.fetch_text(&url).await.unwrap_or_default();
                    if html.is_empty() {
                        None
                    } else {
                        let ext = extract(&html, &url);
                        Some(ext.text)
                    }
                }));
            }

            for handle in handles {
                if let Ok(Some(text)) = handle.await {
                    if !text.trim().is_empty() {
                        let elapsed = start.elapsed().as_millis() as u64;
                        if initial_emitted {
                            if let Some(contradiction) =
                                find_contradiction(&query, &processed_content, &text)
                            {
                                let _ = tx
                                    .send(SrpChunk {
                                        event: SrpEvent::Correction,
                                        content: format!("[CORRECTION] {}", contradiction),
                                        sources: vec![processed_content.len()],
                                        confidence: 0.85,
                                        timestamp_ms: elapsed,
                                    })
                                    .await;
                            }
                        }
                        processed_content.push(text);
                    }
                }
            }

            let elapsed = start.elapsed().as_millis() as u64;

            // Emit Initial once we have enough sources
            if !initial_emitted && processed_content.len() >= config.min_initial_sources {
                let summary = synthesize_summary(&query, &processed_content);
                let _ = tx
                    .send(SrpChunk {
                        event: SrpEvent::Initial,
                        content: format!(
                            "[INITIAL] Based on {} sources: {}",
                            processed_content.len(),
                            summary
                        ),
                        sources: (0..processed_content.len()).collect(),
                        confidence: 0.65,
                        timestamp_ms: elapsed,
                    })
                    .await;
                initial_emitted = true;
            } else if initial_emitted && batch_idx > 0 {
                // Emit Update
                let _ = tx
                    .send(SrpChunk {
                        event: SrpEvent::Update,
                        content: format!(
                            "[UPDATE] {} sources processed. Confidence improving...",
                            processed_content.len()
                        ),
                        sources: (0..processed_content.len()).collect(),
                        confidence: 0.75,
                        timestamp_ms: elapsed,
                    })
                    .await;
            }
        }

        // Emit Final
        let elapsed = start.elapsed().as_millis() as u64;
        let final_summary = synthesize_summary(&query, &processed_content);
        let _ = tx
            .send(SrpChunk {
                event: SrpEvent::Final,
                content: format!(
                    "[FINAL] {} sources analyzed.\n\n{}",
                    processed_content.len(),
                    final_summary
                ),
                sources: (0..processed_content.len()).collect(),
                confidence: 0.90,
                timestamp_ms: elapsed,
            })
            .await;
    });

    rx
}

/// Check new extracted text against query context for simple contradictions
fn find_contradiction(query: &str, _processed: &[String], new_text: &str) -> Option<String> {
    let lower_text = new_text.to_lowercase();
    let lower_query = query.to_lowercase();
    let query_words: Vec<&str> = lower_query.split_whitespace().collect();

    // Heuristic 1: Negation words indicating a refutation of the concept
    let negations = [
        "not true",
        "incorrect",
        "false",
        "disproved",
        "contradicts",
        "however",
        "unlike",
        "instead",
        "misconception",
    ];

    for sentence in lower_text.split(". ") {
        let has_concept = query_words
            .iter()
            .filter(|&&w| w.len() > 3)
            .any(|&w| sentence.contains(w));
        if has_concept {
            for neg in &negations {
                if sentence.contains(neg) {
                    return Some(format!(
                        "Found potential contradiction: \"{}...\"",
                        &sentence.trim().chars().take(100).collect::<String>()
                    ));
                }
            }
        }
    }
    None
}

/// Synthesize a short summary from processed source content.
fn synthesize_summary(query: &str, content: &[String]) -> String {
    let query_words: std::collections::HashSet<&str> = query.split_whitespace().collect();

    let relevant: Vec<String> = content
        .iter()
        .filter_map(|text| {
            // Find the most relevant sentence
            text.split(". ")
                .find(|sentence| {
                    let words: std::collections::HashSet<&str> =
                        sentence.split_whitespace().collect();
                    words.intersection(&query_words).count() > 0
                })
                .map(|s| s.trim().to_string())
        })
        .take(3)
        .collect();

    if relevant.is_empty() {
        format!("Found {} sources related to: {}", content.len(), query)
    } else {
        relevant.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesize_summary_non_empty() {
        let content = vec![
            "Rust is a systems programming language. It focuses on safety.".into(),
            "Go is simple and fast. Go compiles quickly.".into(),
        ];
        let summary = synthesize_summary("Rust programming", &content);
        assert!(!summary.is_empty());
    }

    #[test]
    fn srp_config_defaults() {
        let config = SrpConfig::default();
        assert_eq!(config.min_initial_sources, 2);
        assert_eq!(config.max_wait_ms, 30_000);
    }
}
