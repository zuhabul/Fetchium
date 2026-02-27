//! API key authentication and usage tracking — SQLite-backed.
//!
//! ## Key format
//! `fetchium_` + 32 lowercase hex chars (128-bit random token)
//! Stored as SHA-256 hash; raw key shown exactly once on creation.
//!
//! ## Plan tiers
//! | Plan       | req/min | req/month |
//! |------------|---------|-----------|
//! | free       | 60      | 1,000     |
//! | starter    | 200     | 25,000    |
//! | pro        | 500     | 250,000   |
//! | enterprise | 2,000   | unlimited |

use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;

/// Plan-level limits.
#[derive(Debug, Clone)]
pub struct PlanLimits {
    pub requests_per_min: u32,
    pub requests_per_month: Option<u32>, // None = unlimited
}

impl PlanLimits {
    pub fn for_plan(plan: &str) -> Self {
        match plan {
            "starter" => Self {
                requests_per_min: 200,
                requests_per_month: Some(25_000),
            },
            "pro" => Self {
                requests_per_min: 500,
                requests_per_month: Some(250_000),
            },
            "enterprise" => Self {
                requests_per_min: 2_000,
                requests_per_month: None,
            },
            _ => Self {
                requests_per_min: 60,
                requests_per_month: Some(1_000),
            }, // free
        }
    }
}

/// A stored API key record (no raw key — only hash stored).
#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: String,
    pub name: String,
    pub key_prefix: String, // first 16 chars of raw key for display: `fetchium_xxxxxxx`
    pub plan: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub revoked: bool,
}

/// Monthly usage statistics for one key.
#[derive(Debug, Clone, serde::Serialize)]
pub struct UsageStats {
    pub key_id: String,
    pub plan: String,
    pub requests_this_month: u32,
    pub requests_today: u32,
    pub tokens_this_month: u64,
    pub monthly_limit: Option<u32>,
    pub quota_remaining: Option<u32>,
}

/// SQLite-backed API key database.
pub struct AuthDb {
    conn: Mutex<Connection>,
}

