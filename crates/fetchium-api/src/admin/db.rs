//! AdminDb — SQLite-backed admin user, session, and TOTP management.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;

/// A stored admin user record.
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub name: String,
    pub totp_secret: Option<String>, // base32-encoded
    pub totp_enabled: bool,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

/// A stored admin session record.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdminSession {
    pub id: String,
    pub admin_user_id: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: String,
    pub last_active_at: String,
    pub expires_at: String,
}

/// SQLite-backed admin database.
pub struct AdminDb {
    conn: Mutex<Connection>,
}

impl AdminDb {
    /// Open (or create) the admin.db at `path` and run migrations.
    pub fn open(path: &Path) -> Result<Arc<Self>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating admin db dir {}", parent.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("opening admin.db at {}", path.display()))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )?;
        let db = Arc::new(Self {
            conn: Mutex::new(conn),
        });
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS admin_users (
                id            TEXT PRIMARY KEY,
                email         TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                role          TEXT NOT NULL,
                name          TEXT NOT NULL,
                totp_secret   TEXT,
                totp_enabled  INTEGER NOT NULL DEFAULT 0,
                is_active     INTEGER NOT NULL DEFAULT 1,
                created_at    TEXT NOT NULL,
                updated_at    TEXT NOT NULL,
                last_login_at TEXT,
                last_login_ip TEXT
            );

