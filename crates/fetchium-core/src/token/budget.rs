//! Adaptive Token Budget (ATB) — dynamic token allocation based on query complexity.
//!
//! Simple factual queries get small token budgets (200-500 tokens).
//! Complex deep-analysis queries get large budgets (5000+ tokens).
//! This prevents wasting tokens on easy queries while ensuring thorough
//! coverage for hard ones.

use crate::rank::fusion::QueryIntent;

/// Token budget configuration.
#[derive(Debug, Clone)]
pub struct BudgetConfig {
    /// Minimum token budget for any query.
    pub min_budget: u32,
    /// Maximum token budget for any query.
    pub max_budget: u32,
    /// Number of results that affect budget (more results = more tokens).
    pub tokens_per_result: u32,
    /// Base overhead per query (formatting, metadata, etc.).
    pub base_overhead: u32,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            min_budget: 200,
            max_budget: 10_000,
            tokens_per_result: 150,
            base_overhead: 100,
        }
    }
}

/// A computed token budget with allocation breakdown.
#[derive(Debug, Clone)]
pub struct AdaptiveBudget {
    /// Total allocated tokens.
    pub total: u32,
    /// Tokens allocated for result content.
    pub content_tokens: u32,
    /// Tokens reserved for metadata/formatting.
    pub overhead_tokens: u32,
    /// Recommended max tokens per result.
    pub per_result_limit: u32,
    /// Budget tier for logging.
    pub tier: BudgetTier,
}

/// Budget tier categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetTier {
    /// Minimal: quick factual answers (200-500 tokens).
    Minimal,
    /// Standard: typical searches (500-2000 tokens).
    Standard,
    /// Extended: complex queries (2000-5000 tokens).
    Extended,
    /// Deep: research/analysis queries (5000-10000 tokens).
    Deep,
}

/// Compute the token budget for a query based on intent and result count.
pub fn compute_budget(
    intent: QueryIntent,
    result_count: usize,
    complexity_score: f64,
    config: &BudgetConfig,
) -> AdaptiveBudget {
    // Base budget from intent
    let intent_base = intent_base_budget(intent);

    // Scale by complexity (0.0 → 0.5x, 0.5 → 1.0x, 1.0 → 2.0x)
    let complexity_multiplier = 0.5 + complexity_score * 1.5;

    // Result-based budget
    let result_budget = result_count as u32 * config.tokens_per_result;

    // Combined
    let raw_total =
        (intent_base as f64 * complexity_multiplier) as u32 + result_budget + config.base_overhead;

    let total = raw_total.clamp(config.min_budget, config.max_budget);
    let overhead = config.base_overhead.min(total);
    let content_tokens = total - overhead;

    let per_result_limit = if result_count > 0 {
        content_tokens / result_count as u32
    } else {
        content_tokens
    };

    let tier = if total <= 500 {
        BudgetTier::Minimal
    } else if total <= 2000 {
        BudgetTier::Standard
    } else if total <= 5000 {
        BudgetTier::Extended
    } else {
        BudgetTier::Deep
    };

    AdaptiveBudget {
        total,
        content_tokens,
        overhead_tokens: overhead,
        per_result_limit,
        tier,
    }
}

/// Get the PDS (Progressive Detail Streaming) tier token limits.
pub fn pds_tier_limit(tier_name: &str) -> u32 {
    match tier_name {
        "key_facts" => 200,
        "summary" => 1000,
        "detailed" => 5000,
        "complete" => 10_000,
        _ => 1000,
    }
}

/// Suggest the PDS tier based on available token budget.
pub fn suggest_pds_tier(budget: &AdaptiveBudget) -> &'static str {
    match budget.tier {
        BudgetTier::Minimal => "key_facts",
        BudgetTier::Standard => "summary",
        BudgetTier::Extended => "detailed",
        BudgetTier::Deep => "complete",
    }
}

// ─── Internal ─────────────────────────────────────────────

