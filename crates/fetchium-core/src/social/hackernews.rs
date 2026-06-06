//! HackerNews intelligence via the official Firebase API + Algolia search.
//!
//! 100% free, no API key, very fast (~10-50ms per request).
//!
//! ## Endpoints
//! - Firebase: `https://hacker-news.firebaseio.com/v0/`
//! - Algolia: `https://hn.algolia.com/api/v1/search?query=...`

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::social::types::{score_sentiment, EngagementMetrics, SocialPlatform, SocialPost};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// A HackerNews story.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnStory {
    pub id: u64,
    pub title: String,
    pub url: Option<String>,
    pub author: String,
    pub score: u64,
    pub descendants: u64, // comment count
    pub published: u64,   // unix timestamp
    pub story_type: HnStoryType,
}

/// HN story type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HnStoryType {
    #[default]
    Story,
    Ask,
    Show,
    Job,
}

/// A HN comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnComment {
    pub id: u64,
    pub author: String,
    pub text: String,
    pub score: Option<i64>,
    pub children: Vec<HnComment>,
}

/// Search HN via Algolia (fastest, full-text).
pub async fn search_stories(
    query: &str,
    max: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> FetchiumResult<Vec<HnStory>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    // Sort by relevance, only stories
    let url = format!(
        "https://hn.algolia.com/api/v1/search?query={encoded}&tags=story&hitsPerPage={max}"
    );

    let body = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
        .map_err(|_| FetchiumError::Internal("Request timeout".into()))?
        .map_err(|e| FetchiumError::Search(e.to_string()))?;

    let v: Value = serde_json::from_str(&body)
        .map_err(|e| FetchiumError::Internal(format!("HN Algolia JSON: {e}")))?;

    let hits = match v["hits"].as_array() {
        Some(h) => h,
        None => return Ok(Vec::new()),
    };

    Ok(hits
        .iter()
        .filter_map(|h| {
            let title = h["title"].as_str()?.to_string();
            let id = h["objectID"].as_str()?.parse::<u64>().ok()?;
            Some(HnStory {
                id,
                title,
                url: h["url"].as_str().map(|s| s.to_string()),
                author: h["author"].as_str().unwrap_or("").to_string(),
                score: h["points"].as_u64().unwrap_or(0),
                descendants: h["num_comments"].as_u64().unwrap_or(0),
                published: h["created_at_i"].as_u64().unwrap_or(0),
                story_type: HnStoryType::Story,
            })
        })
        .collect())
}

/// Fetch top/new/best/ask/show stories by category.
pub async fn fetch_category(
    category: HnCategory,
    max: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> FetchiumResult<Vec<HnStory>> {
    let cat_str = category.as_str();
    let url = format!("https://hacker-news.firebaseio.com/v0/{cat_str}.json");

    let body = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
        .map_err(|_| FetchiumError::Internal("Request timeout".into()))?
        .map_err(|e| FetchiumError::Search(e.to_string()))?;

    let ids: Vec<u64> =
        serde_json::from_str(&body).map_err(|e| FetchiumError::Internal(format!("HN IDs: {e}")))?;

    // Fetch top `max` items in parallel
    let ids = ids.into_iter().take(max).collect::<Vec<_>>();
    let mut handles = Vec::with_capacity(ids.len());
    for id in ids {
        let http2 = http.clone();
        let to = timeout_secs;
        handles.push(tokio::spawn(
            async move { fetch_item(id, &http2, to).await },
        ));
    }

    let mut stories = Vec::new();
    for h in handles {
        if let Ok(Ok(story)) = h.await {
            stories.push(story);
        }
    }
    Ok(stories)
}

/// HN story category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HnCategory {
    Top,
    New,
    Best,
    Ask,
    Show,
    Job,
}

impl HnCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::Top => "topstories",
            Self::New => "newstories",
            Self::Best => "beststories",
            Self::Ask => "askstories",
            Self::Show => "showstories",
            Self::Job => "jobstories",
        }
    }
}

