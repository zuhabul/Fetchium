//! Topic subscriptions with SQLite-backed store.

use crate::error::{FetchiumError, FetchiumResult};
use dirs;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// A user-defined subscription to a topic or search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub topic: String,
    /// Interval in seconds.
    pub interval_secs: u64,
    pub notify_method: NotifyMethod,
    pub last_checked_at: Option<String>,
    pub created_at: String,
    pub enabled: bool,
}

/// How to deliver new findings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotifyMethod {
    Stdout,
    File { path: String },
    Webhook { url: String },
}

/// A new finding discovered during a subscription check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFinding {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub published: Option<String>,
}

/// SQLite-backed subscription store.
pub struct SubscriptionStore {
    conn: Connection,
}

impl SubscriptionStore {
    pub fn new(db_path: &std::path::Path) -> FetchiumResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS subscriptions (
                id          TEXT PRIMARY KEY,
                topic       TEXT NOT NULL,
                interval_secs INTEGER NOT NULL DEFAULT 86400,
                notify_json TEXT NOT NULL DEFAULT '\"stdout\"',
                last_checked_at TEXT,
                created_at  TEXT NOT NULL,
                enabled     INTEGER NOT NULL DEFAULT 1
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn add(
        &self,
        topic: &str,
        interval_secs: u64,
        notify: &NotifyMethod,
    ) -> FetchiumResult<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let notify_json = serde_json::to_string(notify)?;
        self.conn.execute(
            "INSERT INTO subscriptions (id, topic, interval_secs, notify_json, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, topic, interval_secs as i64, notify_json, now],
        )?;
        Ok(id)
    }

    pub fn remove(&self, id: &str) -> FetchiumResult<bool> {
        let n = self
            .conn
            .execute("DELETE FROM subscriptions WHERE id=?1", [id])?;
        Ok(n > 0)
    }

    pub fn list(&self) -> FetchiumResult<Vec<Subscription>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic, interval_secs, notify_json, last_checked_at, created_at, enabled
             FROM subscriptions ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, i64>(6)?,
            ))
        })?;
        let mut subs = Vec::new();
        for row in rows {
            let (id, topic, interval_secs, notify_json, last_checked_at, created_at, enabled) =
                row?;
            let notify_method: NotifyMethod =
                serde_json::from_str(&notify_json).unwrap_or(NotifyMethod::Stdout);
            subs.push(Subscription {
                id,
                topic,
                interval_secs: interval_secs as u64,
                notify_method,
                last_checked_at,
                created_at,
                enabled: enabled != 0,
            });
        }
        Ok(subs)
    }

    pub fn mark_checked(&self, id: &str) -> FetchiumResult<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE subscriptions SET last_checked_at=?1 WHERE id=?2",
            params![now, id],
        )?;
        Ok(())
    }

    /// Return subscriptions that are due for a check.
    pub fn due(&self) -> FetchiumResult<Vec<Subscription>> {
        let all = self.list()?;
        let now = chrono::Utc::now();
        Ok(all
            .into_iter()
            .filter(|s| {
                if !s.enabled {
                    return false;
                }
                match &s.last_checked_at {
                    None => true,
                    Some(ts) => {
                        if let Ok(last) = ts.parse::<chrono::DateTime<chrono::Utc>>() {
                            let elapsed = (now - last).num_seconds() as u64;
                            elapsed >= s.interval_secs
                        } else {
                            true
                        }
                    }
                }
            })
            .collect())
    }

    /// Default database path: `~/.fetchium/subscriptions.db`.
    pub fn default_db_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".fetchium")
            .join("subscriptions.db")
    }

    /// Return a human-readable interval string.
    pub fn format_interval(secs: u64) -> String {
        if secs < 3600 {
            format!("{}m", secs / 60)
        } else if secs < 86400 {
            format!("{}h", secs / 3600)
        } else {
            format!("{}d", secs / 86400)
        }
    }
}

/// Parse an interval string like "30m", "1h", "7d" into seconds.
pub fn parse_interval(s: &str) -> FetchiumResult<u64> {
    let bad = || FetchiumError::Config(format!("invalid interval: '{s}'"));
    if let Some(n) = s.strip_suffix('s') {
        Ok(n.parse::<u64>().map_err(|_| bad())?)
    } else if let Some(n) = s.strip_suffix('m') {
        Ok(n.parse::<u64>().map_err(|_| bad())? * 60)
    } else if let Some(n) = s.strip_suffix('h') {
        Ok(n.parse::<u64>().map_err(|_| bad())? * 3600)
    } else if let Some(n) = s.strip_suffix('d') {
        Ok(n.parse::<u64>().map_err(|_| bad())? * 86400)
    } else {
        s.parse::<u64>().map_err(|_| bad())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_interval() {
        assert_eq!(parse_interval("30m").unwrap(), 1800);
        assert_eq!(parse_interval("2h").unwrap(), 7200);
        assert_eq!(parse_interval("7d").unwrap(), 604800);
    }

    #[test]
    fn test_subscription_store_roundtrip() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let store = SubscriptionStore::new(tmp.path()).unwrap();
        let id = store
            .add("Rust async news", 86400, &NotifyMethod::Stdout)
            .unwrap();
        let list = store.list().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, id);
        store.remove(&id).unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
