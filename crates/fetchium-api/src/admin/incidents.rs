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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
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
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let data = db.get_incident(&id).unwrap_or(None);
    let timeline = db.get_incident_timeline(&id).unwrap_or_default();
    Ok(Json(
        serde_json::json!({"data": data, "timeline": timeline}),
    ))
}

pub async fn create(
    auth: AdminAuth,
    State(state): State<AppState>,
    Json(body): Json<CreateIncident>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let id = db
        .create_incident(
            &body.title,
            body.severity.as_deref().unwrap_or("low"),
            Some(&auth.user.id),
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
        "incident",
        Some(&id),
        "incident.create",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true, "id": id})))
}

pub async fn update(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateIncident>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.update_incident(&id, body.status.as_deref(), body.severity.as_deref())
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "incident",
        Some(&id),
        "incident.update",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn add_timeline(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<AddTimeline>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.add_incident_timeline(&id, Some(&auth.user.id), &body.message)
        .map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn resolve(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsManage)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    db.resolve_incident(&id).map_err(|e| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "incident",
        Some(&id),
        "incident.resolve",
        None,
    );
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn postmortem(
    auth: AdminAuth,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::IncidentsRead)?;
    let db = state.admin_db.as_ref().ok_or_else(|| {
        (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({"error": "admin db not initialized"})),
        )
    })?;
    let incident = db.get_incident(&id).unwrap_or(None).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "incident not found"})),
        )
    })?;
    let timeline = db.get_incident_timeline(&id).unwrap_or_default();

    let title = incident
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");
    let severity = incident
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let status = incident
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let started = incident
        .get("started_at")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let resolved = incident
        .get("resolved_at")
        .and_then(|v| v.as_str())
        .unwrap_or("unresolved");

    let timeline_text: String = timeline
        .iter()
        .enumerate()
        .map(|(i, ev)| {
            let t = ev
                .get("event_type")
                .and_then(|v| v.as_str())
                .unwrap_or("update");
            let msg = ev.get("message").and_then(|v| v.as_str()).unwrap_or("");
            let ts = ev.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            format!("  {}. [{}] {} — {}", i + 1, t.to_uppercase(), ts, msg)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let postmortem = format!(
        "INCIDENT POSTMORTEM\n\
         ══════════════════════════════════════════\n\
         Title:     {title}\n\
         Severity:  {severity}\n\
         Status:    {status}\n\
         Started:   {started}\n\
         Resolved:  {resolved}\n\
         \n\
         TIMELINE\n\
         ─────────────────────────────────────────\n\
         {timeline_or_none}\n\
         \n\
         SUMMARY\n\
         ─────────────────────────────────────────\n\
         This incident was classified as {severity} severity. \
         {timeline_count} timeline event(s) were recorded. \
         {resolution_note}\n\
         \n\
         RECOMMENDATIONS\n\
         ─────────────────────────────────────────\n\
         • Review monitoring alerts for early detection\n\
         • Ensure runbooks are up to date for {severity} severity incidents\n\
         • Schedule a follow-up review within 5 business days\n",
        timeline_or_none = if timeline_text.is_empty() {
            "  (no timeline events recorded)".to_string()
        } else {
            timeline_text
        },
        timeline_count = timeline.len(),
        resolution_note = if status == "resolved" {
            format!("Incident was resolved at {resolved}.")
        } else {
            "Incident is still open — resolution pending.".to_string()
        },
    );

    let _ = db.log_audit(
        Some(&auth.user.id),
        Some(&auth.user.role),
        "incident",
        Some(&id),
        "incident.postmortem",
        None,
    );
    Ok(Json(
        serde_json::json!({"ok": true, "postmortem": postmortem}),
    ))
}
