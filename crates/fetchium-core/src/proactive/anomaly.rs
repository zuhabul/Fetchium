//! Anomaly detection on monitored content (PRD §33).
//!
//! Flags significant content changes by comparing the ContentDiff produced
//! by the monitor module against configurable thresholds.

use crate::error::HsxResult;
use crate::monitor::diff::ContentDiff;
use serde::{Deserialize, Serialize};

/// Configuration for anomaly detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    /// Similarity ratio below which we flag an anomaly (0.0–1.0).
    /// Default: 0.7 (30% change triggers anomaly).
    pub similarity_threshold: f32,
    /// Minimum line additions+deletions to be considered anomalous.
    pub min_changed_lines: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.70,
            min_changed_lines: 10,
        }
    }
}

/// Result of an anomaly check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub url: String,
    pub is_anomalous: bool,
    pub similarity: f32,
    pub changed_lines: usize,
    pub summary: String,
}

/// Check whether a `ContentDiff` represents an anomalous change.
pub fn check_anomaly(
    url: &str,
    diff: &ContentDiff,
    cfg: &AnomalyConfig,
) -> HsxResult<AnomalyReport> {
    let changed_lines = diff.additions + diff.deletions;
    let is_anomalous =
        diff.similarity < cfg.similarity_threshold || changed_lines >= cfg.min_changed_lines;

    let summary = if is_anomalous {
        format!(
            "ANOMALY detected: similarity={:.2}, {} lines changed",
            diff.similarity, changed_lines
        )
    } else {
        format!(
            "Normal change: similarity={:.2}, {} lines changed",
            diff.similarity, changed_lines
        )
    };

    Ok(AnomalyReport {
        url: url.to_string(),
        is_anomalous,
        similarity: diff.similarity,
        changed_lines,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_diff(additions: usize, deletions: usize, similarity: f32) -> ContentDiff {
        ContentDiff {
            additions,
            deletions,
            similarity,
            changes: Vec::new(),
        }
    }

    #[test]
    fn test_anomaly_low_similarity() {
        let diff = make_diff(100, 80, 0.20);
        let report =
            check_anomaly("https://example.com", &diff, &AnomalyConfig::default()).unwrap();
        assert!(report.is_anomalous);
    }

    #[test]
    fn test_no_anomaly_high_similarity() {
        let diff = make_diff(1, 0, 0.99);
        let report =
            check_anomaly("https://example.com", &diff, &AnomalyConfig::default()).unwrap();
        assert!(!report.is_anomalous);
    }
}
