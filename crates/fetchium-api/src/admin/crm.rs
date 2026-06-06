//! Admin CRM account handlers.

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
}

#[derive(Deserialize)]
pub struct UpdateCrm {
    pub health: Option<String>,
    pub csm_id: Option<String>,
    pub mrr: Option<i64>,
    pub nps: Option<i64>,
}

#[derive(Deserialize)]
pub struct AddNote {
    pub body: String,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let all = db.list_crm_accounts(10_000, 0).unwrap_or_default();
    let total = all.len() as i64;
    let data = db.list_crm_accounts(limit, offset).unwrap_or_default();
    Ok(Json(
        serde_json::json!({"data": data, "total": total, "limit": limit, "offset": offset}),
    ))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.get_crm_account(&org_id).unwrap_or(None);
    let notes = db.list_crm_notes(&org_id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "notes": notes})))
}

pub async fn update(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(body): Json<UpdateCrm>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.upsert_crm_account(
        &org_id,
        body.health.as_deref(),
        body.csm_id.as_deref(),
        body.mrr,
        body.nps,
    )
    .map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "crm",
        Some(&org_id),
        "crm.update",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn add_note(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(org_id): Path<String>,
    Json(body): Json<AddNote>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::CrmWrite)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.add_crm_note(&org_id, Some(&auth.user.id), &body.body)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(serde_json::json!({"ok": true})))
}
