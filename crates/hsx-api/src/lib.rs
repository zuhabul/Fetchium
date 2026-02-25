//! HyperSearchX REST API Server (PRD §9).
//!
//! Provides HTTP endpoints for all HyperSearchX functionality.
//! Built with axum 0.7 and tower-http middleware (CORS, tracing).
//!
//! # Endpoints
//! | Method | Path           | Description                        |
//! |--------|----------------|------------------------------------|
//! | GET    | /health        | Health check                       |
//! | POST   | /api/search    | Multi-backend web search           |
//! | POST   | /api/fetch     | URL fetch + content extraction     |
//! | POST   | /api/research  | Full research pipeline             |
//! | POST   | /api/estimate  | Token cost estimation              |

pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod types;

use crate::middleware::AppState;
use crate::routes::build_router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Configuration for the REST API server.
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 3000,
        }
    }
}

/// Start the REST API server.
pub async fn start_api_server(
    server_config: ApiServerConfig,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = build_router(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", server_config.host, server_config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("HyperSearchX REST API listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
