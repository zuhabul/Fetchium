//! Fetchium MCP Server — Model Context Protocol integration (PRD §30).
//!
//! Implements the MCP protocol as JSON-RPC 2.0 over stdio.
//! All log output goes to stderr; all MCP protocol output goes to stdout.
//!
//! Provides Fetchium-branded composite tools over MCP.

pub mod handlers;
pub mod tools;

use crate::tools::{
    EstimateInput, ExpandInput, FetchInput, HackerNewsSearchInput, RedditSearchInput,
    ResearchInput, SearchInput, SocialResearchInput, YouTubeAnalyzeInput, YouTubeSearchInput,
    YouTubeTranscriptInput, YouTubeWatchInput,
};
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use fetchium_core::cache::MemoryCache;
use fetchium_core::config::FetchiumConfig;
use fetchium_core::http::client::HttpClient;
use fetchium_core::summarize::SummarizeConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tower_http::trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer};

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

#[derive(Clone)]
struct McpHttpState {
    config: FetchiumConfig,
    http: HttpClient,
    cache: MemoryCache,
}

impl JsonRpcResponse {
    fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

/// Run the MCP server in stdio mode.
///
/// Reads JSON-RPC requests from stdin line by line, dispatches to handlers,
/// and writes JSON-RPC responses to stdout. All diagnostics go to stderr.
pub async fn run_mcp_stdio(config: FetchiumConfig) -> anyhow::Result<()> {
    eprintln!("[fetchium-mcp] Fetchium MCP server starting (stdio transport)");

    let http = HttpClient::new(&config)?;
    let cache = MemoryCache::new(50, 3600, true);

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) if l.trim().is_empty() => continue,
            Ok(l) => l,
            Err(e) => {
                eprintln!("[fetchium-mcp] stdin read error: {e}");
                break;
            }
        };

        let response = handle_message(&line, &config, &http, &cache).await;

        let json_out = serde_json::to_string(&response).unwrap_or_else(|e| {
            format!(
                r#"{{"jsonrpc":"2.0","id":null,"error":{{"code":-32603,"message":"{}"}}}}"#,
                e
            )
        });

        let mut out = stdout.lock();
        let _ = writeln!(out, "{json_out}");
        let _ = out.flush();
    }

    eprintln!("[fetchium-mcp] Server shutting down.");
    Ok(())
}

/// Run the MCP server over HTTP JSON-RPC on `/mcp`.
pub async fn run_mcp_http(config: FetchiumConfig, port: u16) -> anyhow::Result<()> {
    let http = HttpClient::new(&config)?;
    let cache = MemoryCache::new(50, 3600, true);
    let state = Arc::new(McpHttpState {
        config,
        http,
        cache,
    });

    let app = Router::new()
        .route("/health", get(mcp_health))
        .route("/mcp", post(mcp_http_handler))
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    eprintln!("[fetchium-mcp] Fetchium MCP server starting (http transport) on http://{addr}/mcp");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Dispatch a single JSON-RPC message line and return the response.
async fn handle_message(
    line: &str,
    config: &FetchiumConfig,
    http: &HttpClient,
    cache: &MemoryCache,
) -> JsonRpcResponse {
    let req: JsonRpcRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse::err(Value::Null, -32700, format!("Parse error: {e}"));
        }
    };

    handle_request(req, config, http, cache).await
}

async fn handle_request(
    req: JsonRpcRequest,
    config: &FetchiumConfig,
    http: &HttpClient,
    cache: &MemoryCache,
) -> JsonRpcResponse {
    let id = req.id.clone().unwrap_or(Value::Null);
    let params = req.params.unwrap_or(Value::Null);

    match req.method.as_str() {
        // MCP protocol lifecycle
        "initialize" => {
            eprintln!("[fetchium-mcp] initialize");
            JsonRpcResponse::ok(
                id,
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "fetchium",
                        "version": env!("CARGO_PKG_VERSION"),
                    }
                }),
            )
        }

        "notifications/initialized" => {
            // Notification — no response needed; return empty result
            eprintln!("[fetchium-mcp] initialized");
            JsonRpcResponse::ok(id, Value::Null)
        }

        // List available tools
        "tools/list" => JsonRpcResponse::ok(id, json!({ "tools": tools::tool_definitions() })),

        // Tool call dispatch
        "tools/call" => {
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(Value::Object(Default::default()));

            let result = dispatch_tool(tool_name, arguments, config, http, cache).await;
            JsonRpcResponse::ok(
                id,
                json!({
                    "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap_or_default() }]
                }),
            )
        }

        // Ping
        "ping" => JsonRpcResponse::ok(id, json!({})),

        other => JsonRpcResponse::err(id, -32601, format!("Method not found: {other}")),
    }
}

