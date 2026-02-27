//! Speculative Pre-Ranking Engine (SPRE) — show "best so far" results instantly.
//!
//! Novel algorithm: As results stream in from multiple backends concurrently,
//! SPRE maintains a min-heap of top-K candidates with O(log K) insertion.
//! This allows the UI to display progressively improving results instead of
//! waiting for all backends to complete.
//!
//! Key innovation: lightweight BM25 pre-scoring on arrival, combined with a
//! source-diversity bonus that prevents any single backend from dominating
//! early results.

use crate::rank::bm25_score;
use crate::types::ResultItem;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// A scored candidate in the pre-ranking heap.
#[derive(Debug, Clone)]
struct ScoredCandidate {
    item: ResultItem,
    pre_score: f64,
}

impl PartialEq for ScoredCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.pre_score == other.pre_score
    }
}

impl Eq for ScoredCandidate {}

/// Min-heap ordering — lowest score at top so we can evict the weakest.
impl PartialOrd for ScoredCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order: lowest score at the top of the heap (min-heap)
        other
            .pre_score
            .partial_cmp(&self.pre_score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Speculative Pre-Ranking Engine.
///
/// Maintains a bounded min-heap of the best results seen so far.
/// Thread-safe when used behind a mutex (the orchestrator handles this).
pub struct SpreRanker {
    query: String,
    heap: BinaryHeap<ScoredCandidate>,
    capacity: usize,
    /// Track how many results each backend has contributed to the top-K,
    /// used for the diversity bonus.
    backend_counts: std::collections::HashMap<String, usize>,
    /// Total results ingested (for stats).
    total_ingested: usize,
}

/// Configuration for SPRE.
#[derive(Debug, Clone)]
pub struct SpreConfig {
    /// Maximum number of top candidates to maintain.
    pub top_k: usize,
    /// Weight for source-diversity bonus (0.0 = off, 1.0 = strong).
    pub diversity_weight: f64,
    /// Minimum BM25 score to even consider a result (filters noise).
    pub score_threshold: f64,
}

impl Default for SpreConfig {
    fn default() -> Self {
        Self {
            top_k: 20,
            diversity_weight: 0.15,
            score_threshold: 0.01,
        }
    }
}

/// Snapshot of the current top-K results.
#[derive(Debug, Clone)]
pub struct SpreSnapshot {
    /// Current best results, sorted by pre-score descending.
    pub results: Vec<ResultItem>,
    /// How many total results have been ingested.
    pub total_ingested: usize,
    /// How many unique backends have contributed.
    pub backends_contributing: usize,
}

impl SpreRanker {
    /// Create a new SPRE ranker for the given query.
    pub fn new(query: &str, config: SpreConfig) -> Self {
        Self {
            query: query.to_string(),
            heap: BinaryHeap::with_capacity(config.top_k + 1),
            capacity: config.top_k,
            backend_counts: std::collections::HashMap::new(),
            total_ingested: 0,
        }
    }

    /// Ingest a batch of results from a single backend.
    ///
    /// Each result is pre-scored with BM25 + diversity bonus, then
    /// inserted into the min-heap. If the heap exceeds capacity,
    /// the weakest candidate is evicted. O(N log K) per batch.
    pub fn ingest(&mut self, results: Vec<ResultItem>, config: &SpreConfig) {
        for item in results {
            self.total_ingested += 1;

            // Lightweight BM25 pre-score
            let text = format!("{} {}", item.title, item.snippet);
            let bm25 = bm25_score(&text, &self.query);

            if bm25 < config.score_threshold {
                continue;
            }

            // Source-diversity bonus: backends with fewer results in top-K get a boost
            let backend_key = item.backend.to_string();
            let backend_count = self.backend_counts.get(&backend_key).copied().unwrap_or(0);
            let diversity_bonus = if backend_count == 0 {
                config.diversity_weight
            } else {
                config.diversity_weight / (backend_count as f64 + 1.0)
            };

            let pre_score = bm25 + diversity_bonus;

            let candidate = ScoredCandidate {
                item: ResultItem {
                    score: Some(pre_score),
                    ..item
                },
                pre_score,
            };

            if self.heap.len() < self.capacity {
                // Below capacity — always insert
                *self.backend_counts.entry(backend_key).or_insert(0) += 1;
                self.heap.push(candidate);
            } else if let Some(weakest) = self.heap.peek() {
                if pre_score > weakest.pre_score {
                    // Evict the weakest and insert the new candidate
                    let evicted = self.heap.pop().unwrap();
                    let evicted_key = evicted.item.backend.to_string();
                    if let Some(c) = self.backend_counts.get_mut(&evicted_key) {
                        *c = c.saturating_sub(1);
                    }
                    *self.backend_counts.entry(backend_key).or_insert(0) += 1;
                    self.heap.push(candidate);
                }
            }
        }
    }

    /// Get a snapshot of the current top-K, sorted by score descending.
    pub fn snapshot(&self) -> SpreSnapshot {
        let mut results: Vec<ResultItem> = self.heap.iter().map(|c| c.item.clone()).collect();

        // Sort by pre_score descending
        results.sort_by(|a, b| {
            let sa = a.score.unwrap_or(0.0);
            let sb = b.score.unwrap_or(0.0);
            sb.partial_cmp(&sa).unwrap_or(Ordering::Equal)
        });

        // Re-assign contiguous ranks
        for (i, item) in results.iter_mut().enumerate() {
            item.rank = (i + 1) as u32;
        }

        SpreSnapshot {
            results,
            total_ingested: self.total_ingested,
            backends_contributing: self.backend_counts.values().filter(|&&c| c > 0).count(),
        }
    }

    /// How many results are currently in the top-K.
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// Whether the ranker has any results.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_item(title: &str, url: &str, snippet: &str, backend: BackendId) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 0,
            backend,
            score: None,
            published_date: None,
        }
    }

    #[test]
    fn spre_basic_insertion() {
        let config = SpreConfig {
            top_k: 3,
            ..SpreConfig::default()
        };
        let mut ranker = SpreRanker::new("rust programming", config.clone());

        let results = vec![
            make_item(
                "Rust Book",
                "https://doc.rust-lang.org",
                "Learn Rust programming",
                BackendId::DuckDuckGo,
            ),
            make_item(
                "Python Tutorial",
                "https://python.org",
                "Learn Python basics",
                BackendId::DuckDuckGo,
            ),
        ];

        ranker.ingest(results, &config);
        let snap = ranker.snapshot();

        assert!(snap.results.len() <= 3);
        assert_eq!(snap.total_ingested, 2);
        // Rust result should score higher for "rust programming"
        assert_eq!(snap.results[0].title, "Rust Book");
    }

    #[test]
    fn spre_evicts_weakest() {
        let config = SpreConfig {
            top_k: 2,
            diversity_weight: 0.0,
            score_threshold: 0.0,
        };
        let mut ranker = SpreRanker::new("rust", config.clone());

        ranker.ingest(
            vec![make_item(
                "Weak",
                "https://a.com",
                "unrelated content about dogs",
                BackendId::Wikipedia,
            )],
            &config,
        );

        ranker.ingest(
            vec![
                make_item(
                    "Strong",
                    "https://b.com",
                    "rust systems programming",
                    BackendId::DuckDuckGo,
                ),
                make_item(
                    "Strongest",
                    "https://c.com",
                    "rust rust rust programming",
                    BackendId::Google,
                ),
            ],
            &config,
        );

        let snap = ranker.snapshot();
        assert_eq!(snap.results.len(), 2);
        assert!(snap.results.iter().all(|r| r.title != "Weak"));
    }

    #[test]
    fn spre_diversity_bonus() {
        let config = SpreConfig {
            top_k: 5,
            diversity_weight: 0.3,
            score_threshold: 0.0,
        };
        let mut ranker = SpreRanker::new("programming", config.clone());

        ranker.ingest(
            vec![
                make_item(
                    "A",
                    "https://a.com",
                    "programming basics",
                    BackendId::DuckDuckGo,
                ),
                make_item(
                    "B",
                    "https://b.com",
                    "programming intro",
                    BackendId::DuckDuckGo,
                ),
                make_item(
                    "C",
                    "https://c.com",
                    "programming tips",
                    BackendId::DuckDuckGo,
                ),
            ],
            &config,
        );

        ranker.ingest(
            vec![make_item(
                "D",
                "https://d.com",
                "programming guide",
                BackendId::Wikipedia,
            )],
            &config,
        );

        let snap = ranker.snapshot();
        assert!(snap.backends_contributing >= 2);
    }

    #[test]
    fn spre_filters_low_scores() {
        let config = SpreConfig {
            top_k: 10,
            score_threshold: 0.5,
            ..SpreConfig::default()
        };
        let mut ranker = SpreRanker::new("quantum computing", config.clone());

        ranker.ingest(
            vec![make_item(
                "Irrelevant",
                "https://a.com",
                "cooking recipes for dinner",
                BackendId::DuckDuckGo,
            )],
            &config,
        );

        let snap = ranker.snapshot();
        assert_eq!(
            snap.results.len(),
            0,
            "Low-scoring result should be filtered"
        );
        assert_eq!(snap.total_ingested, 1);
    }

    #[test]
    fn spre_snapshot_sorted_descending() {
        let config = SpreConfig::default();
        let mut ranker = SpreRanker::new("rust", config.clone());

        ranker.ingest(
            vec![
                make_item(
                    "Low",
                    "https://a.com",
                    "some text about code",
                    BackendId::Wikipedia,
                ),
                make_item(
                    "High",
                    "https://b.com",
                    "rust programming rust language",
                    BackendId::DuckDuckGo,
                ),
                make_item("Medium", "https://c.com", "rust basics", BackendId::Google),
            ],
            &config,
        );

        let snap = ranker.snapshot();
        for i in 1..snap.results.len() {
            let prev = snap.results[i - 1].score.unwrap_or(0.0);
            let curr = snap.results[i].score.unwrap_or(0.0);
            assert!(prev >= curr, "Results not sorted: {prev} < {curr}");
        }
        for (i, item) in snap.results.iter().enumerate() {
            assert_eq!(item.rank, (i + 1) as u32);
        }
    }
}
