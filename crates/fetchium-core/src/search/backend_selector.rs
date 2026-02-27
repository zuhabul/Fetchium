//! Adaptive Backend Selector (ABS) — smart per-query backend routing.
//!
//! Novel algorithm: Instead of querying all backends for every query, ABS
//! dynamically selects the most promising backends based on:
//!
//! 1. **Query intent** — Academic queries prefer arXiv + Scholar; code queries
//!    prefer GitHub + StackOverflow; news queries prefer HackerNews + Reddit.
//! 2. **Historical success rates** — Backends that have returned good results
//!    for similar queries in the past are preferred.
//! 3. **Current health** — Backends with open circuit breakers are skipped.
//!
//! Uses a **UCB1 multi-armed bandit** to balance exploration (trying backends
//! that haven't been used recently) vs exploitation (using backends known to work).

use crate::rank::fusion::QueryIntent;
use crate::types::BackendId;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Configuration for the Adaptive Backend Selector.
#[derive(Debug, Clone)]
pub struct AbsConfig {
    /// Minimum number of backends to always include.
    pub min_backends: usize,
    /// Maximum number of backends to select.
    pub max_backends: usize,
    /// UCB1 exploration parameter (higher = more exploration).
    pub exploration_factor: f64,
    /// Whether to always include at least one "reliable" backend.
    pub always_include_reliable: bool,
}

impl Default for AbsConfig {
    fn default() -> Self {
        Self {
            min_backends: 3,
            max_backends: 8,
            exploration_factor: 1.41, // sqrt(2), standard UCB1
            always_include_reliable: true,
        }
    }
}

/// Per-backend performance statistics.
#[derive(Debug)]
struct BackendStats {
    /// Total number of times this backend was selected.
    selections: AtomicU64,
    /// Total number of successful queries (returned >= 1 result).
    successes: AtomicU64,
    /// Total reward (sum of quality scores from results).
    total_reward: AtomicU64, // stored as reward * 1000 to avoid floating point atomics
}

impl BackendStats {
    fn new() -> Self {
        Self {
            selections: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            total_reward: AtomicU64::new(0),
        }
    }

    fn success_rate(&self) -> f64 {
        let sel = self.selections.load(Ordering::Relaxed);
        if sel == 0 {
            return 0.5; // optimistic prior
        }
        self.successes.load(Ordering::Relaxed) as f64 / sel as f64
    }

    fn avg_reward(&self) -> f64 {
        let sel = self.selections.load(Ordering::Relaxed);
        if sel == 0 {
            return 0.5; // optimistic prior
        }
        (self.total_reward.load(Ordering::Relaxed) as f64 / 1000.0) / sel as f64
    }
}

/// Intent-to-backend affinity matrix.
///
/// Maps query intents to their preferred backends with affinity scores.
fn intent_affinity(intent: &QueryIntent, backend: &BackendId) -> f64 {
    match intent {
        QueryIntent::Academic | QueryIntent::Data => match backend {
            BackendId::Arxiv => 0.95,
            BackendId::GoogleScholar => 0.90,
            BackendId::Wikipedia => 0.70,
            BackendId::StackOverflow => 0.60,
            BackendId::Github => 0.50,
            _ => 0.30,
        },
        QueryIntent::Code => match backend {
            BackendId::Github => 0.95,
            BackendId::StackOverflow => 0.90,
            BackendId::HackerNews => 0.50,
            BackendId::Reddit => 0.40,
            _ => 0.30,
        },
        QueryIntent::CurrentEvents => match backend {
            BackendId::HackerNews => 0.90,
            BackendId::Reddit => 0.85,
            BackendId::Google => 0.70,
            BackendId::Bing => 0.65,
            BackendId::DuckDuckGo => 0.60,
            _ => 0.30,
        },
        QueryIntent::Comparison => match backend {
            BackendId::Wikipedia => 0.80,
            BackendId::DuckDuckGo => 0.75,
            BackendId::Google => 0.70,
            BackendId::Reddit => 0.65,
            BackendId::StackOverflow => 0.60,
            _ => 0.40,
        },
        QueryIntent::Factual => match backend {
            BackendId::Wikipedia => 0.95,
            BackendId::DuckDuckGo => 0.70,
            BackendId::Google => 0.65,
            BackendId::Brave => 0.60,
            BackendId::Github => 0.20, // repos rarely answer factual questions
            _ => 0.40,
        },
        // Definitional queries: Wikipedia/web search excel; GitHub repos are almost useless
        QueryIntent::Informational => match backend {
            BackendId::Wikipedia => 0.95,
            BackendId::DuckDuckGo => 0.80,
            BackendId::Google => 0.75,
            BackendId::Brave => 0.70,
            BackendId::Bing => 0.65,
            BackendId::Searxng => 0.70,
            BackendId::StackOverflow => 0.55,
            BackendId::Reddit => 0.45,
            BackendId::HackerNews => 0.40,
            BackendId::Arxiv => 0.35,
            BackendId::Github => 0.10, // nearly never relevant for definitions
            _ => 0.40,
        },
        QueryIntent::Verification => match backend {
            BackendId::Wikipedia => 0.90,
            BackendId::Arxiv => 0.85,
            BackendId::GoogleScholar => 0.80,
            BackendId::DuckDuckGo => 0.60,
            _ => 0.35,
        },
        // HowTo, DeepAnalysis, Opinion — use all backends roughly equally
        _ => match backend {
            BackendId::DuckDuckGo => 0.70,
            BackendId::Google => 0.70,
            BackendId::StackOverflow => 0.65,
            BackendId::Bing => 0.60,
            BackendId::Brave => 0.60,
            BackendId::Wikipedia => 0.55,
            _ => 0.50,
        },
    }
}

