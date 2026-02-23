//! AI-specific types: config, model tiers, chat messages, context assembly.

use serde::{Deserialize, Serialize};

/// Which model tier to route to based on query complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    /// Simple factual queries: 1-3B models (gemma3:1b, qwen3:1.7b)
    Small,
    /// Standard queries: 7-9B models (deepseek-r1:7b, qwen3:8b, gemma3:9b)
    Medium,
    /// Complex synthesis: 14B+ models (deepseek-r1:14b, qwen3:14b, llama4:scout)
    Large,
}

/// A single chat message in Ollama API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,    // "system", "user", "assistant"
    pub content: String,
}

/// Configuration for the AI engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// Ollama base URL (default: "http://localhost")
    pub ollama_host: String,
    /// Ollama port (default: 11434)
    pub ollama_port: u16,
    /// Optional model name override (bypasses routing)
    pub default_model: Option<String>,
    /// Optional fast model for latency-sensitive tasks (HyDE, intent classification).
    /// Falls back to `default_model` when unset.
    pub fast_model: Option<String>,
    /// Request timeout in seconds (default: 120)
    pub timeout_secs: u64,
    /// Maximum context tokens to assemble (default: 4096)
    pub max_context_tokens: usize,
    /// Sampling temperature (default: 0.3 for factual synthesis)
    pub temperature: f32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost".into(),
            ollama_port: 11434,
            default_model: None,
            fast_model: None,
            timeout_secs: 120,
            max_context_tokens: 4096,
            temperature: 0.3,
        }
    }
}

impl AiConfig {
    /// Build an `AiConfig` from the top-level `HsxConfig`.
    ///
    /// Reads `ollama_host`, `default_model`, and `max_tokens` from the `[ai]`
    /// section. The host string may include the port (e.g. `"http://localhost:11434"`).
    pub fn from_hsx_config(hsx: &crate::config::HsxConfig) -> Self {
        let (host, port) = parse_ollama_host(&hsx.ai.ollama_host);
        let default_model = if hsx.ai.default_model.is_empty() {
            None
        } else {
            Some(hsx.ai.default_model.clone())
        };
        Self {
            ollama_host: host,
            ollama_port: port,
            default_model,
            fast_model: hsx.ai.fast_model.clone(),
            max_context_tokens: hsx.ai.max_tokens as usize,
            ..Self::default()
        }
    }
}

/// Parse `"http://host:port"` into `("http://host", port)`, defaulting port to 11434.
fn parse_ollama_host(url: &str) -> (String, u16) {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("https://") {
        ("https", r)
    } else if let Some(r) = url.strip_prefix("http://") {
        ("http", r)
    } else {
        return (url.to_string(), 11434);
    };
    if let Some(colon) = rest.rfind(':') {
        let host_part = &rest[..colon];
        let port_str = &rest[colon + 1..];
        if let Ok(p) = port_str.parse::<u16>() {
            return (format!("{}://{}", scheme, host_part), p);
        }
    }
    (format!("{}://{}", scheme, rest), 11434)
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use crate::config::HsxConfig;

    #[test]
    fn parse_host_with_port() {
        let (h, p) = parse_ollama_host("http://localhost:11434");
        assert_eq!(h, "http://localhost");
        assert_eq!(p, 11434);
    }

    #[test]
    fn parse_host_without_port() {
        let (h, p) = parse_ollama_host("http://myhost");
        assert_eq!(h, "http://myhost");
        assert_eq!(p, 11434);
    }

    #[test]
    fn from_hsx_config_reads_ollama_host() {
        let mut hsx = HsxConfig::default();
        hsx.ai.ollama_host = "http://localhost:11434".into();
        hsx.ai.default_model = "deepseek-r1:7b".into();
        hsx.ai.max_tokens = 8192;
        let ai = AiConfig::from_hsx_config(&hsx);
        assert_eq!(ai.ollama_host, "http://localhost");
        assert_eq!(ai.ollama_port, 11434);
        assert_eq!(ai.default_model, Some("deepseek-r1:7b".into()));
        assert_eq!(ai.max_context_tokens, 8192);
    }

    #[test]
    fn from_hsx_config_empty_model_is_none() {
        let mut hsx = HsxConfig::default();
        hsx.ai.default_model = String::new();
        let ai = AiConfig::from_hsx_config(&hsx);
        assert!(ai.default_model.is_none());
    }
}

/// Ollama model info returned by `/api/tags`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    /// Total disk size in bytes
    pub size: u64,
    /// e.g. "7B", "14B"
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// A single streamed chunk from Ollama `/api/chat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatChunk {
    pub model: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

/// A ranked source ready for sandwich layout assembly.
#[derive(Debug, Clone)]
pub struct RankedSource {
    /// Original position in search results
    pub index: usize,
    /// Extracted text content
    pub content: String,
    /// Confidence score 0.0–1.0 from ranking/validation
    pub confidence: f64,
    pub url: String,
    pub title: String,
}

/// Assembled context after sandwich layout reordering.
#[derive(Debug, Clone)]
pub struct SandwichContext {
    pub system_prompt: String,
    /// Sources formatted in sandwich order
    pub user_context: String,
    /// Maps sandwich position -> original source index
    pub source_map: Vec<usize>,
    /// Approximate total tokens used
    pub total_tokens: usize,
}

/// Result of the full AI preview pipeline.
#[derive(Debug, Clone)]
pub struct AiPreviewResult {
    pub answer: String,
    pub model_used: String,
    pub sources_used: usize,
    pub streaming: bool,
    /// True if Ollama was unavailable and we fell back to search results
    pub fallback: bool,
}
