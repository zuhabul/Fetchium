//! REST API request handlers (PRD §9).

use crate::handlers_auth::record_usage_async;
use crate::middleware::{AppState, AuthenticatedKey};
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use fetchium_core::citation::types::CitationStyle;
use fetchium_core::research::pipeline::ResearchPipeline;
use fetchium_core::research::ResearchConfig;
use fetchium_core::validate::types::ValidationMode;
use std::future::Future;
use std::time::Instant;
use tokio::time::{timeout, Duration};

type ApiResult<T> = Result<Json<T>, (StatusCode, Json<ApiError>)>;
type ApiResponse = Result<axum::response::Response, (StatusCode, Json<ApiError>)>;

const SEARCH_TIMEOUT_SECS: u64 = 60;
const FETCH_TIMEOUT_SECS: u64 = 30;
const ESTIMATE_TIMEOUT_SECS: u64 = 10;
const RESEARCH_TIMEOUT_SECS: u64 = 90;
const YOUTUBE_TIMEOUT_SECS: u64 = 45;
const SOCIAL_TIMEOUT_SECS: u64 = 45;

fn request_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

#[allow(clippy::too_many_arguments)]
fn response_meta(
    request_id: String,
    endpoint: &str,
    duration_ms: u64,
    query: Option<String>,
    tier: Option<String>,
    tokens_used: Option<usize>,
    sources_count: Option<usize>,
    result_id: Option<String>,
) -> ResponseMeta {
    ResponseMeta {
        request_id,
        status: "ok".into(),
        endpoint: endpoint.into(),
        duration_ms,
        query,
        tier,
        tokens_used,
        sources_count,
        result_id,
    }
}

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

fn api_timeout(endpoint: &str, secs: u64) -> (StatusCode, Json<ApiError>) {
    api_err(
        StatusCode::GATEWAY_TIMEOUT,
        "request_timeout",
        format!(
            "{endpoint} exceeded the API timeout ({secs}s). Retry later or use async/streaming mode."
        ),
    )
}

