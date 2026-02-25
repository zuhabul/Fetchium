//! REST API request handlers (PRD §9).

use crate::middleware::AppState;
use crate::types::*;
use axum::{extract::State, http::StatusCode, Json};
use hsx_core::citation::types::CitationStyle;
use hsx_core::research::pipeline::ResearchPipeline;
use hsx_core::research::ResearchConfig;
use hsx_core::validate::types::ValidationMode;
use std::sync::Arc;
use std::time::Instant;

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ApiError>)>;

fn api_err(status: StatusCode, err_type: &str, msg: String) -> (StatusCode, Json<ApiError>) {
    (
        status,
        Json(ApiError {
            error: msg,
            error_type: err_type.into(),
            status: status.as_u16(),
        }),
    )
}

/// GET /health — health check endpoint.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// POST /api/search — multi-backend search pipeline.
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> ApiResult<SearchResponse> {
    let max_sources = req.max_sources.unwrap_or(10) as u32;
    let tier = req.tier.as_deref().unwrap_or("summary");
    let token_budget = req.token_budget.unwrap_or(2000);

    let result_json = hsx_core::api_facade::search(
        &req.query,
        max_sources,
        tier,
        token_budget,
        &state.config,
        &state.http,
        Some(&state.cache),
    )
    .await
    .map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "search_failed",
            e.to_string(),
        )
    })?;

    // Deserialize back into SearchResponse or just return Json(result_json)
    // Note: since our SearchResponse type strictly maps, we can just return Json(result_json) if we change the return type.
    // For now, let's deserialize it to ensure the API contract matches:
    let response: SearchResponse = serde_json::from_value(result_json).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;

    Ok(Json(response))
}

/// POST /api/fetch — fetch and extract a URL.
pub async fn fetch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FetchRequest>,
) -> ApiResult<FetchResponse> {
    let budget = req.token_budget.unwrap_or(3000);
    let format = req.format.as_deref().unwrap_or("markdown");

    let result_json =
        hsx_core::api_facade::fetch(&req.url, budget, format, &state.http, Some(&state.cache))
            .await
            .map_err(|e| api_err(StatusCode::BAD_REQUEST, "fetch_failed", e.to_string()))?;

    let response: FetchResponse = serde_json::from_value(result_json).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;

    Ok(Json(response))
}

/// POST /api/research — full multi-source research pipeline.
pub async fn research(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResearchRequest>,
) -> ApiResult<ResearchResponse> {
    let start = Instant::now();

    let citation_style = match req.citation_style.as_deref() {
        Some("apa") => CitationStyle::Apa,
        Some("ieee") => CitationStyle::Ieee,
        Some("mla") => CitationStyle::Mla,
        Some("chicago") => CitationStyle::Chicago,
        Some("bibtex") => CitationStyle::Bibtex,
        _ => CitationStyle::Inline,
    };

    let rc = ResearchConfig {
        query: req.query.clone(),
        max_sources: req
            .max_sources
            .unwrap_or(state.config.general.max_results as usize),
        token_budget: req.token_budget,
        citation_style,
        validation_mode: ValidationMode::Standard,
        strict_evidence: req.strict_evidence.unwrap_or(false),
        evidence_graph: false,
        trace_sources: false,
        trust_verify: false,
        max_rar_loops: 2,
    };

    let report = ResearchPipeline::execute(&rc, &state.config, &state.http)
        .await
        .map_err(|e| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "research_failed",
                e.to_string(),
            )
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let sources: Vec<SourceInfo> = report
        .sources
        .iter()
        .enumerate()
        .map(|(i, s)| SourceInfo {
            index: i + 1,
            title: s.title.clone(),
            url: s.url.clone(),
        })
        .collect();

    Ok(Json(ResearchResponse {
        meta: ResponseMeta {
            query: report.query,
            tier: "detailed".into(),
            tokens_used: req.token_budget.unwrap_or(4000),
            sources_count: report.meta.sources_fetched,
            duration_ms,
            result_id: uuid::Uuid::new_v4().to_string(),
        },
        report: report.synthesis,
        reference_section: report.reference_section,
        sources,
        confidence: report.meta.overall_confidence,
    }))
}

