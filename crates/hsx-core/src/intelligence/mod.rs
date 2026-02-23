//! Intelligence algorithms — PIE, ToTR, CRP, EDF, SGT, CCE, ACS (PRD §31-39).
//!
//! This module houses 7 novel intelligence algorithms that give HyperSearchX
//! persistent learning across sessions and adversarial robustness.

pub mod acs;
pub mod cce;
pub mod crp;
pub mod edf;
pub mod pie;
pub mod sgt;
pub mod totr;

// ─── Shared types ────────────────────────────────────────────────────────────

/// An observation produced by a search, fetch, or extraction operation.
#[derive(Debug, Clone)]
pub enum Observation {
    /// A search completed — record query + result domains.
    SearchCompleted {
        query: String,
        /// URLs returned by the search
        result_urls: Vec<String>,
    },
    /// A single fetch attempt — success or failure.
    FetchAttempt {
        domain: String,
        url: String,
        /// CEP layer attempted (1-5)
        extraction_layer: u8,
        success: bool,
        error: Option<String>,
        duration_ms: u64,
    },
    /// Relevance feedback for a domain after content extraction.
    ContentRelevance {
        domain: String,
        query: String,
        relevance_score: f64,
    },
    /// A named entity discovered during research.
    EntityDiscovered {
        name: String,
        entity_type: String,
        source_url: String,
    },
    /// A relationship between two entities.
    RelationshipDiscovered {
        entity_a: String,
        entity_b: String,
        relation: String,
        source_url: String,
    },
}

/// Statistics for a single intelligence layer.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct LayerStats {
    pub name: String,
    pub record_count: u64,
    pub db_size_bytes: u64,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Returns the default intelligence data directory: `~/.hypersearchx/intelligence/`.
pub fn intelligence_data_dir() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".hypersearchx")
        .join("intelligence")
}

/// Enable WAL mode on a rusqlite connection for concurrent read/write safety.
pub(crate) fn enable_wal(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(())
}

/// SHA-256 hex digest of a UTF-8 string.
pub(crate) fn sha256_hex(input: &str) -> String {
    use sha2::Digest;
    let mut h = sha2::Sha256::new();
    h.update(input.as_bytes());
    h.finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Simple normalised string similarity (Jaccard on character bigrams).
/// Returns 0.0 (totally different) to 1.0 (identical).
pub(crate) fn string_similarity(a: &str, b: &str) -> f64 {
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
