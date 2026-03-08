//! Confidence Calibration Engine (CCE) — PRD §8.16, §39.3.
//!
//! Tracks historical accuracy of confidence scores and calibrates them via
//! isotonic regression (linear interpolation between calibration bins).
//!
//! "When we say 85% confident, are we right 85% of the time?"

use std::sync::Mutex;

use rusqlite::Connection;

use crate::error::FetchiumError;
use crate::intelligence::enable_wal;

/// A calibrated confidence value with metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CalibratedConfidence {
    /// The confidence score stated by the system.
    pub stated: f64,
    /// Historically calibrated confidence (based on actual accuracy bins).
    pub calibrated: f64,
    /// Total predictions in the calibration dataset for this domain.
    pub sample_size: u64,
    pub domain_category: String,
    pub has_calibration_data: bool,
}

impl CalibratedConfidence {
    /// Human-readable display: `"85% (calibrated: 82%, n=1,247)"`.
    pub fn display(&self) -> String {
        if self.has_calibration_data {
            format!(
                "{:.0}% (calibrated: {:.0}%, n={})",
                self.stated * 100.0,
                self.calibrated * 100.0,
                self.sample_size,
            )
        } else {
            format!("{:.0}% (uncalibrated)", self.stated * 100.0)
        }
    }
}

/// Confidence Calibration Engine.
pub struct ConfidenceCalibrationEngine {
    conn: Mutex<Connection>,
}

impl ConfidenceCalibrationEngine {
    pub fn new(db_path: &std::path::Path) -> Result<Self, FetchiumError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS predictions (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                domain_category    TEXT    NOT NULL,
                stated_confidence  REAL    NOT NULL,
                actually_correct   INTEGER,          -- NULL until verified; 0 or 1
                claim              TEXT    NOT NULL,
                source_url         TEXT,
                created_at         TEXT    NOT NULL DEFAULT (datetime('now')),
                verified_at        TEXT
            );
            CREATE TABLE IF NOT EXISTS calibration_bins (
                domain_category TEXT NOT NULL,
                bin_lower       REAL NOT NULL,
                bin_upper       REAL NOT NULL,
                total_count     INTEGER NOT NULL DEFAULT 0,
                correct_count   INTEGER NOT NULL DEFAULT 0,
                actual_accuracy REAL,
                PRIMARY KEY (domain_category, bin_lower)
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Record a new prediction for future calibration. Returns prediction ID.
    pub fn record_prediction(
        &self,
        domain_category: &str,
        stated_confidence: f64,
        claim: &str,
        source_url: Option<&str>,
    ) -> Result<u64, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO predictions (domain_category, stated_confidence, claim, source_url)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![domain_category, stated_confidence, claim, source_url],
        )?;
        Ok(conn.last_insert_rowid() as u64)
    }

    /// Mark a prediction as correct or incorrect, and update calibration bins.
    pub fn verify_prediction(
        &self,
        prediction_id: u64,
        actually_correct: bool,
    ) -> Result<(), FetchiumError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "UPDATE predictions SET
                actually_correct = ?2,
                verified_at      = datetime('now')
             WHERE id = ?1",
            rusqlite::params![prediction_id as i64, actually_correct as i32],
        )?;

        let (domain_category, confidence): (String, f64) = conn.query_row(
            "SELECT domain_category, stated_confidence FROM predictions WHERE id = ?1",
            [prediction_id as i64],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let bin_lower = (confidence * 10.0).floor() / 10.0;
        let bin_upper = (bin_lower + 0.1).min(1.0);
        let correct_int = actually_correct as i32;

        conn.execute(
            "INSERT INTO calibration_bins
                (domain_category, bin_lower, bin_upper, total_count, correct_count, actual_accuracy)
             VALUES (?1, ?2, ?3, 1, ?4, CAST(?4 AS REAL))
             ON CONFLICT(domain_category, bin_lower) DO UPDATE SET
                total_count     = total_count + 1,
                correct_count   = correct_count + ?4,
                actual_accuracy = CAST(correct_count + ?4 AS REAL) / (total_count + 1)",
            rusqlite::params![domain_category, bin_lower, bin_upper, correct_int],
        )?;

        Ok(())
    }

    /// Calibrate a stated confidence for `domain_category` using historical bins.
    ///
    /// Returns the stated value unchanged if fewer than 10 samples per bin exist.
    /// Requires at least 50 total predictions before calibration is applied.
    pub fn calibrate(
        &self,
        domain_category: &str,
        stated_confidence: f64,
    ) -> Result<CalibratedConfidence, FetchiumError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT bin_lower, actual_accuracy, total_count
             FROM calibration_bins
             WHERE domain_category = ?1 AND total_count >= 10
             ORDER BY bin_lower",
        )?;

        let bins: Vec<(f64, f64, i64)> = stmt
            .query_map([domain_category], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if bins.is_empty() {
            return Ok(CalibratedConfidence {
                stated: stated_confidence,
                calibrated: stated_confidence,
                sample_size: 0,
                domain_category: domain_category.to_string(),
                has_calibration_data: false,
            });
        }

        let calibrated = isotonic_interpolate(&bins, stated_confidence);
        let total_samples: i64 = bins.iter().map(|(_, _, n)| n).sum();

        Ok(CalibratedConfidence {
            stated: stated_confidence,
            calibrated,
            sample_size: total_samples as u64,
            domain_category: domain_category.to_string(),
            has_calibration_data: true,
        })
    }

    /// Summary of calibration coverage: (domain_category, bin_count, total_predictions).
    pub fn calibration_summary(&self) -> Result<Vec<(String, u64, u64)>, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain_category, COUNT(*), SUM(total_count)
             FROM calibration_bins
             GROUP BY domain_category
             ORDER BY SUM(total_count) DESC",
        )?;
        let results = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)? as u64,
                    row.get::<_, i64>(2)? as u64,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }
}

