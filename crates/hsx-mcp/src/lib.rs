//! HyperSearchX MCP Server — Model Context Protocol integration (PRD §30).
//!
//! Implements the MCP protocol as JSON-RPC 2.0 over stdio.
//! All log output goes to stderr; all MCP protocol output goes to stdout.
//!
//! Provides 5 composite tools:
//! - `hypersearch_search`   — multi-backend search + ranking
//! - `hypersearch_fetch`    — URL fetching + CEP extraction
//! - `hypersearch_research` — full research pipeline with citations
//! - `hypersearch_estimate` — token cost estimation
//! - `hypersearch_expand`   — PDS tier expansion of previous results

pub mod handlers;
pub mod tools;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use hsx_core::config::HsxConfig;
use hsx_core::http::client::HttpClient;
use hsx_core::cache::MemoryCache;
use crate::tools::{EstimateInput, ExpandInput, FetchInput, ResearchInput, SearchInput};

// ─── JSON-RPC 2.0 types ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn ok(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }

    fn err(id: Value, code: i32, message: String) -> Self {
        Self { jsonrpc: "2.0", id, result: None, error: Some(JsonRpcError { code, message }) }
    }
}

/// Run the MCP server in stdio mode.
///
/// Reads JSON-RPC requests from stdin line by line, dispatches to handlers,
/// and writes JSON-RPC responses to stdout. All diagnostics go to stderr.
pub async fn run_mcp_stdio(config: HsxConfig) -> anyhow::Result<()> {
    eprintln!("[hsx-mcp] HyperSearchX MCP server starting (stdio transport)");

    let http = HttpClient::new(&config)?;
    let cache = MemoryCache::new(50, 3600, true);
    
    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(e) => {
                eprintln!("[hsx-mcp] stdin read error: {e}");
                break;
            }
        };

        let response = handle_message(&line, &config, &http, &cache).await;

        let json_out = serde_json::to_string(&response).unwrap_or_else(|e| {
            format!(r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":-32603,"message":"{}"}}}}"#, e)
        });

        let mut out = stdout.lock();
        let _ = writeln!(out, "{json_out}");
        let _ = out.flush();
    }

    eprintln!("[hsx-mcp] Server shutting down.");
    Ok(())
}

/// Dispatch a single JSON-RPC message line and return the response.
async fn handle_message(line: &str, config: &HsxConfig, http: &HttpClient, cache: &MemoryCache) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse::err(
                Value::Null,
                -32700,
                format!("Parse error: {e}"),
            );
        }
    };

    let id = req.id.clone().unwrap_or(Value::Null);
    let params = req.params.unwrap_or(Value::Null);

    match req.method.as_str() {
        // MCP protocol lifecycle
        "initialize" => {
            eprintln!("[hsx-mcp] initialize");
            JsonRpcResponse::ok(id, json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "hypersearchx",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }))
        }

        "notifications/initialized" => {
            // Notification — no response needed; return empty result
            eprintln!("[hsx-mcp] initialized");
            JsonRpcResponse::ok(id, Value::Null)
        }

        // List available tools
        "tools/list" => {
            JsonRpcResponse::ok(id, json!({ "tools": tools::tool_definitions() }))
        }

        // Tool call dispatch
        "tools/call" => {
            let tool_name = params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));

            let result = dispatch_tool(tool_name, arguments, config, http, cache).await;
            JsonRpcResponse::ok(id, json!({
                "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap_or_default() }]
            }))
        }

        // Ping
        "ping" => JsonRpcResponse::ok(id, json!({})),

        other => {
            JsonRpcResponse::err(id, -32601, format!("Method not found: {other}"))
        }
    }
}

/// Dispatch a `tools/call` to the appropriate handler.
async fn dispatch_tool(name: &str, args: Value, config: &HsxConfig, http: &HttpClient, cache: &MemoryCache) -> Value {
    match name {
        "hypersearch_search" => {
            match serde_json::from_value::<SearchInput>(args) {
                Ok(input) => handlers::handle_search(input, config, http, Some(cache)).await,
                Err(e) => json!({ "error": format!("Invalid input: {e}") }),
            }
        }
        "hypersearch_fetch" => {
            match serde_json::from_value::<FetchInput>(args) {
                Ok(input) => handlers::handle_fetch(input, config, http, Some(cache)).await,
                Err(e) => json!({ "error": format!("Invalid input: {e}") }),
            }
        }
        "hypersearch_research" => {
            match serde_json::from_value::<ResearchInput>(args) {
                Ok(input) => handlers::handle_research(input, config, http, Some(cache)).await,
                Err(e) => json!({ "error": format!("Invalid input: {e}") }),
            }
        }
        "hypersearch_estimate" => {
            match serde_json::from_value::<EstimateInput>(args) {
                Ok(input) => handlers::handle_estimate(input, config, http, Some(cache)).await,
                Err(e) => json!({ "error": format!("Invalid input: {e}") }),
            }
        }
        "hypersearch_expand" => {
            match serde_json::from_value::<ExpandInput>(args) {
                Ok(input) => handlers::handle_expand(input, config, http, Some(cache)).await,
                Err(e) => json!({ "error": format!("Invalid input: {e}") }),
            }
        }
        other => json!({ "error": format!("Unknown tool: {other}") }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definitions_has_five_tools() {
        let tools = tools::tool_definitions();
        assert_eq!(tools.len(), 5);
    }

    #[test]
    fn tool_names_are_correct() {
        let tools = tools::tool_definitions();
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"hypersearch_search"));
        assert!(names.contains(&"hypersearch_fetch"));
        assert!(names.contains(&"hypersearch_research"));
        assert!(names.contains(&"hypersearch_estimate"));
        assert!(names.contains(&"hypersearch_expand"));
    }
}
