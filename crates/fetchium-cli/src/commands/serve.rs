//! `fetchium serve` — start MCP or REST API server (PRD §9, §30).

use crate::cli::{McpTransport, ServeArgs, ServerMode};
use fetchium_core::config::HsxConfig;

pub async fn run(args: ServeArgs, config: &HsxConfig) -> anyhow::Result<()> {
    match args.mode {
        ServerMode::Mcp => {
            if args.transport == McpTransport::Stdio {
                eprintln!("Starting Fetchium MCP server (stdio transport)...");
                fetchium_mcp::run_mcp_stdio(config.clone()).await?;
            } else {
                eprintln!("SSE transport for MCP is planned for a future release.");
                eprintln!("Use `--transport stdio` for now.");
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
            eprintln!(
                "Starting REST API on port {} and MCP on stdio...",
                args.port
            );
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
            fetchium_mcp::run_mcp_stdio(config_clone).await?;
        }
    }
    Ok(())
}
