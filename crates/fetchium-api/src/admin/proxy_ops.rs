//! Admin proxy operations handlers.

use crate::admin::rbac::{require, AdminAuth, Permission};
use crate::middleware::AppState;
use axum::{extract::State, Json};

pub async fn stats(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyRead)?;

    // Live pool summary from the in-process ProxyPool
    let pool_summary = state.http.proxy_pool().map(|p| p.pool_summary());
    let pool_entries: Vec<serde_json::Value> = state
        .http
        .proxy_pool()
        .map(|p| {
            p.stats()
                .into_iter()
                .map(|s| serde_json::to_value(s).unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();

    // Historical audit data (reset/purge counts)
    let history = state
        .admin_db
        .as_ref()
        .map(|db| db.get_proxy_stats())
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(Json(serde_json::json!({
        "summary": pool_summary,
        "proxies": pool_entries,
        "history": history,
    })))
}

pub async fn reset(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyReset)?;

    // Reset every proxy to Active and clear domain assignments
    let proxies_reset = state.http.proxy_pool().map(|p| {
        p.reset_all();
        p.len()
    });

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
    Ok(Json(serde_json::json!({
        "ok": true,
        "proxies_reset": proxies_reset.unwrap_or(0),
        "message": "All proxies reset to active status"
    })))
}

pub async fn purge(
    auth: AdminAuth,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    require(&auth.user, Permission::ProxyReset)?;

    // Remove dead proxies from the pool
    let removed = state.http.proxy_pool().map(|p| p.purge_dead()).unwrap_or(0);

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
    Ok(Json(serde_json::json!({
        "ok": true,
        "proxies_removed": removed,
        "message": format!("Purged {removed} dead proxies from pool")
    })))
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
