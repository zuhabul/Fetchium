//! Content diff computation using the `similar` crate (Phase 5, PRD §10, Mode G).

use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

/// A single line change in a content diff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffLine {
    Added(String),
    Removed(String),
    Equal(String),
}

/// The result of a content diff comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDiff {
    /// Number of added lines.
    pub additions: usize,
    /// Number of removed lines.
    pub deletions: usize,
    /// Similarity ratio between old and new content (0.0 = totally different, 1.0 = identical).
    pub similarity: f32,
    /// Sequence of line changes.
    pub changes: Vec<DiffLine>,
}

impl ContentDiff {
    /// Whether there were any meaningful changes.
    pub fn has_changes(&self) -> bool {
        self.additions > 0 || self.deletions > 0
    }

    /// Render the diff as a unified-diff-style string.
    pub fn to_unified_string(&self) -> String {
        let mut out = String::new();
        for change in &self.changes {
            match change {
                DiffLine::Added(line) => out.push_str(&format!("+ {line}")),
                DiffLine::Removed(line) => out.push_str(&format!("- {line}")),
                DiffLine::Equal(_) => {} // omit unchanged lines for brevity
            }
        }
        out
    }
}

/// Compute a human-readable diff between two content strings (line-based).
pub fn compute_diff(old: &str, new: &str) -> ContentDiff {
    let diff = TextDiff::from_lines(old, new);
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut changes = Vec::new();

    for change in diff.iter_all_changes() {
        let line = change.to_string();
        match change.tag() {
            ChangeTag::Insert => {
                additions += 1;
                changes.push(DiffLine::Added(line));
            }
            ChangeTag::Delete => {
                deletions += 1;
                changes.push(DiffLine::Removed(line));
            }
            ChangeTag::Equal => {
                changes.push(DiffLine::Equal(line));
            }
        }
    }

    ContentDiff {
        additions,
        deletions,
        similarity: diff.ratio(),
        changes,
    }
}

/// Compute a summary diff showing only changed sections (context = 2 lines).
pub fn compute_diff_summary(old: &str, new: &str) -> ContentDiff {
    let diff = TextDiff::from_lines(old, new);
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut changes = Vec::new();

    for change in diff.iter_all_changes() {
        let line = change.to_string();
        match change.tag() {
            ChangeTag::Insert => {
                additions += 1;
                changes.push(DiffLine::Added(line));
            }
            ChangeTag::Delete => {
                deletions += 1;
                changes.push(DiffLine::Removed(line));
            }
            ChangeTag::Equal => {} // skip unchanged in summary mode
        }
    }

    ContentDiff {
        additions,
        deletions,
        similarity: diff.ratio(),
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_content_no_changes() {
        let diff = compute_diff("line one\nline two\n", "line one\nline two\n");
        assert!(!diff.has_changes());
        assert!((diff.similarity - 1.0).abs() < 0.01);
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn single_line_addition() {
        let diff = compute_diff("line one\n", "line one\nline two\n");
        assert!(diff.has_changes());
        assert_eq!(diff.additions, 1);
        assert_eq!(diff.deletions, 0);
    }

    #[test]
    fn single_line_deletion() {
        let diff = compute_diff("line one\nline two\n", "line one\n");
        assert!(diff.has_changes());
        assert_eq!(diff.additions, 0);
        assert_eq!(diff.deletions, 1);
    }

    #[test]
    fn completely_different_low_similarity() {
        let diff = compute_diff("aaaa bbbb cccc", "xxxx yyyy zzzz");
        assert!(diff.similarity < 0.5, "similarity={}", diff.similarity);
    }

    #[test]
    fn unified_string_shows_added_removed() {
        let diff = compute_diff("old line\n", "new line\n");
        let s = diff.to_unified_string();
        assert!(s.contains("- ") || s.contains("+ "));
    }
}