async fn run_with_timeout<T, F>(endpoint: &'static str, secs: u64, fut: F) -> ApiResult<T>
where
    F: Future<Output = ApiResult<T>>,
{
    match timeout(Duration::from_secs(secs), fut).await {
        Ok(result) => result,
        Err(_) => Err(api_timeout(endpoint, secs)),
    }
}

fn job_accepted(request_id: String, job_id: &str, endpoint: &str) -> axum::response::Response {
    (
        StatusCode::ACCEPTED,
        Json(JobAcceptedResponse {
            meta: response_meta(
                request_id,
                endpoint,
                0,
                None,
                None,
                None,
                None,
                Some(job_id.to_string()),
            ),
            job_id: job_id.to_string(),
            status: JobState::Queued,
            status_url: format!("/v1/jobs/{job_id}"),
        }),
    )
        .into_response()
}

fn enqueue_job(state: &AppState, key_id: &str, job_type: &str) -> String {
    let job_id = uuid::Uuid::new_v4().to_string();
    state
        .jobs
        .create(key_id.to_string(), job_id.clone(), job_type.to_string());
    job_id
}

/// GET /health — health check endpoint.
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// POST /v1/search — multi-backend search pipeline.
pub async fn search(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SearchRequest>,
) -> ApiResult<SearchResponse> {
    run_with_timeout("/v1/search", SEARCH_TIMEOUT_SECS, async move {
        let req = req
            .validated()
            .map_err(|e| api_err(StatusCode::BAD_REQUEST, "invalid_request", e.to_string()))?;

        // Acquire search concurrency permit (queues if too many concurrent searches)
        let sem = state.search_semaphore.clone();
        let _permit = sem.acquire().await.map_err(|_| {
            api_err(
                StatusCode::SERVICE_UNAVAILABLE,
                "overloaded",
                "Too many concurrent searches — try again shortly".to_string(),
            )
        })?;

        let start = Instant::now();
        let max_sources = req.max_sources.unwrap_or(10) as u32;
        let tier = req.tier.as_deref().unwrap_or("summary");
        let token_budget = req.token_budget.unwrap_or(2000);

        let include_content = req.include_content.unwrap_or(false);
        let result_json = fetchium_core::api_facade::search(
            fetchium_core::api_facade::SearchRequest {
                query: &req.query,
                max_sources,
                tier,
                token_budget,
                include_content,
            },
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

        let response: SearchResponse = serde_json::from_value(result_json).map_err(|e| {
            api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "serialization_error",
                e.to_string(),
            )
        })?;
        let request_id = request_id_from_headers(&headers);
        let result_id = response.meta.result_id.clone();
        let duration_ms = start.elapsed().as_millis() as u64;
        let response = SearchResponse {
            meta: response_meta(
                request_id,
                "/v1/search",
                duration_ms,
                Some(req.query.clone()),
                Some(tier.to_string()),
                Some(response.meta.tokens_used.unwrap_or(token_budget)),
                Some(response.results.len()),
                result_id,
            ),
            results: response.results,
        };

        record_usage_async(
            state,
            key.id,
            "/v1/search",
            200,
            response.meta.tokens_used.unwrap_or(token_budget) as u64,
            start,
        );
        Ok(Json(response))
    })
    .await
}

async fn fetch_impl(
    key: fetchium_core::error::FetchiumResult<String>,
    state: AppState,
    headers: HeaderMap,
    req: FetchRequest,
    usage_endpoint: &'static str,
) -> ApiResult<FetchResponse> {
    let key_id = key.map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "auth_state_error",
            e.to_string(),
        )
    })?;
    let req = req
        .validated()
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, "invalid_request", e.to_string()))?;
    let start = Instant::now();
    let budget = req.token_budget.unwrap_or(3000);
    let format = req.format.as_deref().unwrap_or("markdown");

    let result_json = fetchium_core::api_facade::fetch(
        &req.url,
        budget,
        format,
        &state.http,
        Some(&state.cache),
        req.schema.as_ref(),
    )
    .await
    .map_err(|e| api_err(StatusCode::BAD_REQUEST, "fetch_failed", e.to_string()))?;

    let response: FetchResponse = serde_json::from_value(result_json).map_err(|e| {
        api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "serialization_error",
            e.to_string(),
        )
    })?;
    let duration_ms = start.elapsed().as_millis() as u64;
    let response = FetchResponse {
        meta: response_meta(
            request_id_from_headers(&headers),
            usage_endpoint,
            duration_ms,
            req.query.clone(),
            Some(format.to_string()),
            Some(response.tokens),
            None,
            Some(response.result_id.clone()),
        ),
        url: response.url,
        title: response.title,
        content: response.content,
        tokens: response.tokens,
        format: response.format,
        result_id: response.result_id,
    };

    record_usage_async(
        state,
        key_id,
        usage_endpoint,
        200,
        response.tokens as u64,
        start,
    );
    Ok(Json(response))
}

/// POST /v1/scrape — fetch and extract a URL.
pub async fn scrape(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<FetchRequest>,
) -> ApiResult<FetchResponse> {
    run_with_timeout("/v1/scrape", FETCH_TIMEOUT_SECS, async move {
        fetch_impl(Ok(key.id), state, headers, req, "/v1/scrape").await
    })
    .await
}

/// POST /v1/fetch — alias for scrape with distinct usage accounting.
pub async fn fetch(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<FetchRequest>,
) -> ApiResult<FetchResponse> {
    run_with_timeout("/v1/fetch", FETCH_TIMEOUT_SECS, async move {
        fetch_impl(Ok(key.id), state, headers, req, "/v1/fetch").await
    })
    .await
}

