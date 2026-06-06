//! Auto-expiring research artifacts (PRD §36.3).

use crate::error::FetchiumError;
use crate::intelligence::enable_wal;
use rusqlite::Connection;
use std::sync::Mutex;

/// Database-backed expiry scheduler for research artifacts.
pub struct ExpiryScheduler {
    conn: Mutex<Connection>,
}

impl ExpiryScheduler {
    pub fn new(db_path: &std::path::Path) -> Result<Self, FetchiumError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS expiry_schedule (
                result_id   TEXT    PRIMARY KEY,
                label       TEXT,
                expire_at   TEXT    NOT NULL,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Schedule `result_id` for deletion after `seconds` seconds.
    pub fn schedule(
        &self,
        result_id: &str,
        label: &str,
        seconds: u64,
    ) -> Result<(), FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let expire_at = chrono::Utc::now() + chrono::Duration::seconds(seconds as i64);
        conn.execute(
            "INSERT OR REPLACE INTO expiry_schedule (result_id, label, expire_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![result_id, label, expire_at.to_rfc3339()],
        )?;
        Ok(())
    }

    /// Returns IDs that have passed their expiry time.
    pub fn expired_ids(&self) -> Result<Vec<String>, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let mut stmt =
            conn.prepare("SELECT result_id FROM expiry_schedule WHERE expire_at < ?1")?;
        let ids = stmt
            .query_map([&now], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(ids)
    }

    /// Remove all expiry records that have passed their expiry time.
    pub fn clear_expired(&self) -> Result<usize, FetchiumError> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        let count = conn.execute("DELETE FROM expiry_schedule WHERE expire_at < ?1", [&now])?;
        Ok(count)
    }

    /// Parse a human-readable duration string into seconds.
    ///
    /// Supported: `30s`, `5m`, `2h`, `7d`.
    pub fn parse_duration(s: &str) -> Option<u64> {
        let s = s.trim();
        if let Some(n) = s.strip_suffix('s') {
            return n.parse().ok();
        }
        if let Some(n) = s.strip_suffix('m') {
            return n.parse::<u64>().ok().map(|x| x * 60);
        }
        if let Some(n) = s.strip_suffix('h') {
            return n.parse::<u64>().ok().map(|x| x * 3600);
        }
        if let Some(n) = s.strip_suffix('d') {
            return n.parse::<u64>().ok().map(|x| x * 86400);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_formats() {
        assert_eq!(ExpiryScheduler::parse_duration("30s"), Some(30));
        assert_eq!(ExpiryScheduler::parse_duration("5m"), Some(300));
        assert_eq!(ExpiryScheduler::parse_duration("2h"), Some(7200));
        assert_eq!(ExpiryScheduler::parse_duration("7d"), Some(604_800));
        assert_eq!(ExpiryScheduler::parse_duration("bad"), None);
    }
}