async fn mcp_health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "fetchium-mcp",
        "transport": "http",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn mcp_http_handler(
    State(state): State<Arc<McpHttpState>>,
    Json(req): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    let response = handle_request(req, &state.config, &state.http, &state.cache).await;
    (StatusCode::OK, Json(response))
}

/// Dispatch a `tools/call` to the appropriate handler.
async fn dispatch_tool(
    name: &str,
    args: Value,
    config: &FetchiumConfig,
    http: &HttpClient,
    cache: &MemoryCache,
) -> Value {
    match name {
        "fetchium_search" => match serde_json::from_value::<SearchInput>(args) {
            Ok(input) => handlers::handle_search(input, config, http, Some(cache)).await,
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "fetchium_fetch" => match serde_json::from_value::<FetchInput>(args) {
            Ok(input) => handlers::handle_fetch(input, config, http, Some(cache)).await,
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "fetchium_research" => match serde_json::from_value::<ResearchInput>(args) {
            Ok(input) => handlers::handle_research(input, config, http, Some(cache)).await,
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "fetchium_estimate" => match serde_json::from_value::<EstimateInput>(args) {
            Ok(input) => handlers::handle_estimate(input, config, http, Some(cache)).await,
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "fetchium_expand" => match serde_json::from_value::<ExpandInput>(args) {
            Ok(input) => handlers::handle_expand(input, config, http, Some(cache)).await,
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "youtube_search" => match serde_json::from_value::<YouTubeSearchInput>(args) {
            Ok(input) => {
                let pipeline_config = fetchium_core::youtube::types::YouTubePipelineConfig {
                    query: input.query,
                    max_videos: input.max_results.unwrap_or(5),
                    fetch_transcript: false,
                    fetch_comments: false,
                    fact_check: input.fact_check.unwrap_or(false),
                    ..Default::default()
                };
                match fetchium_core::youtube::pipeline::run_youtube_pipeline(
                    &pipeline_config,
                    config,
                    http,
                )
                .await
                {
                    Ok(result) => {
                        serde_json::to_value(&result).unwrap_or(json!({"error": "serialization"}))
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "youtube_analyze" => match serde_json::from_value::<YouTubeAnalyzeInput>(args) {
            Ok(input) => {
                match fetchium_core::youtube::pipeline::analyze_single_video(
                    &input.url,
                    config,
                    http,
                    input.comments.unwrap_or(true),
                    input.transcript.unwrap_or(true),
                    input.teaching.unwrap_or(false),
                )
                .await
                {
                    Ok(result) => {
                        serde_json::to_value(&result).unwrap_or(json!({"error": "serialization"}))
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "youtube_watch" => match serde_json::from_value::<YouTubeWatchInput>(args) {
            Ok(input) => {
                match fetchium_core::youtube::pipeline::analyze_single_video(
                    &input.url,
                    config,
                    http,
                    input.comments.unwrap_or(true),
                    input.transcript.unwrap_or(true),
                    false,
                )
                .await
                {
                    Ok(result) => {
                        let summary = fetchium_core::summarize::summarize(
                            &input.url,
                            &SummarizeConfig::default(),
                            config,
                        )
                        .await
                        .ok()
                        .map(|s| s.summary);
                        let highlights = result
                            .videos
                            .first()
                            .and_then(|v| v.transcript.as_ref())
                            .map(|t| {
                                let mut moments = t.key_moments.clone();
                                moments.sort_by(|a, b| {
                                    b.importance
                                        .partial_cmp(&a.importance)
                                        .unwrap_or(std::cmp::Ordering::Equal)
                                });
                                moments
                                    .into_iter()
                                    .take(input.highlights.unwrap_or(5))
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();
                        json!({
                            "analysis": result,
                            "summary": summary,
                            "highlights": highlights
                        })
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "youtube_transcript" => match serde_json::from_value::<YouTubeTranscriptInput>(args) {
            Ok(input) => {
                match fetchium_core::youtube::universal::fetch_universal_transcript(
                    &input.url, http, config,
                )
                .await
                {
                    Ok(transcript) => {
                        let mut highlights = transcript.key_moments.clone();
                        highlights.sort_by(|a, b| {
                            b.importance
                                .partial_cmp(&a.importance)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                        json!({
                            "transcript": transcript,
                            "highlights": highlights.into_iter().take(input.highlights.unwrap_or(5)).collect::<Vec<_>>()
                        })
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "social_research" => match serde_json::from_value::<SocialResearchInput>(args) {
            Ok(input) => {
                use fetchium_core::social::types::{SocialPipelineConfig, SocialPlatform};
                let platforms = input
                    .platforms
                    .as_deref()
                    .map(|ps| {
                        ps.iter()
                            .filter_map(|p| match p.as_str() {
                                "twitter" => Some(SocialPlatform::Twitter),
                                "reddit" => Some(SocialPlatform::Reddit),
                                "tiktok" => Some(SocialPlatform::TikTok),
                                "hackernews" | "hn" => Some(SocialPlatform::HackerNews),
                                "youtube" => Some(SocialPlatform::YouTube),
                                _ => None,
                            })
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        vec![
                            SocialPlatform::Twitter,
                            SocialPlatform::Reddit,
                            SocialPlatform::TikTok,
                            SocialPlatform::HackerNews,
                            SocialPlatform::YouTube,
                        ]
                    });
                let cfg = SocialPipelineConfig {
                    query: input.query,
                    platforms,
                    max_posts_per_platform: input.max_per_platform.unwrap_or(20),
                    include_trends: true,
                    generate_ideas: input.generate_ideas.unwrap_or(true),
                    deep_analysis: false,
                    timeout_secs: 30,
                };
                match fetchium_core::social::unified::engine::run_social_pipeline(
                    &cfg, config, http,
                )
                .await
                {
                    Ok(result) => {
                        serde_json::to_value(&result).unwrap_or(json!({"error": "serialization"}))
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "reddit_search" => match serde_json::from_value::<RedditSearchInput>(args) {
            Ok(input) => {
                use fetchium_core::social::reddit::{
                    pipeline as rd_pipeline, types::RedditPipelineConfig,
                };
                let cfg = RedditPipelineConfig {
                    query: input.query,
                    subreddits: input.subreddits.unwrap_or_default(),
                    max_posts: input.max_posts.unwrap_or(25),
                    ..Default::default()
                };
                match rd_pipeline::run_reddit_pipeline(&cfg, config, http).await {
                    Ok(result) => {
                        serde_json::to_value(&result).unwrap_or(json!({"error": "serialization"}))
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        "hackernews_search" => match serde_json::from_value::<HackerNewsSearchInput>(args) {
            Ok(input) => {
                match fetchium_core::social::hackernews::search_stories(
                    &input.query,
                    input.max_results.unwrap_or(20),
                    http,
                    15,
                )
                .await
                {
                    Ok(stories) => {
                        serde_json::to_value(&stories).unwrap_or(json!({"error": "serialization"}))
                    }
                    Err(e) => json!({ "error": e.to_string() }),
                }
            }
            Err(e) => json!({ "error": format!("Invalid input: {e}") }),
        },
        other => json!({ "error": format!("Unknown tool: {other}") }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fetchium_core::config::FetchiumConfig;

    #[test]
    fn tool_definitions_has_correct_count() {
        let tools = tools::tool_definitions();
        // 5 fetchium core + 4 YouTube + 3 social tools
        assert_eq!(tools.len(), 12);
    }

    #[test]
    fn tool_names_are_correct() {
        let tools = tools::tool_definitions();
        let names: Vec<&str> = tools
            .iter()
            .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
            .collect();
        assert!(names.contains(&"fetchium_search"));
        assert!(names.contains(&"fetchium_fetch"));
        assert!(names.contains(&"fetchium_research"));
        assert!(names.contains(&"fetchium_estimate"));
        assert!(names.contains(&"fetchium_expand"));
    }

    #[tokio::test]
    async fn initialize_request_returns_fetchium_server_info() {
        let config = FetchiumConfig::default();
        let http = HttpClient::new(&config).unwrap();
        let cache = MemoryCache::new(10, 60, true);
        let response = handle_request(
            JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(1)),
                method: "initialize".into(),
                params: Some(json!({})),
            },
            &config,
            &http,
            &cache,
        )
        .await;

        let result = response.result.unwrap();
        assert_eq!(result["serverInfo"]["name"], "fetchium");
    }
}