/// POST /v1/research — full multi-source research pipeline.
pub async fn research(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ResearchRequest>,
) -> ApiResult<ResearchResponse> {
    run_with_timeout("/v1/research", RESEARCH_TIMEOUT_SECS, async move {
        let req = req
            .validated()
            .map_err(|e| api_err(StatusCode::BAD_REQUEST, "invalid_request", e.to_string()))?;
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
            ai_synthesis: true,
            thinking: false,
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

        let tokens_used = req.token_budget.unwrap_or(4000);
        let response = ResearchResponse {
            meta: ResponseMeta {
                request_id: request_id_from_headers(&headers),
                status: "ok".into(),
                endpoint: "/v1/research".into(),
                duration_ms,
                query: Some(report.query),
                tier: Some("detailed".into()),
                tokens_used: Some(tokens_used),
                sources_count: Some(report.meta.sources_fetched),
                result_id: Some(uuid::Uuid::new_v4().to_string()),
            },
            report: report.synthesis,
            reference_section: report.reference_section,
            sources,
            confidence: report.meta.overall_confidence,
        };
        record_usage_async(
            state,
            key.id,
            "/v1/research",
            200,
            tokens_used as u64,
            start,
        );
        Ok(Json(response))
    })
    .await
}

/// POST /v1/youtube/search — YouTube video search with VideoFusion ranking.
pub async fn youtube_search(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::YouTubeSearchRequest>,
) -> ApiResult<DataResponse<serde_json::Value>> {
    run_with_timeout("/v1/youtube/search", YOUTUBE_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let query = req.query.clone();
        let pipeline_config = fetchium_core::youtube::types::YouTubePipelineConfig {
            query: req.query,
            max_videos: req.max_results.unwrap_or(5),
            fetch_transcript: false,
            fetch_comments: false,
            fact_check: req.fact_check.unwrap_or(false),
            ..Default::default()
        };

        let result = fetchium_core::youtube::pipeline::run_youtube_pipeline(
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

        record_usage_async(state, key.id, "/v1/youtube/search", 200, 0, start);
        Ok(Json(DataResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/youtube/search",
                start.elapsed().as_millis() as u64,
                Some(query),
                None,
                Some(0),
                None,
                None,
            ),
            data: json,
        }))
    })
    .await
}

/// POST /v1/youtube/analyze — single YouTube video deep analysis.
pub async fn youtube_analyze(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::YouTubeAnalyzeRequest>,
) -> ApiResult<DataResponse<serde_json::Value>> {
    run_with_timeout("/v1/youtube/analyze", YOUTUBE_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let url = req.url.clone();
        let result = fetchium_core::youtube::pipeline::analyze_single_video(
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

        record_usage_async(state, key.id, "/v1/youtube/analyze", 200, 0, start);
        Ok(Json(DataResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/youtube/analyze",
                start.elapsed().as_millis() as u64,
                Some(url),
                None,
                Some(0),
                None,
                None,
            ),
            data: json,
        }))
    })
    .await
}

/// POST /v1/social/research — unified cross-platform social research.
pub async fn social_research(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::SocialResearchRequest>,
) -> ApiResult<DataResponse<serde_json::Value>> {
    run_with_timeout("/v1/social/research", SOCIAL_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let query = req.query.clone();
        use fetchium_core::social::types::{SocialPipelineConfig, SocialPlatform};

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
            timeout_secs: SOCIAL_TIMEOUT_SECS,
        };

        let result = fetchium_core::social::unified::engine::run_social_pipeline(
            &cfg,
            &state.config,
            &state.http,
        )
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
        record_usage_async(state, key.id, "/v1/social/research", 200, 0, start);
        Ok(Json(DataResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/social/research",
                start.elapsed().as_millis() as u64,
                Some(query),
                None,
                Some(0),
                None,
                None,
            ),
            data: json,
        }))
    })
    .await
}

/// POST /v1/social/reddit — Reddit search.
pub async fn reddit_search(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::RedditSearchRequest>,
) -> ApiResult<DataResponse<serde_json::Value>> {
    run_with_timeout("/v1/social/reddit", SOCIAL_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let query = req.query.clone();
        use fetchium_core::social::reddit::{pipeline as rd, types::RedditPipelineConfig};

        let cfg = RedditPipelineConfig {
            query: req.query,
            subreddits: req.subreddits.unwrap_or_default(),
            max_posts: req.max_posts.unwrap_or(25),
            timeout_secs: SOCIAL_TIMEOUT_SECS,
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
        record_usage_async(state, key.id, "/v1/social/reddit", 200, 0, start);
        Ok(Json(DataResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/social/reddit",
                start.elapsed().as_millis() as u64,
                Some(query),
                None,
                Some(0),
                None,
                None,
            ),
            data: json,
        }))
    })
    .await
}

