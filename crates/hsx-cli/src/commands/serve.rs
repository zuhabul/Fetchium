//! `hsx serve` — start MCP or REST API server (PRD §9, §30).

use crate::cli::{McpTransport, ServeArgs, ServerMode};
use hsx_core::config::HsxConfig;

pub async fn run(args: ServeArgs, config: &HsxConfig) -> anyhow::Result<()> {
    match args.mode {
        ServerMode::Mcp => {
            if args.transport == McpTransport::Stdio {
                eprintln!("Starting HyperSearchX MCP server (stdio transport)...");
                hsx_mcp::run_mcp_stdio(config.clone()).await?;
            } else {
                eprintln!("SSE transport for MCP is planned for a future release.");
                eprintln!("Use `--transport stdio` for now.");
            }
        }

        ServerMode::Rest => {
            eprintln!(
                "Starting HyperSearchX REST API on http://0.0.0.0:{}...",
                args.port
            );
            let state = std::sync::Arc::new(hsx_api::middleware::AppState::new(config.clone())?);
            let server_config = hsx_api::ApiServerConfig {
                host: "0.0.0.0".into(),
                port: args.port,
            };
            hsx_api::start_api_server(server_config, state).await?;
        }

        ServerMode::Both => {
            eprintln!(
                "Starting REST API on port {} and MCP on stdio...",
                args.port
            );
            // Run REST API in background, MCP on stdio
            let config_clone = config.clone();
            let port = args.port;
            let state = std::sync::Arc::new(hsx_api::middleware::AppState::new(config.clone())?);
            tokio::spawn(async move {
                let server_config = hsx_api::ApiServerConfig {
                    host: "0.0.0.0".into(),
                    port,
                };
                if let Err(e) = hsx_api::start_api_server(server_config, state).await {
                    eprintln!("REST API error: {e}");
                }
            });
            hsx_mcp::run_mcp_stdio(config_clone).await?;
        }
    }
    Ok(())
}
