//! API key management handlers — create, list, revoke, usage stats.
//!
//! Admin endpoints use `X-Admin-Secret` header for the MVP.

use crate::middleware::AppState;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

fn err(status: StatusCode, kind: &str, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({
            "type": format!("https://docs.hypersearchx.com/errors/{kind}"),
            "title": msg,
            "status": status.as_u16(),
        })),
    )
}

/// Validate the admin secret from `X-Admin-Secret`.
///
/// Panics at startup if `HSX_ADMIN_SECRET` is not set in the environment,
/// preventing insecure "changeme" defaults in production.
fn check_admin(headers: &HeaderMap) -> bool {
    let secret = std::env::var("HSX_ADMIN_SECRET")
        .expect("HSX_ADMIN_SECRET must be set (generate with: openssl rand -hex 32)");
    let provided = headers
        .get("X-Admin-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // Constant-time comparison to prevent timing attacks
    constant_time_eq(provided.as_bytes(), secret.as_bytes())
}

/// Constant-time byte slice comparison (prevents timing attacks on secret comparison).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

// ─── POST /v1/keys ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub name: String,
    #[serde(default = "default_plan")]
    pub plan: String,
}

fn default_plan() -> String {
    "free".into()
}

#[derive(Serialize)]
pub struct CreateKeyResponse {
    /// Full raw key — shown ONCE. Store it securely.
    pub key: String,
    pub id: String,
    pub name: String,
    pub plan: String,
    pub created_at: String,
    pub warning: &'static str,
}

/// `POST /v1/keys` — create a new API key.
pub async fn create_key(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(req): Json<CreateKeyRequest>,
) -> ApiResult<CreateKeyResponse> {
    if !check_admin(&headers) {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "X-Admin-Secret required",
        ));
    }

    if req.name.is_empty() || req.name.len() > 100 {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "invalid_name",
            "name must be 1–100 characters",
        ));
    }

    let valid_plans = ["free", "starter", "pro", "enterprise"];
    if !valid_plans.contains(&req.plan.as_str()) {
        return Err(err(
            StatusCode::BAD_REQUEST,
            "invalid_plan",
            "plan must be: free | starter | pro | enterprise",
        ));
    }

    let (raw_key, record) =
        tokio::task::block_in_place(|| state.auth_db.create_key(&req.name, &req.plan)).map_err(
            |e| {
                err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_error",
                    &e.to_string(),
                )
            },
        )?;

    Ok(Json(CreateKeyResponse {
        key: raw_key,
        id: record.id,
        name: record.name,
        plan: record.plan,
        created_at: record.created_at,
        warning: "This key will not be shown again. Store it securely.",
    }))
}

// ─── GET /v1/keys ─────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct KeyInfo {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    pub plan: String,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

/// `GET /v1/keys` — list all active API keys (masked).
pub async fn list_keys(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> ApiResult<serde_json::Value> {
    if !check_admin(&headers) {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "X-Admin-Secret required",
        ));
    }

    let keys = tokio::task::block_in_place(|| state.auth_db.list_keys()).map_err(|e| {
        err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "db_error",
            &e.to_string(),
        )
    })?;

    let response: Vec<KeyInfo> = keys
        .into_iter()
        .map(|k| KeyInfo {
            id: k.id,
            name: k.name,
            key_preview: format!("{}...****", &k.key_prefix),
            plan: k.plan,
            created_at: k.created_at,
            last_used_at: k.last_used_at,
        })
        .collect();

    let count = response.len();
    Ok(Json(
        serde_json::json!({ "keys": response, "count": count }),
    ))
}

// ─── DELETE /v1/keys/:id ──────────────────────────────────────────────────────

/// `DELETE /v1/keys/:id` — revoke an API key.
pub async fn revoke_key(
    headers: HeaderMap,
    State(state): State<AppState>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> ApiResult<serde_json::Value> {
    if !check_admin(&headers) {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "X-Admin-Secret required",
        ));
    }

    let revoked =
        tokio::task::block_in_place(|| state.auth_db.revoke_key(&key_id)).map_err(|e| {
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_error",
                &e.to_string(),
            )
        })?;

    if revoked {
        Ok(Json(serde_json::json!({ "id": key_id, "revoked": true })))
    } else {
        Err(err(
            StatusCode::NOT_FOUND,
            "not_found",
            "Key not found or already revoked",
        ))
    }
}

// ─── GET /v1/usage ────────────────────────────────────────────────────────────

/// `GET /v1/usage` — usage statistics for the authenticated key.
pub async fn get_usage(
    crate::middleware::AuthenticatedKey(key): crate::middleware::AuthenticatedKey,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stats = tokio::task::block_in_place(|| state.auth_db.get_usage_stats(&key.id, &key.plan));

    match stats {
        Ok(s) => match serde_json::to_value(&s) {
            Ok(v) => (StatusCode::OK, Json(v)).into_response(),
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize usage stats");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "type": "https://docs.hypersearchx.com/errors/internal_error",
                        "title": "Failed to serialize usage data",
                        "status": 500,
                    })),
                )
                    .into_response()
            }
        },
        Err(e) => {
            tracing::error!(error = %e, key_id = %key.id, "Failed to fetch usage stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "type": "https://docs.hypersearchx.com/errors/db_error",
                    "title": "Failed to retrieve usage statistics",
                    "status": 500,
                })),
            )
                .into_response()
        }
    }
}

// ─── GET /health ──────────────────────────────────────────────────────────────

/// `GET /health` — service liveness + dependency readiness.
///
/// Returns 200 if fully operational, 503 if any critical dependency is down.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let searxng_url = state
        .config
        .search
        .searxng_url
        .as_deref()
        .unwrap_or("http://localhost:4040");
    let health_url = format!("{}/healthz", searxng_url.trim_end_matches('/'));
    let searxng_ok = state.http.fetch_text(&health_url).await.is_ok();

    let status = if searxng_ok { "ok" } else { "degraded" };
    let http_status = if searxng_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        http_status,
        Json(serde_json::json!({
            "status": status,
            "version": env!("CARGO_PKG_VERSION"),
            "searxng": if searxng_ok { "ok" } else { "degraded" },
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

// ─── Usage recording helper ───────────────────────────────────────────────────

/// Record API usage after a request completes. Fire-and-forget with error logging.
pub fn record_usage_async(
    state: AppState,
    key_id: String,
    endpoint: &'static str,
    status: u16,
    tokens: u64,
    start: Instant,
) {
    let duration_ms = start.elapsed().as_millis() as u64;
    let auth_db = state.auth_db.clone();
    tokio::task::spawn_blocking(move || {
        if let Err(e) = auth_db.record_usage(&key_id, endpoint, status, tokens, duration_ms) {
            tracing::error!(
                error = %e,
                key_id = %key_id,
                endpoint = endpoint,
                "Failed to record API usage — quota tracking may be inaccurate"
            );
        }
    });
}
