//! Evidence Decay Function (EDF) — PRD §8.14.
//!
//! Claims decay in reliability based on their content domain.
//! Formula: decayed = base_confidence × e^(−λ × age_days)
//! where λ = ln(2) / half_life_days.
//!
//! Self-calibrating: adjusts half-lives when staleness flags turn out to be wrong.

use std::collections::HashMap;

use rusqlite::Connection;

use crate::error::HsxError;
use crate::intelligence::enable_wal;

// ─── Domain half-lives (in days) ─────────────────────────────────────────────

/// Default half-lives for each content domain category.
static DEFAULT_HALF_LIVES: &[(&str, f64)] = &[
    ("ai_ml_benchmarks", 90.0),    // 3 months — model scores change rapidly
    ("tech_news", 14.0),           // 2 weeks
    ("medical_trials", 730.0),     // 2 years
    ("legal_precedent", 3650.0),   // 10 years
    ("mathematics", 36500.0),      // 100 years — proofs are eternal
    ("stock_prices", 1.0),         // 1 day
    ("software_docs", 180.0),      // 6 months
    ("historical_facts", 18250.0), // 50 years
    ("security_advisories", 30.0), // 1 month
    ("social_media", 7.0),         // 1 week
    ("youtube_video", 180.0),      // 6 months
    ("academic_papers", 1825.0),   // 5 years
    ("news", 7.0),                 // 1 week
    ("wikipedia", 365.0),          // 1 year
];

// ─── URL → domain category classifier ────────────────────────────────────────

/// Return the domain-appropriate half-life (in days) for a URL.
///
/// Looks up the static `DEFAULT_HALF_LIVES` table directly — no heap allocation.
/// Falls back to 180 days (6 months) for unrecognised domains.
pub fn domain_half_life(url: &str) -> f64 {
    let category = classify_url_domain(url);
    DEFAULT_HALF_LIVES
        .iter()
        .find(|(cat, _)| *cat == category)
        .map(|(_, hl)| *hl)
        .unwrap_or(180.0)
}

/// Classify a URL into a content domain category based on hostname patterns.
pub fn classify_url_domain(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.contains("arxiv.org") || lower.contains("scholar.google") {
        return "academic_papers";
    }
    if lower.contains("github.com") || lower.contains("docs.rs") || lower.contains("readthedocs") {
        return "software_docs";
    }
    if lower.contains("cve.mitre") || lower.contains("nvd.nist") || lower.contains("security") {
        return "security_advisories";
    }
    if lower.contains("nih.gov")
        || lower.contains("pubmed")
        || lower.contains("nejm")
        || lower.contains("lancet")
    {
        return "medical_trials";
    }
    if lower.contains("law.")
        || lower.contains("court")
        || lower.contains("supremecourt")
        || lower.contains("justice.gov")
    {
        return "legal_precedent";
    }
    if lower.contains("reddit.com")
        || lower.contains("twitter.com")
        || lower.contains("x.com")
        || lower.contains("mastodon")
        || lower.contains("bluesky")
    {
        return "social_media";
    }
    if lower.contains("wikipedia.org") {
        return "wikipedia";
    }
    if lower.contains("youtube.com") || lower.contains("youtu.be") {
        return "youtube_video";
    }
    if lower.contains("huggingface")
        || lower.contains("paperswithcode")
        || lower.contains("benchmarks")
    {
        return "ai_ml_benchmarks";
    }
    "news" // default
}

// ─── Types ───────────────────────────────────────────────────────────────────

/// Staleness classification of a decayed confidence value.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Staleness {
    Fresh,
    PotentiallyStale,
    Stale,
}

/// Result of applying EDF to a confidence score.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DecayResult {
    pub original_confidence: f64,
    pub decayed_confidence: f64,
    /// Content domain category used for half-life lookup.
    pub domain_category: String,
    pub age_days: f64,
    pub half_life: f64,
    pub staleness: Staleness,
    pub flag: Option<String>,
}

// ─── Engine ──────────────────────────────────────────────────────────────────

/// Evidence Decay Function engine.
pub struct EvidenceDecayFunction {
    half_lives: HashMap<String, f64>,
    /// SQLite connection for self-calibration events.
    calibration_db: Option<Connection>,
}

impl EvidenceDecayFunction {
    /// Create a new EDF with default half-lives and no calibration DB.
    pub fn new() -> Self {
        let mut half_lives = HashMap::new();
        for &(domain, hl) in DEFAULT_HALF_LIVES {
            half_lives.insert(domain.to_string(), hl);
        }
        Self {
            half_lives,
            calibration_db: None,
        }
    }

