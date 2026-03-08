//! Admin org management handlers.

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
pub struct CreateOrg {
    pub name: String,
    pub slug: Option<String>,
    pub owner_email: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePlan {
    pub plan: String,
}

#[derive(Deserialize)]
pub struct OverrideQuota {}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db.list_orgs(limit, offset).unwrap_or_default();
    let total = db.count_orgs().unwrap_or(0);
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateOrg>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let slug = body.slug.unwrap_or_else(|| body.name.to_lowercase().replace(' ', "-"));
    let id = db.create_org(&body.name, &slug, body.owner_email.as_deref()).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "org", Some(&id), "org.create", None);
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let data = db.get_org(&id).unwrap_or(None);
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn update(
    auth: AdminAuth,
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn suspend(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_org_status(&id, "suspended").map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "org", Some(&id), "org.suspend", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn reactivate(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_org_status(&id, "active").map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "org", Some(&id), "org.reactivate", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn change_plan(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ChangePlan>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsPlanChange)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    db.update_org_plan(&id, &body.plan).map_err(|e| (
        axum::http::StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": e.to_string()})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "org", Some(&id), "org.plan_change", None);
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn override_quota(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_body): Json<OverrideQuota>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsQuotaOverride)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    let _ = db.log_audit(Some(&auth.user.id), Some(&auth.user.role), "org", Some(&id), "org.quota_override", None);
    Ok(Json(serde_json::json!({"ok": true})))
}
