//! Admin usage analytics handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn summary(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    // Derive usage stats from audit_events
    let total_requests = db.run_select_query(
        "SELECT COUNT(*) FROM audit_events", 1,
    ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);

    let requests_today = db.run_select_query(
        "SELECT COUNT(*) FROM audit_events WHERE created_at >= date('now')", 1,
    ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);

    let active_orgs = db.run_select_query(
        "SELECT COUNT(DISTINCT target_id) FROM audit_events WHERE target_type='org' AND created_at >= date('now','-30 days')", 1,
    ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);

    let error_count = db.run_select_query(
        "SELECT COUNT(*) FROM audit_events WHERE action LIKE '%.error' OR action LIKE '%.fail'", 1,
    ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);

    Ok(Json(serde_json::json!({
        "total_requests": total_requests,
        "requests_today": requests_today,
        "active_orgs_30d": active_orgs,
        "error_count": error_count,
    })))
}

pub async fn for_org(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    let recent = db.run_select_query(
        &format!("SELECT action, created_at FROM audit_events WHERE target_type='org' AND target_id='{id}' ORDER BY created_at DESC LIMIT 50"),
        50,
    ).map(|r| {
        r.rows.iter().map(|row| serde_json::json!({
            "action": row.first(), "created_at": row.get(1),
        })).collect::<Vec<_>>()
    }).unwrap_or_default();

    let total = db.run_select_query(
        &format!("SELECT COUNT(*) FROM audit_events WHERE target_type='org' AND target_id='{id}'"), 1,
    ).ok().and_then(|r| r.rows.first()?.first()?.as_i64()).unwrap_or(0);

    Ok(Json(serde_json::json!({"recent": recent, "total": total})))
}

pub async fn forensics(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(request_id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    let data = db.run_select_query(
        &format!("SELECT id, admin_user_id, role, target_type, target_id, action, ip, created_at FROM audit_events WHERE id='{request_id}'"),
        1,
    ).map(|r| {
        r.rows.first().map(|row| serde_json::json!({
            "id": row.first(), "user_id": row.get(1), "role": row.get(2),
            "target_type": row.get(3), "target_id": row.get(4),
            "action": row.get(5), "ip": row.get(6), "created_at": row.get(7),
        }))
    }).unwrap_or(None);

    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn top_orgs(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    let data = db.run_select_query(
        "SELECT o.id, o.name, o.plan, COUNT(a.id) as event_count
         FROM organizations o
         LEFT JOIN audit_events a ON a.target_id = o.id AND a.target_type = 'org'
         GROUP BY o.id ORDER BY event_count DESC LIMIT 20",
        20,
    ).map(|r| {
        r.rows.iter().map(|row| serde_json::json!({
            "id": row.first(), "name": row.get(1), "plan": row.get(2), "event_count": row.get(3),
        })).collect::<Vec<_>>()
    }).unwrap_or_default();

    Ok(Json(serde_json::json!({"data": data})))
}

pub async fn endpoint_heatmap(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;

    let data = db.run_select_query(
        "SELECT action, COUNT(*) as count, MAX(created_at) as last_seen
         FROM audit_events
         GROUP BY action ORDER BY count DESC LIMIT 30",
        30,
    ).map(|r| {
        r.rows.iter().map(|row| serde_json::json!({
            "action": row.first(), "count": row.get(1), "last_seen": row.get(2),
        })).collect::<Vec<_>>()
    }).unwrap_or_default();

    Ok(Json(serde_json::json!({"data": data})))
}