            CREATE TABLE IF NOT EXISTS admin_sessions (
                id             TEXT PRIMARY KEY,
                admin_user_id  TEXT NOT NULL REFERENCES admin_users(id),
                token_hash     TEXT NOT NULL UNIQUE,
                ip             TEXT NOT NULL,
                user_agent     TEXT NOT NULL,
                created_at     TEXT NOT NULL,
                last_active_at TEXT NOT NULL,
                expires_at     TEXT NOT NULL,
                revoked_at     TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_token ON admin_sessions(token_hash);
            CREATE INDEX IF NOT EXISTS idx_sessions_user  ON admin_sessions(admin_user_id);

            CREATE TABLE IF NOT EXISTS admin_totp_used (
                admin_user_id TEXT NOT NULL,
                code          TEXT NOT NULL,
                used_at       TEXT NOT NULL,
                PRIMARY KEY (admin_user_id, code)
            );

            CREATE TABLE IF NOT EXISTS admin_backup_codes (
                id            TEXT PRIMARY KEY,
                admin_user_id TEXT NOT NULL REFERENCES admin_users(id),
                code_hash     TEXT NOT NULL,
                used_at       TEXT
            );
        "#,
        )?;
        Ok(())
    }

    // ── User operations ──────────────────────────────────────────────────────

    pub fn has_any_users(&self) -> Result<bool> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM admin_users", [], |r| r.get(0))?;
        Ok(count > 0)
    }

    pub fn create_user(
        &self,
        id: &str,
        email: &str,
        password_hash: &str,
        role: &str,
        name: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO admin_users (id, email, password_hash, role, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, email, password_hash, role, name, now],
        )?;
        Ok(())
    }

    pub fn find_user_by_email(&self, email: &str) -> Result<Option<AdminUser>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, email, password_hash, role, name, totp_secret, totp_enabled,
                     is_active, created_at, last_login_at
             FROM admin_users WHERE email = ?1",
            params![email],
            |r| {
                Ok(AdminUser {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    password_hash: r.get(2)?,
                    role: r.get(3)?,
                    name: r.get(4)?,
                    totp_secret: r.get(5)?,
                    totp_enabled: r.get::<_, i64>(6)? != 0,
                    is_active: r.get::<_, i64>(7)? != 0,
                    created_at: r.get(8)?,
                    last_login_at: r.get(9)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn find_user_by_id(&self, id: &str) -> Result<Option<AdminUser>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, email, password_hash, role, name, totp_secret, totp_enabled,
                     is_active, created_at, last_login_at
             FROM admin_users WHERE id = ?1",
            params![id],
            |r| {
                Ok(AdminUser {
                    id: r.get(0)?,
                    email: r.get(1)?,
                    password_hash: r.get(2)?,
                    role: r.get(3)?,
                    name: r.get(4)?,
                    totp_secret: r.get(5)?,
                    totp_enabled: r.get::<_, i64>(6)? != 0,
                    is_active: r.get::<_, i64>(7)? != 0,
                    created_at: r.get(8)?,
                    last_login_at: r.get(9)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn update_last_login(&self, user_id: &str, ip: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_users SET last_login_at = ?1, last_login_ip = ?2, updated_at = ?1
             WHERE id = ?3",
            params![now, ip, user_id],
        )?;
        Ok(())
    }

    pub fn set_totp(&self, user_id: &str, secret: &str, enabled: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_users SET totp_secret = ?1, totp_enabled = ?2, updated_at = ?3
             WHERE id = ?4",
            params![secret, enabled as i64, now, user_id],
        )?;
        Ok(())
    }

    // ── Session operations ───────────────────────────────────────────────────

    /// Create a new session, return session_id.
    pub fn create_session(
        &self,
        session_id: &str,
        user_id: &str,
        token_hash: &str,
        ip: &str,
        user_agent: &str,
    ) -> Result<()> {
        let now = Utc::now();
        let expires_at = (now + Duration::hours(8)).to_rfc3339();
        let now_str = now.to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO admin_sessions
             (id, admin_user_id, token_hash, ip, user_agent, created_at, last_active_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
            params![session_id, user_id, token_hash, ip, user_agent, now_str, expires_at],
        )?;
        Ok(())
    }

    /// Validate a session token hash, return (AdminUser, session_id) if valid.
    pub fn validate_session(&self, token_hash: &str) -> Result<Option<(AdminUser, String)>> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT id, admin_user_id FROM admin_sessions
             WHERE token_hash = ?1
               AND revoked_at IS NULL
               AND expires_at > ?2",
                params![token_hash, now],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;

        let Some((session_id, user_id)) = row else {
            return Ok(None);
        };

        let user = conn
            .query_row(
                "SELECT id, email, password_hash, role, name, totp_secret, totp_enabled,
                     is_active, created_at, last_login_at
             FROM admin_users WHERE id = ?1 AND is_active = 1",
                params![user_id],
                |r| {
                    Ok(AdminUser {
                        id: r.get(0)?,
                        email: r.get(1)?,
                        password_hash: r.get(2)?,
                        role: r.get(3)?,
                        name: r.get(4)?,
                        totp_secret: r.get(5)?,
                        totp_enabled: r.get::<_, i64>(6)? != 0,
                        is_active: r.get::<_, i64>(7)? != 0,
                        created_at: r.get(8)?,
                        last_login_at: r.get(9)?,
                    })
                },
            )
            .optional()?;

        Ok(user.map(|u| (u, session_id)))
    }

    /// Extend session TTL and update last_active_at.
    pub fn touch_session(&self, session_id: &str) -> Result<()> {
        let now = Utc::now();
        let new_expires = (now + Duration::hours(8)).to_rfc3339();
        let now_str = now.to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_sessions SET last_active_at = ?1, expires_at = ?2 WHERE id = ?3",
            params![now_str, new_expires, session_id],
        )?;
        Ok(())
    }

    pub fn revoke_session(&self, session_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_sessions SET revoked_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self, user_id: &str) -> Result<Vec<AdminSession>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, admin_user_id, ip, user_agent, created_at, last_active_at, expires_at
             FROM admin_sessions
             WHERE admin_user_id = ?1 AND revoked_at IS NULL AND expires_at > ?2
             ORDER BY last_active_at DESC",
        )?;
        let now = Utc::now().to_rfc3339();
        let sessions = stmt
            .query_map(params![user_id, now], |r| {
                Ok(AdminSession {
                    id: r.get(0)?,
                    admin_user_id: r.get(1)?,
                    ip: r.get(2)?,
                    user_agent: r.get(3)?,
                    created_at: r.get(4)?,
                    last_active_at: r.get(5)?,
                    expires_at: r.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(sessions)
    }

    // ── TOTP replay prevention ───────────────────────────────────────────────

    pub fn is_totp_code_used(&self, user_id: &str, code: &str) -> Result<bool> {
        let conn = self.conn.lock();
        // Only check codes used in the last 90 seconds (3 TOTP windows)
        let cutoff = (Utc::now() - Duration::seconds(90)).to_rfc3339();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM admin_totp_used
             WHERE admin_user_id = ?1 AND code = ?2 AND used_at > ?3",
            params![user_id, code, cutoff],
            |r| r.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn mark_totp_code_used(&self, user_id: &str, code: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT OR REPLACE INTO admin_totp_used (admin_user_id, code, used_at)
             VALUES (?1, ?2, ?3)",
            params![user_id, code, now],
        )?;
        Ok(())
    }
}
