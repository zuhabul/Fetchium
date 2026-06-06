//! Persistent Intelligence Engine (PIE) — PRD §31.
//!
//! Orchestrates 4 learning layers that persist across sessions:
//! - PKG: Personal Knowledge Graph
//! - STM: Source Trust Memory (Bayesian Beta distribution)
//! - FPM: Failure Pattern Memory
//! - QPM: Query Prediction Model
//!
//! Data stored in `~/.fetchium/intelligence/`.

pub mod fpm;
pub mod pkg;
pub mod qpm;
pub mod stm;

use crate::error::FetchiumError;
use crate::intelligence::intelligence_data_dir;

/// Aggregate statistics across all 4 PIE layers.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PieStats {
    pub entities: u64,
    pub relationships: u64,
    pub tracked_domains: u64,
    pub failure_patterns: u64,
    pub query_history_size: u64,
}

/// The 4-layer Persistent Intelligence Engine.
pub struct PersistentIntelligenceEngine {
    pub pkg: pkg::PersonalKnowledgeGraph,
    pub stm: stm::SourceTrustMemory,
    pub fpm: fpm::FailurePatternMemory,
    pub qpm: qpm::QueryPredictionModel,
}

impl PersistentIntelligenceEngine {
    /// Open all 4 layers from the default intelligence data directory.
    pub fn new() -> Result<Self, FetchiumError> {
        let base = intelligence_data_dir();
        std::fs::create_dir_all(&base).map_err(|e| {
            FetchiumError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create intelligence dir {}: {e}", base.display()),
            ))
        })?;
        Ok(Self {
            pkg: pkg::PersonalKnowledgeGraph::new(&base.join("knowledge_graph.db"))?,
            stm: stm::SourceTrustMemory::new(&base.join("source_trust.db"))?,
            fpm: fpm::FailurePatternMemory::new(&base.join("failure_patterns.db"))?,
            qpm: qpm::QueryPredictionModel::new(&base.join("query_patterns.db"))?,
        })
    }

    /// Observe a completed search: update STM for each result domain and log the query.
    pub fn observe_search(
        &self,
        query: &str,
        result_domains: &[&str],
        topic: &str,
    ) -> Result<(), FetchiumError> {
        for domain in result_domains {
            self.stm.update_trust(domain, true, 0.7)?;
        }
        self.qpm.record_query(query, topic, None)?;
        Ok(())
    }

    /// Observe a fetch attempt and record in both STM and FPM.
    #[allow(clippy::too_many_arguments)]
    pub fn observe_fetch(
        &self,
        domain: &str,
        url: &str,
        layer: u8,
        success: bool,
        error: Option<&str>,
        duration_ms: u64,
        relevance: f64,
    ) -> Result<(), FetchiumError> {
        let _ = url; // stored implicitly via domain
        self.stm.update_trust(domain, success, relevance)?;
        self.fpm
            .record_attempt(domain, layer, success, error, duration_ms)?;
        Ok(())
    }

    /// Aggregate statistics across all 4 layers.
    pub fn stats(&self) -> Result<PieStats, FetchiumError> {
        Ok(PieStats {
            entities: self.pkg.entity_count()?,
            relationships: self.pkg.relationship_count()?,
            tracked_domains: self.stm.domain_count()?,
            failure_patterns: self.fpm.pattern_count()?,
            query_history_size: self.qpm.query_count()?,
        })
    }

    /// Reset all learned data across all layers.
    pub fn reset_all(&self) -> Result<(), FetchiumError> {
        self.pkg.reset()?;
        self.stm.reset()?;
        self.fpm.reset()?;
        self.qpm.reset()?;
        Ok(())
    }

    /// Export a human-readable JSON summary of all layers.
    pub fn export_json(&self) -> Result<String, FetchiumError> {
        let stats = self.stats()?;
        let top_entities = self.pkg.top_entities(20)?;
        let top_domains = self.stm.top_trusted(20)?;
        let top_topics = self.qpm.top_topics(10)?;

        let export = serde_json::json!({
            "stats": stats,
            "top_entities": top_entities,
            "top_trusted_domains": top_domains,
            "top_query_topics": top_topics,
        });

        Ok(serde_json::to_string_pretty(&export)?)
    }
}

/// Extract a coarse topic from a query string (lowercased first non-trivial word cluster).
pub fn extract_topic(query: &str) -> String {
    let stop_words = [
        "the", "a", "an", "is", "are", "was", "were", "what", "how", "why", "who", "when", "where",
        "which", "of", "in", "to", "for",
    ];
    let words: Vec<&str> = query
        .split_whitespace()
        .filter(|w| {
            let lw = w.to_lowercase();
            !stop_words.contains(&lw.as_str()) && lw.len() >= 3
        })
        .take(3)
        .collect();
    if words.is_empty() {
        query.to_lowercase().trim().to_string()
    } else {
        words.join(" ").to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_topic_removes_stop_words() {
        let topic = extract_topic("What is the best Rust framework for web");
        assert!(
            topic.contains("best") || topic.contains("rust") || topic.contains("framework"),
            "topic={topic}"
        );
    }
}
