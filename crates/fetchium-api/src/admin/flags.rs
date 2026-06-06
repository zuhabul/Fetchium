//! Admin feature flag handlers.

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
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let all = db.list_flags().unwrap_or_default();
    let total = all.len();
    let limit = p.limit.unwrap_or(200).min(500);
    let offset = p.offset.unwrap_or(0).min(total);
    let page: Vec<_> = all.into_iter().skip(offset).take(limit).collect();
    Ok(Json(
        serde_json::json!({"data": page, "total": total, "limit": limit, "offset": offset}),
    ))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateFlag>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::FlagsWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let id = db
        .create_flag(&body.key, body.description.as_deref(), Some(&auth.user.id))
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    if let Some(enabled) = body.enabled {
        db.update_flag_enabled(&id, enabled).map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
