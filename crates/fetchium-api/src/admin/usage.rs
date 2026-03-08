//! Admin usage analytics handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn summary(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn for_org(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": {}}))
}

pub async fn forensics(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_request_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null}))
}

pub async fn top_orgs(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

pub async fn endpoint_heatmap(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}
