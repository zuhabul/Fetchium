//! Fetchium REST API Server (PRD §9).
//!
//! Provides HTTP endpoints for all Fetchium functionality.
//! Built with axum 0.7 and tower-http middleware.
//!
//! # Endpoints
//! | Method | Path              | Auth            | Description                    |
//! |--------|-------------------|-----------------|--------------------------------|
//! | GET    | /health           | public          | Health check                   |
//! | POST   | /v1/search        | Bearer hsx_xxx  | Multi-backend web search       |
//! | POST   | /v1/scrape        | Bearer hsx_xxx  | URL fetch + content extraction |
//! | POST   | /v1/research      | Bearer hsx_xxx  | Full research pipeline         |
//! | GET    | /v1/usage         | Bearer hsx_xxx  | Usage statistics               |
//! | POST   | /v1/keys          | X-Admin-Secret  | Create API key (admin)         |
//! | GET    | /v1/keys          | X-Admin-Secret  | List API keys (admin)          |
//! | DELETE | /v1/keys/:id      | X-Admin-Secret  | Revoke API key (admin)         |

pub mod auth;
pub mod handlers;
pub mod handlers_auth;
pub mod middleware;
pub mod routes;
pub mod types;

use crate::auth::AuthDb;
use crate::middleware::AppState;
use crate::routes::build_router;
use axum::http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    HeaderName, HeaderValue, Method,
};
use std::path::PathBuf;
use std::time::Duration;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// Configuration for the REST API server.
#[derive(Debug, Clone)]
pub struct ApiServerConfig {
    pub host: String,
    pub port: u16,
    /// Directory for auth.db (default: ~/.fetchium/, falls back to ~/.hypersearchx/)
    pub data_dir: PathBuf,
    /// Allowed CORS origins (default: all hypersearchx subdomains)
    pub allowed_origins: Vec<String>,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        // Use ~/.fetchium/ if it exists, otherwise fall back to legacy ~/.hypersearchx/
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let new_dir = home.join(".fetchium");
        let legacy_dir = home.join(".hypersearchx");
        let data_dir = if new_dir.exists() {
            new_dir
        } else if legacy_dir.exists() {
            legacy_dir
        } else {
            new_dir // default to new canonical location when neither exists
        };
        Self {
            host: "0.0.0.0".into(),
            port: 3000,
            data_dir,
            allowed_origins: vec![
                "https://hypersearchx.zuhabul.com".into(),
                "https://app.hypersearchx.zuhabul.com".into(),
                "http://localhost:3200".into(),
                "http://localhost:3100".into(),
            ],
        }
    }
}

/// Start the REST API server with full production middleware stack.
///
/// - CORS: restricted to configured origins only
/// - Request body limit: 1 MB
/// - Request timeout: 60 s
/// - Graceful shutdown: drains in-flight requests on SIGTERM/Ctrl-C
/// - Request ID: `X-Request-Id` injected on every response
pub async fn start_api_server(
    server_config: ApiServerConfig,
    hsx_config: fetchium_core::config::HsxConfig,
) -> anyhow::Result<()> {
    let auth_db = AuthDb::open(&server_config.data_dir.join("auth.db"))?;
    let app_state = AppState::new(hsx_config, auth_db)?;

    // ── CORS: restrict to known dashboard origins ─────────────────────────────
    let allowed: Vec<HeaderValue> = server_config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods(AllowMethods::list([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
        ]))
        .allow_headers(AllowHeaders::list([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-admin-secret"),
            HeaderName::from_static("x-request-id"),
        ]))
        .max_age(Duration::from_secs(3600));

    let x_request_id = HeaderName::from_static("x-request-id");

    let app = build_router(app_state)
        .layer(cors)
        // Propagate or generate X-Request-Id on every request/response
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(PropagateRequestIdLayer::new(x_request_id))
        // Reject bodies > 1 MB
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        // Kill requests that take longer than 60 s
        .layer(TimeoutLayer::new(Duration::from_secs(60)))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", server_config.host, server_config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Fetchium REST API listening on http://{}", addr);

    // ── Graceful shutdown: drain in-flight requests on SIGTERM or Ctrl-C ──────
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shut down gracefully");
    Ok(())
}

/// Wait for SIGTERM (systemd stop) or Ctrl-C, then return to trigger drain.
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received Ctrl-C, shutting down..."),
        _ = sigterm => tracing::info!("Received SIGTERM, shutting down..."),
    }
}
