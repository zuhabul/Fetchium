//! Admin anomaly detection handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn alerts(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

pub async fn suspicious_tenants(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}
