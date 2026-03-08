//! Research radar — surfacing relevant topics from user history (PRD §33.2).

use crate::error::FetchiumResult;
use serde::{Deserialize, Serialize};

/// A proactive suggestion from the radar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadarItem {
    pub topic: String,
    pub title: String,
    pub url: String,
    pub snippet: String,
    /// Score in [0, 1] — higher = more relevant to user interests.
    pub relevance: f64,
    pub reason: String,
}

impl RadarItem {
    pub fn to_markdown(&self) -> String {
        format!(
            "**{}** (relevance: {:.0}%)\n{}\n[{}]({})\n_Reason: {}_",
            self.topic,
            self.relevance * 100.0,
            self.snippet,
            self.title,
            self.url,
            self.reason,
        )
    }
}

/// Build radar items from a set of top topics and candidate results.
pub fn build_radar(
    top_topics: &[(String, f64)],
    candidate_items: Vec<crate::types::ResultItem>,
    limit: usize,
) -> FetchiumResult<Vec<RadarItem>> {
    let mut items: Vec<RadarItem> = Vec::new();

    for item in candidate_items {
        // Score by matching to known top topics
        let best = top_topics
            .iter()
            .map(|(topic, freq)| {
                let sim =
                    topic_similarity(&item.title, topic) + topic_similarity(&item.snippet, topic);
                (topic.clone(), freq * sim)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((topic, score)) = best {
            if score > 0.05 {
                items.push(RadarItem {
                    topic,
                    title: item.title.clone(),
                    url: item.url.clone(),
                    snippet: item.snippet.clone(),
                    relevance: score.min(1.0),
                    reason: "Matches your recent research patterns".into(),
                });
            }
        }
    }

    items.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items.truncate(limit);
    Ok(items)
}

/// Simple word-overlap similarity between text and topic string.
fn topic_similarity(text: &str, topic: &str) -> f64 {
    let text_words: std::collections::HashSet<&str> = text
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| w.len() > 3)
        .collect();
    let topic_words: std::collections::HashSet<&str> =
        topic.split_whitespace().filter(|w| w.len() > 3).collect();
    if topic_words.is_empty() {
        return 0.0;
    }
    let overlap = text_words.intersection(&topic_words).count();
    overlap as f64 / topic_words.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_similarity() {
        assert!(topic_similarity("Rust async programming", "Rust async") > 0.0);
        assert_eq!(topic_similarity("", "topic"), 0.0);
    }

    #[test]
    fn test_build_radar_empty() {
        let items = build_radar(&[], vec![], 10).unwrap();
        assert!(items.is_empty());
    }
}
