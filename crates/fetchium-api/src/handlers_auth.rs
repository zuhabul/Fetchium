//! API key management handlers — create, list, revoke, usage stats.
//!
//! Admin endpoints use `X-Admin-Secret` header for the MVP.

use crate::middleware::AppState;
use crate::types::{ResponseMeta, UsageResponse};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<serde_json::Value>)>;

fn err(status: StatusCode, kind: &str, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({
            "type": format!("https://docs.fetchium.com/errors/{kind}"),
            "title": msg,
            "status": status.as_u16(),
        })),
    )
}

fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn response_meta(request_id: String, endpoint: &str, duration_ms: u64) -> ResponseMeta {
    ResponseMeta {
        request_id,
        status: "ok".into(),
        endpoint: endpoint.into(),
        duration_ms,
        query: None,
        tier: None,
        tokens_used: None,
        sources_count: None,
        result_id: None,
    }
}

/// Validate the admin secret from `X-Admin-Secret`.
///
/// Reads `***REMOVED***` from the environment.
/// Panics at startup if neither is set in the environment,
/// preventing insecure "changeme" defaults in production.
fn check_admin(headers: &HeaderMap) -> bool {
    let secret = std::env::var("***REMOVED***")
        .expect("***REMOVED*** must be set (generate with: openssl rand -hex 32).");
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
    pub meta: ResponseMeta,
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
    let start = Instant::now();
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

    let db = state.auth_db.clone();
    let name = req.name.clone();
    let plan_str = req.plan.clone();
    let (raw_key, record) =
        tokio::task::spawn_blocking(move || db.create_key(&name, &plan_str))
            .await
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?
            .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?;

    Ok(Json(CreateKeyResponse {
        meta: response_meta(
            request_id_from_headers(&headers),
            "/v1/keys",
            start.elapsed().as_millis() as u64,
        ),
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
    let start = Instant::now();
    if !check_admin(&headers) {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "X-Admin-Secret required",
        ));
    }

    let db = state.auth_db.clone();
    let keys = tokio::task::spawn_blocking(move || db.list_keys())
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?;

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
    Ok(Json(serde_json::json!({
        "meta": response_meta(
            request_id_from_headers(&headers),
            "/v1/keys",
            start.elapsed().as_millis() as u64,
        ),
        "keys": response,
        "count": count
    })))
}

// ─── DELETE /v1/keys/:id ──────────────────────────────────────────────────────

/// `DELETE /v1/keys/:id` — revoke an API key.
pub async fn revoke_key(
    headers: HeaderMap,
    State(state): State<AppState>,
    axum::extract::Path(key_id): axum::extract::Path<String>,
) -> ApiResult<serde_json::Value> {
    let start = Instant::now();
    if !check_admin(&headers) {
        return Err(err(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "X-Admin-Secret required",
        ));
    }

    let db = state.auth_db.clone();
    let kid = key_id.clone();
    let revoked = tokio::task::spawn_blocking(move || db.revoke_key(&kid))
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, "db_error", &e.to_string()))?;

    if revoked {
        Ok(Json(serde_json::json!({
            "meta": response_meta(
                request_id_from_headers(&headers),
                "/v1/keys/:id",
                start.elapsed().as_millis() as u64,
            ),
            "id": key_id,
            "revoked": true
        })))
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
    headers: HeaderMap,
) -> impl IntoResponse {
    let start = Instant::now();
    let db = state.auth_db.clone();
    let key_id = key.id.clone();
    let key_plan = key.plan.clone();
    let stats = tokio::task::spawn_blocking(move || db.get_usage_stats(&key_id, &key_plan))
        .await
        .unwrap_or_else(|e| Err(anyhow::anyhow!("task join error: {e}")));

    match stats {
        Ok(s) => match serde_json::to_value(&s) {
            Ok(v) => (
                StatusCode::OK,
                Json(UsageResponse {
                    meta: response_meta(
                        request_id_from_headers(&headers),
                        "/v1/usage",
                        start.elapsed().as_millis() as u64,
                    ),
                    usage: v,
                }),
            )
                .into_response(),
            Err(e) => {
                tracing::error!(error = %e, "Failed to serialize usage stats");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "type": "https://docs.fetchium.com/errors/internal_error",
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
                    "type": "https://docs.fetchium.com/errors/db_error",
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
/// Returns 200 for healthy/degraded states, 503 only when a critical dependency fails.
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let searxng_url = state
        .config
        .search
        .searxng_url
        .clone()
        .or_else(|| std::env::var("SEARXNG_URL").ok())
        .unwrap_or_else(|| "***REMOVED***".to_string());
    let health_url = format!(
        "{}/search?q=test&format=json",
        searxng_url.trim_end_matches('/')
    );
    let search_backbone_ok = tokio::time::timeout(
        Duration::from_secs(3),
        state.http.fetch_text(&health_url),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);
    let db = state.auth_db.clone();
    let auth_store_ok = tokio::task::spawn_blocking(move || db.list_keys().map(|_| ()))
        .await
        .map(|r| r.is_ok())
        .unwrap_or(false);

    let status = if auth_store_ok && search_backbone_ok {
        "ok"
    } else if auth_store_ok {
        "degraded"
    } else {
        "unavailable"
    };
    let http_status = if auth_store_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        http_status,
        Json(serde_json::json!({
            "status": status,
            "version": env!("CARGO_PKG_VERSION"),
            "checks": {
                "api": "ok",
                "search_backbone": if search_backbone_ok { "ok" } else { "degraded" },
                "auth_store": if auth_store_ok { "ok" } else { "error" }
            },
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
