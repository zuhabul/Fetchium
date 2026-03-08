//! Admin billing and subscription handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn list_subscriptions(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": [], "total": 0}))
}

pub async fn for_org(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_org_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null}))
}

pub async fn refund(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_org_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn credit(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_org_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn invoices(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_org_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

pub async fn webhook_log(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

pub async fn webhook_replay(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}
