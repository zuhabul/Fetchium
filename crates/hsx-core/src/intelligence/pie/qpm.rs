//! Query Prediction Model (QPM) — predicts follow-up queries from historical patterns.
//!
//! Schema:
//! - `query_history(id, query_hash, query_text, topic, timestamp)`
//! - `follow_up_patterns(topic, follow_up_query, count)`
//! - `topic_frequency(topic, count, last_queried)`

use rusqlite::Connection;
use std::sync::Mutex;

use crate::error::HsxError;
use crate::intelligence::{enable_wal, sha256_hex};

pub struct QueryPredictionModel {
    conn: Mutex<Connection>,
}

impl QueryPredictionModel {
    pub fn new(db_path: &std::path::Path) -> Result<Self, HsxError> {
        let conn = Connection::open(db_path)?;
        enable_wal(&conn)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS query_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                query_hash  TEXT    NOT NULL,
                query_text  TEXT    NOT NULL,
                topic       TEXT    NOT NULL,
                timestamp   TEXT    NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE IF NOT EXISTS follow_up_patterns (
                topic           TEXT NOT NULL,
                follow_up_query TEXT NOT NULL,
                count           INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (topic, follow_up_query)
            );
            CREATE TABLE IF NOT EXISTS topic_frequency (
                topic        TEXT PRIMARY KEY,
                count        INTEGER NOT NULL DEFAULT 0,
                last_queried TEXT    NOT NULL DEFAULT (datetime('now'))
            );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Record a user query and its extracted topic.
    ///
    /// Optionally also record a follow-up pattern (query that immediately followed
    /// a previous query on the same topic).
    pub fn record_query(
        &self,
        query: &str,
        topic: &str,
        follow_up_of: Option<&str>,
    ) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        let hash = sha256_hex(query);

        conn.execute(
            "INSERT INTO query_history (query_hash, query_text, topic) VALUES (?1, ?2, ?3)",
            rusqlite::params![hash, query, topic],
        )?;

        if let Some(prev_topic) = follow_up_of {
            conn.execute(
                "INSERT INTO follow_up_patterns (topic, follow_up_query, count)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(topic, follow_up_query) DO UPDATE SET count = count + 1",
                rusqlite::params![prev_topic, query],
            )?;
        }

        conn.execute(
            "INSERT INTO topic_frequency (topic, count, last_queried)
             VALUES (?1, 1, datetime('now'))
             ON CONFLICT(topic) DO UPDATE SET
                count        = count + 1,
                last_queried = datetime('now')",
            [topic],
        )?;

        Ok(())
    }

    /// Predict likely follow-up queries given a current topic, sorted by frequency.
    pub fn predict_follow_ups(
        &self,
        current_topic: &str,
        limit: usize,
    ) -> Result<Vec<String>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT follow_up_query FROM follow_up_patterns
             WHERE topic = ?1
             ORDER BY count DESC
             LIMIT ?2",
        )?;
        let results = stmt
            .query_map(rusqlite::params![current_topic, limit as i64], |row| {
                row.get::<_, String>(0)
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Most frequently queried topics.
    pub fn top_topics(&self, limit: usize) -> Result<Vec<(String, u64)>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT topic, count FROM topic_frequency
             ORDER BY count DESC
             LIMIT ?1",
        )?;
        let results = stmt
            .query_map([limit as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    /// Total queries recorded.
    pub fn query_count(&self) -> Result<u64, HsxError> {
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM query_history", [], |row| row.get(0))?;
        Ok(n as u64)
    }

    /// Reset all query prediction data.
    pub fn reset(&self) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM query_history;
             DELETE FROM follow_up_patterns;
             DELETE FROM topic_frequency;
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
    fn predict_returns_most_frequent_follow_up() {
        let tmp = NamedTempFile::new().unwrap();
        let qpm = QueryPredictionModel::new(tmp.path()).unwrap();

        // Record "Rust memory model" 3× after "Rust ownership"
        for _ in 0..3 {
            qpm.record_query("Rust memory model", "Rust", Some("Rust"))
                .unwrap();
        }
        qpm.record_query("Rust async", "Rust", Some("Rust"))
            .unwrap();

        let suggestions = qpm.predict_follow_ups("Rust", 5).unwrap();
        assert!(
            !suggestions.is_empty(),
            "should predict at least one follow-up"
        );
        assert_eq!(suggestions[0], "Rust memory model");
    }

    #[test]
    fn query_count_increments() {
        let tmp = NamedTempFile::new().unwrap();
        let qpm = QueryPredictionModel::new(tmp.path()).unwrap();
        for i in 0..5u32 {
            qpm.record_query(&format!("query {i}"), "topic", None)
                .unwrap();
        }
        assert_eq!(qpm.query_count().unwrap(), 5);
    }
}
