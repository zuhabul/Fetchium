//! AutoML weight tuning for HyperFusion ranking (PRD §39).
//!
//! Uses online gradient descent on click-through feedback events to
//! tune the 8 HyperFusion signal weights. Requires ≥50 feedback events
//! before updating; otherwise returns default weights.

use crate::error::HsxResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// HyperFusion signal weights (all must sum to ~1.0 for normalised scoring).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HyperFusionWeights {
    pub bm25: f64,
    pub semantic: f64,
    pub temporal: f64,
    pub authority: f64,
    pub evidence: f64,
    pub diversity: f64,
    pub depth: f64,
    pub consensus: f64,
}

impl Default for HyperFusionWeights {
    fn default() -> Self {
        // PRD defaults (equal 0.125 × 8 = 1.0)
        Self {
            bm25: 0.20,
            semantic: 0.20,
            temporal: 0.10,
            authority: 0.15,
            evidence: 0.15,
            diversity: 0.08,
            depth: 0.07,
            consensus: 0.05,
        }
    }
}

impl HyperFusionWeights {
    /// Normalise all weights so they sum to 1.0.
    pub fn normalise(&mut self) {
        let sum = self.bm25 + self.semantic + self.temporal + self.authority
            + self.evidence + self.diversity + self.depth + self.consensus;
        if sum <= 0.0 {
            return;
        }
        self.bm25 /= sum;
        self.semantic /= sum;
        self.temporal /= sum;
        self.authority /= sum;
        self.evidence /= sum;
        self.diversity /= sum;
        self.depth /= sum;
        self.consensus /= sum;
    }

    /// Clamp each weight to [0.02, 0.60].
    pub fn clamp(&mut self) {
        let clamp = |w: f64| w.clamp(0.02, 0.60);
        self.bm25 = clamp(self.bm25);
        self.semantic = clamp(self.semantic);
        self.temporal = clamp(self.temporal);
        self.authority = clamp(self.authority);
        self.evidence = clamp(self.evidence);
        self.diversity = clamp(self.diversity);
        self.depth = clamp(self.depth);
        self.consensus = clamp(self.consensus);
    }
}

/// A single feedback event recording which result the user clicked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackEvent {
    pub query: String,
    pub result_url: String,
    /// Position shown (0-indexed).
    pub position_shown: usize,
    /// Signal scores at time of showing.
    pub signals: SignalSnapshot,
    pub clicked: bool,
    pub recorded_at: String,
}

/// Snapshot of 8 signal scores for one result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSnapshot {
    pub bm25: f64,
    pub semantic: f64,
    pub temporal: f64,
    pub authority: f64,
    pub evidence: f64,
    pub diversity: f64,
    pub depth: f64,
    pub consensus: f64,
}

/// AutoML weight tuner backed by a SQLite feedback database.
pub struct AutoMlTuner {
    conn: Connection,
    learning_rate: f64,
    min_events: usize,
}

