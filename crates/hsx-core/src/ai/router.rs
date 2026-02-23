//! Model router — selects the best available Ollama model by query complexity (PRD §23).
//!
//! Default models use 2026 top-tier local LLMs:
//! - Small (1-3B): gemma3:1b, qwen3:1.7b
//! - Medium (7-9B): deepseek-r1:7b, qwen3:8b, gemma3:9b
//! - Large (14B+): deepseek-r1:14b, qwen3:14b, llama4:scout

use crate::ai::types::{AiConfig, ModelTier, OllamaModel};

/// Complexity signals derived from the query for model routing.
#[derive(Debug)]
struct ComplexitySignals {
    word_count: usize,
    has_comparison: bool,
    has_synthesis: bool,
    has_multi_hop: bool,
    question_depth: usize,
    source_count: usize,
}

/// Determine the appropriate model tier for a query + expected source count.
pub fn route_model(query: &str, source_count: usize) -> ModelTier {
    let signals = analyze_complexity(query, source_count);
    let score = compute_complexity_score(&signals);

    match score {
        s if s < 0.3 => ModelTier::Small,
        s if s < 0.7 => ModelTier::Medium,
        _ => ModelTier::Large,
    }
}

fn analyze_complexity(query: &str, source_count: usize) -> ComplexitySignals {
    let lower = query.to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();

    ComplexitySignals {
        word_count: words.len(),
        has_comparison: lower.contains(" vs ")
            || lower.contains("compare")
            || lower.contains("difference between")
            || lower.contains("better than"),
        has_synthesis: lower.contains("explain")
            || lower.contains("analyze")
            || lower.contains("summarize")
            || lower.contains("synthesize")
            || lower.contains("implications"),
        has_multi_hop: lower.contains("and then")
            || lower.contains("because")
            || lower.contains("implications")
            || lower.contains("how does")
            || lower.contains("why does"),
        question_depth: lower.matches('?').count() + lower.matches(" and ").count(),
        source_count,
    }
}

fn compute_complexity_score(signals: &ComplexitySignals) -> f64 {
    let mut score = 0.0_f64;

    score += match signals.word_count {
        0..=5 => 0.1,
        6..=12 => 0.2,
        13..=25 => 0.4,
        _ => 0.6,
    };

    if signals.has_comparison { score += 0.2; }
    if signals.has_synthesis  { score += 0.2; }
    if signals.has_multi_hop  { score += 0.15; }
    score += (signals.question_depth as f64) * 0.1;
    score += (signals.source_count as f64) * 0.02;

    score.min(1.0)
}

/// Select the fastest appropriate model for latency-sensitive tasks (HyDE, intent classification).
///
/// Returns `fast_model` from config if set, otherwise defaults to the top small-tier model.
/// Falls back gracefully without requiring the full Ollama model list.
pub fn select_fast_model(config: &AiConfig) -> &str {
    config.fast_model.as_deref().unwrap_or(SMALL_PREFERRED[0])
}

/// Preferred model names per tier (2026 top local LLMs, in priority order).
const SMALL_PREFERRED: &[&str]  = &["gemma3:1b",      "qwen3:1.7b",   "phi3:mini",    "qwen2.5:1.5b"];
const MEDIUM_PREFERRED: &[&str] = &["deepseek-r1:7b", "qwen3:8b",     "gemma3:9b",    "llama3.2:8b", "mistral:7b"];
const LARGE_PREFERRED: &[&str]  = &["deepseek-r1:14b","qwen3:14b",    "llama4:scout", "llama3.2:70b","mixtral:8x7b"];

