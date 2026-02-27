//! Failure Pattern Memory (FPM) — tracks extraction success/failure per domain+layer.
//!
//! Schema:
//! - `extraction_successes(domain, layer, count, total_time_ms)`
//! - `extraction_failures(domain, layer, error_type, count, last_occurred)`

use rusqlite::Connection;
use std::sync::Mutex;

use crate::error::HsxError;
use crate::intelligence::enable_wal;

pub struct FailurePatternMemory {
    conn: Mutex<Connection>,
}

impl FailurePatternMemory {
    pub fn new(db_path: &std::path::Path) -> Result<Self, HsxError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS extraction_successes (
                domain       TEXT    NOT NULL,
                layer        INTEGER NOT NULL,
                count        INTEGER NOT NULL DEFAULT 0,
                total_time_ms INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (domain, layer)
            );
            CREATE TABLE IF NOT EXISTS extraction_failures (
                domain       TEXT    NOT NULL,
                layer        INTEGER NOT NULL,
                error_type   TEXT    NOT NULL,
                count        INTEGER NOT NULL DEFAULT 0,
                last_occurred TEXT   NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (domain, layer, error_type)
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Record an extraction attempt (success or failure) for `domain` at CEP `layer`.
    pub fn record_attempt(
        &self,
        domain: &str,
        layer: u8,
        success: bool,
        error: Option<&str>,
        duration_ms: u64,
    ) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        if success {
            conn.execute(
                "INSERT INTO extraction_successes (domain, layer, count, total_time_ms)
                 VALUES (?1, ?2, 1, ?3)
                 ON CONFLICT(domain, layer) DO UPDATE SET
                    count         = count + 1,
                    total_time_ms = total_time_ms + ?3",
                rusqlite::params![domain, layer as i64, duration_ms as i64],
            )?;
        } else {
            conn.execute(
                "INSERT INTO extraction_failures (domain, layer, error_type, count, last_occurred)
                 VALUES (?1, ?2, ?3, 1, datetime('now'))
                 ON CONFLICT(domain, layer, error_type) DO UPDATE SET
                    count         = count + 1,
                    last_occurred = datetime('now')",
                rusqlite::params![domain, layer as i64, error.unwrap_or("unknown")],
            )?;
        }
        Ok(())
    }

    /// Recommend the best CEP layer for `domain` based on historical data.
    ///
    /// Returns `(recommended_layer, confidence)`.
    /// Default if no history: layer 2, confidence 0.5.
    pub fn recommend_layer(&self, domain: &str) -> Result<(u8, f64), HsxError> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT s.layer, s.count,
                    COALESCE((SELECT SUM(f.count)
                              FROM extraction_failures f
                              WHERE f.domain = s.domain AND f.layer = s.layer), 0)
             FROM extraction_successes s
             WHERE s.domain = ?1
             ORDER BY s.count DESC",
        )?;

        let results: Vec<(u8, i64, i64)> = stmt
            .query_map([domain], |row| {
                Ok((
                    row.get::<_, i64>(0)? as u8,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if let Some((layer, successes, failures)) = results.first() {
            let total = (*successes + *failures).max(1) as f64;
            let confidence = *successes as f64 / total;
            Ok((*layer, confidence))
        } else {
            Ok((2, 0.5)) // default: Layer 2 (HTTP + readability)
        }
    }

    /// Total number of distinct failure patterns recorded.
    pub fn pattern_count(&self) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM extraction_failures", [], |row| {
            row.get(0)
        })?;
        Ok(n as u64)
    }

    /// Reset all failure/success patterns.
    pub fn reset(&self) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM extraction_successes;
             DELETE FROM extraction_failures;
             VACUUM;",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn recommend_layer_returns_best_success_layer() {
        let tmp = NamedTempFile::new().unwrap();
        let fpm = FailurePatternMemory::new(tmp.path()).unwrap();
        // Layer 3 succeeds 5× for "spa.com"
        for _ in 0..5 {
            fpm.record_attempt("spa.com", 1, false, Some("timeout"), 0)
                .unwrap();
            fpm.record_attempt("spa.com", 3, true, None, 500).unwrap();
        }
        let (layer, conf) = fpm.recommend_layer("spa.com").unwrap();
        assert_eq!(layer, 3);
        assert!(conf > 0.5, "conf={conf:.2}");
    }

    #[test]
    fn no_history_returns_default() {
        let tmp = NamedTempFile::new().unwrap();
        let fpm = FailurePatternMemory::new(tmp.path()).unwrap();
        let (layer, conf) = fpm.recommend_layer("new-site.io").unwrap();
        assert_eq!(layer, 2);
        assert!((conf - 0.5).abs() < 1e-9);
    }
}