    /// Create with a calibration database that persists half-life adjustments.
    pub fn with_calibration(db_path: &std::path::Path) -> Result<Self, HsxError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS calibration_events (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                domain_category   TEXT    NOT NULL,
                flagged_stale     INTEGER NOT NULL,
                actually_valid    INTEGER NOT NULL,
                recorded_at       TEXT    NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS half_life_overrides (
                domain_category TEXT PRIMARY KEY,
                half_life_days  REAL NOT NULL,
                updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        let mut edf = Self::new();

        // Load any persisted half-life overrides.
        {
            let mut stmt =
                conn.prepare("SELECT domain_category, half_life_days FROM half_life_overrides")?;
            let overrides: Vec<(String, f64)> = stmt
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .filter_map(|r| r.ok())
                .collect();
            for (cat, hl) in overrides {
                edf.half_lives.insert(cat, hl);
            }
        }

        edf.calibration_db = Some(conn);
        Ok(edf)
    }

    /// Calculate decayed confidence for a claim of `age_days` old in `domain_category`.
    ///
    /// Clamps the result to [0.01, base_confidence] to prevent underflow or inflation.
    pub fn decay(&self, base_confidence: f64, domain_category: &str, age_days: f64) -> DecayResult {
        let half_life = self.get_half_life(domain_category);
        let lambda = std::f64::consts::LN_2 / half_life;
        let decayed = (base_confidence * (-lambda * age_days).exp())
            .clamp(0.01_f64.min(base_confidence), base_confidence);

        let staleness = if decayed < 0.3 {
            Staleness::Stale
        } else if decayed < 0.5 {
            Staleness::PotentiallyStale
        } else {
            Staleness::Fresh
        };

        let flag = match &staleness {
            Staleness::Stale => Some(format!(
                "[stale: {age_days:.0} days old, half-life {half_life:.0} days]"
            )),
            Staleness::PotentiallyStale => Some(format!(
                "[potentially stale: {age_days:.0} days old, half-life {half_life:.0} days]"
            )),
            Staleness::Fresh => None,
        };

        DecayResult {
            original_confidence: base_confidence,
            decayed_confidence: decayed,
            domain_category: domain_category.to_string(),
            age_days,
            half_life,
            staleness,
            flag,
        }
    }

    /// Self-calibration feedback: adjust half-life based on false-positive / false-negative flags.
    ///
    /// Requires at least 10 events per category before making adjustments.
    pub fn calibrate(
        &mut self,
        domain_category: &str,
        was_flagged_stale: bool,
        actually_still_valid: bool,
    ) -> Result<(), HsxError> {
        if let Some(conn) = &self.calibration_db {
            conn.execute(
                "INSERT INTO calibration_events (domain_category, flagged_stale, actually_valid)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    domain_category,
                    was_flagged_stale as i32,
                    actually_still_valid as i32,
                ],
            )?;

            // Count events for this category.
            let event_count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM calibration_events WHERE domain_category = ?1",
                [domain_category],
                |row| row.get(0),
            )?;

            if event_count >= 10 {
                let current = self.get_half_life(domain_category);
                let new_half_life = if was_flagged_stale && actually_still_valid {
                    // False positive: content was still valid → increase half-life by 10%
                    current * 1.1
                } else if !was_flagged_stale && !actually_still_valid {
                    // False negative: content was stale but we didn't flag it → decrease by 10%
                    (current * 0.9).max(1.0)
                } else {
                    current
                };

                if (new_half_life - current).abs() > 0.01 {
                    self.half_lives
                        .insert(domain_category.to_string(), new_half_life);
                    conn.execute(
                        "INSERT INTO half_life_overrides (domain_category, half_life_days)
                         VALUES (?1, ?2)
                         ON CONFLICT(domain_category) DO UPDATE SET
                            half_life_days = ?2,
                            updated_at = datetime('now')",
                        rusqlite::params![domain_category, new_half_life],
                    )?;
                    tracing::info!(
                        domain = domain_category,
                        old_hl = current,
                        new_hl = new_half_life,
                        "EDF: half-life adjusted"
                    );
                }
            }
        }
        Ok(())
    }

    fn get_half_life(&self, domain_category: &str) -> f64 {
        self.half_lives
            .get(domain_category)
            .copied()
            .unwrap_or(180.0) // default: 6 months
    }
}

impl Default for EvidenceDecayFunction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ai_benchmarks_halve_after_90_days() {
        let edf = EvidenceDecayFunction::new();
        let result = edf.decay(0.9, "ai_ml_benchmarks", 90.0);
        // After one half-life, confidence should be ~0.45
        assert!(
            (result.decayed_confidence - 0.45).abs() < 0.02,
            "decayed={:.3}",
            result.decayed_confidence
        );
    }

    #[test]
    fn math_barely_decays_in_a_year() {
        let edf = EvidenceDecayFunction::new();
        let result = edf.decay(0.9, "mathematics", 365.0);
        assert!(
            result.decayed_confidence > 0.88,
            "decayed={:.3}",
            result.decayed_confidence
        );
        assert_eq!(result.staleness, Staleness::Fresh);
    }

    #[test]
    fn stale_content_gets_flag() {
        let edf = EvidenceDecayFunction::new();
        let result = edf.decay(0.9, "social_media", 60.0); // 60 days, 7-day half-life
        assert!(result.staleness == Staleness::Stale);
        assert!(result.flag.is_some());
    }

    #[test]
    fn classify_url_domain_arxiv() {
        assert_eq!(
            classify_url_domain("https://arxiv.org/abs/2301.12345"),
            "academic_papers"
        );
    }

    #[test]
    fn classify_url_domain_github() {
        assert_eq!(
            classify_url_domain("https://github.com/rust-lang/rust"),
            "software_docs"
        );
    }
}
