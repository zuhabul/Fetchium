//! REST API route definitions.

use axum::{routing::{get, post}, Router};
use std::sync::Arc;
use crate::handlers;
use crate::middleware::AppState;

/// Build the axum router with all API routes attached.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health",     get(handlers::health_check))
        .route("/api/health", get(handlers::health_check))

        // Core endpoints
        .route("/api/search",   post(handlers::search))
        .route("/api/fetch",    post(handlers::fetch))
        .route("/api/research", post(handlers::research))
        .route("/api/estimate", post(handlers::estimate))

        .with_state(state)
}
