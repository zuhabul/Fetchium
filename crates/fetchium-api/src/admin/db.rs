//! AdminDb — SQLite-backed admin user, session, and TOTP management.

use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

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

/// Result of a read-only SQL query.
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// SQLite-backed admin database.
pub struct AdminDb {
    conn: Mutex<Connection>,
}

impl AdminDb {
    /// Open an in-memory SQLite database and run migrations. Used in tests.
    pub fn open_in_memory() -> Result<Arc<Self>> {
        let conn = Connection::open_in_memory().context("opening in-memory admin db")?;
        let db = Arc::new(Self {
            conn: Mutex::new(conn),
        });
        db.migrate()?;
        Ok(db)
    }

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
            -- Migration 1: admin auth tables
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

            -- Migration 2: organizations
            CREATE TABLE IF NOT EXISTS organizations (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                slug        TEXT NOT NULL UNIQUE,
                status      TEXT NOT NULL DEFAULT 'active',
                plan        TEXT NOT NULL DEFAULT 'free',
                owner_email TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS organization_members (
                id          TEXT PRIMARY KEY,
                org_id      TEXT NOT NULL REFERENCES organizations(id),
                user_id     TEXT NOT NULL,
                role        TEXT NOT NULL DEFAULT 'member',
                joined_at   TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS customer_users (
                id         TEXT PRIMARY KEY,
                org_id     TEXT NOT NULL REFERENCES organizations(id),
                email      TEXT NOT NULL,
                name       TEXT,
                status     TEXT NOT NULL DEFAULT 'active',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Migration 3: billing
            CREATE TABLE IF NOT EXISTS subscriptions (
                id              TEXT PRIMARY KEY,
                org_id          TEXT NOT NULL REFERENCES organizations(id),
                plan            TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'active',
                current_period_start TEXT,
                current_period_end   TEXT,
                created_at      TEXT NOT NULL,
                updated_at      TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS invoices (
                id         TEXT PRIMARY KEY,
                org_id     TEXT NOT NULL REFERENCES organizations(id),
                amount     INTEGER NOT NULL DEFAULT 0,
                currency   TEXT NOT NULL DEFAULT 'usd',
                status     TEXT NOT NULL DEFAULT 'draft',
                due_date   TEXT,
                paid_at    TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS credits_ledger (
                id         TEXT PRIMARY KEY,
                org_id     TEXT NOT NULL REFERENCES organizations(id),
                amount     INTEGER NOT NULL,
                reason     TEXT,
                granted_by TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS payment_events (
                id         TEXT PRIMARY KEY,
                org_id     TEXT REFERENCES organizations(id),
                event_type TEXT NOT NULL,
                payload    TEXT,
                created_at TEXT NOT NULL
            );

            -- Migration 4: support
            CREATE TABLE IF NOT EXISTS support_tickets (
                id          TEXT PRIMARY KEY,
                org_id      TEXT REFERENCES organizations(id),
                subject     TEXT NOT NULL,
                body        TEXT,
                status      TEXT NOT NULL DEFAULT 'open',
                priority    TEXT NOT NULL DEFAULT 'normal',
                assignee_id TEXT REFERENCES admin_users(id),
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS support_notes (
                id         TEXT PRIMARY KEY,
                ticket_id  TEXT NOT NULL REFERENCES support_tickets(id),
                author_id  TEXT REFERENCES admin_users(id),
                body       TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS support_macros (
                id         TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                body       TEXT NOT NULL,
                created_by TEXT REFERENCES admin_users(id),
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- Migration 5: CRM
            CREATE TABLE IF NOT EXISTS crm_accounts (
                id          TEXT PRIMARY KEY,
                org_id      TEXT NOT NULL UNIQUE REFERENCES organizations(id),
                health      TEXT NOT NULL DEFAULT 'healthy',
                csm_id      TEXT REFERENCES admin_users(id),
                mrr         INTEGER NOT NULL DEFAULT 0,
                nps         INTEGER,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS crm_notes (
                id         TEXT PRIMARY KEY,
                org_id     TEXT NOT NULL REFERENCES organizations(id),
                author_id  TEXT REFERENCES admin_users(id),
                body       TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            -- Migration 6: campaigns
            CREATE TABLE IF NOT EXISTS campaigns (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                type        TEXT NOT NULL DEFAULT 'email',
                status      TEXT NOT NULL DEFAULT 'draft',
                created_by  TEXT REFERENCES admin_users(id),
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS attribution_touches (
                id          TEXT PRIMARY KEY,
                campaign_id TEXT NOT NULL REFERENCES campaigns(id),
                org_id      TEXT REFERENCES organizations(id),
                touch_type  TEXT NOT NULL,
                occurred_at TEXT NOT NULL
            );

            -- Migration 7: incidents
            CREATE TABLE IF NOT EXISTS incidents (
                id          TEXT PRIMARY KEY,
                title       TEXT NOT NULL,
                severity    TEXT NOT NULL DEFAULT 'low',
                status      TEXT NOT NULL DEFAULT 'open',
                owner_id    TEXT REFERENCES admin_users(id),
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL,
                resolved_at TEXT
            );

            CREATE TABLE IF NOT EXISTS incident_timeline (
                id          TEXT PRIMARY KEY,
                incident_id TEXT NOT NULL REFERENCES incidents(id),
                author_id   TEXT REFERENCES admin_users(id),
                message     TEXT NOT NULL,
                created_at  TEXT NOT NULL
            );

            -- Migration 8: audit + feature flags
            CREATE TABLE IF NOT EXISTS audit_events (
                id            TEXT PRIMARY KEY,
                admin_user_id TEXT REFERENCES admin_users(id),
                role          TEXT,
                target_type   TEXT NOT NULL,
                target_id     TEXT,
                action        TEXT NOT NULL,
                ip            TEXT,
                created_at    TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_audit_user   ON audit_events(admin_user_id);
            CREATE INDEX IF NOT EXISTS idx_audit_target ON audit_events(target_type, target_id);
            CREATE INDEX IF NOT EXISTS idx_audit_time   ON audit_events(created_at);

            CREATE TABLE IF NOT EXISTS feature_flags (
                id          TEXT PRIMARY KEY,
                key         TEXT NOT NULL UNIQUE,
                enabled     INTEGER NOT NULL DEFAULT 0,
                description TEXT,
                owner_id    TEXT REFERENCES admin_users(id),
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );

            -- Migration 9: approval requests
            CREATE TABLE IF NOT EXISTS approval_requests (
                id           TEXT PRIMARY KEY,
                action_type  TEXT NOT NULL,
                payload      TEXT,
                status       TEXT NOT NULL DEFAULT 'pending',
                requested_by TEXT REFERENCES admin_users(id),
                reviewed_by  TEXT REFERENCES admin_users(id),
                review_note  TEXT,
                created_at   TEXT NOT NULL,
                updated_at   TEXT NOT NULL
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

    pub fn set_user_active(&self, user_id: &str, active: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_users SET is_active = ?1, updated_at = ?2 WHERE id = ?3",
            params![active as i64, now, user_id],
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

    pub fn revoke_all_sessions_for_user(&self, user_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_sessions SET revoked_at = ?1 WHERE admin_user_id = ?2 AND revoked_at IS NULL",
            params![now, user_id],
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

    pub fn list_users(&self, limit: i64, offset: i64, search: Option<&str>, status: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut where_parts: Vec<String> = vec![];
        if let Some(s) = search.filter(|s| !s.is_empty()) {
            let s = s.replace('\'', "''");
            where_parts.push(format!("(email LIKE '%{s}%' OR name LIKE '%{s}%')"));
        }
        if let Some(st) = status.filter(|s| !s.is_empty()) {
            let active = if st == "active" { "1" } else { "0" };
            where_parts.push(format!("is_active = {active}"));
        }
        let where_str = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };
        let sql = format!("SELECT id, email, role, name, is_active, created_at, last_login_at FROM admin_users {where_str} ORDER BY created_at DESC LIMIT {limit} OFFSET {offset}");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, i64>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(
                |(id, email, role, name, is_active, created_at, last_login_at)| {
                    serde_json::json!({
                        "id": id,
                        "email": email,
                        "role": role,
                        "name": name,
                        "is_active": is_active != 0,
                        "created_at": created_at,
                        "last_login_at": last_login_at,
                    })
                },
            )
            .collect())
    }

    // ── Org operations ───────────────────────────────────────────────────────

    pub fn list_orgs(&self, limit: i64, offset: i64, search: Option<&str>, plan: Option<&str>, status: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        // organizations table added in migration 2 — guard against missing table
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='organizations'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(vec![]);
        }
        let mut where_parts: Vec<String> = vec![];
        if let Some(s) = search.filter(|s| !s.is_empty()) {
            let s = s.replace('\'', "''");
            where_parts.push(format!("(name LIKE '%{s}%' OR slug LIKE '%{s}%' OR owner_email LIKE '%{s}%')"));
        }
        if let Some(p) = plan.filter(|s| !s.is_empty()) {
            let p = p.replace('\'', "''");
            where_parts.push(format!("plan = '{p}'"));
        }
        if let Some(st) = status.filter(|s| !s.is_empty()) {
            let st = st.replace('\'', "''");
            where_parts.push(format!("status = '{st}'"));
        }
        let where_str = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };
        let sql = format!("SELECT id, name, slug, status, plan, owner_email, created_at FROM organizations {where_str} ORDER BY created_at DESC LIMIT {limit} OFFSET {offset}");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(|(id, name, slug, status, plan, owner_email, created_at)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "slug": slug,
                    "status": status,
                    "plan": plan,
                    "owner_email": owner_email,
                    "created_at": created_at,
                })
            })
            .collect())
    }

    pub fn get_org(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        let row: Option<(
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            String,
        )> = conn
            .query_row(
                "SELECT id, name, slug, status, plan, owner_email, created_at
                 FROM organizations WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
            .optional()?;
        Ok(
            row.map(|(id, name, slug, status, plan, owner_email, created_at)| {
                serde_json::json!({
                    "id": id,
                    "name": name,
                    "slug": slug,
                    "status": status,
                    "plan": plan,
                    "owner_email": owner_email,
                    "created_at": created_at,
                })
            }),
        )
    }

    pub fn create_org(&self, name: &str, slug: &str, owner_email: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO organizations (id, name, slug, status, plan, owner_email, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'active', 'free', ?4, ?5, ?5)",
            params![id, name, slug, owner_email, now],
        )?;
        Ok(id)
    }

    pub fn update_org_status(&self, id: &str, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE organizations SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )?;
        Ok(())
    }

    pub fn update_org_plan(&self, id: &str, plan: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE organizations SET plan = ?1, updated_at = ?2 WHERE id = ?3",
            params![plan, now, id],
        )?;
        Ok(())
    }

    pub fn count_orgs(&self) -> Result<i64> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='organizations'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(0);
        }
        Ok(conn.query_row("SELECT COUNT(*) FROM organizations", [], |r| r.get(0))?)
    }

    // ── Audit log ────────────────────────────────────────────────────────────

    pub fn log_audit(
        &self,
        admin_user_id: Option<&str>,
        role: Option<&str>,
        target_type: &str,
        target_id: Option<&str>,
        action: &str,
        ip: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='audit_events'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(());
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO audit_events (id, admin_user_id, role, target_type, target_id, action, ip, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, admin_user_id, role, target_type, target_id, action, ip, now],
        )?;
        Ok(())
    }

    // ── Feature flags ────────────────────────────────────────────────────────

    pub fn list_flags(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='feature_flags'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(vec![]);
        }
        let mut stmt = conn.prepare(
            "SELECT id, key, enabled, description, owner_id, created_at, updated_at
             FROM feature_flags ORDER BY key",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, Option<String>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(
                |(id, key, enabled, description, owner_id, created_at, updated_at)| {
                    serde_json::json!({
                        "id": id,
                        "key": key,
                        "enabled": enabled != 0,
                        "description": description,
                        "owner_id": owner_id,
                        "created_at": created_at,
                        "updated_at": updated_at,
                    })
                },
            )
            .collect())
    }

    pub fn create_flag(
        &self,
        key: &str,
        description: Option<&str>,
        owner_id: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO feature_flags (id, key, enabled, description, owner_id, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, ?4, ?5, ?5)",
            params![id, key, description, owner_id, now],
        )?;
        Ok(id)
    }

