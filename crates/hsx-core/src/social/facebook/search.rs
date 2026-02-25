//! Facebook search via free public methods.
//!
//! ## Strategy (in priority order)
//!
//! 1. **DDG `site:facebook.com` search** — completely free, no auth, returns
//!    URLs of public Facebook posts/pages. We extract Open Graph metadata from
//!    each URL to enrich results.
//! 2. **Graph API** — richer data (engagement counts, page info) when an
//!    optional `graph_api_token` is provided in config. The token can be a
//!    no-auth public token `{APP_ID}|{APP_SECRET}`.
//! 3. **Graceful degradation** — if all methods fail, returns empty Vec.
//!
//! ## Why not direct scraping?
//!
//! Facebook aggressively blocks scrapers: CAPTCHA, bot-detection, and
//! login walls. DDG + Open Graph is the most reliable free approach.

use crate::error::{HsxError, HsxResult};
use crate::http::client::HttpClient;
use crate::social::facebook::types::*;
use serde_json::Value;
use std::time::Duration;

/// Search Facebook via DuckDuckGo `site:facebook.com` and enrich with
/// Open Graph metadata from each discovered URL.
pub async fn search_posts(
    query: &str,
    config: &FacebookPipelineConfig,
    http: &HttpClient,
) -> HsxResult<(Vec<FacebookPost>, Vec<FacebookPage>, FacebookDataSource)> {
    // Try Graph API first if token is available
    if let Some(ref token) = config.graph_api_token {
        if let Ok(result) =
            search_graph_api(query, config.max_results, token, http, config.timeout_secs).await
        {
            if !result.0.is_empty() {
                return Ok((result.0, result.1, FacebookDataSource::GraphApi));
            }
        }
    }

    // Tier 2: Reddit + HN in parallel (3-5s faster than sequential)
    let ((reddit_posts, reddit_pages), (hn_posts, _hn_pages)) = tokio::join!(
        search_via_reddit(query, config.max_results, http),
        search_via_hackernews(query, config.max_results, http),
    );

    // Merge Reddit + HN results, dedup by ID
    if !reddit_posts.is_empty() || !hn_posts.is_empty() {
        let mut combined = reddit_posts;
        let existing_ids: std::collections::HashSet<String> =
            combined.iter().map(|p| p.id.clone()).collect();
        for p in hn_posts {
            if !existing_ids.contains(&p.id) {
                combined.push(p);
            }
        }
        let combined_pages = reddit_pages;
        if !combined.is_empty() || !combined_pages.is_empty() {
            tracing::info!("Facebook: {} posts from Reddit+HN", combined.len());
            return Ok((combined, combined_pages, FacebookDataSource::OpenGraph));
        }
    }

    // Tier 4: DDG/SearxNG site:facebook.com search (often CAPTCHA-blocked)
    let (posts, pages) = search_ddg_facebook_with_snippets(
        query,
        config.max_results,
        http,
        config.timeout_secs,
    )
    .await;

    let source = if posts.is_empty() && pages.is_empty() {
        FacebookDataSource::DdgSearch
    } else {
        FacebookDataSource::OpenGraph
    };
    Ok((posts, pages, source))
}

/// Search using Meta Graph API (requires optional token).
///
/// Token format: `{APP_ID}|{APP_SECRET}` — available with any Meta app,
/// or a user access token from the Graph API Explorer.
async fn search_graph_api(
    query: &str,
    max: usize,
    token: &str,
    http: &HttpClient,
    timeout_secs: u64,
) -> HsxResult<(Vec<FacebookPost>, Vec<FacebookPage>)> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let url = format!(
        "https://graph.facebook.com/v20.0/search?q={encoded}&type=page&fields=id,name,fan_count,about,category,link&limit={max}&access_token={token}"
    );

    let body = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url))
        .await
        .map_err(|_| HsxError::Internal("Graph API timeout".into()))?
        .map_err(|e| HsxError::Search(e.to_string()))?;

    let v: Value = serde_json::from_str(&body)
        .map_err(|e| HsxError::Internal(format!("Graph API JSON: {e}")))?;

    let items = match v["data"].as_array() {
        Some(a) => a,
        None => return Ok((Vec::new(), Vec::new())),
    };

    let pages: Vec<FacebookPage> = items
        .iter()
        .filter_map(|item| {
            let id = item["id"].as_str()?.to_string();
            let name = item["name"].as_str().unwrap_or("").to_string();
            Some(FacebookPage {
                url: item["link"]
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("https://www.facebook.com/{id}")),
                id,
                name,
                followers: item["fan_count"].as_u64(),
                likes: item["fan_count"].as_u64(),
                category: item["category"].as_str().unwrap_or("").to_string(),
                about: item["about"]
                    .as_str()
                    .unwrap_or("")
                    .chars()
                    .take(300)
                    .collect(),
                verified: false,
            })
        })
        .collect();

    Ok((Vec::new(), pages))
}

