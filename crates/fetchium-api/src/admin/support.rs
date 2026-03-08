//! Admin support ticket handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddNote {
    pub body: String,
}

#[derive(Deserialize)]
pub struct Assign {
    pub assignee_id: String,
}

#[derive(Deserialize)]
pub struct SetStatus {
    pub status: String,
}

#[derive(Deserialize)]
pub struct CreateMacro {
    pub name: String,
    pub body: String,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_tickets().unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.get_ticket(&id).unwrap_or(None);
    let notes = db.get_ticket_notes(&id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "notes": notes})))
}

pub async fn for_org(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_tickets_for_org(&id).unwrap_or_default();
    let total = data.len() as i64;
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn add_note(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddNote>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportReply)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.add_ticket_note(&id, Some(&auth.user.id), &body.body)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn assign(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<Assign>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportClose)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_ticket_assignee(&id, &body.assignee_id)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn set_status(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<SetStatus>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportClose)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_ticket_status(&id, &body.status)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "ticket", Some(&id), &format!("ticket.{}", body.status), None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn list_macros(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_support_macros().unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn create_macro(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateMacro>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::SupportClose)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let id = db.create_support_macro(&body.name, &body.body, Some(&auth.user.id))
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}
