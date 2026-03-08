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

    pub fn list_users(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, email, role, name, is_active, created_at, last_login_at
             FROM admin_users ORDER BY created_at DESC",
        )?;
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

    pub fn list_orgs(&self, limit: i64, offset: i64) -> Result<Vec<serde_json::Value>> {
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
        let mut stmt = conn.prepare(
            "SELECT id, name, slug, status, plan, owner_email, created_at
             FROM organizations ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;
        let rows = stmt
            .query_map([limit, offset], |r| {
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
