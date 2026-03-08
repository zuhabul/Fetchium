//! Admin audit log handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db.list_audit(limit, offset).unwrap_or_default();
    let total = db.count_audit().unwrap_or(0);
    Ok(Json(serde_json::json!({"data": data, "total": total, "limit": limit, "offset": offset})))
}

pub async fn get(
    auth: AdminAuth,
    State(_state): State<AppState>,
    axum::extract::Path(_id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    Ok(Json(serde_json::json!({"data": null})))
}
