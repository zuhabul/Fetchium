//! TikTok data fetching via public/unofficial endpoints.
//!
//! TikTok has no official public API. We use:
//! 1. tikwm.com public search API — returns full video data (plays, likes, author)
//! 2. DDG `site:tiktok.com <query>` — URL-level search, no engagement metrics
//! 3. TikTok internal search API (requires proper headers)
//! 4. TikTok discover page HTML as last resort

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::social::tiktok::types::*;
use serde_json::Value;
use std::time::Duration;

/// Search TikTok videos using a four-tier approach:
/// 1. tikwm.com API — full data (play counts, likes, real usernames, always free)
/// 2. DDG `site:tiktok.com <query>` — URL-level fallback
/// 3. TikTok internal search API (requires proper headers)
/// 4. TikTok discover page HTML as last resort
pub async fn search_videos(
    query: &str,
    config: &TikTokPipelineConfig,
    http: &HttpClient,
) -> FetchiumResult<Vec<TikTokVideo>> {
    // ── Tier 1: tikwm.com public search API (full metrics, no auth) ───
    if let Ok(videos) = search_via_tikwm(query, config.max_videos, http).await {
        if !videos.is_empty() {
            return Ok(videos);
        }
    }

    // ── Tier 2: DuckDuckGo site:tiktok.com ────────────────────────────
    if let Ok(videos) = search_via_ddg(query, config.max_videos, http).await {
        if !videos.is_empty() {
            return Ok(videos);
        }
    }

    // ── Tier 3: TikTok internal search API ────────────────────────────
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let api_url = format!(
        "https://www.tiktok.com/api/search/general/full/?keyword={encoded}&count={}&cursor=0",
        config.max_videos
    );

    let api_result = tokio::time::timeout(Duration::from_secs(config.timeout_secs), async {
        http.client()
            .get(&api_url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Referer", "https://www.tiktok.com/")
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
    })
    .await;

    if let Ok(Ok(resp)) = api_result {
        if resp.status().is_success() {
            if let Ok(body) = resp.text().await {
                if let Ok(videos) = parse_tiktok_search_api(&body, config.max_videos) {
                    if !videos.is_empty() {
                        return Ok(videos);
                    }
                }
            }
        }
    }

    // ── Tier 4: TikTok discover page HTML ─────────────────────────────
    let discover_url = "https://www.tiktok.com/discover";
    if let Ok(Ok(html)) = tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        http.fetch_text(discover_url),
    )
    .await
    {
        let videos = parse_tiktok_html(&html, config.max_videos);
        if !videos.is_empty() {
            return Ok(videos);
        }
    }

    Ok(Vec::new())
}

// ─── tikwm.com Tier 1 ──────────────────────────────────────────

/// tikwm.com is a public TikTok data API with no auth requirement.
///
/// Returns full video metadata: play counts, likes, comments, shares,
/// real usernames, thumbnails, descriptions — better data than DDG.
async fn search_via_tikwm(
    query: &str,
    max: usize,
    http: &HttpClient,
) -> FetchiumResult<Vec<TikTokVideo>> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!("https://www.tikwm.com/api/feed/search?keywords={encoded}&count={max}");

    let body = match tokio::time::timeout(
        Duration::from_secs(10),
        http.client()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (compatible; Fetchium/1.0)")
            .header("Referer", "https://www.tikwm.com/")
            .header("Accept", "application/json")
            .send(),
    )
    .await
    {
        Ok(Ok(r)) if r.status().is_success() => r.text().await.unwrap_or_default(),
        _ => return Ok(Vec::new()),
    };

    parse_tikwm_response(&body, max)
}

