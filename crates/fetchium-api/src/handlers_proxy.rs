//! Proxy management admin endpoints.

use crate::middleware::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

/// GET /v1/proxy/stats — Get proxy pool status and per-proxy stats.
pub async fn proxy_stats(State(state): State<AppState>) -> impl IntoResponse {
    match state.http.proxy_pool() {
        Some(pool) => {
            let summary = pool.pool_summary();
            let per_proxy = pool.stats();
            Json(json!({
                "enabled": true,
                "summary": summary,
                "proxies": per_proxy,
            }))
        }
        None => Json(json!({
            "enabled": false,
            "summary": null,
            "proxies": [],
            "message": "Proxy rotation is not enabled. Set [proxy] enabled = true in config.toml"
        })),
    }
}

/// POST /v1/proxy/reset — Reset all proxies to active status.
pub async fn proxy_reset(State(state): State<AppState>) -> impl IntoResponse {
    match state.http.proxy_pool() {
        Some(pool) => {
            pool.reset_all();
            Json(json!({
                "status": "ok",
                "message": "All proxies reset to active",
                "active_count": pool.len(),
            }))
        }
        None => Json(json!({
            "status": "error",
            "message": "Proxy rotation is not enabled"
        })),
    }
}

/// POST /v1/proxy/purge — Remove dead proxies from the pool.
pub async fn proxy_purge(State(state): State<AppState>) -> impl IntoResponse {
    match state.http.proxy_pool() {
        Some(pool) => {
            let removed = pool.purge_dead();
            Json(json!({
                "status": "ok",
                "removed": removed,
                "remaining": pool.len(),
            }))
        }
        None => Json(json!({
            "status": "error",
            "message": "Proxy rotation is not enabled"
        })),
    }
}

/// POST /v1/proxy/test — Test a specific proxy by making a request through it.
pub async fn proxy_test(State(state): State<AppState>) -> impl IntoResponse {
    match state.http.proxy_pool() {
        Some(pool) => {
            let proxy = pool.next_proxy();
            match proxy {
                Some(p) => {
                    let start = std::time::Instant::now();
                    let test_url = "https://httpbin.org/ip";
                    match fetchium_core::proxy::ProxyPool::build_client_with_proxy(
                        &p,
                        "Fetchium/1.0",
                        std::time::Duration::from_secs(10),
                    ) {
                        Ok(client) => match client.get(test_url).send().await {
                            Ok(resp) => {
                                let latency = start.elapsed().as_millis() as u64;
                                let status = resp.status().as_u16();
                                let body = resp.text().await.unwrap_or_default();
                                p.record_success(latency);
                                Json(json!({
                                    "status": "ok",
                                    "proxy": format!("{}:{}", p.host, p.port),
                                    "test_url": test_url,
                                    "http_status": status,
                                    "latency_ms": latency,
                                    "response": body,
                                }))
                            }
                            Err(e) => {
                                p.record_failure();
                                Json(json!({
                                    "status": "error",
                                    "proxy": format!("{}:{}", p.host, p.port),
                                    "error": e.to_string(),
                                }))
                            }
                        },
                        Err(e) => Json(json!({
                            "status": "error",
                            "error": format!("Failed to build proxy client: {e}"),
                        })),
                    }
                }
                None => Json(json!({
                    "status": "error",
                    "message": "No available proxies"
                })),
            }
        }
        None => Json(json!({
            "status": "error",
            "message": "Proxy rotation is not enabled"
        })),
    }
}
