//! Source Trust Memory (STM) — Bayesian Beta-distribution trust tracking per domain.
//!
//! Schema: `domain_trust(domain, trust_score, alpha, beta, fetch_count, success_count,
//!          avg_relevance, last_updated)`
//!
//! Trust score = E[Beta(alpha, beta)] = alpha / (alpha + beta).
//! Prior: Beta(1, 1) — uninformed, 0.5 trust for unknown domains.

use rusqlite::Connection;
use std::sync::Mutex;

use crate::error::HsxError;
use crate::intelligence::enable_wal;

pub struct SourceTrustMemory {
    conn: Mutex<Connection>,
}

impl SourceTrustMemory {
    /// Open or create the STM database at `db_path`.
    pub fn new(db_path: &std::path::Path) -> Result<Self, HsxError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS domain_trust (
                domain         TEXT PRIMARY KEY,
                trust_score    REAL    NOT NULL DEFAULT 0.5,
                alpha          REAL    NOT NULL DEFAULT 1.0,
                beta           REAL    NOT NULL DEFAULT 1.0,
                fetch_count    INTEGER NOT NULL DEFAULT 0,
                success_count  INTEGER NOT NULL DEFAULT 0,
                avg_relevance  REAL    NOT NULL DEFAULT 0.0,
                last_updated   TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Bayesian trust update.
    ///
    /// - Success: Beta(α+1, β) → trust_score = (α+1)/(α+β+1)
    /// - Failure: Beta(α, β+1) → trust_score = α/(α+β+1)
    pub fn update_trust(
        &self,
        domain: &str,
        success: bool,
        relevance: f64,
    ) -> Result<f64, HsxError> {
        let conn = self.conn.lock().unwrap();

        // Insert-or-ignore to seed the row.
        conn.execute(
            "INSERT OR IGNORE INTO domain_trust (domain) VALUES (?1)",
            [domain],
        )?;

        if success {
            conn.execute(
                "UPDATE domain_trust SET
                    alpha         = alpha + 1.0,
                    fetch_count   = fetch_count + 1,
                    success_count = success_count + 1,
                    avg_relevance = (avg_relevance * fetch_count + ?2) / (fetch_count + 1.0),
                    trust_score   = (alpha + 1.0) / (alpha + 1.0 + beta),
                    last_updated  = datetime('now')
                 WHERE domain = ?1",
                rusqlite::params![domain, relevance],
            )?;
        } else {
            conn.execute(
                "UPDATE domain_trust SET
                    beta         = beta + 1.0,
                    fetch_count  = fetch_count + 1,
                    trust_score  = alpha / (alpha + beta + 1.0),
                    last_updated = datetime('now')
                 WHERE domain = ?1",
                [domain],
            )?;
        }

        let score: f64 = conn.query_row(
            "SELECT trust_score FROM domain_trust WHERE domain = ?1",
            [domain],
            |row| row.get(0),
        )?;
        Ok(score)
    }

    /// Get trust score for a domain. Returns `0.5` (uninformed prior) if unknown.
    pub fn get_trust(&self, domain: &str) -> Result<f64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let score = conn
            .query_row(
                "SELECT trust_score FROM domain_trust WHERE domain = ?1",
                [domain],
                |row| row.get::<_, f64>(0),
            )
            .unwrap_or(0.5);
        Ok(score)
    }

    /// Top `limit` trusted domains (requiring at least 5 fetches).
    pub fn top_trusted(&self, limit: usize) -> Result<Vec<(String, f64)>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain, trust_score FROM domain_trust
             WHERE fetch_count >= 5
             ORDER BY trust_score DESC
             LIMIT ?1",
        )?;
        let results = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Count of tracked domains.
    pub fn domain_count(&self) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM domain_trust", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    /// Reset all trust data (VACUUM for privacy).
    pub fn reset(&self) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("DELETE FROM domain_trust; VACUUM;")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn bayesian_update_success_increases_trust() {
        let tmp = NamedTempFile::new().unwrap();
        let stm = SourceTrustMemory::new(tmp.path()).unwrap();
        for _ in 0..10 {
            stm.update_trust("example.com", true, 0.9).unwrap();
        }
        let trust = stm.get_trust("example.com").unwrap();
        assert!(trust > 0.7, "trust={trust:.3}");
    }

    #[test]
    fn bayesian_update_failure_decreases_trust() {
        let tmp = NamedTempFile::new().unwrap();
        let stm = SourceTrustMemory::new(tmp.path()).unwrap();
        for _ in 0..10 {
            stm.update_trust("bad.com", false, 0.0).unwrap();
        }
        let trust = stm.get_trust("bad.com").unwrap();
        assert!(trust < 0.3, "trust={trust:.3}");
    }

    #[test]
    fn unknown_domain_returns_prior() {
        let tmp = NamedTempFile::new().unwrap();
        let stm = SourceTrustMemory::new(tmp.path()).unwrap();
        let trust = stm.get_trust("unknown.io").unwrap();
        assert!((trust - 0.5).abs() < 1e-9);
    }
}