/// Search via SearXNG meta-engine and DDG for `site:facebook.com` content.
///
/// SearXNG instances aggregate multiple search engines server-side, bypassing
/// per-IP bot detection. DDG HTML is used as a secondary fallback.
async fn search_ddg_facebook_with_snippets(
    query: &str,
    max: usize,
    http: &HttpClient,
    timeout_secs: u64,
) -> (Vec<FacebookPost>, Vec<FacebookPage>) {
    // Tier 1: SearXNG public instances (aggregate Google/Bing/DDG server-side)
    let searxng_instances = [
        "https://search.ononoki.org",
        "https://search.sapti.me",
        "https://searx.be",
        "https://paulgo.io",
    ];
    let site_query = format!("site:facebook.com {query}");
    let encoded_q: String = url::form_urlencoded::byte_serialize(site_query.as_bytes()).collect();

    for instance in &searxng_instances {
        let url = format!("{instance}/search?q={encoded_q}&format=json&pageno=1&language=en");
        let resp = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            http.fetch_text(&url),
        )
        .await;
        if let Ok(Ok(body)) = resp {
            let (posts, pages) = parse_searxng_facebook_results(&body, query, max);
            if !posts.is_empty() || !pages.is_empty() {
                return (posts, pages);
            }
        }
    }

    // Tier 2: DDG HTML (may hit bot challenge, but worth trying)
    let queries = [
        format!("site:facebook.com {query}"),
        format!("site:facebook.com/posts {query}"),
        format!("site:facebook.com/groups {query}"),
    ];
    for ddg_query in &queries {
        let form: &[(&str, &str)] = &[("q", ddg_query), ("b", ""), ("kl", "en-us")];
        let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), async {
            http.client()
                .post("https://html.duckduckgo.com/html/")
                .form(form)
                .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
                .header("Accept-Language", "en-US,en;q=0.9")
                .header("Referer", "https://duckduckgo.com/")
                .send()
                .await
        })
        .await;
        if let Ok(Ok(r)) = resp {
            if r.status().is_success() {
                if let Ok(body) = r.text().await {
                    let (posts, pages) = parse_ddg_facebook_snippets(&body, query, max);
                    if !posts.is_empty() || !pages.is_empty() {
                        return (posts, pages);
                    }
                }
            }
        }
    }

    (Vec::new(), Vec::new())
}