fn parse_tikwm_response(json_str: &str, max: usize) -> FetchiumResult<Vec<TikTokVideo>> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| FetchiumError::Internal(format!("tikwm JSON: {e}")))?;

    if v["code"].as_i64().unwrap_or(-1) != 0 {
        return Ok(Vec::new());
    }

    let videos_arr = match v["data"]["videos"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    let mut results = Vec::new();
    for item in videos_arr.iter().take(max) {
        let video_id = item["video_id"]
            .as_str()
            .or_else(|| item["video_id"].as_u64().map(|_| ""))
            .unwrap_or("")
            .to_string();
        let video_id = if video_id.is_empty() {
            match item["video_id"].as_u64() {
                Some(n) => n.to_string(),
                None => continue,
            }
        } else {
            video_id
        };

        let author_unique = item["author"]["unique_id"]
            .as_str()
            .unwrap_or("tiktok_user");
        let author_nick = item["author"]["nickname"]
            .as_str()
            .unwrap_or(author_unique)
            .to_string();
        let username = author_unique.to_string();

        let title = item["title"].as_str().unwrap_or("").to_string();
        let description = if title.is_empty() {
            "TikTok video".to_string()
        } else {
            title
        };

        let play_count = item["play_count"].as_u64().unwrap_or(0);
        let digg_count = item["digg_count"].as_u64().unwrap_or(0); // likes
        let comment_count = item["comment_count"].as_u64().unwrap_or(0);
        let share_count = item["share_count"].as_u64().unwrap_or(0);
        let duration = item["duration"].as_u64().unwrap_or(0);
        let create_time = item["create_time"].as_u64().unwrap_or(0);
        let published = if create_time > 0 {
            // Unix timestamp to rough date string
            format_unix_ts(create_time)
        } else {
            String::new()
        };

        let hashtags = extract_hashtags_from_desc(&description);
        let url = format!("https://www.tiktok.com/@{username}/video/{video_id}");

        results.push(TikTokVideo {
            id: video_id,
            url,
            author: TikTokUser {
                username: username.clone(),
                display_name: author_nick,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            description,
            published,
            duration_secs: duration as u32,
            likes: digg_count,
            comments: comment_count,
            shares: share_count,
            plays: play_count,
            hashtags,
            music: None,
            is_duet: false,
            is_stitch: false,
        });
    }

    Ok(results)
}

/// Format a Unix timestamp as a human-readable date string.
fn format_unix_ts(ts: u64) -> String {
    // Approximate: seconds since epoch → "YYYY-MM-DD"
    // We avoid chrono dep — compute manually with integer math
    let days_since_epoch = ts / 86400;
    // Days from 1970-01-01
    let mut year = 1970u64;
    let mut remaining = days_since_epoch;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }
    let months = [
        31u64,
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            29
        } else {
            28
        },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &days_in_month in &months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }
    format!("{year}-{month:02}-{:02}", remaining + 1)
}

const DDG_HTML_URL: &str = "https://html.duckduckgo.com/html/";

