//! Admin user management handlers.

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
    pub search: Option<String>,
    pub status: Option<String>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::UsersRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db.list_users(limit, offset, p.search.as_deref(), p.status.as_deref()).unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::UsersRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.find_user_by_id(&id).unwrap_or(None).map(|u| serde_json::json!({
        "id": u.id, "email": u.email, "role": u.role, "name": u.name,
        "is_active": u.is_active, "totp_enabled": u.totp_enabled,
        "created_at": u.created_at, "last_login_at": u.last_login_at,
    }));
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn suspend(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::UsersSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.set_user_active(&id, false).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "user", Some(&id), "user.suspend", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn force_logout(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::UsersSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.revoke_all_sessions_for_user(&id).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "user", Some(&id), "user.force_logout", None);
    Ok(Json(serde_json::json!({"ok": true})))
}
