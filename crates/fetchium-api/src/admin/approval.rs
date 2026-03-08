//! Admin approval workflow handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn list(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": [], "total": 0}))
}

pub async fn create(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null}))
}

pub async fn approve(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn reject(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}