/// Parse DDG HTML to extract Facebook posts and pages from search snippets.
///
/// This avoids fetching Facebook pages directly (which get blocked).
/// Parse SearXNG JSON response to extract Facebook posts and pages.
fn parse_searxng_facebook_results(
    json_str: &str,
    query: &str,
    max: usize,
) -> (Vec<FacebookPost>, Vec<FacebookPage>) {
    let v: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return (Vec::new(), Vec::new()),
    };
    let results = match v["results"].as_array() {
        Some(a) => a,
        None => return (Vec::new(), Vec::new()),
    };

    let mut posts = Vec::new();
    let mut pages = Vec::new();

    for item in results {
        if posts.len() + pages.len() >= max {
            break;
        }
        let raw_url = item["url"].as_str().unwrap_or("").to_string();
        if !raw_url.contains("facebook.com") {
            continue;
        }
        if raw_url.contains("/login") || raw_url.contains("/signin") || raw_url.contains("l.facebook.com") {
            continue;
        }
        let title = item["title"].as_str().unwrap_or("").to_string();
        let description = item["content"]
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| !s.contains("We would like to show you a description"))
            .unwrap_or_else(|| format!("Facebook content about {query}"));

        let is_post = raw_url.contains("/posts/")
            || raw_url.contains("/videos/")
            || raw_url.contains("story_fbid")
            || raw_url.contains("fbid=");

        if is_post {
            let page_name = extract_fb_page_name_from_url(&raw_url);
            let post_type = if raw_url.contains("/videos/") {
                FacebookPostType::Video
            } else {
                FacebookPostType::Link
            };
            posts.push(FacebookPost {
                id: extract_fb_id(&raw_url),
                url: raw_url.clone(),
                page_name,
                page_url: extract_page_url(&raw_url),
                message: description.chars().take(500).collect(),
                likes: 0,
                comments: 0,
                shares: 0,
                post_type,
                published: String::new(),
                media_url: None,
            });
        } else {
            pages.push(FacebookPage {
                id: extract_fb_id(&raw_url),
                name: if title.is_empty() { extract_fb_page_name_from_url(&raw_url) } else { title },
                url: raw_url,
                followers: None,
                likes: None,
                category: String::new(),
                about: description.chars().take(300).collect(),
                verified: false,
            });
        }
    }

    (posts, pages)
}

/// Parse DDG HTML response to extract Facebook posts and pages from search snippets.
fn parse_ddg_facebook_snippets(
    html: &str,
    query: &str,
    max: usize,
) -> (Vec<FacebookPost>, Vec<FacebookPage>) {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let result_sel = Selector::parse("div.result").expect("valid");
    let link_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid");
    let snippet_sel = Selector::parse("a.result__snippet, .result__snippet").expect("valid");

    let mut posts = Vec::new();
    let mut pages = Vec::new();

    for result in doc.select(&result_sel) {
        if posts.len() + pages.len() >= max {
            break;
        }
        let classes = result.value().attr("class").unwrap_or("");
        if classes.contains("result--ad") || classes.contains("result--more") {
            continue;
        }

        let (title, href) = match result.select(&link_sel).next() {
            Some(el) => {
                let t = el.text().collect::<String>().trim().to_string();
                let h = el.value().attr("href").unwrap_or("").to_string();
                (t, h)
            }
            None => continue,
        };

        let raw_url = clean_ddg_url(&href);
        if !raw_url.contains("facebook.com") {
            continue;
        }

        // Skip login/auth pages
        if raw_url.contains("/login") || raw_url.contains("/signin") || raw_url.contains("l.facebook.com") {
            continue;
        }

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Skip DDG placeholder text
        if snippet.contains("We would like to show you a description") {
            continue;
        }

        let description = if snippet.is_empty() {
            format!("Facebook content about {query}")
        } else {
            snippet
        };

        let is_post = raw_url.contains("/posts/")
            || raw_url.contains("/videos/")
            || raw_url.contains("story_fbid")
            || raw_url.contains("fbid=");

        if is_post {
            let page_name = extract_fb_page_name_from_url(&raw_url);
            let post_type = if raw_url.contains("/videos/") {
                FacebookPostType::Video
            } else {
                FacebookPostType::Link
            };
            posts.push(FacebookPost {
                id: extract_fb_id(&raw_url),
                url: raw_url.clone(),
                page_name,
                page_url: extract_page_url(&raw_url),
                message: description.chars().take(500).collect(),
                likes: 0,
                comments: 0,
                shares: 0,
                post_type,
                published: String::new(),
                media_url: None,
            });
        } else {
            // It's a page or group
            pages.push(FacebookPage {
                id: extract_fb_id(&raw_url),
                name: if title.is_empty() { extract_fb_page_name_from_url(&raw_url) } else { title },
                url: raw_url,
                followers: None,
                likes: None,
                category: String::new(),
                about: description.chars().take(300).collect(),
                verified: false,
            });
        }
    }

    (posts, pages)
}