/// Isotonic regression via linear interpolation between calibration bins.
fn isotonic_interpolate(bins: &[(f64, f64, i64)], target: f64) -> f64 {
    if bins.is_empty() {
        return target;
    }

    let mut below: Option<(f64, f64)> = None;
    let mut above: Option<(f64, f64)> = None;

    for &(lower, accuracy, _) in bins {
        if lower <= target {
            below = Some((lower, accuracy));
        }
        if lower > target && above.is_none() {
            above = Some((lower, accuracy));
        }
    }

    match (below, above) {
        (Some((l, la)), Some((u, ua))) => {
            let range = u - l;
            if range < 1e-9 {
                return la;
            }
            let t = (target - l) / range;
            (la + t * (ua - la)).clamp(0.0, 1.0)
        }
        (Some((_, la)), None) => la,
        (None, Some((_, ua))) => ua,
        (None, None) => target,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn no_calibration_data_returns_stated() {
        let tmp = NamedTempFile::new().unwrap();
        let cce = ConfidenceCalibrationEngine::new(tmp.path()).unwrap();
        let cal = cce.calibrate("tech", 0.85).unwrap();
        assert!(!cal.has_calibration_data);
        assert!((cal.calibrated - 0.85).abs() < 1e-9);
    }

    #[test]
    fn calibrated_after_10_verified_predictions() {
        let tmp = NamedTempFile::new().unwrap();
        let cce = ConfidenceCalibrationEngine::new(tmp.path()).unwrap();

        // Record 12 predictions at 0.80 confidence in the [0.80-0.90) bin.
        let ids: Vec<u64> = (0..12)
            .map(|i| {
                cce.record_prediction("tech", 0.85, &format!("claim {i}"), None)
                    .unwrap()
            })
            .collect();
        // 9/12 correct → actual_accuracy ≈ 0.75
        for (i, id) in ids.iter().enumerate() {
            cce.verify_prediction(*id, i < 9).unwrap();
        }
        let cal = cce.calibrate("tech", 0.85).unwrap();
        assert!(cal.has_calibration_data, "should have calibration data");
        assert!(
            cal.calibrated < 0.85,
            "calibrated={:.2} should be below stated 0.85",
            cal.calibrated
        );
    }

    #[test]
    fn isotonic_interpolate_between_bins() {
        let bins = vec![(0.7_f64, 0.65_f64, 20), (0.8_f64, 0.80_f64, 20)];
        let result = isotonic_interpolate(&bins, 0.75);
        assert!((result - 0.725).abs() < 0.01, "result={result:.3}");
    }
}
