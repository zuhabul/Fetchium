//! Admin CSV export handler.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn export_csv(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(entity): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::AuditRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;

    let sql = match entity.as_str() {
        "orgs" => "SELECT id, name, slug, status, plan, owner_email, created_at FROM organizations ORDER BY created_at DESC LIMIT 10000",
        "users" => "SELECT id, email, role, name, is_active, created_at, last_login_at FROM admin_users ORDER BY created_at DESC LIMIT 10000",
        "audit" => "SELECT id, admin_user_id, role, target_type, target_id, action, ip, created_at FROM audit_events ORDER BY created_at DESC LIMIT 10000",
        "incidents" => "SELECT id, title, severity, status, owner_id, created_at, resolved_at FROM incidents ORDER BY created_at DESC LIMIT 10000",
        "tickets" => "SELECT id, org_id, subject, status, priority, assignee_id, created_at FROM support_tickets ORDER BY created_at DESC LIMIT 10000",
        "flags" => "SELECT id, key, enabled, description, created_at, updated_at FROM feature_flags ORDER BY key LIMIT 10000",
        "campaigns" => "SELECT id, name, type, status, created_by, created_at FROM campaigns ORDER BY created_at DESC LIMIT 10000",
        _ => return Err((
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": format!("unknown entity: {entity}. Valid: orgs, users, audit, incidents, tickets, flags, campaigns")})),
        )),
    };

    let result = db.run_select_query(sql, 10000).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    // Build CSV string
    let mut csv = result.columns.join(",") + "\n";
    for row in &result.rows {
        let line = row
            .iter()
            .map(|v| {
                let s = match v {
                    serde_json::Value::Null => String::new(),
                    serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
                    other => other.to_string(),
                };
                s
            })
            .collect::<Vec<_>>()
            .join(",");
        csv.push_str(&line);
        csv.push('\n');
    }

    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        entity.as_str(),
        None,
        &format!("export.{entity}"),
        None,
    );

    Ok(Json(serde_json::json!({
        "ok": true,
        "entity": entity,
        "rows": result.rows.len(),
        "columns": result.columns,
        "csv": csv,
    })))
}