/// Extract page name from a Facebook URL path component.
fn extract_fb_page_name_from_url(url: &str) -> String {
    // https://www.facebook.com/PageName/posts/123 → PageName
    let trimmed = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.facebook.com/")
        .trim_start_matches("facebook.com/");
    trimmed
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}


/// Parse DDG HTML to find `facebook.com` URLs in search results.
fn extract_facebook_urls_from_ddg_html(html: &str, max: usize) -> Vec<String> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let mut urls = Vec::new();

    // DDG wraps results in <a class="result__url"> or <a class="result__a">
    let sel = Selector::parse("a.result__a, a.result__url").ok();
    if let Some(sel) = sel {
        for a in doc.select(&sel) {
            let href = a.value().attr("href").unwrap_or("");
            if href.contains("facebook.com") {
                let clean = clean_ddg_url(href);
                if !clean.is_empty() && !urls.contains(&clean) {
                    urls.push(clean);
                    if urls.len() >= max {
                        break;
                    }
                }
            }
        }
    }

    // Fallback: scan all links in the page
    if urls.is_empty() {
        let all_sel = Selector::parse("a[href*='facebook.com']").ok();
        if let Some(sel) = all_sel {
            for a in doc.select(&sel).take(max * 3) {
                let href = a.value().attr("href").unwrap_or("");
                let clean = clean_ddg_url(href);
                if !clean.is_empty() && clean.contains("facebook.com") && !urls.contains(&clean) {
                    urls.push(clean);
                    if urls.len() >= max {
                        break;
                    }
                }
            }
        }
    }

    urls
}

/// Strip DDG redirect wrappers to get the raw Facebook URL.
fn clean_ddg_url(href: &str) -> String {
    // DDG sometimes wraps: /l/?uddg=https%3A%2F%2Fwww.facebook.com%2F...
    if href.contains("uddg=") {
        if let Some(start) = href.find("uddg=") {
            let raw = &href[start + 5..];
            let decoded: String = url::form_urlencoded::parse(raw.as_bytes())
                .map(|(k, _)| k.to_string())
                .next()
                .unwrap_or_default();
            if decoded.contains("facebook.com") {
                return decoded;
            }
        }
    }
    if href.contains("facebook.com") {
        return href.to_string();
    }
    String::new()
}

/// Extract a Facebook entity ID from a URL heuristically.
fn extract_fb_id(url: &str) -> String {
    // Try /posts/ID, /videos/ID, story_fbid=ID patterns
    for prefix in &["/posts/", "/videos/", "story_fbid=", "fbid=", "/"] {
        if let Some(idx) = url.find(prefix) {
            let after = &url[idx + prefix.len()..];
            let id: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !id.is_empty() {
                return id;
            }
        }
    }
    // Hash of URL as fallback ID
    format!(
        "{:x}",
        url.as_bytes()
            .iter()
            .fold(0u64, |a, &b| a.wrapping_mul(31).wrapping_add(b as u64))
    )
}

/// Extract the page URL component from a post URL.
fn extract_page_url(url: &str) -> String {
    // https://www.facebook.com/PageName/posts/123 → https://www.facebook.com/PageName
    let trimmed = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
    if parts.len() >= 2 {
        format!("https://{}/{}", parts[0], parts[1])
    } else {
        url.to_string()
    }
}

/// Fetch trending topics on Facebook via DDG search.
pub async fn fetch_trends(
    config: &FacebookPipelineConfig,
    http: &HttpClient,
) -> HsxResult<Vec<FacebookTrend>> {
    // Search for "trending" on Facebook via DDG
    let url = "https://html.duckduckgo.com/html/?q=site%3Afacebook.com+trending";
    let body = match tokio::time::timeout(
        Duration::from_secs(config.timeout_secs),
        http.fetch_text(url),
    )
    .await
    {
        Ok(Ok(b)) => b,
        _ => return Ok(Vec::new()),
    };

    let urls = extract_facebook_urls_from_ddg_html(&body, 10);
    if urls.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![FacebookTrend {
        topic: "Facebook Trending".into(),
        result_count: urls.len() as u64,
        sample_urls: urls,
    }])
}

