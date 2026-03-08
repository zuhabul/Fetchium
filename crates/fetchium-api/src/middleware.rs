//! REST API middleware — auth extraction, rate limiting, request logging.

use crate::auth::{ApiKeyRecord, AuthDb, PlanLimits};
use crate::types::{JobState, JobStatusResponse};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use fetchium_core::cache::MemoryCache;
use fetchium_core::config::FetchiumConfig;
use fetchium_core::http::client::HttpClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Shared application state injected into all axum handlers.
/// AppState is Clone (cheap — all inner fields are Arc-wrapped).
#[derive(Clone)]
pub struct AppState {
    pub config: FetchiumConfig,
    pub http: HttpClient,
    pub cache: MemoryCache,
    pub auth_db: Arc<AuthDb>,
    pub rate_limiter: Arc<PerKeyRateLimiter>,
    pub jobs: Arc<JobStore>,
    /// Global concurrency limiter for search operations.
    /// Prevents backend overload when multiple requests arrive simultaneously.
    pub search_semaphore: Arc<tokio::sync::Semaphore>,
}

impl AppState {
    pub fn new(config: FetchiumConfig, auth_db: Arc<AuthDb>) -> anyhow::Result<Self> {
        let http = HttpClient::new(&config)?;
        let cache = MemoryCache::new(
            config.cache.memory_max_entries,
            config.cache.ttl_secs,
            config.cache.enabled,
        );
        Ok(Self {
            config,
            http,
            cache,
            auth_db,
            rate_limiter: Arc::new(PerKeyRateLimiter::new()),
            jobs: Arc::new(JobStore::new()),
            // Serialize search operations: each search dispatches 7-10 backend
            // requests in parallel through a shared proxy pool. Concurrent searches
            // cause connection pool starvation since client_for_domain creates new
            // reqwest clients per call. With serialization, each search completes in
            // ~3-5s, so 5 queued searches = 15-25s — well within the 25s timeout.
            // TODO: Cache proxied reqwest clients to enable true concurrency.
            search_semaphore: Arc::new(tokio::sync::Semaphore::new(1)),
        })
    }
}

#[derive(Debug, Clone)]
struct StoredJob {
    owner_key_id: String,
    payload: JobStatusResponse,
}

pub struct JobStore {
    jobs: Mutex<HashMap<String, StoredJob>>,
}

impl Default for JobStore {
    fn default() -> Self {
        Self::new()
    }
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: Mutex::new(HashMap::new()),
        }
    }

    pub fn create(&self, owner_key_id: String, job_id: String, job_type: String) {
        let payload = JobStatusResponse {
            meta: crate::types::ResponseMeta {
                request_id: job_id.clone(),
                status: "queued".into(),
                endpoint: "/v1/jobs/:id".into(),
                duration_ms: 0,
                query: None,
                tier: None,
                tokens_used: None,
                sources_count: None,
                result_id: Some(job_id.clone()),
            },
            job_id: job_id.clone(),
            job_type,
            status: JobState::Queued,
            created_at: chrono::Utc::now().to_rfc3339(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        };
        self.jobs.lock().insert(
            job_id,
            StoredJob {
                owner_key_id,
                payload,
            },
        );
    }

    pub fn mark_running(&self, job_id: &str) {
        if let Some(job) = self.jobs.lock().get_mut(job_id) {
            job.payload.status = JobState::Running;
            job.payload.meta.status = "running".into();
            job.payload.started_at = Some(chrono::Utc::now().to_rfc3339());
        }
    }

    pub fn complete(&self, job_id: &str, result: serde_json::Value) {
        if let Some(job) = self.jobs.lock().get_mut(job_id) {
            job.payload.status = JobState::Completed;
            job.payload.meta.status = "completed".into();
            job.payload.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.payload.result = Some(result);
            job.payload.error = None;
        }
    }

    pub fn fail(&self, job_id: &str, error: String) {
        if let Some(job) = self.jobs.lock().get_mut(job_id) {
            job.payload.status = JobState::Failed;
            job.payload.meta.status = "failed".into();
            job.payload.completed_at = Some(chrono::Utc::now().to_rfc3339());
            job.payload.error = Some(error);
            job.payload.result = None;
        }
    }

    pub fn get_owned(&self, job_id: &str, owner_key_id: &str) -> Option<JobStatusResponse> {
        self.jobs.lock().get(job_id).and_then(|job| {
            if job.owner_key_id == owner_key_id {
                Some(job.payload.clone())
            } else {
                None
            }
        })
    }
}

