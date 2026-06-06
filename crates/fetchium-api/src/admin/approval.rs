//! Admin approval workflow handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct CreateApproval {
    pub action_type: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ReviewBody {
    pub note: Option<String>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let all = db.list_approvals().unwrap_or_default();
    let total = all.len();
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0).min(total);
    let page: Vec<_> = all.into_iter().skip(offset).take(limit).collect();
    Ok(Json(
        serde_json::json!({"data": page, "total": total, "limit": limit, "offset": offset}),
    ))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateApproval>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let payload_str = body.payload.as_ref().map(|v| v.to_string());
    let id = db
        .create_approval(
            &body.action_type,
            payload_str.as_deref(),
            Some(&auth.user.id),
        )
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn approve(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AdminStaffManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_approval_status(&id, "approved", Some(&auth.user.id), body.note.as_deref())
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "approval",
        Some(&id),
        "approval.approve",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn reject(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ReviewBody>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AdminStaffManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_approval_status(&id, "rejected", Some(&auth.user.id), body.note.as_deref())
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "approval",
        Some(&id),
        "approval.reject",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}
