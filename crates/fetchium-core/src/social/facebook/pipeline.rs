//! Facebook intelligence pipeline.

use crate::error::FetchiumResult;
use crate::http::client::HttpClient;
use crate::social::facebook::types::*;
use crate::social::facebook::{analysis, search};
use std::time::Instant;

/// Run the Facebook intelligence pipeline.
///
/// Uses DDG `site:facebook.com` search → Open Graph metadata as primary method,
/// or Graph API if token is configured.
pub async fn run_facebook_pipeline(
    config: &FacebookPipelineConfig,
    http: &HttpClient,
) -> FetchiumResult<FacebookPipelineResult> {
    let started = Instant::now();

    let (posts, pages, data_source) = search::search_posts(&config.query, config, http).await?;

    let a = analysis::analyse_posts(&posts, &pages);

    Ok(FacebookPipelineResult {
        query: config.query.clone(),
        posts,
        pages,
        analysis: a,
        duration_ms: started.elapsed().as_millis() as u64,
        data_source,
    })
}