impl AuthDb {
    /// Open (or create) the auth database at the given path and run migrations.
    pub fn open(path: &Path) -> Result<Arc<Self>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create auth db dir")?;
        }

        let conn = Connection::open(path).context("open auth.db")?;

        // WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        // Run migrations
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS api_keys (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                key_hash    TEXT NOT NULL UNIQUE,
                key_prefix  TEXT NOT NULL,
                plan        TEXT NOT NULL DEFAULT 'free',
                created_at  TEXT NOT NULL,
                last_used_at TEXT,
                revoked     INTEGER NOT NULL DEFAULT 0
            );

            CREATE INDEX IF NOT EXISTS idx_key_hash ON api_keys(key_hash);

            CREATE TABLE IF NOT EXISTS usage_logs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                key_id      TEXT NOT NULL REFERENCES api_keys(id),
                endpoint    TEXT NOT NULL,
                status      INTEGER NOT NULL,
                tokens_used INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_usage_key_date
                ON usage_logs(key_id, created_at);
        ",
        )?;

        tracing::info!("AuthDb: opened at {}", path.display());
        Ok(Arc::new(Self {
            conn: Mutex::new(conn),
        }))
    }

    /// Create a new API key. Returns `(raw_key, record)`.
    /// The raw key is shown exactly once — store it or it's lost.
    pub fn create_key(&self, name: &str, plan: &str) -> Result<(String, ApiKeyRecord)> {
        let id = uuid::Uuid::new_v4().to_string();
        let token: String = (0..32)
            .map(|_| format!("{:02x}", rand::random::<u8>()))
            .collect();
        let raw_key = format!("fetchium_{token}");
        let key_prefix = raw_key[..16].to_string(); // "fetchium_" + 7 hex chars
        let key_hash = hash_key(&raw_key);
        let created_at = Utc::now().to_rfc3339();

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO api_keys (id, name, key_hash, key_prefix, plan, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, key_hash, key_prefix, plan, created_at],
        )
        .context("insert api key")?;

        let record = ApiKeyRecord {
            id,
            name: name.to_string(),
            key_prefix,
            plan: plan.to_string(),
            created_at,
            last_used_at: None,
            revoked: false,
        };

        tracing::info!(
            "AuthDb: created key '{}' (plan={})",
            record.name,
            record.plan
        );
        Ok((raw_key, record))
    }

    /// Validate a raw API key. Returns the record if valid (not revoked).
    /// Also updates `last_used_at` on success.
    pub fn validate_key(&self, raw_key: &str) -> Result<Option<ApiKeyRecord>> {
        let key_hash = hash_key(raw_key);
        let conn = self.conn.lock();

        let result = conn.query_row(
            "SELECT id, name, key_prefix, plan, created_at, last_used_at, revoked
             FROM api_keys WHERE key_hash = ?1",
            params![key_hash],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, bool>(6)?,
                ))
            },
        );

        match result {
            Ok((id, name, key_prefix, plan, created_at, last_used_at, revoked)) => {
                if revoked {
                    return Ok(None);
                }
                // Update last_used_at (best-effort, ignore errors)
                let now = Utc::now().to_rfc3339();
                let _ = conn.execute(
                    "UPDATE api_keys SET last_used_at = ?1 WHERE id = ?2",
                    params![now, id],
                );
                Ok(Some(ApiKeyRecord {
                    id,
                    name,
                    key_prefix,
                    plan,
                    created_at,
                    last_used_at,
                    revoked,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Revoke an API key by ID.
    pub fn revoke_key(&self, key_id: &str) -> Result<bool> {
        let conn = self.conn.lock();
        let rows = conn.execute(
            "UPDATE api_keys SET revoked = 1 WHERE id = ?1 AND revoked = 0",
            params![key_id],
        )?;
        Ok(rows > 0)
    }

    /// List all non-revoked keys (for display — no hashes).
    pub fn list_keys(&self) -> Result<Vec<ApiKeyRecord>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, key_prefix, plan, created_at, last_used_at, revoked
             FROM api_keys WHERE revoked = 0 ORDER BY created_at DESC",
        )?;

        let keys = stmt
            .query_map([], |row| {
                Ok(ApiKeyRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    key_prefix: row.get(2)?,
                    plan: row.get(3)?,
                    created_at: row.get(4)?,
                    last_used_at: row.get(5)?,
                    revoked: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("list keys")?;

        Ok(keys)
    }

    /// Record a single API request in the usage log.
    pub fn record_usage(
        &self,
        key_id: &str,
        endpoint: &str,
        status: u16,
        tokens_used: u64,
        duration_ms: u64,
    ) -> Result<()> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO usage_logs (key_id, endpoint, status, tokens_used, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![key_id, endpoint, status, tokens_used, duration_ms, now],
        ).context("record usage")?;
        Ok(())
    }

    /// Get usage statistics for a key (current month + today).
    pub fn get_usage_stats(&self, key_id: &str, plan: &str) -> Result<UsageStats> {
        let conn = self.conn.lock();
        let month_prefix = Utc::now().format("%Y-%m").to_string();
        let today_prefix = Utc::now().format("%Y-%m-%d").to_string();

        let requests_this_month: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_logs WHERE key_id = ?1 AND created_at LIKE ?2",
                params![key_id, format!("{month_prefix}%")],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let requests_today: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_logs WHERE key_id = ?1 AND created_at LIKE ?2",
                params![key_id, format!("{today_prefix}%")],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let tokens_this_month: i64 = conn
            .query_row(
                "SELECT COALESCE(SUM(tokens_used), 0) FROM usage_logs
             WHERE key_id = ?1 AND created_at LIKE ?2",
                params![key_id, format!("{month_prefix}%")],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let limits = PlanLimits::for_plan(plan);
        let quota_remaining = limits
            .requests_per_month
            .map(|limit| limit.saturating_sub(requests_this_month));

        Ok(UsageStats {
            key_id: key_id.to_string(),
            plan: plan.to_string(),
            requests_this_month,
            requests_today,
            tokens_this_month: tokens_this_month as u64,
            monthly_limit: limits.requests_per_month,
            quota_remaining,
        })
    }

    /// Check whether a key has exceeded its monthly quota. Returns `true` if allowed.
    pub fn check_quota(&self, key_id: &str, plan: &str) -> bool {
        let limits = PlanLimits::for_plan(plan);
        let Some(monthly_limit) = limits.requests_per_month else {
            return true; // enterprise = unlimited
        };

        let conn = self.conn.lock();
        let month_prefix = Utc::now().format("%Y-%m").to_string();
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_logs WHERE key_id = ?1 AND created_at LIKE ?2",
                params![key_id, format!("{month_prefix}%")],
                |row| row.get(0),
            )
            .unwrap_or(0);

        count < monthly_limit
    }
}

/// SHA-256 hash a raw API key for storage.
fn hash_key(raw_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_db() -> Arc<AuthDb> {
        let dir = tempdir().unwrap();
        AuthDb::open(&dir.path().join("auth.db")).unwrap()
    }

    #[test]
    fn create_and_validate_key() {
        let db = test_db();
        let (raw_key, record) = db.create_key("test key", "free").unwrap();
        assert!(raw_key.starts_with("fetchium_"));
        assert_eq!(raw_key.len(), 73); // "fetchium_" + 64 hex (32 random bytes)
        assert_eq!(record.plan, "free");

        let found = db.validate_key(&raw_key).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test key");
    }

    #[test]
    fn invalid_key_returns_none() {
        let db = test_db();
        let result = db
            .validate_key("fetchium_nonexistent00000000000000000000")
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn revoke_key() {
        let db = test_db();
        let (raw_key, record) = db.create_key("revoke me", "free").unwrap();
        assert!(db.revoke_key(&record.id).unwrap());
        // Revoked key should not validate
        let found = db.validate_key(&raw_key).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn usage_tracking() {
        let db = test_db();
        let (_, record) = db.create_key("usage test", "free").unwrap();
        db.record_usage(&record.id, "/v1/search", 200, 1500, 234)
            .unwrap();
        db.record_usage(&record.id, "/v1/scrape", 200, 3000, 500)
            .unwrap();

        let stats = db.get_usage_stats(&record.id, "free").unwrap();
        assert_eq!(stats.requests_this_month, 2);
        assert_eq!(stats.tokens_this_month, 4500);
    }

    #[test]
    fn quota_enforcement() {
        let db = test_db();
        let (_, record) = db.create_key("quota test", "free").unwrap();
        // Free = 1000/month
        assert!(db.check_quota(&record.id, "free"));
    }

    #[test]
    fn plan_limits() {
        assert_eq!(PlanLimits::for_plan("free").requests_per_month, Some(1_000));
        assert_eq!(
            PlanLimits::for_plan("starter").requests_per_month,
            Some(25_000)
        );
        assert_eq!(
            PlanLimits::for_plan("pro").requests_per_month,
            Some(250_000)
        );
        assert_eq!(PlanLimits::for_plan("enterprise").requests_per_month, None);
    }

    #[test]
    fn list_keys() {
        let db = test_db();
        db.create_key("key one", "free").unwrap();
        db.create_key("key two", "pro").unwrap();
        let keys = db.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
    }
}
