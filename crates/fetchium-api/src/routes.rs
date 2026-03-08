//! REST API route definitions — v1 API under /v1/ prefix.

use crate::admin;
use crate::middleware::AppState;
use crate::{handlers, handlers_auth, handlers_proxy};
use axum::{
    routing::{delete, get, post},
    Router,
};

/// Build the axum router with all v1 API routes.
pub fn build_router(state: AppState) -> Router {
    // Authenticated endpoints (require Bearer fetchium_xxx)
    let v1_authed = Router::new()
        .route("/search", post(handlers::search))
        .route("/scrape", post(handlers::scrape))
        .route("/fetch", post(handlers::fetch))
        .route("/research", post(handlers::research))
        .route("/youtube/search", post(handlers::youtube_search))
        .route("/youtube/analyze", post(handlers::youtube_analyze))
        .route("/social/research", post(handlers::social_research))
        .route(
            "/social/research/jobs",
            post(handlers::submit_social_research_job),
        )
        .route("/social/reddit", post(handlers::reddit_search))
        .route(
            "/social/reddit/jobs",
            post(handlers::submit_reddit_search_job),
        )
        .route("/social/hackernews", post(handlers::hackernews_search))
        .route(
            "/social/hackernews/jobs",
            post(handlers::submit_hackernews_search_job),
        )
        .route("/estimate", post(handlers::estimate))
        .route("/research/jobs", post(handlers::submit_research_job))
        .route(
            "/youtube/search/jobs",
            post(handlers::submit_youtube_search_job),
        )
        .route(
            "/youtube/analyze/jobs",
            post(handlers::submit_youtube_analyze_job),
        )
        .route("/jobs/:id", get(handlers::get_job_status))
        .route("/usage", get(handlers_auth::get_usage));

    // Admin endpoints (require X-Admin-Secret header, for MVP)
    let v1_admin = Router::new()
        .route("/keys", post(handlers_auth::create_key))
        .route("/keys", get(handlers_auth::list_keys))
        .route("/keys/:id", delete(handlers_auth::revoke_key))
        // Proxy management
        .route("/proxy/stats", get(handlers_proxy::proxy_stats))
        .route("/proxy/reset", post(handlers_proxy::proxy_reset))
        .route("/proxy/purge", post(handlers_proxy::proxy_purge))
        .route("/proxy/test", post(handlers_proxy::proxy_test));

    // Internal admin routes — session-authenticated, staff only
    let internal_admin = Router::new()
        // Auth
        .route("/auth/bootstrap", post(admin::auth::bootstrap))
        .route("/auth/login", post(admin::auth::login))
        .route("/auth/logout", post(admin::auth::logout))
        .route("/auth/me", get(admin::auth::me))
        .route("/auth/totp/setup", post(admin::auth::totp_setup))
        .route("/auth/totp/confirm", post(admin::auth::totp_confirm))
        // Sessions
        .route("/sessions", get(admin::auth::list_sessions))
        .route("/sessions/:id", delete(admin::auth::revoke_session));

    Router::new()
        // Public endpoints (no auth)
        .route("/health", get(handlers_auth::health))
        .route("/v1/health", get(handlers_auth::health))
        .route("/", get(api_root))
        // Versioned public/customer API
        .nest("/v1", v1_authed)
        .nest("/v1", v1_admin)
        // Internal admin namespace (session auth, not X-Admin-Secret)
        .nest("/internal/admin", internal_admin)
        // Legacy unversioned routes (backwards compat)
        .route("/api/health", get(handlers_auth::health))
        .with_state(state)
}

async fn api_root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "Fetchium API",
        "version": env!("CARGO_PKG_VERSION"),
        "docs": "https://docs.fetchium.com",
        "endpoints": {
            "search":   "POST /v1/search",
            "scrape":   "POST /v1/scrape",
            "research": "POST /v1/research",
            "research_jobs": "POST /v1/research/jobs",
            "job_status": "GET /v1/jobs/:id",
            "usage":    "GET  /v1/usage",
            "health":   "GET  /v1/health",
            "keys":     "POST /v1/keys  (X-Admin-Secret required)",
        }
    }))
}