/// Select the best available model for the desired tier.
///
/// Checks installed models against preferred lists and falls back gracefully
/// to neighbouring tiers if the desired tier has no model installed.
pub fn select_model(
    available: &[OllamaModel],
    tier: ModelTier,
    override_model: Option<&str>,
) -> Option<String> {
    // User-specified override takes absolute priority.
    if let Some(name) = override_model {
        if available.iter().any(|m| m.name == name || m.name.starts_with(name)) {
            return Some(name.to_string());
        }
    }

    // Classify installed models by parameter count.
    let mut small: Vec<&OllamaModel>  = Vec::new();
    let mut medium: Vec<&OllamaModel> = Vec::new();
    let mut large: Vec<&OllamaModel>  = Vec::new();

    for model in available {
        let pb = estimate_param_billions(model);
        match pb {
            b if b <= 3.5  => small.push(model),
            b if b <= 11.0 => medium.push(model),
            _              => large.push(model),
        }
    }

    // Sort each tier by the preferred list order.
    let pick_preferred = |bucket: &Vec<&OllamaModel>, preferred: &[&str]| -> Option<String> {
        for pref in preferred {
            if let Some(m) = bucket.iter().find(|m| m.name.contains(*pref) || m.name.starts_with(pref.split(':').next().unwrap_or(pref))) {
                return Some(m.name.clone());
            }
        }
        // Fall back to whatever is installed in this tier
        bucket.first().map(|m| m.name.clone())
    };

    let candidates: [(&Vec<&OllamaModel>, &[&str]); 3] = match tier {
        ModelTier::Small  => [(&small, SMALL_PREFERRED),  (&medium, MEDIUM_PREFERRED), (&large, LARGE_PREFERRED)],
        ModelTier::Medium => [(&medium, MEDIUM_PREFERRED), (&small, SMALL_PREFERRED),  (&large, LARGE_PREFERRED)],
        ModelTier::Large  => [(&large, LARGE_PREFERRED),  (&medium, MEDIUM_PREFERRED), (&small, SMALL_PREFERRED)],
    };

    for (bucket, preferred) in candidates {
        if let Some(name) = pick_preferred(bucket, preferred) {
            return Some(name);
        }
    }

    None
}

/// Estimate parameter count in billions from Ollama model metadata.
fn estimate_param_billions(model: &OllamaModel) -> f64 {
    if let Some(ref ps) = model.parameter_size {
        let cleaned = ps.to_lowercase().replace('b', "");
        if let Ok(val) = cleaned.trim().parse::<f64>() {
            return val;
        }
    }
    // Fallback: Q4 quantized ≈ 0.5 GB per billion parameters
    let gb = model.size as f64 / (1024.0 * 1024.0 * 1024.0);
    gb / 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_model(name: &str, size_gb: f64, param_b: &str) -> OllamaModel {
        OllamaModel {
            name: name.into(),
            size: (size_gb * 1024.0 * 1024.0 * 1024.0) as u64,
            parameter_size: Some(param_b.into()),
            quantization_level: None,
        }
    }

    #[test]
    fn simple_query_routes_small() {
        assert_eq!(route_model("what is Rust", 3), ModelTier::Small);
    }

    #[test]
    fn medium_query_routes_medium() {
        assert_eq!(
            route_model("explain async Rust programming with examples", 5),
            ModelTier::Medium
        );
    }

    #[test]
    fn complex_query_routes_large() {
        let q = "compare Rust vs Go vs C++ for systems programming with tradeoffs and implications";
        assert_eq!(route_model(q, 10), ModelTier::Large);
    }

    #[test]
    fn select_model_picks_preferred_medium() {
        let available = vec![
            make_model("deepseek-r1:7b", 4.0, "7B"),
            make_model("gemma3:1b", 0.8, "1B"),
        ];
        let chosen = select_model(&available, ModelTier::Medium, None);
        assert_eq!(chosen, Some("deepseek-r1:7b".into()));
    }

    #[test]
    fn select_model_falls_back_to_small() {
        let available = vec![make_model("gemma3:1b", 0.8, "1B")];
        let chosen = select_model(&available, ModelTier::Medium, None);
        assert!(chosen.is_some());
    }

    #[test]
    fn select_model_override_wins() {
        let available = vec![
            make_model("deepseek-r1:7b", 4.0, "7B"),
            make_model("gemma3:1b", 0.8, "1B"),
        ];
        let chosen = select_model(&available, ModelTier::Small, Some("deepseek-r1:7b"));
        assert_eq!(chosen, Some("deepseek-r1:7b".into()));
    }

    #[test]
    fn no_models_returns_none() {
        let chosen = select_model(&[], ModelTier::Medium, None);
        assert!(chosen.is_none());
    }

    #[test]
    fn select_fast_model_uses_fast_model_when_set() {
        let config = AiConfig {
            fast_model: Some("qwen3:1.7b".into()),
            ..AiConfig::default()
        };
        assert_eq!(select_fast_model(&config), "qwen3:1.7b");
    }

    #[test]
    fn select_fast_model_falls_back_to_small_preferred() {
        let config = AiConfig::default(); // fast_model is None
        let model = select_fast_model(&config);
        assert!(!model.is_empty());
        assert_eq!(model, SMALL_PREFERRED[0]);
    }
}
