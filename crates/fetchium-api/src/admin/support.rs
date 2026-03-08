//! Admin support ticket handlers.

use crate::admin::rbac::AdminAuth;
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn list(_auth: AdminAuth, State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": [], "total": 0}))
}

pub async fn get(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null}))
}

pub async fn for_org(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": [], "total": 0}))
}

pub async fn add_note(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn assign(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn set_status(
    _auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true}))
}

pub async fn list_macros(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": []}))
}

pub async fn create_macro(
    _auth: AdminAuth,
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({"ok": true, "data": null}))
}
