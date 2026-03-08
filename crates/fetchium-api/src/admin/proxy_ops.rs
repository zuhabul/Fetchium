//! Admin proxy operations handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn stats(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn reset(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn purge(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn geo_distribution(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}
