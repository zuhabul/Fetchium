//! Change detection and URL monitoring engine (Phase 5, PRD §10, Mode G).
//!
//! ## Commands
//! ```bash
//! fetchium monitor <url>                     # one-shot check for changes
//! fetchium monitor <url> --diff              # show diff since last snapshot
//! fetchium monitor <url> --interval 1h       # register for periodic polling
//! fetchium monitor list                      # list all monitored URLs
//! fetchium monitor remove <url>              # stop monitoring
//! ```

pub mod diff;
pub mod snapshot;

pub use diff::{compute_diff, compute_diff_summary, ContentDiff, DiffLine};
pub use snapshot::{MonitorEntry, Snapshot, SnapshotStore};

use serde::{Deserialize, Serialize};

/// Result of a monitor check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResult {
    /// The URL that was checked.
    pub url: String,
    /// Whether content changed since last check.
    pub changed: bool,
    /// Diff details (only populated when `changed = true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<ContentDiff>,
    /// When this check was performed.
    pub checked_at: String,
}

/// Parse a human-readable interval string to seconds.
///
/// Supported units: `s`, `m`, `h`, `d`.
/// Examples: `"30s"`, `"5m"`, `"1h"`, `"7d"`.
pub fn parse_interval(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num_part, unit) = s.split_at(s.len().saturating_sub(1));
    let (digits, unit) = match unit {
        "s" | "m" | "h" | "d" => (num_part, unit),
        c if c.chars().all(|x| x.is_ascii_digit()) => (s, "s"), // bare number = seconds
        _ => return None,
    };
    let n: u64 = digits.parse().ok()?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86400,
        _ => return None,
    };
    Some(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_seconds() {
        assert_eq!(parse_interval("30s"), Some(30));
    }

    #[test]
    fn parse_interval_minutes() {
        assert_eq!(parse_interval("5m"), Some(300));
    }

    #[test]
    fn parse_interval_hours() {
        assert_eq!(parse_interval("1h"), Some(3600));
    }

    #[test]
    fn parse_interval_days() {
        assert_eq!(parse_interval("7d"), Some(7 * 86400));
    }

    #[test]
    fn parse_interval_bare_number() {
        assert_eq!(parse_interval("120"), Some(120));
    }

    #[test]
    fn parse_interval_invalid() {
        assert_eq!(parse_interval("abc"), None);
        assert_eq!(parse_interval(""), None);
    }
}
