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
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub key_id: String,
    pub plan: String,
    pub requests_this_month: u32,
    pub requests_today: u32,
    pub tokens_this_month: u64,
    pub monthly_limit: Option<u32>,
    pub quota_remaining: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewRecentRequest {
    pub endpoint: String,
    pub status: u16,
    pub duration_ms: u64,
    pub tokens_used: u64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewTimeseriesPoint {
    pub date: String,
    pub requests: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewEndpointStat {
    pub endpoint: String,
    pub requests: u32,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewSummary {
    pub key_id: String,
    pub plan: String,
    pub requests_today: u32,
    pub requests_this_month: u32,
    pub tokens_this_month: u64,
    pub monthly_limit: Option<u32>,
    pub quota_remaining: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avg_latency_ms_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_rate_7d: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub summary: OverviewSummary,
    pub timeseries: Vec<OverviewTimeseriesPoint>,
    pub top_endpoints: Vec<OverviewEndpointStat>,
    pub recent_requests: Vec<OverviewRecentRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickstartStatus {
    pub session: QuickstartSession,
    pub connectivity: QuickstartConnectivity,
    pub first_success: QuickstartFirstSuccess,
    pub recent_activity: QuickstartRecentActivity,
    pub recommended_next_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickstartSession {
    pub plan: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickstartConnectivity {
    pub api_reachable: bool,
    pub usage_check_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickstartFirstSuccess {
    pub has_successful_request: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_success_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_success_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickstartRecentActivity {
    pub request_count_7d: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_request_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_request_status: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSettings {
    pub workspace_name: String,
    pub email_updates: bool,
    pub incident_alerts: bool,
    pub changelog_notifications: bool,
    pub default_search_tier: String,
    pub default_max_sources: u32,
    pub theme: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSettingsResponse {
    pub workspace: DashboardSettingsWorkspace,
    pub session: DashboardSettingsSession,
    pub preferences: CustomerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSettingsWorkspace {
    pub name: String,
    pub plan: String,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSettingsSession {
    pub key_id: String,
    pub api_key_preview: String,
    pub api_base: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTimeseriesPoint {
    pub date: String,
    pub requests: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointUsageBreakdown {
    pub endpoint: String,
    pub requests: u32,
    pub tokens_used: u64,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHealthSummary {
    pub success_rate: f64,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageAnalyticsSummary {
    pub key_id: String,
    pub plan: String,
    pub requests_today: u32,
    pub requests_this_month: u32,
    pub tokens_this_month: u64,
    pub monthly_limit: Option<u32>,
    pub quota_remaining: Option<u32>,
    pub requests_per_minute_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardUsageResponse {
    pub summary: UsageAnalyticsSummary,
    pub timeseries: Vec<UsageTimeseriesPoint>,
    pub endpoint_breakdown: Vec<EndpointUsageBreakdown>,
    pub health: UsageHealthSummary,
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

            CREATE TABLE IF NOT EXISTS customer_settings (
                key_id          TEXT PRIMARY KEY REFERENCES api_keys(id),
                workspace_name  TEXT NOT NULL DEFAULT '',
                email_updates   INTEGER NOT NULL DEFAULT 1,
                incident_alerts INTEGER NOT NULL DEFAULT 0,
                changelog_notifications INTEGER NOT NULL DEFAULT 1,
                default_search_tier TEXT NOT NULL DEFAULT 'summary',
                default_max_sources INTEGER NOT NULL DEFAULT 5,
                theme           TEXT NOT NULL DEFAULT 'system',
                updated_at      TEXT NOT NULL DEFAULT ''
            );
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

    pub fn get_recent_requests(
        &self,
        key_id: &str,
        limit: usize,
    ) -> Result<Vec<OverviewRecentRequest>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT endpoint, status, duration_ms, tokens_used, created_at
             FROM usage_logs
             WHERE key_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
        )?;

        let rows = stmt
            .query_map(params![key_id, limit as i64], |row| {
                Ok(OverviewRecentRequest {
                    endpoint: row.get(0)?,
                    status: row.get(1)?,
                    duration_ms: row.get::<_, i64>(2)? as u64,
                    tokens_used: row.get::<_, i64>(3)? as u64,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn get_overview_timeseries(
        &self,
        key_id: &str,
        days: usize,
    ) -> Result<Vec<OverviewTimeseriesPoint>> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let mut stmt = conn.prepare(
            "SELECT substr(created_at, 1, 10) AS day, COUNT(*) AS requests
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)
             GROUP BY day
             ORDER BY day ASC",
        )?;

        let rows = stmt
            .query_map(params![key_id, since_days], |row| {
                Ok(OverviewTimeseriesPoint {
                    date: row.get(0)?,
                    requests: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn get_top_endpoints(
        &self,
        key_id: &str,
        days: usize,
        limit: usize,
    ) -> Result<Vec<OverviewEndpointStat>> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let mut stmt = conn.prepare(
            "SELECT endpoint, COUNT(*) AS requests, MAX(created_at) AS last_seen_at
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)
             GROUP BY endpoint
             ORDER BY requests DESC, last_seen_at DESC
             LIMIT ?3",
        )?;

        let rows = stmt
            .query_map(params![key_id, since_days, limit as i64], |row| {
                Ok(OverviewEndpointStat {
                    endpoint: row.get(0)?,
                    requests: row.get(1)?,
                    last_seen_at: row.get(2)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(rows)
    }

    pub fn get_latency_success_summary(
        &self,
        key_id: &str,
        days: usize,
    ) -> Result<(Option<u64>, Option<f64>)> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let (count, success_count, avg_latency_ms): (u32, u32, Option<f64>) = conn.query_row(
            "SELECT
                COUNT(*) AS total_requests,
                COALESCE(SUM(CASE WHEN status >= 200 AND status < 300 THEN 1 ELSE 0 END), 0) AS success_count,
                AVG(duration_ms) AS avg_latency_ms
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)",
            params![key_id, since_days],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;

        if count == 0 {
            return Ok((None, None));
        }

        Ok((
            avg_latency_ms.map(|value| value.round() as u64),
            Some(((success_count as f64) / (count as f64) * 1000.0).round() / 10.0),
        ))
    }

    pub fn get_dashboard_overview(&self, key_id: &str, plan: &str) -> Result<DashboardOverview> {
        let usage = self.get_usage_stats(key_id, plan)?;
        let (avg_latency_ms_7d, success_rate_7d) = self.get_latency_success_summary(key_id, 7)?;

        Ok(DashboardOverview {
            summary: OverviewSummary {
                key_id: usage.key_id,
                plan: usage.plan,
                requests_today: usage.requests_today,
                requests_this_month: usage.requests_this_month,
                tokens_this_month: usage.tokens_this_month,
                monthly_limit: usage.monthly_limit,
                quota_remaining: usage.quota_remaining,
                avg_latency_ms_7d,
                success_rate_7d,
            },
            timeseries: self.get_overview_timeseries(key_id, 14)?,
            top_endpoints: self.get_top_endpoints(key_id, 30, 5)?,
            recent_requests: self.get_recent_requests(key_id, 20)?,
        })
    }

    pub fn get_quickstart_status(&self, key_id: &str, plan: &str) -> Result<QuickstartStatus> {
        let conn = self.conn.lock();

        // First successful request (status 2xx)
        let first_success: Option<(String, String)> = conn
            .query_row(
                "SELECT created_at, endpoint FROM usage_logs
                 WHERE key_id = ?1 AND status >= 200 AND status < 300
                 ORDER BY created_at ASC LIMIT 1",
                params![key_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        // Recent activity (last 7 days)
        let request_count_7d: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_logs
                 WHERE key_id = ?1 AND created_at >= datetime('now', '-7 days')",
                params![key_id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let last_request: Option<(String, u16)> = conn
            .query_row(
                "SELECT created_at, status FROM usage_logs
                 WHERE key_id = ?1
                 ORDER BY created_at DESC LIMIT 1",
                params![key_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok();

        // Determine recommended next steps based on state
        let mut steps = Vec::new();
        let has_success = first_success.is_some();
        let has_recent_failures = if let Some((_, status)) = &last_request {
            *status >= 400
        } else {
            false
        };

        if has_recent_failures {
            steps.push("check_api_health".to_string());
            steps.push("usage_check".to_string());
        }

        if !has_success {
            steps.push("send_first_request".to_string());
        } else {
            // Check which endpoints they've used
            let used_endpoints: Vec<String> = {
                let mut stmt = conn.prepare(
                    "SELECT DISTINCT endpoint FROM usage_logs
                     WHERE key_id = ?1 AND status >= 200 AND status < 300",
                )?;
                let results: Vec<String> = stmt
                    .query_map(params![key_id], |row| row.get(0))?
                    .filter_map(|r| r.ok())
                    .collect();
                results
            };

            let tried_search = used_endpoints.iter().any(|e| e == "/v1/search");
            let tried_scrape = used_endpoints
                .iter()
                .any(|e| e == "/v1/scrape" || e == "/v1/fetch");
            let tried_research = used_endpoints.iter().any(|e| e == "/v1/research");

            if !tried_search {
                steps.push("playground_search".to_string());
            }
            if tried_search && !tried_scrape {
                steps.push("try_fetch".to_string());
            }
            if tried_search && !tried_research {
                steps.push("try_research".to_string());
            }
            steps.push("usage_check".to_string());
        }

        Ok(QuickstartStatus {
            session: QuickstartSession {
                plan: plan.to_string(),
                key_id: key_id.to_string(),
            },
            connectivity: QuickstartConnectivity {
                api_reachable: true,
                usage_check_ok: true,
            },
            first_success: QuickstartFirstSuccess {
                has_successful_request: has_success,
                first_success_at: first_success.as_ref().map(|(ts, _)| ts.clone()),
                first_success_endpoint: first_success.map(|(_, ep)| ep),
            },
            recent_activity: QuickstartRecentActivity {
                request_count_7d,
                last_request_at: last_request.as_ref().map(|(ts, _)| ts.clone()),
                last_request_status: last_request.map(|(_, s)| s),
            },
            recommended_next_steps: steps,
        })
    }

    pub fn get_customer_settings(&self, key_id: &str) -> Result<CustomerSettings> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();

        // Upsert default row if not exists
        conn.execute(
            "INSERT OR IGNORE INTO customer_settings (key_id, updated_at) VALUES (?1, ?2)",
            params![key_id, now],
        )?;

        let settings = conn
            .query_row(
                "SELECT workspace_name, email_updates, incident_alerts, changelog_notifications,
                    default_search_tier, default_max_sources, theme, updated_at
             FROM customer_settings WHERE key_id = ?1",
                params![key_id],
                |row| {
                    Ok(CustomerSettings {
                        workspace_name: row.get(0)?,
                        email_updates: row.get::<_, i32>(1)? != 0,
                        incident_alerts: row.get::<_, i32>(2)? != 0,
                        changelog_notifications: row.get::<_, i32>(3)? != 0,
                        default_search_tier: row.get(4)?,
                        default_max_sources: row.get::<_, i32>(5)? as u32,
                        theme: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                },
            )
            .context("get customer settings")?;

        Ok(settings)
    }

    pub fn update_customer_settings(
        &self,
        key_id: &str,
        patch: &serde_json::Value,
    ) -> Result<CustomerSettings> {
        let conn = self.conn.lock();
        let now = Utc::now().to_rfc3339();

        // Upsert default row if not exists
        conn.execute(
            "INSERT OR IGNORE INTO customer_settings (key_id, updated_at) VALUES (?1, ?2)",
            params![key_id, now],
        )?;

        // Apply each field from the patch
        if let Some(v) = patch.get("workspace_name").and_then(|v| v.as_str()) {
            conn.execute(
                "UPDATE customer_settings SET workspace_name = ?1, updated_at = ?2 WHERE key_id = ?3",
                params![v, now, key_id],
            )?;
        }
        if let Some(v) = patch.get("email_updates").and_then(|v| v.as_bool()) {
            conn.execute(
                "UPDATE customer_settings SET email_updates = ?1, updated_at = ?2 WHERE key_id = ?3",
                params![v as i32, now, key_id],
            )?;
        }
        if let Some(v) = patch.get("incident_alerts").and_then(|v| v.as_bool()) {
            conn.execute(
                "UPDATE customer_settings SET incident_alerts = ?1, updated_at = ?2 WHERE key_id = ?3",
                params![v as i32, now, key_id],
            )?;
        }
        if let Some(v) = patch
            .get("changelog_notifications")
            .and_then(|v| v.as_bool())
        {
            conn.execute(
                "UPDATE customer_settings SET changelog_notifications = ?1, updated_at = ?2 WHERE key_id = ?3",
                params![v as i32, now, key_id],
            )?;
        }
        if let Some(v) = patch.get("default_search_tier").and_then(|v| v.as_str()) {
            let valid = ["key_facts", "summary", "detailed", "complete"];
            if valid.contains(&v) {
                conn.execute(
                    "UPDATE customer_settings SET default_search_tier = ?1, updated_at = ?2 WHERE key_id = ?3",
                    params![v, now, key_id],
                )?;
            }
        }
        if let Some(v) = patch.get("default_max_sources").and_then(|v| v.as_u64()) {
            let clamped = v.clamp(1, 20) as i32;
            conn.execute(
                "UPDATE customer_settings SET default_max_sources = ?1, updated_at = ?2 WHERE key_id = ?3",
                params![clamped, now, key_id],
            )?;
        }
        if let Some(v) = patch.get("theme").and_then(|v| v.as_str()) {
            let valid = ["system", "dark", "light"];
            if valid.contains(&v) {
                conn.execute(
                    "UPDATE customer_settings SET theme = ?1, updated_at = ?2 WHERE key_id = ?3",
                    params![v, now, key_id],
                )?;
            }
        }

        // Drop the lock before calling get_customer_settings which re-acquires it
        drop(conn);
        self.get_customer_settings(key_id)
    }

    pub fn get_dashboard_settings(
        &self,
        key_id: &str,
        plan: &str,
    ) -> Result<DashboardSettingsResponse> {
        let conn = self.conn.lock();

        // Get key record for session info
        let (created_at, last_used_at, key_prefix): (String, Option<String>, String) = conn
            .query_row(
                "SELECT created_at, last_used_at, key_prefix FROM api_keys WHERE id = ?1",
                params![key_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap_or_else(|_| (Utc::now().to_rfc3339(), None, String::new()));

        drop(conn);

        let preferences = self.get_customer_settings(key_id)?;

        Ok(DashboardSettingsResponse {
            workspace: DashboardSettingsWorkspace {
                name: if preferences.workspace_name.is_empty() {
                    format!("{plan} workspace")
                } else {
                    preferences.workspace_name.clone()
                },
                plan: plan.to_string(),
                key_id: key_id.to_string(),
            },
            session: DashboardSettingsSession {
                key_id: key_id.to_string(),
                api_key_preview: if key_prefix.len() >= 12 {
                    format!(
                        "{}...{}",
                        &key_prefix[..12],
                        &key_prefix[key_prefix.len().saturating_sub(4)..]
                    )
                } else {
                    key_prefix
                },
                api_base: "***REMOVED***".to_string(),
                created_at,
                last_used_at,
            },
            preferences,
        })
    }

    pub fn get_usage_timeseries_with_tokens(
        &self,
        key_id: &str,
        days: usize,
    ) -> Result<Vec<UsageTimeseriesPoint>> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let mut stmt = conn.prepare(
            "SELECT substr(created_at, 1, 10) AS day,
                    COUNT(*) AS requests,
                    COALESCE(SUM(tokens_used), 0) AS tokens
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)
             GROUP BY day
             ORDER BY day ASC",
        )?;
        let rows = stmt
            .query_map(params![key_id, since_days], |row| {
                Ok(UsageTimeseriesPoint {
                    date: row.get(0)?,
                    requests: row.get(1)?,
                    tokens: Some(row.get::<_, i64>(2)? as u64),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_endpoint_usage_breakdown(
        &self,
        key_id: &str,
        days: usize,
        limit: usize,
    ) -> Result<Vec<EndpointUsageBreakdown>> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let mut stmt = conn.prepare(
            "SELECT endpoint,
                    COUNT(*) AS requests,
                    COALESCE(SUM(tokens_used), 0) AS tokens_used,
                    COALESCE(SUM(CASE WHEN status >= 400 THEN 1 ELSE 0 END), 0) AS error_count
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)
             GROUP BY endpoint
             ORDER BY requests DESC
             LIMIT ?3",
        )?;
        let rows = stmt
            .query_map(params![key_id, since_days, limit as i64], |row| {
                Ok(EndpointUsageBreakdown {
                    endpoint: row.get(0)?,
                    requests: row.get(1)?,
                    tokens_used: row.get::<_, i64>(2)? as u64,
                    error_count: row.get(3)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn get_usage_health_summary(
        &self,
        key_id: &str,
        days: usize,
    ) -> Result<UsageHealthSummary> {
        let conn = self.conn.lock();
        let since_days = format!("-{} days", days.saturating_sub(1));
        let (total, success_count): (u32, u32) = conn.query_row(
            "SELECT COUNT(*),
                    COALESCE(SUM(CASE WHEN status >= 200 AND status < 300 THEN 1 ELSE 0 END), 0)
             FROM usage_logs
             WHERE key_id = ?1 AND created_at >= datetime('now', ?2)",
            params![key_id, since_days],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let error_count = total.saturating_sub(success_count);
        let success_rate = if total > 0 {
            ((success_count as f64 / total as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        Ok(UsageHealthSummary {
            success_rate,
            error_count,
        })
    }

    pub fn get_dashboard_usage(&self, key_id: &str, plan: &str) -> Result<DashboardUsageResponse> {
        let stats = self.get_usage_stats(key_id, plan)?;
        let limits = PlanLimits::for_plan(plan);
        let timeseries = self.get_usage_timeseries_with_tokens(key_id, 30)?;
        let endpoint_breakdown = self.get_endpoint_usage_breakdown(key_id, 30, 10)?;
        let health = self.get_usage_health_summary(key_id, 30)?;

        Ok(DashboardUsageResponse {
            summary: UsageAnalyticsSummary {
                key_id: stats.key_id,
                plan: stats.plan,
                requests_today: stats.requests_today,
                requests_this_month: stats.requests_this_month,
                tokens_this_month: stats.tokens_this_month,
                monthly_limit: stats.monthly_limit,
                quota_remaining: stats.quota_remaining,
                requests_per_minute_limit: limits.requests_per_min,
            },
            timeseries,
            endpoint_breakdown,
            health,
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

    #[test]
    fn dashboard_overview_aggregates_usage_logs() {
        let db = test_db();
        let (_, record) = db.create_key("overview test", "pro").unwrap();
        db.record_usage(&record.id, "/v1/search", 200, 1200, 100)
            .unwrap();
        db.record_usage(&record.id, "/v1/search", 500, 0, 300)
            .unwrap();
        db.record_usage(&record.id, "/v1/fetch", 200, 800, 200)
            .unwrap();

        let overview = db.get_dashboard_overview(&record.id, "pro").unwrap();

        assert_eq!(overview.summary.requests_this_month, 3);
        assert_eq!(overview.summary.requests_today, 3);
        assert_eq!(overview.summary.tokens_this_month, 2000);
        assert_eq!(overview.recent_requests.len(), 3);
        assert_eq!(overview.top_endpoints.len(), 2);
        assert_eq!(overview.top_endpoints[0].endpoint, "/v1/search");
        assert_eq!(overview.top_endpoints[0].requests, 2);
        assert_eq!(overview.summary.avg_latency_ms_7d, Some(200));
        assert_eq!(overview.summary.success_rate_7d, Some(66.7));
    }
}
