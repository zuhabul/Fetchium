//! Content snapshot store for change detection (Phase 5, PRD §10, Mode G).
//!
//! Stores SHA-256 content hashes + full content snapshots in SQLite.
//! Change detection is O(1) via hash comparison.

use crate::error::HsxError;
use rusqlite::{Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::debug;

/// A saved content snapshot.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: i64,
    pub url: String,
    pub content_hash: String,
    pub content: String,
    pub fetched_at: String,
}

/// A registered monitor entry.
#[derive(Debug, Clone)]
pub struct MonitorEntry {
    pub url: String,
    pub interval_secs: u64,
    pub last_checked: Option<String>,
    pub notify_method: Option<String>,
}

/// SQLite-backed snapshot store.
pub struct SnapshotStore {
    conn: Mutex<Connection>,
}

impl SnapshotStore {
    /// Open or create the snapshot database.
    pub fn new() -> Result<Self, HsxError> {
        let path = default_db_path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let conn = Connection::open(&path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS snapshots (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 url          TEXT    NOT NULL,
                 content_hash TEXT    NOT NULL,
                 content      TEXT    NOT NULL,
                 fetched_at   TEXT    DEFAULT (datetime('now')),
                 UNIQUE(url, content_hash)
             );
             CREATE TABLE IF NOT EXISTS monitors (
                 url             TEXT PRIMARY KEY,
                 interval_secs   INTEGER NOT NULL DEFAULT 3600,
                 last_checked    TEXT,
                 notify_method   TEXT
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Open the snapshot database at a custom path (useful for tests).
    pub fn new_at(path: &std::path::Path) -> Result<Self, HsxError> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS snapshots (
                 id           INTEGER PRIMARY KEY AUTOINCREMENT,
                 url          TEXT    NOT NULL,
                 content_hash TEXT    NOT NULL,
                 content      TEXT    NOT NULL,
                 fetched_at   TEXT    DEFAULT (datetime('now')),
                 UNIQUE(url, content_hash)
             );
             CREATE TABLE IF NOT EXISTS monitors (
                 url             TEXT PRIMARY KEY,
                 interval_secs   INTEGER NOT NULL DEFAULT 3600,
                 last_checked    TEXT,
                 notify_method   TEXT
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Save a new snapshot. Returns `true` if the content changed since last snapshot.
    pub fn save_snapshot(&self, url: &str, content: &str) -> Result<bool, HsxError> {
        let hash = sha256_hex(content);
        let conn = self.conn.lock().unwrap();

        // Check last hash for this URL
        let last_hash: Option<String> = conn
            .query_row(
                "SELECT content_hash FROM snapshots WHERE url = ?1 ORDER BY id DESC LIMIT 1",
                [url],
                |r| r.get(0),
            )
            .optional()?;

        let changed = last_hash.as_deref() != Some(hash.as_str());

        if changed {
            conn.execute(
                "INSERT OR IGNORE INTO snapshots (url, content_hash, content) VALUES (?1, ?2, ?3)",
                rusqlite::params![url, hash, content],
            )?;
            debug!("Snapshot saved for {url} (hash={:.8})", hash);
        } else {
            debug!("No change detected for {url}");
        }

        Ok(changed)
    }

    /// Get the most recent snapshot for a URL.
    pub fn get_latest(&self, url: &str) -> Result<Option<Snapshot>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, url, content_hash, content, fetched_at
                 FROM snapshots WHERE url = ?1 ORDER BY id DESC LIMIT 1",
                [url],
                |r| {
                    Ok(Snapshot {
                        id: r.get(0)?,
                        url: r.get(1)?,
                        content_hash: r.get(2)?,
                        content: r.get(3)?,
                        fetched_at: r.get(4)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Get the previous snapshot (second-latest) for a URL.
    pub fn get_previous(&self, url: &str) -> Result<Option<Snapshot>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let result = conn
            .query_row(
                "SELECT id, url, content_hash, content, fetched_at
                 FROM snapshots WHERE url = ?1 ORDER BY id DESC LIMIT 1 OFFSET 1",
                [url],
                |r| {
                    Ok(Snapshot {
                        id: r.get(0)?,
                        url: r.get(1)?,
                        content_hash: r.get(2)?,
                        content: r.get(3)?,
                        fetched_at: r.get(4)?,
                    })
                },
            )
            .optional()?;
        Ok(result)
    }

    /// Register a URL for monitoring.
    pub fn register(
        &self,
        url: &str,
        interval_secs: u64,
        notify_method: Option<&str>,
    ) -> Result<(), HsxError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO monitors (url, interval_secs, notify_method) VALUES (?1, ?2, ?3)
             ON CONFLICT(url) DO UPDATE SET interval_secs = excluded.interval_secs,
                                            notify_method = excluded.notify_method",
            rusqlite::params![url, interval_secs, notify_method],
        )?;
        Ok(())
    }

    /// Unregister a URL from monitoring.
    pub fn unregister(&self, url: &str) -> Result<bool, HsxError> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM monitors WHERE url = ?1", [url])?;
        Ok(rows > 0)
    }

    /// List all monitored URLs.
    pub fn list_monitors(&self) -> Result<Vec<MonitorEntry>, HsxError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT url, interval_secs, last_checked, notify_method FROM monitors")?;
        let entries = stmt
            .query_map([], |r| {
                Ok(MonitorEntry {
                    url: r.get(0)?,
                    interval_secs: r.get(1)?,
                    last_checked: r.get(2)?,
                    notify_method: r.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }

    /// Prune old snapshots, keeping at most `keep` per URL.
    pub fn prune_old_snapshots(&self, keep: usize) -> Result<usize, HsxError> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            &format!(
                "DELETE FROM snapshots WHERE id NOT IN (
                     SELECT id FROM snapshots s2
                     WHERE s2.url = snapshots.url
                     ORDER BY id DESC LIMIT {keep}
                 )"
            ),
            [],
        )?;
        Ok(deleted)
    }
}

fn sha256_hex(content: &str) -> String {
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

fn default_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hypersearchx")
        .join("monitor.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn make_store() -> (SnapshotStore, NamedTempFile) {
        let tmp = NamedTempFile::new().unwrap();
        let store = SnapshotStore::new_at(tmp.path()).unwrap();
        (store, tmp)
    }

    #[test]
    fn first_snapshot_is_changed() {
        let (store, _tmp) = make_store();
        let changed = store
            .save_snapshot("https://example.com", "content v1")
            .unwrap();
        assert!(changed, "first snapshot should report changed");
    }

    #[test]
    fn same_content_not_changed() {
        let (store, _tmp) = make_store();
        store
            .save_snapshot("https://example.com", "content")
            .unwrap();
        let changed = store
            .save_snapshot("https://example.com", "content")
            .unwrap();
        assert!(!changed, "identical content should not report changed");
    }

    #[test]
    fn different_content_is_changed() {
        let (store, _tmp) = make_store();
        store
            .save_snapshot("https://example.com", "version 1")
            .unwrap();
        let changed = store
            .save_snapshot("https://example.com", "version 2")
            .unwrap();
        assert!(changed, "different content should report changed");
    }

    #[test]
    fn get_latest_returns_most_recent() {
        let (store, _tmp) = make_store();
        store.save_snapshot("https://example.com", "v1").unwrap();
        store.save_snapshot("https://example.com", "v2").unwrap();
        let snap = store.get_latest("https://example.com").unwrap().unwrap();
        assert_eq!(snap.content, "v2");
    }

    #[test]
    fn register_and_list_monitors() {
        let (store, _tmp) = make_store();
        store.register("https://a.com", 3600, None).unwrap();
        store
            .register("https://b.com", 1800, Some("webhook"))
            .unwrap();
        let monitors = store.list_monitors().unwrap();
        assert_eq!(monitors.len(), 2);
    }

    #[test]
    fn unregister_removes_entry() {
        let (store, _tmp) = make_store();
        store.register("https://x.com", 3600, None).unwrap();
        let removed = store.unregister("https://x.com").unwrap();
        assert!(removed);
        let monitors = store.list_monitors().unwrap();
        assert!(monitors.is_empty());
    }
}
