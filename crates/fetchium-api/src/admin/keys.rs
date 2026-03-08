//! Admin API key management handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub status: Option<String>,
    pub plan: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateKey {
    pub name: String,
    pub org_id: Option<String>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db.list_api_keys(limit, offset, p.status.as_deref(), p.plan.as_deref()).unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateKey>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysCreate)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let (id, raw_key) = db.create_api_key(body.org_id.as_deref(), &body.name, Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "key", Some(&id), "key.create", None);
    Ok(Json(serde_json::json!({"ok": true, "id": id, "key": raw_key})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.get_api_key(&id).unwrap_or(None);
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn revoke(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysRevoke)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.revoke_api_key(&id, Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "key", Some(&id), "key.revoke", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn rotate(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysCreate)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let old = db.get_api_key(&id).unwrap_or(None);
    db.revoke_api_key(&id, Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let name = old
        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .unwrap_or_else(|| "rotated".to_string());
    let (new_id, raw_key) = db.create_api_key(None, &name, Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "key", Some(&id), "key.rotate", None);
    Ok(Json(serde_json::json!({"ok": true, "id": new_id, "key": raw_key})))
}

pub async fn list_for_org(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::KeysRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_api_keys_for_org(&id).unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}
