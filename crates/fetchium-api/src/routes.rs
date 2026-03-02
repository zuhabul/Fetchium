//! REST API route definitions — v1 API under /v1/ prefix.

use crate::middleware::AppState;
use crate::{handlers, handlers_auth};
use axum::{
    routing::{delete, get, post},
    Router,
};

/// Build the axum router with all v1 API routes.
pub fn build_router(state: AppState) -> Router {
    // Authenticated endpoints (require Bearer fetchium_xxx)
    let v1_authed = Router::new()
        .route("/search", post(handlers::search))
        .route("/scrape", post(handlers::fetch))
        .route("/fetch", post(handlers::fetch)) // alias
        .route("/research", post(handlers::research))
        .route("/youtube/search", post(handlers::youtube_search))
        .route("/youtube/analyze", post(handlers::youtube_analyze))
        .route("/social/research", post(handlers::social_research))
        .route("/social/reddit", post(handlers::reddit_search))
        .route("/social/hackernews", post(handlers::hackernews_search))
        .route("/estimate", post(handlers::estimate))
        .route("/usage", get(handlers_auth::get_usage));

    // Admin endpoints (require X-Admin-Secret header, for MVP)
    let v1_admin = Router::new()
        .route("/keys", post(handlers_auth::create_key))
        .route("/keys", get(handlers_auth::list_keys))
        .route("/keys/:id", delete(handlers_auth::revoke_key));

    Router::new()
        // Public endpoints (no auth)
        .route("/health", get(handlers_auth::health))
        .route("/v1/health", get(handlers_auth::health))
        .route("/", get(api_root))
        // Versioned API
        .nest("/v1", v1_authed)
        .nest("/v1", v1_admin)
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
            "usage":    "GET  /v1/usage",
            "health":   "GET  /v1/health",
            "keys":     "POST /v1/keys  (X-Admin-Secret required)",
        }
    }))
}
