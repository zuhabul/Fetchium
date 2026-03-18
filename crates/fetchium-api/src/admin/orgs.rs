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
    pub search: Option<String>,
    pub plan: Option<String>,
    pub status: Option<String>,
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
pub struct UpdateOrg {
    pub name: Option<String>,
    pub owner_email: Option<String>,
    pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct OverrideQuota {
    pub monthly_limit: i64,
}

pub async fn list(
    auth: AdminAuth,
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let limit = p.limit.unwrap_or(50).min(200);
    let offset = p.offset.unwrap_or(0);
    let data = db
        .list_orgs(
            limit,
            offset,
            p.search.as_deref(),
            p.plan.as_deref(),
            p.status.as_deref(),
        )
        .unwrap_or_default();
    let total = db.count_orgs().unwrap_or(0);
    Ok(Json(serde_json::json!({"data": data, "total": total})))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateOrg>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let slug = body
        .slug
        .unwrap_or_else(|| body.name.to_lowercase().replace(' ', "-"));
    let id = db
        .create_org(&body.name, &slug, body.owner_email.as_deref())
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&id),
        "org.create",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn get(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.get_org(&id).unwrap_or(None);
    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn update(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateOrg>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_org_fields(
        &id,
        body.name.as_deref(),
        body.owner_email.as_deref(),
        body.notes.as_deref(),
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
        "org",
        Some(&id),
        "org.update",
        None,
    );
    let data = db.get_org(&id).unwrap_or(None);
    Ok(Json(serde_json::json!({"ok": true, "data": data})))
}

pub async fn suspend(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_org_status(&id, "suspended").map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&id),
        "org.suspend",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn reactivate(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsSuspend)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_org_status(&id, "active").map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&id),
        "org.reactivate",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn change_plan(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ChangePlan>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsPlanChange)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_org_plan(&id, &body.plan).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&id),
        "org.plan_change",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn members(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.list_org_members(&id).unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "total": data.len()})))
}

pub async fn org_audit(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db
        .run_select_query_params1(
            "SELECT a.id, a.action, a.role, u.email, a.ip, a.created_at
              FROM audit_events a LEFT JOIN admin_users u ON u.id = a.admin_user_id
              WHERE a.target_type='org' AND a.target_id=?1
              ORDER BY a.created_at DESC LIMIT 100",
            &id,
            100,
        )
        .map(|r| {
            r.rows
                .iter()
                .map(|row| {
                    serde_json::json!({
                        "id": row.first(), "action": row.get(1), "role": row.get(2),
                        "actor_email": row.get(3), "ip": row.get(4), "created_at": row.get(5),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(Json(serde_json::json!({"data": data, "total": data.len()})))
}

pub async fn override_quota(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<OverrideQuota>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::OrgsQuotaOverride)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_org_quota(&id, body.monthly_limit).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "org",
        Some(&id),
        "org.quota_override",
        None,
    );
    Ok(Json(
        serde_json::json!({"ok": true, "quota_override": body.monthly_limit}),
    ))
}