    pub fn get_flag(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        let row: Option<(
            String,
            String,
            i64,
            Option<String>,
            Option<String>,
            String,
            String,
        )> = conn
            .query_row(
                "SELECT id, key, enabled, description, owner_id, created_at, updated_at
                 FROM feature_flags WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                    ))
                },
            )
            .optional()?;
        Ok(row.map(
            |(id, key, enabled, description, owner_id, created_at, updated_at)| {
                serde_json::json!({
                    "id": id,
                    "key": key,
                    "enabled": enabled != 0,
                    "description": description,
                    "owner_id": owner_id,
                    "created_at": created_at,
                    "updated_at": updated_at,
                })
            },
        ))
    }

    pub fn update_flag_enabled(&self, id: &str, enabled: bool) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE feature_flags SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![enabled as i64, now, id],
        )?;
        Ok(())
    }

    // ── Incidents ────────────────────────────────────────────────────────────

    pub fn list_incidents(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='incidents'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(vec![]);
        }
        let mut stmt = conn.prepare(
            "SELECT id, title, severity, status, owner_id, created_at, resolved_at
             FROM incidents ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(
                |(id, title, severity, status, owner_id, created_at, resolved_at)| {
                    serde_json::json!({
                        "id": id,
                        "title": title,
                        "severity": severity,
                        "status": status,
                        "owner_id": owner_id,
                        "created_at": created_at,
                        "resolved_at": resolved_at,
                    })
                },
            )
            .collect())
    }

    pub fn create_incident(
        &self,
        title: &str,
        severity: &str,
        owner_id: Option<&str>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO incidents (id, title, severity, status, owner_id, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'open', ?4, ?5, ?5)",
            params![id, title, severity, owner_id, now],
        )?;
        Ok(id)
    }

    // ── Support tickets ──────────────────────────────────────────────────────

    pub fn list_tickets(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='support_tickets'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(vec![]);
        }
        let mut stmt = conn.prepare(
            "SELECT id, org_id, subject, status, priority, assignee_id, created_at
             FROM support_tickets ORDER BY created_at DESC",
        )?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, Option<String>>(5)?,
                    r.get::<_, String>(6)?,
                ))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows
            .into_iter()
            .map(
                |(id, org_id, subject, status, priority, assignee_id, created_at)| {
                    serde_json::json!({
                        "id": id,
                        "org_id": org_id,
                        "subject": subject,
                        "status": status,
                        "priority": priority,
                        "assignee_id": assignee_id,
                        "created_at": created_at,
                    })
                },
            )
            .collect())
    }

    /// Returns the current size of the admin DB in kilobytes.
    pub fn db_size_kb(&self) -> Result<u64, String> {
        let conn = self.conn.lock();
        let pages: u64 = conn.query_row("PRAGMA page_count", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        let page_size: u64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        Ok((pages * page_size) / 1024)
    }

    pub fn count_tickets_by_status(&self, status: &str) -> Result<i64> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='support_tickets'",
            [],
            |r| r.get(0),
        )?;
        if exists == 0 {
            return Ok(0);
        }
        Ok(conn.query_row(
            "SELECT COUNT(*) FROM support_tickets WHERE status = ?1",
            params![status],
            |r| r.get(0),
        )?)
    }

    // ── DB query runner ──────────────────────────────────────────────────────

    pub fn run_select_query(&self, sql: &str, limit: usize) -> Result<QueryResult, String> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let columns: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
        let col_count = columns.len();

        let rows: Vec<Vec<serde_json::Value>> = stmt
            .query_map([], |row| {
                let mut vals = Vec::new();
                for i in 0..col_count {
                    let val: serde_json::Value = row.get_ref(i)
                        .map(|v| match v {
                            rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                            rusqlite::types::ValueRef::Integer(n) => serde_json::json!(n),
                            rusqlite::types::ValueRef::Real(f) => serde_json::json!(f),
                            rusqlite::types::ValueRef::Text(s) => {
                                serde_json::Value::String(String::from_utf8_lossy(s).to_string())
                            },
                            rusqlite::types::ValueRef::Blob(b) => {
                                serde_json::Value::String(format!("<blob {} bytes>", b.len()))
                            },
                        })
                        .unwrap_or(serde_json::Value::Null);
                    vals.push(val);
                }
                Ok(vals)
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .take(limit)
            .collect();

        Ok(QueryResult { columns, rows })
    }

    // ── Universal search ─────────────────────────────────────────────────────

    pub fn search_orgs(&self, q: &str, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.conn.lock();
        let pattern = format!("%{}%", q);
        let mut stmt = conn.prepare(
            "SELECT id, name, slug, plan, status FROM organizations
             WHERE lower(name) LIKE ?1 OR lower(slug) LIKE ?1 OR lower(owner_email) LIKE ?1
             LIMIT ?2"
        ).map_err(|e| e.to_string())?;
        let rows: Vec<_> = stmt.query_map(
            rusqlite::params![pattern, limit as i64],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "slug": row.get::<_, String>(2)?,
                    "plan": row.get::<_, String>(3)?,
                    "status": row.get::<_, String>(4)?,
                }))
            }
        ).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn search_incidents(&self, q: &str, limit: usize) -> Result<Vec<serde_json::Value>, String> {
        let conn = self.conn.lock();
        let pattern = format!("%{}%", q);
        let mut stmt = conn.prepare(
            "SELECT id, title, severity, status FROM incidents
             WHERE lower(title) LIKE ?1 LIMIT ?2"
        ).map_err(|e| e.to_string())?;
        let rows: Vec<_> = stmt.query_map(
            rusqlite::params![pattern, limit as i64],
            |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, String>(0)?,
                    "title": row.get::<_, String>(1)?,
                    "severity": row.get::<_, String>(2)?,
                    "status": row.get::<_, String>(3)?,
                }))
            }
        ).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
        Ok(rows)
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

    pub fn list_audit(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT a.id, a.admin_user_id, u.email, u.name, a.role,
                    a.target_type, a.target_id, a.action, a.ip, a.created_at
             FROM audit_events a
             LEFT JOIN admin_users u ON u.id = a.admin_user_id
             ORDER BY a.created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt
            .query_map(params![limit, offset], |row| {
                Ok(serde_json::json!({
                    "id":          row.get::<_, String>(0)?,
                    "user_id":     row.get::<_, Option<String>>(1)?,
                    "user_email":  row.get::<_, Option<String>>(2)?,
                    "user_name":   row.get::<_, Option<String>>(3)?,
                    "role":        row.get::<_, Option<String>>(4)?,
                    "target_type": row.get::<_, String>(5)?,
                    "target_id":   row.get::<_, Option<String>>(6)?,
                    "action":      row.get::<_, String>(7)?,
                    "ip":          row.get::<_, Option<String>>(8)?,
                    "created_at":  row.get::<_, String>(9)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn count_audit(&self) -> Result<i64> {
        Ok(self.conn.lock().query_row(
            "SELECT COUNT(*) FROM audit_events", [], |r| r.get(0),
        )?)
    }

    pub fn list_campaigns(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT c.id, c.name, c.type, c.status, c.created_at, c.updated_at,
                    COUNT(at.id) as touch_count
             FROM campaigns c
             LEFT JOIN attribution_touches at ON at.campaign_id = c.id
             GROUP BY c.id
             ORDER BY c.created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt
            .query_map(params![limit, offset], |row| {
                Ok(serde_json::json!({
                    "id":          row.get::<_, String>(0)?,
                    "name":        row.get::<_, String>(1)?,
                    "type":        row.get::<_, String>(2)?,
                    "status":      row.get::<_, String>(3)?,
                    "created_at":  row.get::<_, String>(4)?,
                    "updated_at":  row.get::<_, String>(5)?,
                    "touch_count": row.get::<_, i64>(6)?,
                }))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(rows)
    }

    pub fn count_campaigns(&self) -> Result<i64> {
        Ok(self.conn.lock().query_row(
            "SELECT COUNT(*) FROM campaigns", [], |r| r.get(0),
        )?)
    }

    pub fn create_campaign(&self, id: &str, name: &str, campaign_type: &str, created_by: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO campaigns (id, name, type, status, created_by, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'draft', ?4, ?5, ?5)",
            params![id, name, campaign_type, created_by, now],
        )?;
        Ok(())
    }

    pub fn update_staff_role(&self, user_id: &str, role: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE admin_users SET role = ?1, updated_at = ?2 WHERE id = ?3",
            params![role, now, user_id],
        )?;
        Ok(())
    }

    pub fn list_org_members(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, email, name, status, created_at FROM customer_users WHERE org_id = ?1 ORDER BY created_at DESC LIMIT 200",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "email": r.get::<_,String>(1)?,
                "name": r.get::<_,Option<String>>(2)?, "status": r.get::<_,String>(3)?,
                "created_at": r.get::<_,String>(4)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    // ── Incident helpers ─────────────────────────────────────────────────────

    pub fn get_incident(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, title, severity, status, owner_id, created_at, resolved_at FROM incidents WHERE id = ?1",
            params![id],
            |r| Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "title": r.get::<_,String>(1)?,
                "severity": r.get::<_,String>(2)?, "status": r.get::<_,String>(3)?,
                "owner_id": r.get::<_,Option<String>>(4)?,
                "created_at": r.get::<_,String>(5)?, "resolved_at": r.get::<_,Option<String>>(6)?,
            })),
        ).optional().map_err(Into::into)
    }

    pub fn get_incident_timeline(&self, incident_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.message, t.created_at, u.name, u.email
             FROM incident_timeline t LEFT JOIN admin_users u ON u.id = t.author_id
             WHERE t.incident_id = ?1 ORDER BY t.created_at ASC",
        )?;
        let rows = stmt.query_map(params![incident_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "message": r.get::<_,String>(1)?,
                "created_at": r.get::<_,String>(2)?,
                "author_name": r.get::<_,Option<String>>(3)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn add_incident_timeline(&self, incident_id: &str, author_id: Option<&str>, message: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO incident_timeline (id, incident_id, author_id, message, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, incident_id, author_id, message, now],
        )?;
        Ok(())
    }

    pub fn update_incident(&self, id: &str, status: Option<&str>, severity: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        if let Some(s) = status {
            self.conn.lock().execute(
                "UPDATE incidents SET status = ?1, updated_at = ?2 WHERE id = ?3",
                params![s, now, id],
            )?;
        }
        if let Some(s) = severity {
            self.conn.lock().execute(
                "UPDATE incidents SET severity = ?1, updated_at = ?2 WHERE id = ?3",
                params![s, now, id],
            )?;
        }
        Ok(())
    }

    pub fn resolve_incident(&self, id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE incidents SET status='resolved', resolved_at=?1, updated_at=?1 WHERE id=?2",
            params![now, id],
        )?;
        Ok(())
    }

    // ── Support helpers ──────────────────────────────────────────────────────

    pub fn get_ticket(&self, id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, org_id, subject, status, priority, assignee_id, created_at FROM support_tickets WHERE id = ?1",
            params![id],
            |r| Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,Option<String>>(1)?,
                "subject": r.get::<_,String>(2)?, "status": r.get::<_,String>(3)?,
                "priority": r.get::<_,String>(4)?, "assignee_id": r.get::<_,Option<String>>(5)?,
                "created_at": r.get::<_,String>(6)?,
            })),
        ).optional().map_err(Into::into)
    }

    pub fn list_tickets_for_org(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, org_id, subject, status, priority, assignee_id, created_at
             FROM support_tickets WHERE org_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,Option<String>>(1)?,
                "subject": r.get::<_,String>(2)?, "status": r.get::<_,String>(3)?,
                "priority": r.get::<_,String>(4)?, "assignee_id": r.get::<_,Option<String>>(5)?,
                "created_at": r.get::<_,String>(6)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_ticket_notes(&self, ticket_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.body, n.created_at, u.name
             FROM support_notes n LEFT JOIN admin_users u ON u.id = n.author_id
             WHERE n.ticket_id = ?1 ORDER BY n.created_at ASC",
        )?;
        let rows = stmt.query_map(params![ticket_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "body": r.get::<_,String>(1)?,
                "created_at": r.get::<_,String>(2)?, "author_name": r.get::<_,Option<String>>(3)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn add_ticket_note(&self, ticket_id: &str, author_id: Option<&str>, body: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO support_notes (id, ticket_id, author_id, body, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, ticket_id, author_id, body, now],
        )?;
        Ok(())
    }

    pub fn update_ticket_status(&self, id: &str, status: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE support_tickets SET status=?1, updated_at=?2 WHERE id=?3",
            params![status, now, id],
        )?;
        Ok(())
    }

    pub fn update_ticket_assignee(&self, id: &str, assignee_id: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE support_tickets SET assignee_id=?1, updated_at=?2 WHERE id=?3",
            params![assignee_id, now, id],
        )?;
        Ok(())
    }

    pub fn list_support_macros(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare("SELECT id, name, body, created_at FROM support_macros ORDER BY name")?;
        let rows = stmt.query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "name": r.get::<_,String>(1)?,
                "body": r.get::<_,String>(2)?, "created_at": r.get::<_,String>(3)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn create_support_macro(&self, name: &str, body: &str, created_by: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO support_macros (id, name, body, created_by, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?5)",
            params![id, name, body, created_by, now],
        )?;
        Ok(id)
    }

    // ── Billing helpers ──────────────────────────────────────────────────────

    pub fn list_subscriptions(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT s.id, s.org_id, o.name, s.plan, s.status,
                    s.current_period_start, s.current_period_end, s.created_at
             FROM subscriptions s LEFT JOIN organizations o ON o.id = s.org_id
             ORDER BY s.created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit, offset], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,String>(1)?,
                "org_name": r.get::<_,Option<String>>(2)?, "plan": r.get::<_,String>(3)?,
                "status": r.get::<_,String>(4)?,
                "current_period_start": r.get::<_,Option<String>>(5)?,
                "current_period_end": r.get::<_,Option<String>>(6)?,
                "created_at": r.get::<_,String>(7)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_subscription_for_org(&self, org_id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, plan, status, current_period_start, current_period_end, created_at
             FROM subscriptions WHERE org_id = ?1 ORDER BY created_at DESC LIMIT 1",
            params![org_id],
            |r| Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "plan": r.get::<_,String>(1)?,
                "status": r.get::<_,String>(2)?,
                "current_period_start": r.get::<_,Option<String>>(3)?,
                "current_period_end": r.get::<_,Option<String>>(4)?,
                "created_at": r.get::<_,String>(5)?,
            })),
        ).optional().map_err(Into::into)
    }

    pub fn list_invoices_for_org(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, amount, currency, status, due_date, paid_at, created_at
             FROM invoices WHERE org_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "amount": r.get::<_,i64>(1)?,
                "currency": r.get::<_,String>(2)?, "status": r.get::<_,String>(3)?,
                "due_date": r.get::<_,Option<String>>(4)?, "paid_at": r.get::<_,Option<String>>(5)?,
                "created_at": r.get::<_,String>(6)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn list_credits_for_org(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, amount, reason, granted_by, created_at FROM credits_ledger WHERE org_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "amount": r.get::<_,i64>(1)?,
                "reason": r.get::<_,Option<String>>(2)?, "granted_by": r.get::<_,Option<String>>(3)?,
                "created_at": r.get::<_,String>(4)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn add_credit(&self, org_id: &str, amount_cents: i64, reason: Option<&str>, granted_by: Option<&str>) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO credits_ledger (id, org_id, amount, reason, granted_by, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, org_id, amount_cents, reason, granted_by, now],
        )?;
        Ok(())
    }

    pub fn list_payment_events(&self, limit: usize) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.org_id, o.name, p.event_type, p.payload, p.created_at
             FROM payment_events p LEFT JOIN organizations o ON o.id = p.org_id
             ORDER BY p.created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |r| {
            let payload_str: Option<String> = r.get(4)?;
            let payload = payload_str
                .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                .unwrap_or(serde_json::Value::Object(Default::default()));
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,Option<String>>(1)?,
                "org_name": r.get::<_,Option<String>>(2)?, "event_type": r.get::<_,String>(3)?,
                "payload": payload, "status": "processed", "created_at": r.get::<_,String>(5)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    // ── CRM helpers ──────────────────────────────────────────────────────────

    pub fn list_crm_accounts(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='crm_accounts'", [], |r| r.get(0),
        )?;
        if exists == 0 { return Ok(vec![]); }
        let mut stmt = conn.prepare(
            "SELECT c.id, c.org_id, o.name, c.health, c.mrr, c.nps, c.created_at
             FROM crm_accounts c LEFT JOIN organizations o ON o.id = c.org_id
             ORDER BY c.mrr DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt.query_map(params![limit, offset], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,String>(1)?,
                "org_name": r.get::<_,Option<String>>(2)?, "health": r.get::<_,String>(3)?,
                "mrr": r.get::<_,i64>(4)?, "nps": r.get::<_,Option<i64>>(5)?,
                "created_at": r.get::<_,String>(6)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_crm_account(&self, org_id: &str) -> Result<Option<serde_json::Value>> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, org_id, health, mrr, nps, created_at FROM crm_accounts WHERE org_id = ?1",
            params![org_id],
            |r| Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,String>(1)?,
                "health": r.get::<_,String>(2)?, "mrr": r.get::<_,i64>(3)?,
                "nps": r.get::<_,Option<i64>>(4)?, "created_at": r.get::<_,String>(5)?,
            })),
        ).optional().map_err(Into::into)
    }

    pub fn upsert_crm_account(&self, org_id: &str, health: Option<&str>, csm_id: Option<&str>, mrr: Option<i64>, nps: Option<i64>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let id = Uuid::new_v4().to_string();
        self.conn.lock().execute(
            "INSERT INTO crm_accounts (id, org_id, health, csm_id, mrr, nps, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?7)
             ON CONFLICT(org_id) DO UPDATE SET
               health=COALESCE(?3,health), csm_id=COALESCE(?4,csm_id),
               mrr=COALESCE(?5,mrr), nps=COALESCE(?6,nps), updated_at=?7",
            params![id, org_id, health.unwrap_or("healthy"), csm_id, mrr.unwrap_or(0), nps, now],
        )?;
        Ok(())
    }

    pub fn add_crm_note(&self, org_id: &str, author_id: Option<&str>, body: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO crm_notes (id, org_id, author_id, body, created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, org_id, author_id, body, now],
        )?;
        Ok(())
    }

    pub fn list_crm_notes(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT n.id, n.body, n.created_at, u.name
             FROM crm_notes n LEFT JOIN admin_users u ON u.id = n.author_id
             WHERE n.org_id = ?1 ORDER BY n.created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "body": r.get::<_,String>(1)?,
                "created_at": r.get::<_,String>(2)?, "author_name": r.get::<_,Option<String>>(3)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    // ── Approval helpers ─────────────────────────────────────────────────────

    pub fn list_approvals(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let exists: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='approval_requests'", [], |r| r.get(0),
        )?;
        if exists == 0 { return Ok(vec![]); }
        let mut stmt = conn.prepare(
            "SELECT a.id, a.action_type, a.payload, a.status, a.created_at,
                    u.email, r.email, a.review_note
             FROM approval_requests a
             LEFT JOIN admin_users u ON u.id = a.requested_by
             LEFT JOIN admin_users r ON r.id = a.reviewed_by
             ORDER BY a.created_at DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "action_type": r.get::<_,String>(1)?,
                "payload": r.get::<_,Option<String>>(2)?, "status": r.get::<_,String>(3)?,
                "created_at": r.get::<_,String>(4)?,
                "requested_by_email": r.get::<_,Option<String>>(5)?,
                "reviewed_by_email": r.get::<_,Option<String>>(6)?,
                "review_note": r.get::<_,Option<String>>(7)?,
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn create_approval(&self, action_type: &str, payload: Option<&str>, requested_by: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO approval_requests (id, action_type, payload, status, requested_by, created_at, updated_at)
             VALUES (?1,?2,?3,'pending',?4,?5,?5)",
            params![id, action_type, payload, requested_by, now],
        )?;
        Ok(id)
    }

    pub fn update_approval_status(&self, id: &str, status: &str, reviewed_by: Option<&str>, note: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE approval_requests SET status=?1, reviewed_by=?2, review_note=?3, updated_at=?4 WHERE id=?5",
            params![status, reviewed_by, note, now, id],
        )?;
        Ok(())
    }

    // ── API key helpers ──────────────────────────────────────────────────────

    fn ensure_api_keys_table(&self) -> Result<()> {
        self.conn.lock().execute_batch(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id         TEXT PRIMARY KEY,
                org_id     TEXT REFERENCES organizations(id),
                name       TEXT NOT NULL,
                key_prefix TEXT NOT NULL,
                key_hash   TEXT NOT NULL UNIQUE,
                created_by TEXT REFERENCES admin_users(id),
                revoked_at TEXT,
                revoked_by TEXT REFERENCES admin_users(id),
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_api_keys_org ON api_keys(org_id);",
        )?;
        Ok(())
    }

    pub fn list_api_keys(&self, limit: i64, offset: i64, status: Option<&str>, plan: Option<&str>) -> Result<Vec<serde_json::Value>> {
        self.ensure_api_keys_table()?;
        let conn = self.conn.lock();
        let mut where_parts: Vec<String> = vec![];
        if let Some(st) = status.filter(|s| !s.is_empty()) {
            if st == "active" { where_parts.push("k.revoked_at IS NULL".into()); }
            else if st == "revoked" { where_parts.push("k.revoked_at IS NOT NULL".into()); }
        }
        if let Some(p) = plan.filter(|s| !s.is_empty()) {
            let p = p.replace('\'', "''");
            where_parts.push(format!("o.plan = '{p}'"));
        }
        let where_str = if where_parts.is_empty() { String::new() } else { format!("WHERE {}", where_parts.join(" AND ")) };
        let sql = format!("SELECT k.id, k.org_id, o.name, k.name, k.key_prefix, k.revoked_at, k.created_at FROM api_keys k LEFT JOIN organizations o ON o.id = k.org_id {where_str} ORDER BY k.created_at DESC LIMIT {limit} OFFSET {offset}");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], |r| {
            let revoked: Option<String> = r.get(5)?;
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "org_id": r.get::<_,Option<String>>(1)?,
                "org_name": r.get::<_,Option<String>>(2)?, "name": r.get::<_,String>(3)?,
                "key_prefix": r.get::<_,String>(4)?, "revoked_at": revoked,
                "created_at": r.get::<_,String>(6)?,
                "active": revoked.is_none(),
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_api_key(&self, id: &str) -> Result<Option<serde_json::Value>> {
        self.ensure_api_keys_table()?;
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT id, org_id, name, key_prefix, revoked_at, created_at FROM api_keys WHERE id = ?1",
            params![id],
            |r| {
                let revoked: Option<String> = r.get(4)?;
                Ok(serde_json::json!({
                    "id": r.get::<_,String>(0)?, "org_id": r.get::<_,Option<String>>(1)?,
                    "name": r.get::<_,String>(2)?, "key_prefix": r.get::<_,String>(3)?,
                    "revoked_at": revoked, "created_at": r.get::<_,String>(5)?,
                    "active": revoked.is_none(),
                }))
            },
        ).optional().map_err(Into::into)
    }

    pub fn create_api_key(&self, org_id: Option<&str>, name: &str, created_by: Option<&str>) -> Result<(String, String)> {
        self.ensure_api_keys_table()?;
        let id = Uuid::new_v4().to_string();
        let raw = format!("fxm_{}", Uuid::new_v4().to_string().replace('-', ""));
        let prefix = raw[..12].to_string();
        let key_hash = format!("{:x}", raw.bytes().fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64)));
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "INSERT INTO api_keys (id, org_id, name, key_prefix, key_hash, created_by, created_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![id, org_id, name, prefix, key_hash, created_by, now],
        )?;
        Ok((id, raw))
    }

    pub fn revoke_api_key(&self, id: &str, revoked_by: Option<&str>) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.lock().execute(
            "UPDATE api_keys SET revoked_at=?1, revoked_by=?2 WHERE id=?3",
            params![now, revoked_by, id],
        )?;
        Ok(())
    }

    pub fn list_api_keys_for_org(&self, org_id: &str) -> Result<Vec<serde_json::Value>> {
        self.ensure_api_keys_table()?;
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, name, key_prefix, revoked_at, created_at FROM api_keys WHERE org_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![org_id], |r| {
            let revoked: Option<String> = r.get(3)?;
            Ok(serde_json::json!({
                "id": r.get::<_,String>(0)?, "name": r.get::<_,String>(1)?,
                "key_prefix": r.get::<_,String>(2)?, "revoked_at": revoked,
                "created_at": r.get::<_,String>(4)?, "active": revoked.is_none(),
            }))
        })?.filter_map(|r| r.ok()).collect();
        Ok(rows)
    }

    pub fn get_proxy_stats(&self) -> serde_json::Value {
        // Derive live proxy info from audit_events (proxy resets, purges)
        let conn = self.conn.lock();
        let reset_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_events WHERE action='proxy.reset'", [], |r| r.get(0))
            .unwrap_or(0);
        let last_reset: Option<String> = conn
            .query_row("SELECT created_at FROM audit_events WHERE action='proxy.reset' ORDER BY created_at DESC LIMIT 1", [], |r| r.get(0))
            .ok();
        serde_json::json!({
            "provider": "DataImpulse",
            "endpoint": "gw.dataimpulse.com:823",
            "reset_count": reset_count,
            "last_reset": last_reset,
            "status": "active",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Arc<AdminDb> {
        AdminDb::open_in_memory().expect("open in-memory db")
    }

    #[test]
    fn test_create_and_find_user() {
        let db = test_db();
        let id = Uuid::new_v4().to_string();
        db.create_user(&id, "test@example.com", "hashed_pw", "ops", "Test User")
            .expect("create user");
        let user = db
            .find_user_by_email("test@example.com")
            .expect("find user")
            .expect("user should exist");
        assert_eq!(user.id, id);
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
    }

    #[test]
    fn test_has_any_users_empty() {
        let db = test_db();
        assert!(!db.has_any_users().expect("should not error"), "fresh db has no users");
    }

    #[test]
    fn test_has_any_users_after_create() {
        let db = test_db();
        let id = Uuid::new_v4().to_string();
        db.create_user(&id, "admin@test.com", "pw", "owner", "Admin").unwrap();
        assert!(db.has_any_users().expect("should not error"), "should have users now");
    }

    #[test]
    fn test_session_create_and_validate() {
        let db = test_db();
        let user_id = Uuid::new_v4().to_string();
        db.create_user(&user_id, "alice@test.com", "pw", "ops", "Alice").unwrap();
        let token_hash = "abc123hash";
        let session_id = Uuid::new_v4().to_string();
        db.create_session(&session_id, &user_id, token_hash, "127.0.0.1", "TestAgent/1.0")
            .expect("create session");
        let result = db.validate_session(token_hash).expect("validate session");
        assert!(result.is_some(), "session should be valid");
        let (user, sid) = result.unwrap();
        assert_eq!(user.email, "alice@test.com");
        assert_eq!(sid, session_id);
    }

    #[test]
    fn test_revoke_session() {
        let db = test_db();
        let user_id = Uuid::new_v4().to_string();
        db.create_user(&user_id, "bob@test.com", "pw", "support", "Bob").unwrap();
        let token_hash = "revoketest123";
        let session_id = Uuid::new_v4().to_string();
        db.create_session(&session_id, &user_id, token_hash, "10.0.0.1", "UA/1").unwrap();
        db.revoke_session(&session_id).expect("revoke");
        let result = db.validate_session(token_hash).expect("validate after revoke");
        assert!(result.is_none(), "revoked session should be invalid");
    }

    #[test]
    fn test_totp_replay_prevention() {
        let db = test_db();
        let user_id = Uuid::new_v4().to_string();
        db.create_user(&user_id, "carol@test.com", "pw", "owner", "Carol").unwrap();
        let code = "123456";
        assert!(!db.is_totp_code_used(&user_id, code).unwrap(), "code not yet used");
        db.mark_totp_code_used(&user_id, code).unwrap();
        assert!(db.is_totp_code_used(&user_id, code).unwrap(), "code should be used now");
    }
}
