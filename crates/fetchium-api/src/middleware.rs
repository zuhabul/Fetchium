//! REST API middleware — auth extraction, rate limiting, request logging.

use crate::auth::{ApiKeyRecord, AuthDb, PlanLimits};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use fetchium_core::cache::MemoryCache;
use fetchium_core::config::HsxConfig;
use fetchium_core::http::client::HttpClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// Shared application state injected into all axum handlers.
/// AppState is Clone (cheap — all inner fields are Arc-wrapped).
#[derive(Clone)]
pub struct AppState {
    pub config: HsxConfig,
    pub http: HttpClient,
    pub cache: MemoryCache,
    pub auth_db: Arc<AuthDb>,
    pub rate_limiter: Arc<PerKeyRateLimiter>,
}

impl AppState {
    pub fn new(config: HsxConfig, auth_db: Arc<AuthDb>) -> anyhow::Result<Self> {
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
                "type": format!("https://docs.hypersearchx.com/errors/{error_type}"),
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

        // Validate against DB (blocking operation — short, so acceptable inline)
        let record = tokio::task::block_in_place(|| app_state.auth_db.validate_key(raw_key))
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
        let within_quota =
            tokio::task::block_in_place(|| app_state.auth_db.check_quota(&record.id, &record.plan));
        if !within_quota {
            return Err(AuthError::QuotaExceeded);
        }

        Ok(AuthenticatedKey(record))
    }
}