/// Search Reddit for Facebook content using dual-phase approach.
///
/// **Phase 1:** URL-filtered (`url:facebook.com`) — finds actual FB links on Reddit.
/// **Phase 2:** Topic-broadened (`facebook <query>`) — finds discussions about Facebook content.
async fn search_via_reddit(
    query: &str,
    max: usize,
    http: &HttpClient,
) -> (Vec<FacebookPost>, Vec<FacebookPage>) {
    let client = http.client();
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let limit = max.min(25);

    // Phase 1: URL-filtered (strict — actual facebook.com links)
    let url_filtered = format!(
        "https://www.reddit.com/search.json?q={encoded}+url%3Afacebook.com&limit={limit}&sort=relevance&type=link"
    );
    let mut posts = reddit_json_to_fb_posts(client, &url_filtered, true).await;

    // Phase 2: Topic-broadened (if Phase 1 returned fewer than half max)
    if posts.len() < max / 2 {
        let remaining = max.saturating_sub(posts.len());
        let topic_url = format!(
            "https://www.reddit.com/search.json?q=facebook+{encoded}&limit={}&sort=relevance&type=link",
            remaining.min(25)
        );
        let topic_posts = reddit_json_to_fb_posts(client, &topic_url, false).await;
        let existing_ids: std::collections::HashSet<String> =
            posts.iter().map(|p| p.id.clone()).collect();
        for p in topic_posts {
            if !existing_ids.contains(&p.id) {
                posts.push(p);
            }
        }
    }

    posts.truncate(max);
    (posts, Vec::new())
}

