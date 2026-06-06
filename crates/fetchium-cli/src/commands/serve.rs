//! `fetchium serve` — start MCP or REST API server (PRD §9, §30).

use crate::cli::{McpTransport, ServeArgs, ServerMode};
use fetchium_core::config::FetchiumConfig;

pub async fn run(args: ServeArgs, config: &FetchiumConfig) -> anyhow::Result<()> {
    match args.mode {
        ServerMode::Mcp => {
            if args.transport == McpTransport::Stdio {
                eprintln!("Starting Fetchium MCP server (stdio transport)...");
                fetchium_mcp::run_mcp_stdio(config.clone()).await?;
            } else {
                eprintln!("Starting Fetchium MCP server (HTTP transport) on /mcp...");
                fetchium_mcp::run_mcp_http(config.clone(), args.port).await?;
            }
        }

        ServerMode::Rest => {
            eprintln!(
                "Starting Fetchium REST API on http://0.0.0.0:{}...",
                args.port
            );
            let server_config = fetchium_api::ApiServerConfig {
                host: "0.0.0.0".into(),
                port: args.port,
                data_dir: config.data_dir(),
                ..Default::default()
            };
            fetchium_api::start_api_server(server_config, config.clone()).await?;
        }

        ServerMode::Both => {
            eprintln!("Starting REST API on port {}...", args.port);
            let config_clone = config.clone();
            let config2 = config.clone();
            let port = args.port;
            tokio::spawn(async move {
                let server_config = fetchium_api::ApiServerConfig {
                    host: "0.0.0.0".into(),
                    port,
                    data_dir: config2.data_dir(),
                    ..Default::default()
                };
                if let Err(e) = fetchium_api::start_api_server(server_config, config2).await {
                    eprintln!("REST API error: {e}");
                }
            });

            if args.transport == McpTransport::Stdio {
                eprintln!("Starting Fetchium MCP server (stdio transport)...");
                fetchium_mcp::run_mcp_stdio(config_clone).await?;
            } else {
                let mcp_port = args.port.saturating_add(1);
                eprintln!(
                    "Starting Fetchium MCP server (HTTP transport) on port {}...",
                    mcp_port
                );
                fetchium_mcp::run_mcp_http(config_clone, mcp_port).await?;
            }
        }
    }
    Ok(())
}