/// POST /v1/social/hackernews — HackerNews search.
pub async fn hackernews_search(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::HackerNewsSearchRequest>,
) -> ApiResult<DataResponse<serde_json::Value>> {
    run_with_timeout("/v1/social/hackernews", SOCIAL_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let query = req.query.clone();
        let stories = fetchium_core::social::hackernews::search_stories(
            &req.query,
            req.max_results.unwrap_or(20),
            &state.http,
            SOCIAL_TIMEOUT_SECS,
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
        record_usage_async(state, key.id, "/v1/social/hackernews", 200, 0, start);
        Ok(Json(DataResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/social/hackernews",
                start.elapsed().as_millis() as u64,
                Some(query),
                None,
                Some(0),
                None,
                None,
            ),
            data: json,
        }))
    })
    .await
}

/// POST /v1/estimate — heuristic token cost estimation.
pub async fn estimate(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<EstimateRequest>,
) -> ApiResult<EstimateResponse> {
    run_with_timeout("/v1/estimate", ESTIMATE_TIMEOUT_SECS, async move {
        let start = Instant::now();
        let client = state.http.client();
        let head = client.head(&req.url).send().await;

        let (estimated_tokens, content_type) = match head {
            Ok(resp) if resp.status().is_success() => {
                let content_type = resp
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("text/html")
                    .to_string();
                let content_length = resp.content_length().unwrap_or(32_000);
                let estimated_tokens = (content_length as usize / 4).clamp(128, 12_000);
                (estimated_tokens, content_type)
            }
            Ok(resp) => {
                return Err(api_err(
                    StatusCode::BAD_REQUEST,
                    "estimate_failed",
                    format!("HTTP {} from {}", resp.status(), req.url),
                ));
            }
            Err(_) => (2000, "text/html".to_string()),
        };

        let response = EstimateResponse {
            meta: response_meta(
                request_id_from_headers(&headers),
                "/v1/estimate",
                start.elapsed().as_millis() as u64,
                Some(req.url.clone()),
                None,
                Some(estimated_tokens),
                None,
                None,
            ),
            url: req.url,
            estimated_tokens,
            estimated_relevant_tokens: estimated_tokens / 2,
            extraction_layer: 1,
            content_type,
        };
        record_usage_async(state, key.id, "/v1/estimate", 200, 0, start);
        Ok(Json(response))
    })
    .await
}

pub async fn get_job_status(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
) -> ApiResponse {
    let Some(mut job) = state.jobs.get_owned(&job_id, &key.id) else {
        return Err(api_err(
            StatusCode::NOT_FOUND,
            "job_not_found",
            format!("No job found for id {job_id}"),
        ));
    };
    job.meta.request_id = request_id_from_headers(&headers);
    job.meta.endpoint = "/v1/jobs/:id".into();

    let status = match job.status {
        JobState::Queued | JobState::Running => StatusCode::ACCEPTED,
        JobState::Completed => StatusCode::OK,
        JobState::Failed => StatusCode::OK,
    };
    Ok((status, Json(job)).into_response())
}

pub async fn submit_research_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ResearchRequest>,
) -> ApiResponse {
    let req = req
        .validated()
        .map_err(|e| api_err(StatusCode::BAD_REQUEST, "invalid_request", e.to_string()))?;
    let job_id = enqueue_job(&state, &key.id, "research");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
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
                .unwrap_or(state_clone.config.general.max_results as usize),
            token_budget: req.token_budget,
            citation_style,
            validation_mode: ValidationMode::Standard,
            strict_evidence: req.strict_evidence.unwrap_or(false),
            evidence_graph: false,
            trace_sources: false,
            trust_verify: false,
            max_rar_loops: 2,
            ai_synthesis: true,
            thinking: false,
        };
        match ResearchPipeline::execute(&rc, &state_clone.config, &state_clone.http).await {
            Ok(report) => {
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
                let payload = ResearchResponse {
                    meta: ResponseMeta {
                        request_id: job_id_clone.clone(),
                        status: "completed".into(),
                        endpoint: "/v1/research/jobs".into(),
                        duration_ms: report.meta.duration_ms,
                        query: Some(report.query),
                        tier: Some("detailed".into()),
                        tokens_used: Some(req.token_budget.unwrap_or(4000)),
                        sources_count: Some(report.meta.sources_fetched),
                        result_id: Some(uuid::Uuid::new_v4().to_string()),
                    },
                    report: report.synthesis,
                    reference_section: report.reference_section,
                    sources,
                    confidence: report.meta.overall_confidence,
                };
                match serde_json::to_value(payload) {
                    Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                    Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
                }
            }
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/research/jobs",
    ))
}

