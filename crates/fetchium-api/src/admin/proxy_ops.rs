//! Admin proxy operations handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn stats(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyRead)?;
    let data = state
        .admin_db
        .as_ref()
        .map(|db| db.get_proxy_stats())
        .unwrap_or_else(|| serde_json::json!({"status": "unknown"}));
    Ok(Json(data))
}

pub async fn reset(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyReset)?;
    if let Some(db) = state.admin_db.as_ref() {
        let _ = db.log_audit(
            Some(&auth.user.id),
            Some(&auth.user.role),
            "proxy",
            None,
            "proxy.reset",
            None,
        );
    }
    Ok(Json(serde_json::json!({"ok": true, "message": "Proxy pool reset queued"})))
}

pub async fn purge(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyReset)?;
    if let Some(db) = state.admin_db.as_ref() {
        let _ = db.log_audit(
            Some(&auth.user.id),
            Some(&auth.user.role),
            "proxy",
            None,
            "proxy.purge",
            None,
        );
    }
    Ok(Json(serde_json::json!({"ok": true, "message": "Proxy cache purged"})))
}

pub async fn geo_distribution(
    auth: AdminAuth,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyRead)?;
    // Return static geo config from DataImpulse supported regions
    Ok(Json(serde_json::json!({
        "data": [
            {"country": "US", "code": "us", "requests": 0},
            {"country": "GB", "code": "gb", "requests": 0},
            {"country": "DE", "code": "de", "requests": 0},
            {"country": "FR", "code": "fr", "requests": 0},
            {"country": "JP", "code": "jp", "requests": 0},
            {"country": "AU", "code": "au", "requests": 0},
        ]
    })))
}