/// Search TikTok via DuckDuckGo — tries `site:tiktok.com/@` then general `site:tiktok.com`.
async fn search_via_ddg(
    query: &str,
    max: usize,
    http: &HttpClient,
) -> FetchiumResult<Vec<TikTokVideo>> {
    // Try targeting user videos specifically, then fall back to general TikTok search
    let ddg_query = format!("site:tiktok.com/@ {query}");
    let form: &[(&str, &str)] = &[("q", &ddg_query), ("b", ""), ("kl", "en-us")];

    let resp = tokio::time::timeout(Duration::from_secs(12), async {
        http.client()
            .post(DDG_HTML_URL)
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

    let videos = parse_ddg_tiktok_results(&body, max);
    if !videos.is_empty() {
        return Ok(videos);
    }

    // Retry with broader query (no /@ restriction) if first pass got nothing
    let fallback_query = format!("site:tiktok.com {query}");
    let form2: &[(&str, &str)] = &[("q", &fallback_query), ("b", ""), ("kl", "en-us")];
    let resp2 = tokio::time::timeout(Duration::from_secs(12), async {
        http.client()
            .post(DDG_HTML_URL)
            .form(form2)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .send()
            .await
    })
    .await;
    if let Ok(Ok(r)) = resp2 {
        if r.status().is_success() {
            let body2 = r.text().await.unwrap_or_default();
            return Ok(parse_ddg_tiktok_results_broad(&body2, query, max));
        }
    }
    Ok(Vec::new())
}

/// Parse DDG HTML for TikTok video results.
fn parse_ddg_tiktok_results(html: &str, max: usize) -> Vec<TikTokVideo> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let result_sel = Selector::parse("div.result").expect("valid");
    let link_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid");
    let snippet_sel = Selector::parse("a.result__snippet, .result__snippet").expect("valid");

    let mut videos = Vec::new();

    for result in doc.select(&result_sel) {
        if videos.len() >= max {
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

        let raw_url = resolve_ddg_url(href);
        if !is_tiktok_video_url(&raw_url) {
            continue;
        }

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        if snippet.is_empty() {
            continue;
        }

        let (mut username, video_id) = extract_tiktok_id_and_author(&raw_url);

        // Fallback: extract @username from snippet text when URL doesn't contain it
        if username.is_empty() {
            username = extract_username_from_snippet(&snippet);
        }
        if username.is_empty() {
            username = "tiktok_user".to_string();
        }

        let hashtags = extract_hashtags_from_desc(&snippet);

        videos.push(TikTokVideo {
            id: if video_id.is_empty() {
                format!("ddg_{}", videos.len())
            } else {
                video_id
            },
            url: raw_url,
            author: TikTokUser {
                username: username.clone(),
                display_name: username,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            description: snippet,
            published: String::new(),
            duration_secs: 0,
            likes: 0,
            comments: 0,
            shares: 0,
            plays: 0,
            hashtags,
            music: None,
            is_duet: false,
            is_stitch: false,
        });
    }
    videos
}

/// Extract @username from snippet text (e.g. "TikTok video from Name (@username)")
fn extract_username_from_snippet(snippet: &str) -> String {
    // Pattern 1: "(@username)" common in TikTok share snippets
    if let Some(start) = snippet.find("(@") {
        let rest = &snippet[start + 2..];
        let end = rest.find(')').unwrap_or(rest.len());
        let user = &rest[..end];
        if !user.is_empty() && user.len() < 40 {
            return user.to_string();
        }
    }
    // Pattern 2: "@username" anywhere in text
    for word in snippet.split_whitespace() {
        if let Some(stripped) = word.strip_prefix('@') {
            let clean: String = stripped
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if clean.len() >= 2 {
                return clean;
            }
        }
    }
    String::new()
}

/// Broad DDG parser — accepts any tiktok.com URL (profiles, hashtags, videos).
/// Used as fallback when video-specific search returns nothing.
fn parse_ddg_tiktok_results_broad(html: &str, query: &str, max: usize) -> Vec<TikTokVideo> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let result_sel = Selector::parse("div.result").expect("valid");
    let link_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid");
    let snippet_sel = Selector::parse("a.result__snippet, .result__snippet").expect("valid");

    let mut videos = Vec::new();

    for result in doc.select(&result_sel) {
        if videos.len() >= max {
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
        let raw_url = resolve_ddg_url(href);

        // Accept any TikTok URL
        if !raw_url.contains("tiktok.com") {
            continue;
        }

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let description = if snippet.is_empty() {
            format!("TikTok content about {query}")
        } else {
            snippet
        };

        let (mut username, video_id) = if raw_url.contains("/video/") {
            extract_tiktok_id_and_author(&raw_url)
        } else {
            let u = raw_url
                .split("/@")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or("")
                .to_string();
            let id = format!("ddg_{}", videos.len());
            (u, id)
        };

        if username.is_empty() {
            username = extract_username_from_snippet(&description);
        }
        if username.is_empty() {
            username = "tiktok_user".to_string();
        }

        let hashtags = extract_hashtags_from_desc(&description);
        videos.push(TikTokVideo {
            id: video_id,
            url: raw_url,
            author: TikTokUser {
                username: username.clone(),
                display_name: username,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            description,
            published: String::new(),
            duration_secs: 0,
            likes: 0,
            comments: 0,
            shares: 0,
            plays: 0,
            hashtags,
            music: None,
            is_duet: false,
            is_stitch: false,
        });
    }
    videos
}

fn is_tiktok_video_url(url: &str) -> bool {
    url.contains("tiktok.com/@") && url.contains("/video/")
}

fn extract_tiktok_id_and_author(url: &str) -> (String, String) {
    // https://www.tiktok.com/@username/video/1234567890
    let username = url
        .split("/@")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .unwrap_or("")
        .to_string();
    let video_id = url
        .split("/video/")
        .nth(1)
        .and_then(|s| s.split('?').next())
        .unwrap_or("")
        .to_string();
    (username, video_id)
}

fn resolve_ddg_url(href: &str) -> String {
    if href.is_empty() {
        return String::new();
    }
    if href.contains("uddg=") {
        let qs_start = href.find('?').map(|i| i + 1).unwrap_or(0);
        for (k, v) in url::form_urlencoded::parse(&href.as_bytes()[qs_start..]) {
            if k == "uddg" && v.starts_with("http") {
                return v.into_owned();
            }
        }
    }
    if (href.starts_with("https://") || href.starts_with("http://"))
        && !href.contains("duckduckgo.com")
    {
        return href.to_string();
    }
    String::new()
}

/// Fetch trending hashtags on TikTok.
pub async fn fetch_trends(
    config: &TikTokPipelineConfig,
    http: &HttpClient,
) -> FetchiumResult<Vec<TikTokTrend>> {
    // TikTok's trending hashtags via public Creative Center API
    let url = "https://ads.tiktok.com/creative_radar_api/v1/popular_trend/hashtag/list?page=1&limit=20&period=7&country_code=US";

    if let Ok(Ok(body)) = tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        http.fetch_text(url),
    )
    .await
    {
        if let Ok(trends) = parse_creative_center_trends(&body) {
            return Ok(trends);
        }
    }

    // Fallback: scrape TikTok explore page
    let explore_url = "https://www.tiktok.com/explore";
    if let Ok(Ok(html)) = tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        http.fetch_text(explore_url),
    )
    .await
    {
        return Ok(parse_trends_from_html(&html));
    }

    Ok(Vec::new())
}

// ─── Parsers ─────────────────────────────────────────────────────

fn parse_tiktok_search_api(body: &str, max: usize) -> FetchiumResult<Vec<TikTokVideo>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| FetchiumError::Internal(format!("TikTok search API: {e}")))?;

    let items = match v["data"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    Ok(items
        .iter()
        .take(max)
        .filter_map(|item| {
            let av = &item["item"]["video"];
            let author = &item["item"]["author"];
            let stats = &item["item"]["stats"];
            let music = &item["item"]["music"];

            let id = item["item"]["id"].as_str()?.to_string();
            let desc = item["item"]["desc"].as_str().unwrap_or("").to_string();
            let hashtags = extract_hashtags_from_desc(&desc);

            Some(TikTokVideo {
                id: id.clone(),
                url: format!(
                    "https://www.tiktok.com/@{}/video/{}",
                    author["uniqueId"].as_str().unwrap_or("unknown"),
                    id
                ),
                author: TikTokUser {
                    username: author["uniqueId"].as_str().unwrap_or("").to_string(),
                    display_name: author["nickname"].as_str().unwrap_or("").to_string(),
                    followers: None,
                    following: None,
                    verified: author["verified"].as_bool().unwrap_or(false),
                    bio: author["signature"].as_str().unwrap_or("").to_string(),
                },
                description: desc,
                published: item["item"]["createTime"]
                    .as_u64()
                    .map(|t| t.to_string())
                    .unwrap_or_default(),
                duration_secs: av["duration"].as_u64().unwrap_or(0) as u32,
                likes: stats["diggCount"].as_u64().unwrap_or(0),
                comments: stats["commentCount"].as_u64().unwrap_or(0),
                shares: stats["shareCount"].as_u64().unwrap_or(0),
                plays: stats["playCount"].as_u64().unwrap_or(0),
                hashtags,
                music: music["title"].as_str().map(|title| TikTokMusic {
                    title: title.to_string(),
                    artist: music["authorName"].as_str().unwrap_or("").to_string(),
                    is_original: music["original"].as_bool().unwrap_or(false),
                }),
                is_duet: false,
                is_stitch: false,
            })
        })
        .collect())
}

fn parse_tiktok_html(html: &str, max: usize) -> Vec<TikTokVideo> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let mut videos = Vec::new();

    // TikTok hydrates state into a JSON script tag
    let script_sel = Selector::parse("script#__UNIVERSAL_DATA_FOR_REHYDRATION__").ok();
    if let Some(sel) = script_sel {
        if let Some(el) = doc.select(&sel).next() {
            let script_content = el.text().collect::<String>();
            if let Ok(v) = serde_json::from_str::<Value>(&script_content) {
                // Navigate the hydration tree for video items
                if let Some(items) = find_video_items(&v) {
                    for item in items.iter().take(max) {
                        if let Some(video) = hydration_to_video(item) {
                            videos.push(video);
                        }
                    }
                    return videos;
                }
            }
        }
    }

    // Fallback: parse visible video links
    let link_sel = Selector::parse("a[href*='/video/']").ok();
    if let Some(sel) = link_sel {
        for link in doc.select(&sel).take(max) {
            let href = link.value().attr("href").unwrap_or("").to_string();
            let text = link.text().collect::<String>().trim().to_string();
            if href.contains("/video/") && !text.is_empty() {
                let id = href.split("/video/").nth(1).unwrap_or("").to_string();
                videos.push(TikTokVideo {
                    id: id.clone(),
                    url: format!("https://www.tiktok.com{href}"),
                    author: TikTokUser {
                        username: "unknown".into(),
                        display_name: String::new(),
                        followers: None,
                        following: None,
                        verified: false,
                        bio: String::new(),
                    },
                    description: text,
                    published: String::new(),
                    duration_secs: 0,
                    likes: 0,
                    comments: 0,
                    shares: 0,
                    plays: 0,
                    hashtags: Vec::new(),
                    music: None,
                    is_duet: false,
                    is_stitch: false,
                });
            }
        }
    }
    videos
}

