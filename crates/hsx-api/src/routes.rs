//! REST API route definitions.

use crate::handlers;
use crate::middleware::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Build the axum router with all API routes attached.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        .route("/api/health", get(handlers::health_check))
        // Core endpoints
        .route("/api/search", post(handlers::search))
        .route("/api/fetch", post(handlers::fetch))
        .route("/api/research", post(handlers::research))
        .route("/api/estimate", post(handlers::estimate))
        // YouTube Intelligence endpoints
        .route("/api/youtube/search", post(handlers::youtube_search))
        .route("/api/youtube/analyze", post(handlers::youtube_analyze))
        // Social Media Intelligence endpoints
        .route("/api/social/research", post(handlers::social_research))
        .route("/api/social/reddit", post(handlers::reddit_search))
        .route("/api/social/hackernews", post(handlers::hackernews_search))
        .with_state(state)
}