pub async fn submit_youtube_search_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::YouTubeSearchRequest>,
) -> ApiResponse {
    let job_id = enqueue_job(&state, &key.id, "youtube_search");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
        let pipeline_config = fetchium_core::youtube::types::YouTubePipelineConfig {
            query: req.query,
            max_videos: req.max_results.unwrap_or(5),
            fetch_transcript: false,
            fetch_comments: false,
            fact_check: req.fact_check.unwrap_or(false),
            ..Default::default()
        };
        match fetchium_core::youtube::pipeline::run_youtube_pipeline(
            &pipeline_config,
            &state_clone.config,
            &state_clone.http,
        )
        .await
        {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
            },
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/youtube/search/jobs",
    ))
}

pub async fn submit_youtube_analyze_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::YouTubeAnalyzeRequest>,
) -> ApiResponse {
    let job_id = enqueue_job(&state, &key.id, "youtube_analyze");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
        match fetchium_core::youtube::pipeline::analyze_single_video(
            &req.url,
            &state_clone.config,
            &state_clone.http,
            req.comments.unwrap_or(true),
            req.transcript.unwrap_or(true),
            req.teaching.unwrap_or(false),
        )
        .await
        {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
            },
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/youtube/analyze/jobs",
    ))
}

pub async fn submit_social_research_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::SocialResearchRequest>,
) -> ApiResponse {
    let job_id = enqueue_job(&state, &key.id, "social_research");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
        use fetchium_core::social::types::{SocialPipelineConfig, SocialPlatform};
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
            timeout_secs: SOCIAL_TIMEOUT_SECS,
        };
        match fetchium_core::social::unified::engine::run_social_pipeline(
            &cfg,
            &state_clone.config,
            &state_clone.http,
        )
        .await
        {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
            },
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/social/research/jobs",
    ))
}

pub async fn submit_reddit_search_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::RedditSearchRequest>,
) -> ApiResponse {
    let job_id = enqueue_job(&state, &key.id, "reddit_search");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
        use fetchium_core::social::reddit::{pipeline as rd, types::RedditPipelineConfig};
        let cfg = RedditPipelineConfig {
            query: req.query,
            subreddits: req.subreddits.unwrap_or_default(),
            max_posts: req.max_posts.unwrap_or(25),
            timeout_secs: SOCIAL_TIMEOUT_SECS,
            ..Default::default()
        };
        match rd::run_reddit_pipeline(&cfg, &state_clone.config, &state_clone.http).await {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
            },
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/social/reddit/jobs",
    ))
}

pub async fn submit_hackernews_search_job(
    AuthenticatedKey(key): AuthenticatedKey,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::types::HackerNewsSearchRequest>,
) -> ApiResponse {
    let job_id = enqueue_job(&state, &key.id, "hackernews_search");
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        state_clone.jobs.mark_running(&job_id_clone);
        match fetchium_core::social::hackernews::search_stories(
            &req.query,
            req.max_results.unwrap_or(20),
            &state_clone.http,
            SOCIAL_TIMEOUT_SECS,
        )
        .await
        {
            Ok(result) => match serde_json::to_value(result) {
                Ok(value) => state_clone.jobs.complete(&job_id_clone, value),
                Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
            },
            Err(e) => state_clone.jobs.fail(&job_id_clone, e.to_string()),
        }
    });
    Ok(job_accepted(
        request_id_from_headers(&headers),
        &job_id,
        "/v1/social/hackernews/jobs",
    ))
}