impl AutoMlTuner {
    pub fn new(db_path: &std::path::Path) -> HsxResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS feedback_events (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                query       TEXT NOT NULL,
                result_url  TEXT NOT NULL,
                pos_shown   INTEGER NOT NULL,
                signals     TEXT NOT NULL,
                clicked     INTEGER NOT NULL,
                recorded_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tuned_weights (
                id          INTEGER PRIMARY KEY CHECK (id = 1),
                weights     TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn, learning_rate: 0.01, min_events: 50 })
    }

    /// Record a feedback event.
    pub fn record(&self, event: &FeedbackEvent) -> HsxResult<()> {
        let signals = serde_json::to_string(&event.signals)?;
        self.conn.execute(
            "INSERT INTO feedback_events (query, result_url, pos_shown, signals, clicked, recorded_at)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                event.query,
                event.result_url,
                event.position_shown as i64,
                signals,
                event.clicked as i64,
                event.recorded_at,
            ],
        )?;
        Ok(())
    }

    /// Count recorded events.
    pub fn event_count(&self) -> HsxResult<usize> {
        let n: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM feedback_events", [], |r| r.get(0))?;
        Ok(n as usize)
    }

    /// Load current tuned weights (falls back to Default if not yet tuned).
    pub fn load_weights(&self) -> HsxResult<HyperFusionWeights> {
        let row: rusqlite::Result<String> = self.conn.query_row(
            "SELECT weights FROM tuned_weights WHERE id=1",
            [],
            |r| r.get(0),
        );
        match row {
            Ok(json) => Ok(serde_json::from_str(&json)?),
            Err(_) => Ok(HyperFusionWeights::default()),
        }
    }

    /// Run one gradient-descent update pass over all stored events.
    ///
    /// Returns `None` if fewer than `min_events` events are stored.
    pub fn tune(&self) -> HsxResult<Option<HyperFusionWeights>> {
        if self.event_count()? < self.min_events {
            return Ok(None);
        }

        let mut weights = self.load_weights()?;
        let mut stmt = self.conn.prepare(
            "SELECT signals, clicked FROM feedback_events",
        )?;

        let rows: Vec<(SignalSnapshot, bool)> = stmt
            .query_map([], |r| {
                let s: String = r.get(0)?;
                let c: i64 = r.get(1)?;
                Ok((s, c != 0))
            })?
            .filter_map(|r| r.ok())
            .filter_map(|(s, c)| serde_json::from_str::<SignalSnapshot>(&s).ok().map(|snap| (snap, c)))
            .collect();

        // Simple perceptron update: for each event, if clicked, increase
        // weights for the signals that were high; if not, decrease them.
        for (snap, clicked) in &rows {
            let label = if *clicked { 1.0_f64 } else { 0.0_f64 };
            let scores = [
                snap.bm25, snap.semantic, snap.temporal, snap.authority,
                snap.evidence, snap.diversity, snap.depth, snap.consensus,
            ];
            let prediction: f64 = scores.iter().zip([
                weights.bm25, weights.semantic, weights.temporal, weights.authority,
                weights.evidence, weights.diversity, weights.depth, weights.consensus,
            ]).map(|(s, w)| s * w).sum();

            let error = label - prediction;
            let lr = self.learning_rate;
            weights.bm25      += lr * error * snap.bm25;
            weights.semantic   += lr * error * snap.semantic;
            weights.temporal   += lr * error * snap.temporal;
            weights.authority  += lr * error * snap.authority;
            weights.evidence   += lr * error * snap.evidence;
            weights.diversity  += lr * error * snap.diversity;
            weights.depth      += lr * error * snap.depth;
            weights.consensus  += lr * error * snap.consensus;
        }

        weights.clamp();
        weights.normalise();

        let json = serde_json::to_string(&weights)?;
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO tuned_weights (id, weights, updated_at) VALUES (1,?1,?2)",
            params![json, now],
        )?;

        Ok(Some(weights))
    }

    /// Return a human-readable training report.
    pub fn report(&self) -> HsxResult<String> {
        let count = self.event_count()?;
        let weights = self.load_weights()?;
        Ok(format!(
            "AutoML Report\n  Events recorded: {}\n  Min required: {}\n  \
             Weights:\n    bm25={:.3}  semantic={:.3}  temporal={:.3}  authority={:.3}\n    \
             evidence={:.3}  diversity={:.3}  depth={:.3}  consensus={:.3}",
            count,
            self.min_events,
            weights.bm25, weights.semantic, weights.temporal, weights.authority,
            weights.evidence, weights.diversity, weights.depth, weights.consensus,
        ))
    }
}

/// Return the path to the AutoML feedback database.
pub fn automl_db_path() -> std::path::PathBuf {
    crate::intelligence::intelligence_data_dir().join("automl_feedback.db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weights_normalise() {
        let mut w = HyperFusionWeights {
            bm25: 2.0, semantic: 2.0, temporal: 1.0, authority: 1.0,
            evidence: 1.0, diversity: 1.0, depth: 0.5, consensus: 0.5,
        };
        w.normalise();
        let sum = w.bm25 + w.semantic + w.temporal + w.authority
            + w.evidence + w.diversity + w.depth + w.consensus;
        assert!((sum - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_tuner_insufficient_events() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let tuner = AutoMlTuner::new(tmp.path()).unwrap();
        let result = tuner.tune().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_defaults_when_empty() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let tuner = AutoMlTuner::new(tmp.path()).unwrap();
        let w = tuner.load_weights().unwrap();
        assert!((w.bm25 - HyperFusionWeights::default().bm25).abs() < 1e-9);
    }
}