/// Adaptive Backend Selector using UCB1 multi-armed bandit.
#[derive(Clone)]
pub struct AdaptiveBackendSelector {
    stats: Arc<DashMap<String, BackendStats>>,
    total_rounds: Arc<AtomicU64>,
    config: AbsConfig,
}

/// Selection result with metadata.
#[derive(Debug, Clone)]
pub struct BackendSelection {
    /// The selected backends, ordered by expected value.
    pub backends: Vec<BackendId>,
    /// UCB1 scores for each selected backend (parallel with `backends`).
    pub scores: Vec<f64>,
}

impl AdaptiveBackendSelector {
    /// Create a new selector.
    pub fn new(config: AbsConfig) -> Self {
        Self {
            stats: Arc::new(DashMap::new()),
            total_rounds: Arc::new(AtomicU64::new(0)),
            config,
        }
    }

    /// Select backends for a query based on intent and historical performance.
    pub fn select(
        &self,
        intent: &QueryIntent,
        available_backends: &[BackendId],
        unhealthy_backends: &[BackendId],
    ) -> BackendSelection {
        let total = self.total_rounds.fetch_add(1, Ordering::Relaxed) + 1;

        // Filter out unhealthy backends
        let healthy: Vec<&BackendId> = available_backends
            .iter()
            .filter(|b| !unhealthy_backends.contains(b))
            .collect();

        if healthy.is_empty() {
            return BackendSelection {
                backends: vec![],
                scores: vec![],
            };
        }

        // Score each backend with UCB1 + intent affinity
        let mut scored: Vec<(BackendId, f64)> = healthy
            .iter()
            .map(|&backend| {
                let key = backend.to_string();
                let ucb1 = self.ucb1_score(&key, total);
                let affinity = intent_affinity(intent, backend);
                let combined = ucb1 * 0.4 + affinity * 0.6; // 60% intent, 40% history
                (backend.clone(), combined)
            })
            .collect();

        // Sort by combined score (descending)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select top N backends (bounded by config)
        let n = scored
            .len()
            .min(self.config.max_backends)
            .max(self.config.min_backends.min(scored.len()));

        let mut backends: Vec<BackendId> = scored[..n].iter().map(|(b, _)| b.clone()).collect();
        let mut scores: Vec<f64> = scored[..n].iter().map(|(_, s)| *s).collect();

        // Ensure at least one reliable backend is included
        if self.config.always_include_reliable {
            let reliable = [BackendId::Wikipedia, BackendId::DuckDuckGo];
            let has_reliable = backends.iter().any(|b| reliable.contains(b));
            if !has_reliable {
                if let Some(reliable_b) = reliable
                    .iter()
                    .find(|b| healthy.contains(b) && !backends.contains(b))
                {
                    if backends.len() >= self.config.max_backends {
                        backends.pop();
                        scores.pop();
                    }
                    backends.push(reliable_b.clone());
                    scores.push(0.5);
                }
            }
        }

        BackendSelection { backends, scores }
    }

    /// Report the outcome of using a backend.
    pub fn report_outcome(&self, backend: &BackendId, result_count: usize, quality: f64) {
        let key = backend.to_string();
        let entry = self.stats.entry(key).or_insert_with(BackendStats::new);
        entry.selections.fetch_add(1, Ordering::Relaxed);
        if result_count > 0 {
            entry.successes.fetch_add(1, Ordering::Relaxed);
        }
        let reward_int = (quality.clamp(0.0, 1.0) * 1000.0) as u64;
        entry.total_reward.fetch_add(reward_int, Ordering::Relaxed);
    }