fn intent_base_budget(intent: QueryIntent) -> u32 {
    match intent {
        QueryIntent::Factual => 300,
        QueryIntent::HowTo => 800,
        QueryIntent::Comparison => 1500,
        QueryIntent::Verification => 600,
        QueryIntent::CurrentEvents => 700,
        QueryIntent::DeepAnalysis => 3000,
        QueryIntent::Code => 1200,
        QueryIntent::Academic => 2000,
        QueryIntent::Opinion => 800,
        QueryIntent::Data => 1000,
        QueryIntent::Informational => 500, // concise definitions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factual_query_small_budget() {
        let config = BudgetConfig::default();
        let budget = compute_budget(QueryIntent::Factual, 1, 0.1, &config);
        assert!(
            budget.total <= 1000,
            "factual should be small, got {}",
            budget.total
        );
        assert!(matches!(
            budget.tier,
            BudgetTier::Minimal | BudgetTier::Standard
        ));
    }

    #[test]
    fn deep_analysis_large_budget() {
        let config = BudgetConfig::default();
        let budget = compute_budget(QueryIntent::DeepAnalysis, 10, 0.9, &config);
        assert!(
            budget.total >= 2000,
            "deep analysis should be large, got {}",
            budget.total
        );
        assert!(matches!(
            budget.tier,
            BudgetTier::Extended | BudgetTier::Deep
        ));
    }

    #[test]
    fn budget_respects_min_max() {
        let config = BudgetConfig {
            min_budget: 100,
            max_budget: 500,
            ..BudgetConfig::default()
        };
        let budget = compute_budget(QueryIntent::DeepAnalysis, 100, 1.0, &config);
        assert!(budget.total <= 500);
        assert!(budget.total >= 100);
    }

    #[test]
    fn zero_results_uses_full_content_budget() {
        let config = BudgetConfig::default();
        let budget = compute_budget(QueryIntent::Factual, 0, 0.5, &config);
        assert_eq!(budget.per_result_limit, budget.content_tokens);
    }

    #[test]
    fn complexity_scales_budget() {
        let config = BudgetConfig::default();
        let low = compute_budget(QueryIntent::HowTo, 5, 0.1, &config);
        let high = compute_budget(QueryIntent::HowTo, 5, 0.9, &config);
        assert!(
            high.total > low.total,
            "higher complexity should mean bigger budget"
        );
    }

    #[test]
    fn per_result_limit_divides_evenly() {
        let config = BudgetConfig::default();
        let budget = compute_budget(QueryIntent::Code, 5, 0.5, &config);
        assert!(budget.per_result_limit > 0);
        assert!(budget.per_result_limit <= budget.content_tokens);
    }

    #[test]
    fn pds_tier_suggestions() {
        let minimal = AdaptiveBudget {
            total: 300,
            content_tokens: 200,
            overhead_tokens: 100,
            per_result_limit: 200,
            tier: BudgetTier::Minimal,
        };
        assert_eq!(suggest_pds_tier(&minimal), "key_facts");

        let deep = AdaptiveBudget {
            total: 8000,
            content_tokens: 7900,
            overhead_tokens: 100,
            per_result_limit: 790,
            tier: BudgetTier::Deep,
        };
        assert_eq!(suggest_pds_tier(&deep), "complete");
    }

    #[test]
    fn pds_tier_limits() {
        assert_eq!(pds_tier_limit("key_facts"), 200);
        assert_eq!(pds_tier_limit("summary"), 1000);
        assert_eq!(pds_tier_limit("detailed"), 5000);
        assert_eq!(pds_tier_limit("complete"), 10_000);
        assert_eq!(pds_tier_limit("unknown"), 1000);
    }

    #[test]
    fn overhead_never_exceeds_total() {
        let config = BudgetConfig {
            base_overhead: 5000,
            min_budget: 200,
            max_budget: 300,
            ..BudgetConfig::default()
        };
        let budget = compute_budget(QueryIntent::Factual, 0, 0.0, &config);
        assert!(budget.overhead_tokens <= budget.total);
    }
}
