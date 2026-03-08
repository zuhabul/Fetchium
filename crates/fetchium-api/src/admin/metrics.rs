//! Admin metrics and observability handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn realtime(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn summary(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn provider_health(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}