/// Per-API-key sliding-window rate limiter (in-memory).
///
/// Limits based on plan tier: free=60/min, starter=200/min, pro=500/min, enterprise=2000/min.
pub struct PerKeyRateLimiter {
    /// key_id → (request_count, window_start)
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
}

impl Default for PerKeyRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl PerKeyRateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Returns `Some((remaining, limit, reset_secs))` if allowed, `None` if rate limited.
    pub fn check_with_info(&self, key_id: &str, plan: &str) -> Option<(u32, u32, u64)> {
        let limits = PlanLimits::for_plan(plan);
        let max = limits.requests_per_min;

        let mut buckets = self.buckets.lock();
        let now = Instant::now();
        let entry = buckets.entry(key_id.to_string()).or_insert((0, now));

        let elapsed = now.duration_since(entry.1).as_secs();
        if elapsed >= 60 {
            *entry = (0, now);
        }

        if entry.0 >= max {
            return None;
        }
        entry.0 += 1;
        let remaining = max - entry.0;
        let reset_secs = 60u64.saturating_sub(elapsed);
        Some((remaining, max, reset_secs))
    }

    /// Returns `true` if the key is within its per-minute rate limit.
    pub fn check(&self, key_id: &str, plan: &str) -> bool {
        self.check_with_info(key_id, plan).is_some()
    }
}

// ─── Axum extractor: authenticated API key ────────────────────────────────────

/// Rate limit info injected into request extensions by `AuthenticatedKey`.
#[derive(Clone, Copy)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset: u64,
}

/// Axum extractor that validates `Authorization: Bearer fetchium_xxx` and injects
/// the key record into the handler. Returns 401/429 on failure.
pub struct AuthenticatedKey(pub ApiKeyRecord);

/// Error type for auth failures.
pub enum AuthError {
    MissingToken,
    InvalidToken,
    RateLimited,
    QuotaExceeded,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            AuthError::MissingToken => (
                StatusCode::UNAUTHORIZED,
                "missing_token",
                "Authorization header required: Bearer fetchium_...",
            ),
            AuthError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "invalid_token",
                "Invalid or revoked API key",
            ),
            AuthError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Rate limit exceeded — slow down requests",
            ),
            AuthError::QuotaExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "quota_exceeded",
                "Monthly request quota exceeded — upgrade your plan",
            ),
        };
        let mut response = (
            status,
            Json(serde_json::json!({
                "type": format!("https://docs.fetchium.com/errors/{error_type}"),
                "title": message,
                "status": status.as_u16(),
            })),
        )
            .into_response();
        if status == StatusCode::TOO_MANY_REQUESTS {
            let headers = response.headers_mut();
            headers.insert("Retry-After", "60".parse().unwrap());
        }
        response
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedKey
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, AuthError> {
        let app_state = AppState::from_ref(state);

        // Extract Bearer token
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(AuthError::MissingToken)?;

        let raw_key = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AuthError::MissingToken)?
            .trim();

        if !raw_key.starts_with("fetchium_") || raw_key.len() < 20 {
            return Err(AuthError::InvalidToken);
        }

        // Validate against DB (non-blocking via spawn_blocking thread pool)
        let db = app_state.auth_db.clone();
        let key_str = raw_key.to_string();
        let record = tokio::task::spawn_blocking(move || db.validate_key(&key_str))
            .await
            .map_err(|_| AuthError::InvalidToken)?
            .map_err(|_| AuthError::InvalidToken)?
            .ok_or(AuthError::InvalidToken)?;

        // Per-minute rate limit — check and capture remaining for headers
        let rate_info = app_state
            .rate_limiter
            .check_with_info(&record.id, &record.plan);
        if rate_info.is_none() {
            return Err(AuthError::RateLimited);
        }
        // Store rate limit info in request extensions for handlers to use
        if let Some((remaining, limit, reset)) = rate_info {
            parts.extensions.insert(RateLimitInfo {
                remaining,
                limit,
                reset,
            });
        }

        // Monthly quota
        let db2 = app_state.auth_db.clone();
        let key_id = record.id.clone();
        let plan = record.plan.clone();
        let within_quota = tokio::task::spawn_blocking(move || db2.check_quota(&key_id, &plan))
            .await
            .unwrap_or(false);
        if !within_quota {
            return Err(AuthError::QuotaExceeded);
        }

        Ok(AuthenticatedKey(record))
    }
}
