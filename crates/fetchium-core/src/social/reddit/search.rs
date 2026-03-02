//! Reddit search and data fetching via Reddit's public JSON API.
//!
//! No authentication required for public subreddits.
//! Appending `.json` to any Reddit URL returns JSON data.

use crate::error::{HsxError, HsxResult};
use crate::http::client::HttpClient;
use crate::social::reddit::types::*;
use serde_json::Value;
use std::time::Duration;

/// User-agent Reddit requires to avoid 429s.
const REDDIT_UA: &str = "Fetchium/1.0 (research tool; https://github.com/fetchium/fetchium)";

/// Search Reddit posts via a multi-tier approach:
///
/// 1. PullPush API (api.pullpush.io) — bypasses Reddit's Cloudflare/auth requirement
/// 2. Old Reddit JSON API (www.reddit.com/search.json) — fallback with Reddit UA
/// 3. DDG `site:reddit.com <query>` — last resort, parse post URLs
pub async fn search_posts(
    query: &str,
    config: &RedditPipelineConfig,
    http: &HttpClient,
) -> HsxResult<Vec<RedditPost>> {
    // ── Tier 1: PullPush API (no auth, bypasses Cloudflare) ────────────
    if let Ok(posts) = search_via_pullpush(query, config, http).await {
        if !posts.is_empty() {
            return Ok(posts);
        }
    }

    // ── Tier 2: Old Reddit JSON API with proper UA ──────────────────────
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let base = if config.subreddits.is_empty() {
        format!(
            "https://www.reddit.com/search.json?q={encoded}&sort=relevance&limit={}&type=link",
            config.max_posts
        )
    } else {
        let sr = config.subreddits.join("+");
        format!(
            "https://www.reddit.com/r/{sr}/search.json?q={encoded}&sort=relevance&restrict_sr=1&limit={}",
            config.max_posts
        )
    };

    let timeout = Duration::from_secs(config.timeout_secs.min(8)); // cap at 8s per attempt
    if let Ok(Ok(resp)) = tokio::time::timeout(timeout, async {
        http.client()
            .get(&base)
            .header("User-Agent", REDDIT_UA)
            .header("Accept", "application/json")
            .send()
            .await
    })
    .await
    {
        if let Ok(body) = resp.text().await {
            if let Ok(posts) = parse_reddit_listing(&body) {
                if !posts.is_empty() {
                    return Ok(posts);
                }
            }
        }
    }

    // ── Tier 3: DDG site:reddit.com search ─────────────────────────────
    search_via_ddg(query, config.max_posts, http).await
}

/// Search Reddit via PullPush API (Pushshift successor).
///
/// PullPush archives Reddit posts and is accessible without auth or Cloudflare.
async fn search_via_pullpush(
    query: &str,
    config: &RedditPipelineConfig,
    http: &HttpClient,
) -> HsxResult<Vec<RedditPost>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();

    let url = if config.subreddits.is_empty() {
        format!(
            "https://api.pullpush.io/reddit/search/submission/?q={encoded}&size={}&sort=score",
            config.max_posts
        )
    } else {
        let sr = config
            .subreddits
            .first()
            .map(|s| s.trim_start_matches("r/"))
            .unwrap_or("");
        format!(
            "https://api.pullpush.io/reddit/search/submission/?q={encoded}&subreddit={sr}&size={}&sort=score",
            config.max_posts
        )
    };

    let body = match tokio::time::timeout(
        Duration::from_secs(8),
        http.client()
            .get(&url)
            .header("User-Agent", REDDIT_UA)
            .send(),
    )
    .await
    {
        Ok(Ok(r)) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => return Ok(Vec::new()),
    };

    let v: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let data = match v["data"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let mut posts: Vec<RedditPost> = data
        .iter()
        .filter_map(parse_pullpush_post)
        .filter(|p| p.score >= 2) // skip very low-quality posts
        .collect();

    // Relevance filter: require at least one query word in title or selftext
    if !config.subreddits.is_empty() {
        // subreddit-specific: trust all results
    } else {
        let query_words: Vec<&str> = query.split_whitespace().filter(|w| w.len() >= 3).collect();
        if !query_words.is_empty() {
            posts.retain(|p| {
                let text = format!("{} {}", p.title, p.selftext).to_lowercase();
                query_words.iter().any(|w| text.contains(&w.to_lowercase()))
            });
        }
    }

    Ok(posts)
}

