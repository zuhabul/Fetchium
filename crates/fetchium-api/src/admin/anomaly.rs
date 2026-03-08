//! Admin anomaly detection handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn alerts(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    // High-frequency actors: >20 audit actions in last hour
    let alerts = db.run_select_query(
        "SELECT a.admin_user_id, u.email, COUNT(*) as action_count, MAX(a.created_at) as last_action
         FROM audit_events a
         LEFT JOIN admin_users u ON u.id = a.admin_user_id
         WHERE a.created_at > datetime('now', '-1 hour')
         GROUP BY a.admin_user_id
         HAVING COUNT(*) > 20
         ORDER BY action_count DESC
         LIMIT 20",
        20,
    ).map(|r| {
        r.rows.iter().map(|row| serde_json::json!({
            "type": "high_frequency_actor",
            "severity": "warning",
            "user_id": row.first(),
            "user_email": row.get(1),
            "action_count": row.get(2),
            "last_action": row.get(3),
        })).collect::<Vec<_>>()
    }).unwrap_or_default();

    Ok(Json(serde_json::json!({"data": alerts, "total": alerts.len()})))
}

pub async fn suspicious_tenants(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({"error": "admin db not initialized"})),
    ))?;
    // Suspended orgs
    let data = db.run_select_query(
        "SELECT id, name, slug, status, plan, created_at
         FROM organizations
         WHERE status = 'suspended'
         ORDER BY updated_at DESC
         LIMIT 50",
        50,
    ).map(|r| {
        r.rows.iter().map(|row| serde_json::json!({
            "id": row.first(),
            "name": row.get(1),
            "slug": row.get(2),
            "status": row.get(3),
            "plan": row.get(4),
            "created_at": row.get(5),
            "reason": "suspended",
        })).collect::<Vec<_>>()
    }).unwrap_or_default();

    Ok(Json(serde_json::json!({"data": data, "total": data.len()})))
}
