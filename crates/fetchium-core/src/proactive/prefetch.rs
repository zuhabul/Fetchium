//! Predictive prefetching — pre-cache likely follow-up queries (PRD §33).

use crate::error::FetchiumResult;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A prefetch request with associated priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefetchEntry {
    pub query: String,
    /// Priority in [0.0, 1.0]; higher = fetch sooner.
    pub priority: f64,
    pub requested_at: String,
}

/// In-memory priority queue for prefetch requests.
#[derive(Default)]
pub struct PrefetchQueue {
    queue: VecDeque<PrefetchEntry>,
}

impl PrefetchQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue a query for prefetching.
    pub fn enqueue(&mut self, query: &str, priority: f64) {
        let entry = PrefetchEntry {
            query: query.to_string(),
            priority: priority.clamp(0.0, 1.0),
            requested_at: chrono::Utc::now().to_rfc3339(),
        };
        // Insert in priority order (highest first)
        let pos = self
            .queue
            .iter()
            .position(|e| e.priority < entry.priority)
            .unwrap_or(self.queue.len());
        self.queue.insert(pos, entry);
    }

    /// Dequeue the highest-priority prefetch request.
    pub fn dequeue(&mut self) -> Option<PrefetchEntry> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Generate prefetch candidates from a completed search result.
///
/// Heuristically generates likely follow-up queries:
/// - Adds "tutorial" and "examples" variants for informational queries.
/// - Adds "vs <top_result>" for comparison queries.
pub fn generate_candidates(
    query: &str,
    result_titles: &[String],
) -> FetchiumResult<Vec<(String, f64)>> {
    let mut candidates: Vec<(String, f64)> = Vec::new();
    let q = query.trim().to_ascii_lowercase();

    // Tutorial follow-up for informational queries
    if !q.contains("tutorial") && !q.contains("example") {
        candidates.push((format!("{query} tutorial"), 0.6));
        candidates.push((format!("{query} examples"), 0.5));
    }

    // "Latest news" follow-up
    if !q.contains("news") && !q.contains("latest") {
        candidates.push((format!("{query} latest news"), 0.7));
    }

    // Top result title as a follow-up query
    if let Some(title) = result_titles.first() {
        let short: String = title
            .split_whitespace()
            .take(5)
            .collect::<Vec<_>>()
            .join(" ");
        if !short.is_empty() && short.len() > 5 {
            candidates.push((short, 0.4));
        }
    }

    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_priority_ordering() {
        let mut q = PrefetchQueue::new();
        q.enqueue("low", 0.2);
        q.enqueue("high", 0.9);
        q.enqueue("mid", 0.5);
        let first = q.dequeue().unwrap();
        assert_eq!(first.query, "high");
    }

    #[test]
    fn test_generate_candidates() {
        let titles = vec!["Rust async book".to_string()];
        let cands = generate_candidates("Rust async", &titles).unwrap();
        assert!(!cands.is_empty());
    }
}