/// Parse Reddit JSON into FacebookPost objects.
async fn reddit_json_to_fb_posts(
    client: &reqwest::Client,
    url: &str,
    require_platform_url: bool,
) -> Vec<FacebookPost> {
    // Reddit requires a descriptive UA to avoid 429s
    let resp = match client
        .get(url)
        .header("User-Agent", "HyperSearchX:fb-intel:v0.1 (research tool)")
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let v: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let children = match v["data"]["children"].as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    children
        .iter()
        .filter_map(|child| {
            let post = &child["data"];
            let post_url = post["url"].as_str().unwrap_or("");
            let is_fb_url = post_url.contains("facebook.com");

            if require_platform_url && !is_fb_url {
                return None;
            }

            let title = post["title"].as_str().unwrap_or("").to_string();
            let subreddit = post["subreddit"].as_str().unwrap_or("unknown");
            let score = post["score"].as_i64().unwrap_or(0);
            let comments = post["num_comments"].as_u64().unwrap_or(0);
            let created = post["created_utc"].as_f64().unwrap_or(0.0);
            let published = chrono::DateTime::from_timestamp(created as i64, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default();

            let display_url = if is_fb_url {
                post_url.to_string()
            } else {
                format!(
                    "https://reddit.com{}",
                    post["permalink"].as_str().unwrap_or("")
                )
            };

            let post_id = post["id"].as_str().unwrap_or("").to_string();

            Some(FacebookPost {
                id: if is_fb_url {
                    extract_fb_id(post_url)
                } else {
                    post_id
                },
                url: display_url.clone(),
                page_name: title.clone(),
                page_url: display_url,
                message: format!("{title} (via r/{subreddit})"),
                likes: score.max(0) as u64,
                comments,
                shares: 0,
                post_type: FacebookPostType::Link,
                published,
                media_url: None,
            })
        })
        .collect()
}

/// Search HackerNews for Facebook content using dual-phase approach.
///
/// Phase 1: URL-filtered (facebook.com links on HN).
/// Phase 2: Topic-broadened (HN stories mentioning Facebook/Meta).
async fn search_via_hackernews(
    query: &str,
    max: usize,
    http: &HttpClient,
) -> (Vec<FacebookPost>, Vec<FacebookPage>) {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();

    // Phase 1: URL-filtered
    let url1 = format!(
        "https://hn.algolia.com/api/v1/search?query={encoded}&tags=story&hitsPerPage={}&restrictSearchableAttributes=url,title",
        max.min(30)
    );
    let mut posts = hn_json_to_fb_posts(http, &url1, true).await;

    // Phase 2: Topic-broadened (if Phase 1 has few results)
    if posts.len() < max / 2 {
        let url2 = format!(
            "https://hn.algolia.com/api/v1/search?query=facebook+{encoded}&tags=story&hitsPerPage={}",
            max.min(30)
        );
        let topic_posts = hn_json_to_fb_posts(http, &url2, false).await;
        let existing_ids: std::collections::HashSet<String> =
            posts.iter().map(|p| p.id.clone()).collect();
        for p in topic_posts {
            if !existing_ids.contains(&p.id) {
                posts.push(p);
            }
        }
    }

    posts.truncate(max);
    (posts, Vec::new())
}

/// Parse HN Algolia JSON into FacebookPost objects.
async fn hn_json_to_fb_posts(
    http: &HttpClient,
    url: &str,
    require_platform_url: bool,
) -> Vec<FacebookPost> {
    let body = match http.fetch_text(url).await {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };

    let v: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let hits = match v["hits"].as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    hits.iter()
        .filter_map(|hit| {
            let hit_url = hit["url"].as_str().unwrap_or("");
            let is_fb_url = hit_url.contains("facebook.com");

            if require_platform_url && !is_fb_url {
                return None;
            }

            let title = hit["title"].as_str().unwrap_or("").to_string();
            let points = hit["points"].as_u64().unwrap_or(0);
            let comments = hit["num_comments"].as_u64().unwrap_or(0);
            let created = hit["created_at"].as_str().unwrap_or("").to_string();
            let obj_id = hit["objectID"].as_str().unwrap_or("").to_string();
            let display_url = if is_fb_url && !hit_url.is_empty() {
                hit_url.to_string()
            } else {
                format!("https://news.ycombinator.com/item?id={obj_id}")
            };

            Some(FacebookPost {
                id: if is_fb_url {
                    extract_fb_id(hit_url)
                } else {
                    obj_id
                },
                url: display_url.clone(),
                page_name: title.clone(),
                page_url: display_url,
                message: format!("{title} (via HackerNews)"),
                likes: points,
                comments,
                shares: 0,
                post_type: FacebookPostType::Link,
                published: created,
                media_url: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_fb_id_from_post_url() {
        let url = "https://www.facebook.com/SomePage/posts/1234567890";
        assert_eq!(extract_fb_id(url), "1234567890");
    }

    #[test]
    fn extract_fb_id_from_story_fbid() {
        let url = "https://www.facebook.com/photo?fbid=9876543210";
        assert_eq!(extract_fb_id(url), "9876543210");
    }

    #[test]
    fn extract_page_url_basic() {
        let url = "https://www.facebook.com/RustLang/posts/123";
        assert_eq!(extract_page_url(url), "https://www.facebook.com/RustLang");
    }

    #[test]
    fn clean_ddg_url_passthrough() {
        let url = "https://www.facebook.com/SomePage";
        assert_eq!(clean_ddg_url(url), url);
    }

    #[test]
    fn clean_ddg_url_non_facebook_empty() {
        let url = "https://www.example.com/page";
        assert_eq!(clean_ddg_url(url), "");
    }

    #[test]
    fn extract_facebook_urls_from_ddg_html_empty() {
        let urls = extract_facebook_urls_from_ddg_html("<html></html>", 10);
        assert!(urls.is_empty());
    }

    #[test]
    fn facebook_post_type_display() {
        assert_eq!(FacebookPostType::Video.to_string(), "video");
        assert_eq!(FacebookPostType::Reel.to_string(), "reel");
        assert_eq!(FacebookPostType::Photo.to_string(), "photo");
    }

    #[test]
    fn pipeline_config_defaults() {
        let cfg = FacebookPipelineConfig::default();
        assert_eq!(cfg.max_results, 20);
        assert!(cfg.graph_api_token.is_none());
        assert_eq!(cfg.timeout_secs, 15);
    }
}