/// Fetch a single HN item by ID.
pub async fn fetch_item(id: u64, http: &HttpClient, timeout_secs: u64) -> FetchiumResult<HnStory> {
    let url = format!("https://hacker-news.firebaseio.com/v0/item/{id}.json");
    let body = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
        .map_err(|_| FetchiumError::Internal("Request timeout".into()))?
        .map_err(|e| FetchiumError::Search(e.to_string()))?;

    let v: Value = serde_json::from_str(&body)
        .map_err(|e| FetchiumError::Internal(format!("HN item {id}: {e}")))?;

    let story_type = match v["type"].as_str() {
        Some("ask") => HnStoryType::Ask,
        Some("show") => HnStoryType::Show,
        Some("job") => HnStoryType::Job,
        _ => HnStoryType::Story,
    };

    Ok(HnStory {
        id,
        title: v["title"].as_str().unwrap_or("").to_string(),
        url: v["url"].as_str().map(|s| s.to_string()),
        author: v["by"].as_str().unwrap_or("").to_string(),
        score: v["score"].as_u64().unwrap_or(0),
        descendants: v["descendants"].as_u64().unwrap_or(0),
        published: v["time"].as_u64().unwrap_or(0),
        story_type,
    })
}

/// Convert HN story to normalised SocialPost.
pub fn to_social_post(story: &HnStory) -> SocialPost {
    let content = format!(
        "{} — {} points, {} comments",
        story.title, story.score, story.descendants
    );
    let sentiment = score_sentiment(&story.title);
    let mut engagement = EngagementMetrics {
        likes: story.score,
        shares: 0,
        comments: story.descendants,
        views: None,
        score: 0.0,
    };
    engagement.compute_score();

    SocialPost {
        platform: SocialPlatform::HackerNews,
        id: story.id.to_string(),
        url: story
            .url
            .clone()
            .unwrap_or_else(|| format!("https://news.ycombinator.com/item?id={}", story.id)),
        author: story.author.clone(),
        content,
        published: story.published.to_string(),
        engagement,
        media: Vec::new(),
        hashtags: Vec::new(),
        mentions: Vec::new(),
        sentiment,
        authenticity: 0.9, // HN has strong moderation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hn_category_strings() {
        assert_eq!(HnCategory::Top.as_str(), "topstories");
        assert_eq!(HnCategory::Ask.as_str(), "askstories");
    }

    #[test]
    fn to_social_post_basic() {
        let story = HnStory {
            id: 12345,
            title: "Amazing new Rust framework".into(),
            url: Some("https://example.com".into()),
            author: "rustacean".into(),
            score: 500,
            descendants: 120,
            published: 1700000000,
            story_type: HnStoryType::Story,
        };
        let post = to_social_post(&story);
        assert_eq!(post.platform, SocialPlatform::HackerNews);
        assert!(post.content.contains("500 points"));
    }

    #[test]
    fn to_social_post_no_url_fallback() {
        let story = HnStory {
            id: 99999,
            title: "Ask HN: Favourite tool?".into(),
            url: None,
            author: "curious".into(),
            score: 300,
            descendants: 80,
            published: 1700001000,
            story_type: HnStoryType::Ask,
        };
        let post = to_social_post(&story);
        assert!(
            post.url.contains("news.ycombinator.com"),
            "fallback should point to YC: {}",
            post.url
        );
        assert!(post.url.contains("99999"));
    }

    #[test]
    fn to_social_post_engagement_computed() {
        let story = HnStory {
            id: 1,
            title: "Great story".into(),
            url: Some("https://example.com".into()),
            author: "user".into(),
            score: 1000,
            descendants: 300,
            published: 1700000000,
            story_type: HnStoryType::Story,
        };
        let post = to_social_post(&story);
        assert!(post.engagement.score > 0.0);
    }

    #[test]
    fn to_social_post_authenticity_high() {
        let story = HnStory {
            id: 2,
            title: "HN has moderation".into(),
            url: None,
            author: "mod".into(),
            score: 50,
            descendants: 10,
            published: 0,
            story_type: HnStoryType::Story,
        };
        let post = to_social_post(&story);
        assert!(post.authenticity >= 0.8, "HN authenticity should be high");
    }

    #[test]
    fn hn_story_type_default_is_story() {
        assert_eq!(HnStoryType::default(), HnStoryType::Story);
    }

    #[test]
    fn hn_category_all_str_variants() {
        assert_eq!(HnCategory::Top.as_str(), "topstories");
        assert_eq!(HnCategory::New.as_str(), "newstories");
        assert_eq!(HnCategory::Best.as_str(), "beststories");
        assert_eq!(HnCategory::Ask.as_str(), "askstories");
        assert_eq!(HnCategory::Show.as_str(), "showstories");
        assert_eq!(HnCategory::Job.as_str(), "jobstories");
    }
}
