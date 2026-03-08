//! Admin incident management handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateIncident {
    pub title: String,
    pub severity: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateIncident {
    pub status: Option<String>,
    pub severity: Option<String>,
}

#[derive(Deserialize)]
pub struct AddTimeline {
    pub message: String,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_incidents().unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.get_incident(&id).unwrap_or(None);
    let timeline = db.get_incident_timeline(&id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "timeline": timeline})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateIncident>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let id = db.create_incident(&body.title, body.severity.as_deref().unwrap_or("low"), Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "incident", Some(&id), "incident.create", None);
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn update(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateIncident>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_incident(&id, body.status.as_deref(), body.severity.as_deref())
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "incident", Some(&id), "incident.update", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn add_timeline(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddTimeline>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.add_incident_timeline(&id, Some(&auth.user.id), &body.message)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn resolve(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.resolve_incident(&id)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "incident", Some(&id), "incident.resolve", None);
    Ok(Json(serde_json::json!({"ok": true})))
}
