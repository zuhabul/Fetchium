//! AI-powered URL/text summarization pipeline.
//!
//! Pipeline: `input → detect_type → fetch (if URL) → CEP extract → QATBE → AI summarize`
//!
//! Used by `fetchium summarize` CLI command and the REST API.

use crate::ai::prompt::summarize_prompt;
use crate::ai::provider_client::chat_with_fallback;
use crate::ai::types::{AiConfig, ChatMessage};
use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::http::client::HttpClient;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Desired summary length.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryLength {
    /// ~100 words
    Short,
    /// ~300 words
    #[default]
    Medium,
    /// ~700 words
    Long,
}

impl SummaryLength {
    fn description(self) -> &'static str {
        match self {
            Self::Short => "short (~100 words)",
            Self::Medium => "medium (~300 words)",
            Self::Long => "long (~700 words)",
        }
    }

    /// Number of sentences for heuristic fallback.
    fn fallback_sentences(self) -> usize {
        match self {
            Self::Short => 3,
            Self::Medium => 7,
            Self::Long => 15,
        }
    }
}

/// Configuration for the summarize pipeline.
#[derive(Debug, Clone)]
pub struct SummarizeConfig {
    /// Desired summary length.
    pub length: SummaryLength,
    /// Optional model override for AI synthesis.
    pub model: Option<String>,
}

impl Default for SummarizeConfig {
    fn default() -> Self {
        Self {
            length: SummaryLength::Medium,
            model: None,
        }
    }
}

/// Result of summarization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResult {
    /// The generated summary text.
    pub summary: String,
    /// Source URL if input was a URL.
    pub source_url: Option<String>,
    /// Title extracted from the source (if URL).
    pub source_title: Option<String>,
    /// Whether AI was used (false = heuristic fallback).
    pub ai_used: bool,
    /// Duration in milliseconds.
    pub duration_ms: u64,
}

/// Summarize a URL or raw text.
///
/// If `input` starts with `http://` or `https://`, it is fetched and extracted first.
/// Otherwise it is treated as raw text.
pub async fn summarize(
    input: &str,
    config: &SummarizeConfig,
    hsx_config: &HsxConfig,
) -> Result<SummaryResult, HsxError> {
    let start = Instant::now();

    let is_url = input.starts_with("http://") || input.starts_with("https://");

    let (content, source_url, source_title) = if is_url {
        // YouTube URLs: extract transcript instead of HTML for much better summaries
        let is_youtube = input.contains("youtube.com/watch")
            || input.contains("youtu.be/")
            || input.contains("youtube.com/shorts/");

        if is_youtube {
            let http = HttpClient::new(hsx_config)?;
            match crate::youtube::universal::fetch_universal_transcript(input, &http, hsx_config)
                .await
            {
                Ok(transcript) => {
                    let video_id = transcript.video_id.clone();
                    let text = transcript.full_text.clone();
                    let title = format!("YouTube Video ({})", video_id);
                    (text, Some(input.to_string()), Some(title))
                }
                Err(_) => {
                    // Fall back to regular HTML fetch if transcript unavailable
                    let html = http.fetch_text(input).await?;
                    let extracted = crate::extract::pipeline::extract(&html, input);
                    let title = if extracted.title.is_empty() {
                        None
                    } else {
                        Some(extracted.title)
                    };
                    (extracted.text, Some(input.to_string()), title)
                }
            }
        } else {
            let http = HttpClient::new(hsx_config)?;
            let html = http.fetch_text(input).await?;
            let extracted = crate::extract::pipeline::extract(&html, input);
            let title = if extracted.title.is_empty() {
                None
            } else {
                Some(extracted.title)
            };
            (extracted.text, Some(input.to_string()), title)
        }
    } else {
        (input.to_string(), None, None)
    };

    if content.trim().is_empty() {
        return Ok(SummaryResult {
            summary: String::new(),
            source_url,
            source_title,
            ai_used: false,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Try AI synthesis
    let ai_config = AiConfig::from_hsx_config(hsx_config);
    let has_ai =
        !ai_config.providers.fallback_chain.is_empty() || ai_config.default_model.is_some();

    if has_ai {
        let system = summarize_prompt(config.length.description());
        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system,
            },
            ChatMessage {
                role: "user".into(),
                content: content.clone(),
            },
        ];

        let providers = ai_config.providers.clone();
        let model_override = config.model.clone();

        // Use thread isolation to keep future Send-safe (required for axum handlers).
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let result = rt.block_on(async move {
                let mut noop = |_: &str| {};
                match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    chat_with_fallback(
                        &messages,
                        model_override.as_deref(),
                        &ai_config,
                        &providers,
                        false,
                        &mut noop,
                    ),
                )
                .await
                {
                    Ok(Ok(result)) => Some(result.content),
                    _ => None,
                }
            });
            let _ = tx.send(result);
        });

        if let Ok(Some(summary)) = rx.await {
            if !summary.is_empty() {
                return Ok(SummaryResult {
                    summary,
                    source_url,
                    source_title,
                    ai_used: true,
                    duration_ms: start.elapsed().as_millis() as u64,
                });
            }
        }
    }

    // Heuristic fallback: take first N sentences
    let sentences: Vec<&str> = content.split(". ").collect();
    let n = config.length.fallback_sentences().min(sentences.len());
    let summary = sentences[..n]
        .iter()
        .map(|s| format!("{}.", s.trim()))
        .collect::<Vec<_>>()
        .join(" ");

    Ok(SummaryResult {
        summary,
        source_url,
        source_title,
        ai_used: false,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_length_descriptions() {
        assert!(SummaryLength::Short.description().contains("100"));
        assert!(SummaryLength::Medium.description().contains("300"));
        assert!(SummaryLength::Long.description().contains("700"));
    }

    #[test]
    fn default_config() {
        let config = SummarizeConfig::default();
        assert_eq!(config.length, SummaryLength::Medium);
        assert!(config.model.is_none());
    }

    #[tokio::test]
    async fn summarize_raw_text() {
        let text = "Rust is a systems language. It ensures memory safety. It has zero-cost abstractions. It supports async/await. It has great tooling.";
        let config = SummarizeConfig {
            length: SummaryLength::Short,
            model: None,
        };
        let hsx = HsxConfig::default();
        let result = summarize(text, &config, &hsx).await.unwrap();
        assert!(!result.summary.is_empty());
        assert!(result.source_url.is_none());
    }

    #[tokio::test]
    async fn summarize_empty_text() {
        let config = SummarizeConfig::default();
        let hsx = HsxConfig::default();
        let result = summarize("", &config, &hsx).await.unwrap();
        assert!(result.summary.is_empty());
        assert!(!result.ai_used);
    }
}
