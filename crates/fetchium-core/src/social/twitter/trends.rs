//! Twitter/X trend aggregator scraping — free, no API key needed.

use crate::error::FetchiumError;
use crate::http::client::HttpClient;
use serde::{Deserialize, Serialize};

/// A trending topic from aggregator sites.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingTopic {
    pub name: String,
    pub tweet_volume: Option<u64>,
    pub rank: usize,
    pub category: Option<String>,
    pub source: String,
}

/// Fetch trending topics from free aggregator sites.
pub async fn fetch_trends(
    country: &str,
    http: &HttpClient,
) -> Result<Vec<TrendingTopic>, FetchiumError> {
    let mut trends = Vec::new();

    // Try trends24.in first (realtime trends)
    match fetch_trends24(country, http).await {
        Ok(mut t) => trends.append(&mut t),
        Err(e) => tracing::debug!("trends24 failed: {e}"),
    }

    // Fallback to getdaytrends.com
    if trends.is_empty() {
        match fetch_getdaytrends(country, http).await {
            Ok(mut t) => trends.append(&mut t),
            Err(e) => tracing::debug!("getdaytrends failed: {e}"),
        }
    }

    // Re-rank by position
    for (i, trend) in trends.iter_mut().enumerate() {
        trend.rank = i + 1;
    }

    Ok(trends)
}

async fn fetch_trends24(country: &str, http: &HttpClient) -> Result<Vec<TrendingTopic>, FetchiumError> {
    let country_path = match country.to_lowercase().as_str() {
        "us" | "united states" => "united-states",
        "uk" | "united kingdom" => "united-kingdom",
        _ => country,
    };
    let url = format!("https://trends24.in/{}/", country_path);

    let html = match tokio::time::timeout(std::time::Duration::from_secs(8), http.fetch_text(&url))
        .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(FetchiumError::Internal("trends24 timeout".into())),
    };

    // Parse trending topics from HTML
    let mut topics = Vec::new();
    for line in html.lines() {
        if line.contains("trend-card__list") || line.contains("trend-name") {
            // Extract text between > and <
            let text = extract_inner_text(line);
            if !text.is_empty() && text.len() < 100 {
                topics.push(TrendingTopic {
                    name: text,
                    tweet_volume: None,
                    rank: topics.len() + 1,
                    category: None,
                    source: "trends24.in".into(),
                });
            }
        }
        if topics.len() >= 30 {
            break;
        }
    }

    Ok(topics)
}

async fn fetch_getdaytrends(
    country: &str,
    http: &HttpClient,
) -> Result<Vec<TrendingTopic>, FetchiumError> {
    let url = format!("https://getdaytrends.com/{}/", country.to_lowercase());

    let html = match tokio::time::timeout(std::time::Duration::from_secs(8), http.fetch_text(&url))
        .await
    {
        Ok(Ok(html)) => html,
        Ok(Err(e)) => return Err(e),
        Err(_) => return Err(FetchiumError::Internal("getdaytrends timeout".into())),
    };

    let mut topics = Vec::new();
    for line in html.lines() {
        if line.contains("trending-topic") || line.contains("trend-name") {
            let text = extract_inner_text(line);
            if !text.is_empty() && text.len() < 100 {
                topics.push(TrendingTopic {
                    name: text,
                    tweet_volume: None,
                    rank: topics.len() + 1,
                    category: None,
                    source: "getdaytrends.com".into(),
                });
            }
        }
        if topics.len() >= 30 {
            break;
        }
    }

    Ok(topics)
}

fn extract_inner_text(html_line: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html_line.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_inner_text_works() {
        assert_eq!(
            extract_inner_text("<a href=\"#\">Trending Topic</a>"),
            "Trending Topic"
        );
    }
}
