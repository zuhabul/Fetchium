//! Research session merging with deduplication (PRD §37).

use crate::error::FetchiumError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub session_id: String,
    pub content: String,
    pub source_url: Option<String>,
}

#[derive(Debug)]
pub struct MergeResult {
    pub merged: Vec<Finding>,
    pub added: usize,
    pub deduplicated: usize,
}

/// Merge findings from two sessions with bigram-Jaccard deduplication.
///
/// Threshold of 0.85 similarity is treated as a duplicate.
pub fn merge_sessions(
    sessions_dir: &std::path::Path,
    session_a: &str,
    session_b: &str,
) -> Result<MergeResult, FetchiumError> {
    let findings_a = load_findings(sessions_dir, session_a)?;
    let findings_b = load_findings(sessions_dir, session_b)?;

    let mut merged = findings_a;
    let mut added = 0;
    let mut deduplicated = 0;

    for finding in findings_b {
        let is_duplicate = merged
            .iter()
            .any(|existing| bigram_similarity(&existing.content, &finding.content) > 0.85);
        if is_duplicate {
            deduplicated += 1;
        } else {
            merged.push(finding);
            added += 1;
        }
    }

    Ok(MergeResult {
        merged,
        added,
        deduplicated,
    })
}

fn load_findings(
    sessions_dir: &std::path::Path,
    session_id: &str,
) -> Result<Vec<Finding>, FetchiumError> {
    let path = sessions_dir.join(session_id).join("findings.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
}

fn bigram_similarity(a: &str, b: &str) -> f64 {
    let bigrams = |s: &str| -> std::collections::HashSet<[char; 2]> {
        let chars: Vec<char> = s.to_lowercase().chars().collect();
        chars.windows(2).map(|w| [w[0], w[1]]).collect()
    };
    let a_set = bigrams(a);
    let b_set = bigrams(b);
    if a_set.is_empty() && b_set.is_empty() {
        return 1.0;
    }
    if a_set.is_empty() || b_set.is_empty() {
        return 0.0;
    }
    let intersection = a_set.intersection(&b_set).count();
    let union = a_set.union(&b_set).count();
    intersection as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_findings_are_deduplicated() {
        let content = "Rust is fast and safe".to_string();
        let merged: Vec<Finding> = vec![Finding {
            id: "a".into(),
            session_id: "s1".into(),
            content: content.clone(),
            source_url: None,
        }];
        let new_f = Finding {
            id: "b".into(),
            session_id: "s2".into(),
            content: content.clone(),
            source_url: None,
        };
        let is_dup = merged
            .iter()
            .any(|e| bigram_similarity(&e.content, &new_f.content) > 0.85);
        assert!(is_dup);
        assert_eq!(merged.len(), 1);
    }
}