fn parse_creative_center_trends(body: &str) -> FetchiumResult<Vec<TikTokTrend>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| FetchiumError::Internal(format!("TikTok Creative Center: {e}")))?;

    let list = match v["data"]["list"].as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    Ok(list
        .iter()
        .filter_map(|item| {
            let tag = item["hashtag_name"].as_str()?;
            Some(TikTokTrend {
                hashtag: format!("#{tag}"),
                view_count: item["publish_cnt"].as_u64().unwrap_or(0),
                video_count: item["video_cnt"].as_u64(),
                is_challenge: item["is_challenge"].as_bool().unwrap_or(false),
            })
        })
        .collect())
}

fn parse_trends_from_html(html: &str) -> Vec<TikTokTrend> {
    use scraper::{Html, Selector};
    let doc = Html::parse_document(html);
    let mut trends = Vec::new();

    let hashtag_sel = Selector::parse("a[href*='/tag/']").ok();
    if let Some(sel) = hashtag_sel {
        for el in doc.select(&sel).take(30) {
            let text = el.text().collect::<String>().trim().to_string();
            if text.is_empty() || (!text.starts_with('#') && text.len() < 2) {
                continue;
            }
            let tag = if text.starts_with('#') {
                text
            } else {
                format!("#{text}")
            };
            trends.push(TikTokTrend {
                hashtag: tag,
                view_count: 0,
                video_count: None,
                is_challenge: false,
            });
        }
    }
    trends
}