    /// Get performance summary for all backends.
    pub fn summary(&self) -> Vec<(String, f64, u64)> {
        self.stats
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let rate = entry.value().success_rate();
                let count = entry.value().selections.load(Ordering::Relaxed);
                (key, rate, count)
            })
            .collect()
    }

    /// UCB1 score for a backend.
    ///
    /// UCB1 = avg_reward + C * sqrt(ln(total_rounds) / selections)
    fn ucb1_score(&self, key: &str, total_rounds: u64) -> f64 {
        match self.stats.get(key) {
            Some(stats) => {
                let selections = stats.selections.load(Ordering::Relaxed);
                if selections == 0 {
                    return 1.0; // moderate exploration bonus for unseen
                }
                let avg = stats.avg_reward();
                let exploration = self.config.exploration_factor
                    * ((total_rounds as f64).ln() / selections as f64).sqrt();
                (avg + exploration).min(1.0)
            }
            None => 1.0, // unseen backend — moderate exploration bonus
        }
    }
}

impl Default for AdaptiveBackendSelector {
    fn default() -> Self {
        Self::new(AbsConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_backends() -> Vec<BackendId> {
        vec![
            BackendId::DuckDuckGo,
            BackendId::Google,
            BackendId::Bing,
            BackendId::Wikipedia,
            BackendId::HackerNews,
            BackendId::Arxiv,
            BackendId::Github,
            BackendId::Reddit,
            BackendId::StackOverflow,
            BackendId::Brave,
        ]
    }

    #[test]
    fn academic_query_prefers_arxiv() {
        let abs = AdaptiveBackendSelector::default();
        let selection = abs.select(&QueryIntent::Academic, &all_backends(), &[]);

        // arXiv should be in the top 3 for academic queries
        let top3: Vec<_> = selection.backends.iter().take(3).collect();
        assert!(
            top3.contains(&&BackendId::Arxiv),
            "Academic query should include arXiv in top 3, got: {:?}",
            top3
        );
    }

    #[test]
    fn code_query_prefers_github() {
        let abs = AdaptiveBackendSelector::default();
        let selection = abs.select(&QueryIntent::Code, &all_backends(), &[]);

        let top3: Vec<_> = selection.backends.iter().take(3).collect();
        assert!(
            top3.contains(&&BackendId::Github),
            "Code query should include GitHub in top 3, got: {:?}",
            top3
        );
    }

    #[test]
    fn unhealthy_backends_excluded() {
        let abs = AdaptiveBackendSelector::default();
        let unhealthy = vec![BackendId::Google, BackendId::Bing];
        let selection = abs.select(&QueryIntent::Factual, &all_backends(), &unhealthy);

        assert!(!selection.backends.contains(&BackendId::Google));
        assert!(!selection.backends.contains(&BackendId::Bing));
    }

    #[test]
    fn respects_max_backends() {
        let config = AbsConfig {
            max_backends: 3,
            min_backends: 1,
            ..AbsConfig::default()
        };
        let abs = AdaptiveBackendSelector::new(config);
        let selection = abs.select(&QueryIntent::Factual, &all_backends(), &[]);

        assert!(selection.backends.len() <= 3);
    }

    #[test]
    fn learning_improves_selection() {
        let abs = AdaptiveBackendSelector::default();

        // Report that Wikipedia consistently gives great results
        for _ in 0..20 {
            abs.report_outcome(&BackendId::Wikipedia, 10, 0.95);
        }
        // Report that Bing consistently fails
        for _ in 0..20 {
            abs.report_outcome(&BackendId::Bing, 0, 0.0);
        }

        let selection = abs.select(&QueryIntent::Factual, &all_backends(), &[]);
        let wiki_pos = selection
            .backends
            .iter()
            .position(|b| *b == BackendId::Wikipedia);
        let bing_pos = selection
            .backends
            .iter()
            .position(|b| *b == BackendId::Bing);

        // Wikipedia should rank higher than Bing after learning
        match (wiki_pos, bing_pos) {
            (Some(w), Some(b)) => assert!(w < b, "Wikipedia should rank above Bing after learning"),
            (Some(_), None) => {} // Bing might be excluded entirely — that's fine
            _ => panic!("Wikipedia should be selected"),
        }
    }

    #[test]
    fn reliable_backend_always_included() {
        let config = AbsConfig {
            always_include_reliable: true,
            max_backends: 3,
            min_backends: 1,
            ..AbsConfig::default()
        };
        let abs = AdaptiveBackendSelector::new(config);

        // Only offer backends that are NOT reliable
        let backends = vec![
            BackendId::Arxiv,
            BackendId::Github,
            BackendId::HackerNews,
            BackendId::Wikipedia, // reliable
        ];

        let selection = abs.select(&QueryIntent::Code, &backends, &[]);
        assert!(
            selection.backends.contains(&BackendId::Wikipedia)
                || selection.backends.contains(&BackendId::DuckDuckGo),
            "Should include at least one reliable backend"
        );
    }

    #[test]
    fn empty_available_returns_empty() {
        let abs = AdaptiveBackendSelector::default();
        let selection = abs.select(&QueryIntent::Factual, &[], &[]);
        assert!(selection.backends.is_empty());
    }
}
