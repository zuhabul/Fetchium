//! Admin feature flag handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateFlag {
    pub key: String,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateFlag {
    pub enabled: Option<bool>,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.list_flags().unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "total": data.len()})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateFlag>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let id = db.create_flag(
        &body.key,
        body.description.as_deref(),
        Some(&auth.user.id),
    ).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    if body.enabled.unwrap_or(false) {
        let _ = db.update_flag_enabled(&id, true);
    }
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.get_flag(&id).unwrap_or(None);
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn update(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateFlag>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    if let Some(enabled) = body.enabled {
        db.update_flag_enabled(&id, enabled).map_err(|e| (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        ))?;
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