/// Search Reddit via DuckDuckGo `site:reddit.com <query>`.
///
/// Returns post stubs from DDG snippets — useful when Reddit API is inaccessible.
async fn search_via_ddg(query: &str, max: usize, http: &HttpClient) -> HsxResult<Vec<RedditPost>> {
    let ddg_query = format!("site:reddit.com {query}");
    let form: &[(&str, &str)] = &[("q", &ddg_query), ("b", ""), ("kl", "en-us")];

    let resp = tokio::time::timeout(Duration::from_secs(12), async {
        http.client()
            .post("https://html.duckduckgo.com/html/")
            .form(form)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .send()
            .await
    })
    .await;

    let body = match resp {
        Ok(Ok(r)) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => return Ok(Vec::new()),
    };

    Ok(parse_ddg_reddit_results(&body, max))
}

/// Parse DDG HTML results for Reddit post links.
fn parse_ddg_reddit_results(html: &str, max: usize) -> Vec<RedditPost> {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let result_sel = Selector::parse("div.result").expect("valid");
    let link_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid");
    let snippet_sel = Selector::parse("a.result__snippet, .result__snippet").expect("valid");

    let mut posts = Vec::new();
    for result in doc.select(&result_sel) {
        if posts.len() >= max {
            break;
        }
        let classes = result.value().attr("class").unwrap_or("");
        if classes.contains("result--ad") {
            continue;
        }

        let href = result
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("");

        // Resolve DDG redirect
        let raw_url = if href.contains("uddg=") {
            let qs_start = href.find('?').map(|i| i + 1).unwrap_or(0);
            let mut resolved = String::new();
            for (k, v) in url::form_urlencoded::parse(&href.as_bytes()[qs_start..]) {
                if k == "uddg" && v.starts_with("http") {
                    resolved = v.into_owned();
                    break;
                }
            }
            resolved
        } else if href.starts_with("https://") || href.starts_with("http://") {
            href.to_string()
        } else {
            continue;
        };

        if !raw_url.contains("reddit.com/r/") {
            continue;
        }

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Extract subreddit from URL
        let subreddit = raw_url
            .split("/r/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("reddit")
            .to_string();

        // Extract title from URL path
        let title = raw_url
            .split("/comments/")
            .nth(1)
            .and_then(|s| s.split('/').nth(1))
            .map(|s| s.replace('_', " "))
            .unwrap_or_else(|| snippet.chars().take(80).collect());

        let id = raw_url
            .split("/comments/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .unwrap_or("ddg")
            .to_string();

        posts.push(RedditPost {
            id,
            url: raw_url.clone(),
            permalink: raw_url.replace("https://www.reddit.com", ""),
            title,
            selftext: snippet,
            author: String::new(),
            subreddit,
            score: 0,
            upvote_ratio: 0.5,
            num_comments: 0,
            created_utc: 0.0,
            flair: None,
            is_self: true,
            link_url: None,
            awards: 0,
            crossposts: 0,
        });
    }
    posts
}

/// Parse a PullPush API submission entry.
fn parse_pullpush_post(d: &serde_json::Value) -> Option<RedditPost> {
    let title = d["title"].as_str()?.to_string();
    if title.is_empty() {
        return None;
    }

    let id = d["id"].as_str().unwrap_or("").to_string();
    let subreddit = d["subreddit"].as_str().unwrap_or("").to_string();
    let permalink = d["permalink"].as_str().unwrap_or("").to_string();
    let url = if permalink.is_empty() {
        format!("https://reddit.com/r/{subreddit}/comments/{id}/")
    } else {
        format!("https://reddit.com{permalink}")
    };

    Some(RedditPost {
        id,
        url,
        permalink,
        title,
        selftext: d["selftext"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(500)
            .collect(),
        author: d["author"].as_str().unwrap_or("[deleted]").to_string(),
        subreddit,
        score: d["score"].as_i64().unwrap_or(0),
        upvote_ratio: d["upvote_ratio"].as_f64().unwrap_or(0.5),
        num_comments: d["num_comments"].as_u64().unwrap_or(0),
        created_utc: d["created_utc"].as_f64().unwrap_or(0.0),
        flair: d["link_flair_text"].as_str().map(|s| s.to_string()),
        is_self: d["is_self"].as_bool().unwrap_or(false),
        link_url: d["url"].as_str().map(|s| s.to_string()),
        awards: d["total_awards_received"].as_u64().unwrap_or(0) as u32,
        crossposts: d["num_crossposts"].as_u64().unwrap_or(0) as u32,
    })
}

/// Fetch posts from a subreddit category (hot/new/rising/top/controversial).
pub async fn fetch_subreddit_posts(
    subreddit: &str,
    category: RedditCategory,
    limit: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> HsxResult<Vec<RedditPost>> {
    let url = format!("https://www.reddit.com/r/{subreddit}/{category}.json?limit={limit}",);
    let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        http.client()
            .get(&url)
            .header("User-Agent", REDDIT_UA)
            .header("Accept", "application/json")
            .send()
            .await
    })
    .await
    .map_err(|_| HsxError::Internal("Request timeout".into()))?
    .map_err(|e| HsxError::Search(e.to_string()))?;
    let body = resp
        .text()
        .await
        .map_err(|e| HsxError::Search(e.to_string()))?;
    parse_reddit_listing(&body)
}

/// Fetch top posts from multiple subreddits in parallel.
pub async fn fetch_multi_subreddit_hot(
    subreddits: &[String],
    per_sub: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> Vec<RedditPost> {
    let mut handles = Vec::new();

    for sub in subreddits {
        let sub = sub.clone();
        let http = http.clone();
        let handle = tokio::spawn(async move {
            fetch_subreddit_posts(&sub, RedditCategory::Hot, per_sub, &http, timeout_secs).await
        });
        handles.push(handle);
    }

    let mut posts = Vec::new();
    for h in handles {
        if let Ok(Ok(p)) = h.await {
            posts.extend(p);
        }
    }
    posts
}

/// Fetch top-level comments for a post.
pub async fn fetch_comments(
    permalink: &str,
    max: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> HsxResult<Vec<RedditComment>> {
    let url = format!("https://www.reddit.com{permalink}.json?limit={max}");
    let body = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
        .map_err(|_| HsxError::Internal("Request timeout".into()))?
        .map_err(|e| HsxError::Search(e.to_string()))?;

    parse_comments(&body)
}

/// Search historical Reddit posts via PullPush API (Pushshift successor).
///
/// Useful for deeper historical research. Rate limit: ~1000 req/hr.
pub async fn search_historical(
    query: &str,
    max: usize,
    after_utc: Option<u64>,
    http: &HttpClient,
    timeout_secs: u64,
) -> HsxResult<Vec<RedditPost>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let after_param = after_utc.map(|t| format!("&after={t}")).unwrap_or_default();
    let url = format!(
        "https://api.pullpush.io/reddit/search/submission/?q={encoded}&size={max}{after_param}&sort=score"
    );

    let body = match tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
    {
        Ok(Ok(b)) => b,
        _ => return Ok(Vec::new()), // PullPush is optional
    };

    let v: Value = serde_json::from_str(&body)
        .map_err(|e| HsxError::Internal(format!("PullPush JSON: {e}")))?;

    let children = match v["data"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    Ok(children
        .iter()
        .filter_map(|c| parse_post(&c["data"]))
        .collect())
}

/// Fetch subreddit info.
pub async fn fetch_subreddit_info(
    subreddit: &str,
    http: &HttpClient,
    timeout_secs: u64,
) -> HsxResult<SubredditStats> {
    let url = format!("https://www.reddit.com/r/{subreddit}/about.json");
    let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
        http.client()
            .get(&url)
            .header("User-Agent", REDDIT_UA)
            .header("Accept", "application/json")
            .send()
            .await
    })
    .await
    .map_err(|_| HsxError::Internal("Request timeout".into()))?
    .map_err(|e| HsxError::Search(e.to_string()))?;
    let body = resp
        .text()
        .await
        .map_err(|e| HsxError::Search(e.to_string()))?;

    let v: Value = serde_json::from_str(&body)
        .map_err(|e| HsxError::Internal(format!("Reddit about JSON: {e}")))?;
    let data = &v["data"];

    Ok(SubredditStats {
        name: data["display_name"]
            .as_str()
            .unwrap_or(subreddit)
            .to_string(),
        subscribers: data["subscribers"].as_u64().unwrap_or(0),
        active_users: data["active_user_count"].as_u64(),
        title: data["title"].as_str().unwrap_or("").to_string(),
        description: data["public_description"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        over18: data["over18"].as_bool().unwrap_or(false),
    })
}

// ─── Parsers ─────────────────────────────────────────────────────

fn parse_reddit_listing(body: &str) -> HsxResult<Vec<RedditPost>> {
    let v: Value =
        serde_json::from_str(body).map_err(|e| HsxError::Internal(format!("Reddit JSON: {e}")))?;

    let children = v["data"]["children"]
        .as_array()
        .ok_or_else(|| HsxError::Internal("No children in Reddit listing".into()))?;

    Ok(children
        .iter()
        .filter_map(|c| parse_post(&c["data"]))
        .collect())
}

fn parse_post(d: &Value) -> Option<RedditPost> {
    let id = d["id"].as_str()?.to_string();
    let permalink = d["permalink"].as_str().unwrap_or("").to_string();
    let title = d["title"].as_str().unwrap_or("").to_string();

    if title.is_empty() {
        return None;
    }

    Some(RedditPost {
        id,
        url: format!("https://reddit.com{permalink}"),
        permalink,
        title,
        selftext: d["selftext"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(500)
            .collect(),
        author: d["author"].as_str().unwrap_or("[deleted]").to_string(),
        subreddit: d["subreddit"].as_str().unwrap_or("").to_string(),
        score: d["score"].as_i64().unwrap_or(0),
        upvote_ratio: d["upvote_ratio"].as_f64().unwrap_or(0.5),
        num_comments: d["num_comments"].as_u64().unwrap_or(0),
        created_utc: d["created_utc"].as_f64().unwrap_or(0.0),
        flair: d["link_flair_text"].as_str().map(|s| s.to_string()),
        is_self: d["is_self"].as_bool().unwrap_or(false),
        link_url: d["url"].as_str().map(|s| s.to_string()),
        awards: d["total_awards_received"].as_u64().unwrap_or(0) as u32,
        crossposts: d["num_crossposts"].as_u64().unwrap_or(0) as u32,
    })
}

fn parse_comments(body: &str) -> HsxResult<Vec<RedditComment>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| HsxError::Internal(format!("Reddit comments JSON: {e}")))?;

    // Reddit returns [post_data, comments_data]
    let comments_listing = &v[1]["data"]["children"];
    let children = match comments_listing.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    Ok(children
        .iter()
        .filter_map(|c| parse_comment(&c["data"], 0))
        .collect())
}

fn parse_comment(d: &Value, depth: u32) -> Option<RedditComment> {
    let body = d["body"].as_str()?;
    if body == "[deleted]" || body == "[removed]" {
        return None;
    }
    let id = d["id"].as_str()?.to_string();

    let replies = d["replies"]["data"]["children"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|c| parse_comment(&c["data"], depth + 1))
                .take(5)
                .collect()
        })
        .unwrap_or_default();

    Some(RedditComment {
        id,
        author: d["author"].as_str().unwrap_or("[deleted]").to_string(),
        body: body.chars().take(400).collect(),
        score: d["score"].as_i64().unwrap_or(0),
        depth,
        replies,
    })
}

#[allow(dead_code)]
pub fn get_user_agent() -> &'static str {
    REDDIT_UA
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_listing_ok() {
        let json = r#"{"data":{"children":[]}}"#;
        let posts = parse_reddit_listing(json).unwrap();
        assert!(posts.is_empty());
    }

    #[test]
    fn parse_post_basic() {
        let d = serde_json::json!({
            "id": "abc123",
            "permalink": "/r/rust/comments/abc123/hello/",
            "title": "Hello Rust",
            "selftext": "This is a test post",
            "author": "rustacean",
            "subreddit": "rust",
            "score": 1234,
            "upvote_ratio": 0.97,
            "num_comments": 42,
            "created_utc": 1700000000.0,
            "is_self": true,
            "total_awards_received": 2,
            "num_crossposts": 0
        });
        let post = parse_post(&d).unwrap();
        assert_eq!(post.id, "abc123");
        assert_eq!(post.score, 1234);
        assert_eq!(post.subreddit, "rust");
    }

    #[test]
    fn parse_deleted_comment_none() {
        let d = serde_json::json!({
            "id": "x1",
            "body": "[deleted]",
            "author": "[deleted]",
            "score": 0
        });
        assert!(parse_comment(&d, 0).is_none());
    }
}