/// POST /api/youtube/search — YouTube video search with VideoFusion ranking.
pub async fn youtube_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::types::YouTubeSearchRequest>,
) -> ApiResult<serde_json::Value> {
    let pipeline_config = hsx_core::youtube::types::YouTubePipelineConfig {
        query: req.query,
        max_videos: req.max_results.unwrap_or(5),
        fetch_transcript: false,
        fetch_comments: false,
        fact_check: req.fact_check.unwrap_or(false),
        ..Default::default()
    };

    let result = hsx_core::youtube::pipeline::run_youtube_pipeline(
        &pipeline_config,
        &state.config,
        &state.http,
    )
    .await
    .map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "youtube_search_failed",
            e.to_string(),
        )
    })?;

    let json = serde_json::to_value(&result).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;

    Ok(Json(json))
}

/// POST /api/youtube/analyze — single YouTube video deep analysis.
pub async fn youtube_analyze(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::types::YouTubeAnalyzeRequest>,
) -> ApiResult<serde_json::Value> {
    let result = hsx_core::youtube::pipeline::analyze_single_video(
        &req.url,
        &state.config,
        &state.http,
        req.comments.unwrap_or(true),
        req.transcript.unwrap_or(true),
        req.teaching.unwrap_or(false),
    )
    .await
    .map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "youtube_analyze_failed",
            e.to_string(),
        )
    })?;

    let json = serde_json::to_value(&result).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;

    Ok(Json(json))
}

/// POST /api/social/research — unified cross-platform social research.
pub async fn social_research(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::types::SocialResearchRequest>,
) -> ApiResult<serde_json::Value> {
    use hsx_core::social::types::{SocialPipelineConfig, SocialPlatform};

    let platforms = req
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
        query: req.query,
        platforms,
        max_posts_per_platform: req.max_per_platform.unwrap_or(20),
        include_trends: true,
        generate_ideas: req.generate_ideas.unwrap_or(true),
        deep_analysis: false,
        timeout_secs: 30,
    };

    let result =
        hsx_core::social::unified::engine::run_social_pipeline(&cfg, &state.config, &state.http)
            .await
            .map_err(|e| {
                api_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "social_research_failed",
                    e.to_string(),
                )
            })?;

    let json = serde_json::to_value(&result).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;
    Ok(Json(json))
}

/// POST /api/social/reddit — Reddit search.
pub async fn reddit_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::types::RedditSearchRequest>,
) -> ApiResult<serde_json::Value> {
    use hsx_core::social::reddit::{pipeline as rd, types::RedditPipelineConfig};

    let cfg = RedditPipelineConfig {
        query: req.query,
        subreddits: req.subreddits.unwrap_or_default(),
        max_posts: req.max_posts.unwrap_or(25),
        ..Default::default()
    };

    let result = rd::run_reddit_pipeline(&cfg, &state.config, &state.http)
        .await
        .map_err(|e| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "reddit_search_failed",
                e.to_string(),
            )
        })?;

    let json = serde_json::to_value(&result).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;
    Ok(Json(json))
}

/// GET /api/social/hackernews — HackerNews search.
pub async fn hackernews_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::types::HackerNewsSearchRequest>,
) -> ApiResult<serde_json::Value> {
    let stories = hsx_core::social::hackernews::search_stories(
        &req.query,
        req.max_results.unwrap_or(20),
        &state.http,
        15,
    )
    .await
    .map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "hn_search_failed",
            e.to_string(),
        )
    })?;

    let json = serde_json::to_value(&stories).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;
    Ok(Json(json))
}

/// POST /api/estimate — heuristic token cost estimation.
pub async fn estimate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EstimateRequest>,
) -> ApiResult<EstimateResponse> {
    let html = state
        .http
        .fetch_text(&req.url)
        .await
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, "estimate_failed", e.to_string()))?;

    let raw_tokens = hsx_core::extract::layer1::estimate_tokens(&html) as usize;
    let estimated_tokens = raw_tokens / 4; // post-extraction estimate

    Ok(Json(EstimateResponse {
        url: req.url,
        estimated_tokens,
        estimated_relevant_tokens: estimated_tokens / 2,
        extraction_layer: 1,
        content_type: "text/html".into(),
    }))
}