// ─── Helpers ─────────────────────────────────────────────────────

fn extract_hashtags_from_desc(desc: &str) -> Vec<String> {
    desc.split_whitespace()
        .filter(|w| w.starts_with('#') && w.len() > 1)
        .map(|w| {
            w.trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .collect()
}

fn find_video_items(v: &Value) -> Option<&Vec<Value>> {
    // Try common TikTok hydration paths
    v["__DEFAULT_SCOPE__"]["webapp.video-detail"]["itemInfo"]["itemStruct"]
        .as_array()
        .or_else(|| v["items"].as_array())
}

fn hydration_to_video(item: &Value) -> Option<TikTokVideo> {
    let id = item["id"].as_str()?.to_string();
    let author = &item["author"];
    let stats = &item["stats"];
    let desc = item["desc"].as_str().unwrap_or("").to_string();
    let hashtags = extract_hashtags_from_desc(&desc);

    Some(TikTokVideo {
        id: id.clone(),
        url: format!(
            "https://www.tiktok.com/@{}/video/{}",
            author["uniqueId"].as_str().unwrap_or(""),
            id
        ),
        author: TikTokUser {
            username: author["uniqueId"].as_str().unwrap_or("").to_string(),
            display_name: author["nickname"].as_str().unwrap_or("").to_string(),
            followers: None,
            following: None,
            verified: author["verified"].as_bool().unwrap_or(false),
            bio: String::new(),
        },
        description: desc,
        published: item["createTime"]
            .as_u64()
            .map(|t| t.to_string())
            .unwrap_or_default(),
        duration_secs: item["video"]["duration"].as_u64().unwrap_or(0) as u32,
        likes: stats["diggCount"].as_u64().unwrap_or(0),
        comments: stats["commentCount"].as_u64().unwrap_or(0),
        shares: stats["shareCount"].as_u64().unwrap_or(0),
        plays: stats["playCount"].as_u64().unwrap_or(0),
        hashtags,
        music: None,
        is_duet: false,
        is_stitch: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashtags_from_desc() {
        let tags = extract_hashtags_from_desc("Learn #Rust #Programming today #foryou");
        assert!(tags.contains(&"#Rust".to_string()));
        assert!(tags.contains(&"#Programming".to_string()));
        assert!(tags.contains(&"#foryou".to_string()));
    }

    #[test]
    fn parse_search_empty_ok() {
        let json = r#"{"data":[]}"#;
        let videos = parse_tiktok_search_api(json, 10).unwrap();
        assert!(videos.is_empty());
    }
}
