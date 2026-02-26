//! Twitter/X search via three-tier approach:
//!
//! **Tier 1 – DuckDuckGo site search** (most reliable, no key)
//!   - `site:x.com <query>` via DDG HTML endpoint
//!   - Always returns results; snippets contain tweet content
//!
//! **Tier 2 – Nitter HTML scraping** (good when instances are up)
//!   - Multiple instances tried in parallel
//!   - Robust CSS selector fallbacks for different nitter variants
//!
//! **Tier 3 – Nitter RSS** (unreliable; many instances now require whitelist)
//!   - Skipped when body contains whitelist-error strings

use crate::error::{HsxError, HsxResult};
use crate::http::client::HttpClient;
use crate::social::twitter::types::*;
use std::time::Duration;

const DDG_HTML_URL: &str = "https://html.duckduckgo.com/html/";

/// Search Twitter/X tweets using a parallel multi-tier approach.
///
/// ## Tier 0 (best): Local SearXNG site:x.com (aggregates Google/Bing/DDG server-side)
/// ## Tier 1 (parallel): Reddit API + HackerNews Algolia (free, reliable APIs)
/// ## Tier 2 (fallback): Nitter HTML scraping (all instances in parallel)
/// ## Tier 3 (fallback): DDG site:x.com search (often CAPTCHA-blocked)
pub async fn search_tweets(
    query: &str,
    max: usize,
    config: &TwitterPipelineConfig,
    http: &HttpClient,
) -> HsxResult<Vec<Tweet>> {
    // ── Tier 0: Local SearXNG (fastest, most reliable when available) ───
    if let Some(ref searxng_url) = config.searxng_url {
        let tweets = search_via_searxng(query, max, searxng_url, http, config.timeout_secs).await;
        if !tweets.is_empty() {
            tracing::info!("Twitter: {} results from local SearXNG", tweets.len());
            return Ok(tweets);
        }
    }

    // ── Tier 1: Reddit + HN in parallel (3-5s faster than sequential) ──
    let (reddit_tweets, hn_tweets) = tokio::join!(
        search_via_reddit(query, max, http),
        search_via_hackernews(query, max, http),
    );

    // Prefer Reddit (usually more results), merge with HN for breadth
    if !reddit_tweets.is_empty() || !hn_tweets.is_empty() {
        let mut combined = reddit_tweets;
        let existing_urls: std::collections::HashSet<String> =
            combined.iter().map(|t| t.url.clone()).collect();
        for t in hn_tweets {
            if !existing_urls.contains(&t.url) {
                combined.push(t);
            }
        }
        combined.truncate(max);
        if !combined.is_empty() {
            tracing::info!("Twitter: {} results from Reddit+HN", combined.len());
            return Ok(combined);
        }
    }

    // ── Tier 2: Nitter HTML scraping (all instances in parallel) ────────
    let tweets = search_nitter_html_parallel(query, max, config, http).await;
    if !tweets.is_empty() {
        return Ok(tweets);
    }

    // ── Tier 3: DuckDuckGo site:x.com search (often CAPTCHA-blocked) ───
    if let Ok(tweets) = search_via_ddg(query, max, http).await {
        if !tweets.is_empty() {
            return Ok(tweets);
        }
    }

    Ok(Vec::new())
}

/// Search Twitter/X via local SearXNG JSON API with `site:x.com` query.
///
/// SearXNG aggregates Google, Bing, and DDG results server-side — bypasses
/// per-IP bot detection. Returns snippets from indexed tweets.
async fn search_via_searxng(
    query: &str,
    max: usize,
    searxng_url: &str,
    http: &HttpClient,
    timeout_secs: u64,
) -> Vec<Tweet> {
    let site_q = format!("site:x.com {query}");
    let encoded: String = url::form_urlencoded::byte_serialize(site_q.as_bytes()).collect();
    let url = format!("{searxng_url}/search?q={encoded}&format=json&pageno=1&language=en-US");

    let resp = tokio::time::timeout(Duration::from_secs(timeout_secs), http.fetch_text(&url)).await;
    let body = match resp {
        Ok(Ok(b)) => b,
        _ => return Vec::new(),
    };

    let v: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let results = match v["results"].as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    let mut tweets = Vec::new();
    for (i, item) in results.iter().take(max).enumerate() {
        let raw_url = item["url"].as_str().unwrap_or("").to_string();
        if !raw_url.contains("x.com") && !raw_url.contains("twitter.com") {
            continue;
        }
        let title = item["title"].as_str().unwrap_or("").to_string();
        let snippet = item["content"].as_str().unwrap_or("").to_string();
        let text = if snippet.is_empty() {
            title.clone()
        } else {
            snippet
        };

        let (username, tweet_id) = extract_twitter_id_and_author(&raw_url);
        let hashtags = extract_hashtags(&text);
        let mentions = extract_mentions(&text);
        tweets.push(Tweet {
            id: if tweet_id.is_empty() {
                format!("searxng-{i}")
            } else {
                tweet_id
            },
            url: raw_url,
            author: TwitterUser {
                username: username.clone(),
                display_name: username,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            text,
            published: String::new(),
            likes: 0,
            retweets: 0,
            replies: 0,
            views: None,
            hashtags,
            mentions,
            media_urls: Vec::new(),
            is_reply: false,
            is_retweet: false,
            quoted_tweet: None,
        });
    }
    tweets
}

/// Search Reddit for Twitter/X content using a dual-phase approach.
///
/// **Phase 1:** URL-filtered search (`url:x.com OR url:twitter.com`) — finds actual
/// tweet links shared on Reddit. High-fidelity but low volume.
///
/// **Phase 2:** Topic-based search (`twitter <query>`) — finds Reddit discussions
/// about Twitter content. Higher volume, broader coverage.
///
/// Results are merged and deduplicated by URL.
async fn search_via_reddit(query: &str, max: usize, http: &HttpClient) -> Vec<Tweet> {
    let client = http.client();
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let limit = max.min(25);

    // Phase 1: URL-filtered (strict — actual tweet links)
    let url_filtered = format!(
        "https://www.reddit.com/search.json?q={encoded}+url%3Ax.com+OR+url%3Atwitter.com&limit={limit}&sort=relevance&type=link"
    );
    let mut tweets = reddit_json_to_tweets(client, &url_filtered, true).await;

    // Phase 2: Topic-broadened (if Phase 1 returned fewer than half max)
    if tweets.len() < max / 2 {
        let remaining = max.saturating_sub(tweets.len());
        let topic_url = format!(
            "https://www.reddit.com/search.json?q=twitter+{encoded}&limit={}&sort=relevance&type=link",
            remaining.min(25)
        );
        let topic_tweets = reddit_json_to_tweets(client, &topic_url, false).await;

        // Merge, dedup by URL
        let existing_urls: std::collections::HashSet<String> =
            tweets.iter().map(|t| t.url.clone()).collect();
        for t in topic_tweets {
            if !existing_urls.contains(&t.url) {
                tweets.push(t);
            }
        }
    }

    tweets.truncate(max);
    tweets
}

/// Parse Reddit JSON search results into Tweet objects.
///
/// If `require_platform_url` is true, only includes posts linking to x.com/twitter.com.
/// If false, includes all results (topic-based search).
async fn reddit_json_to_tweets(
    client: &reqwest::Client,
    url: &str,
    require_platform_url: bool,
) -> Vec<Tweet> {
    // Reddit requires a descriptive UA to avoid 429s
    let resp = match client
        .get(url)
        .header(
            "User-Agent",
            "HyperSearchX:twitter-intel:v0.1 (research tool)",
        )
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
            let is_platform_url = post_url.contains("x.com") || post_url.contains("twitter.com");

            if require_platform_url && !is_platform_url {
                return None;
            }

            let title = post["title"].as_str().unwrap_or("").to_string();
            let selftext = post["selftext"].as_str().unwrap_or("");
            let subreddit = post["subreddit"].as_str().unwrap_or("unknown");
            let score = post["score"].as_i64().unwrap_or(0);
            let created = post["created_utc"].as_f64().unwrap_or(0.0);

            let username = if is_platform_url {
                extract_twitter_username(post_url)
            } else {
                None
            };

            let text = if selftext.is_empty() || selftext == "[deleted]" || selftext == "[removed]"
            {
                title.clone()
            } else {
                format!("{title}\n\n{}", truncate_text_safe(selftext, 280))
            };

            let published = chrono::DateTime::from_timestamp(created as i64, 0)
                .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                .unwrap_or_default();

            let display_url = if is_platform_url {
                post_url.to_string()
            } else {
                format!(
                    "https://reddit.com{}",
                    post["permalink"].as_str().unwrap_or("")
                )
            };

            Some(Tweet {
                id: post["id"].as_str().unwrap_or("").to_string(),
                url: display_url,
                author: TwitterUser {
                    username: username.unwrap_or_else(|| format!("via_r/{subreddit}")),
                    display_name: title,
                    verified: false,
                    followers: None,
                    following: None,
                    bio: format!("Shared on r/{subreddit}"),
                },
                text,
                published,
                likes: score.max(0) as u64,
                retweets: 0,
                replies: post["num_comments"].as_u64().unwrap_or(0),
                views: None,
                hashtags: Vec::new(),
                mentions: Vec::new(),
                media_urls: Vec::new(),
                is_reply: false,
                is_retweet: false,
                quoted_tweet: None,
            })
        })
        .collect()
}

/// Search HackerNews for Twitter/X content using dual-phase approach.
///
/// Phase 1: URL-filtered (x.com/twitter.com links on HN).
/// Phase 2: Topic-broadened (HN stories mentioning Twitter/X).
async fn search_via_hackernews(query: &str, max: usize, http: &HttpClient) -> Vec<Tweet> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();

    // Phase 1: URL-filtered — find actual tweet links
    let url1 = format!(
        "https://hn.algolia.com/api/v1/search?query={encoded}&tags=story&hitsPerPage={}&restrictSearchableAttributes=url,title",
        max.min(30)
    );
    let mut tweets = hn_json_to_tweets(http, &url1, true).await;

    // Phase 2: Topic-broadened — find HN discussions about Twitter/X content
    if tweets.len() < max / 2 {
        let url2 = format!(
            "https://hn.algolia.com/api/v1/search?query=twitter+{encoded}&tags=story&hitsPerPage={}",
            max.min(30)
        );
        let topic_tweets = hn_json_to_tweets(http, &url2, false).await;
        let existing_ids: std::collections::HashSet<String> =
            tweets.iter().map(|t| t.id.clone()).collect();
        for t in topic_tweets {
            if !existing_ids.contains(&t.id) {
                tweets.push(t);
            }
        }
    }

    tweets.truncate(max);
    tweets
}

/// Parse HN Algolia JSON into Tweet objects.
async fn hn_json_to_tweets(http: &HttpClient, url: &str, require_platform_url: bool) -> Vec<Tweet> {
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
            let is_platform = hit_url.contains("x.com") || hit_url.contains("twitter.com");

            if require_platform_url && !is_platform {
                return None;
            }

            let title = hit["title"].as_str().unwrap_or("").to_string();
            let hn_author = hit["author"].as_str().unwrap_or("unknown").to_string();
            let points = hit["points"].as_u64().unwrap_or(0);
            let comments = hit["num_comments"].as_u64().unwrap_or(0);
            let created = hit["created_at"].as_str().unwrap_or("").to_string();
            let username = if is_platform {
                extract_twitter_username(hit_url)
            } else {
                None
            };
            let obj_id = hit["objectID"].as_str().unwrap_or("").to_string();
            let display_url = if is_platform && !hit_url.is_empty() {
                hit_url.to_string()
            } else {
                format!("https://news.ycombinator.com/item?id={obj_id}")
            };

            Some(Tweet {
                id: obj_id,
                url: display_url,
                author: TwitterUser {
                    username: username.unwrap_or_else(|| format!("via_HN/{hn_author}")),
                    display_name: title.clone(),
                    verified: false,
                    followers: None,
                    following: None,
                    bio: format!("Shared on HN by {hn_author}"),
                },
                text: title,
                published: created,
                likes: points,
                retweets: 0,
                replies: comments,
                views: None,
                hashtags: Vec::new(),
                mentions: Vec::new(),
                media_urls: Vec::new(),
                is_reply: false,
                is_retweet: false,
                quoted_tweet: None,
            })
        })
        .collect()
}

/// Extract Twitter username from a tweet URL.
fn extract_twitter_username(url: &str) -> Option<String> {
    let path = url
        .strip_prefix("https://x.com/")
        .or_else(|| url.strip_prefix("https://twitter.com/"))
        .or_else(|| url.strip_prefix("http://x.com/"))
        .or_else(|| url.strip_prefix("http://twitter.com/"))?;
    let username = path.split('/').next()?;
    if username.is_empty() || username == "search" || username == "i" || username == "home" {
        return None;
    }
    Some(format!("@{username}"))
}

fn truncate_text_safe(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut idx = max;
        while idx > 0 && !s.is_char_boundary(idx) {
            idx -= 1;
        }
        format!("{}...", &s[..idx])
    }
}

/// Tier 1: Search via DuckDuckGo using `site:xcancel.com` (nitter mirror indexed by DDG).
///
/// xcancel.com is indexed by search engines and returns real tweet content.
/// Falls back to broader x.com/twitter.com search if nitter returns nothing.
async fn search_via_ddg(query: &str, max: usize, http: &HttpClient) -> HsxResult<Vec<Tweet>> {
    // Try xcancel.com (nitter mirror) first — best indexed nitter instance
    let ddg_query = format!("site:xcancel.com {query}");
    let form: &[(&str, &str)] = &[("q", &ddg_query), ("b", ""), ("kl", "en-us")];

    let resp_result = tokio::time::timeout(Duration::from_secs(12), async {
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
    .await
    .map_err(|_| HsxError::OperationTimeout {
        operation: "ddg_twitter_search".into(),
        timeout_ms: 12_000,
        suggestion: "Try again or check network".into(),
    })?
    .map_err(HsxError::Network)?;

    let status = resp_result.status().as_u16();
    let body: String = resp_result.text().await.map_err(HsxError::Network)?;

    if status >= 400 || body.len() < 200 {
        return Ok(Vec::new());
    }

    let tweets = parse_ddg_twitter_results(&body, max);
    if !tweets.is_empty() {
        return Ok(tweets);
    }

    // Fallback: try broader search on x.com + twitter.com
    let ddg_query2 = format!("site:x.com OR site:twitter.com \"{query}\"");
    let form2: &[(&str, &str)] = &[("q", &ddg_query2), ("b", ""), ("kl", "en-us")];
    let resp2 = tokio::time::timeout(Duration::from_secs(10), async {
        http.client()
            .post(DDG_HTML_URL)
            .form(form2)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Referer", "https://duckduckgo.com/")
            .send()
            .await
    }).await;
    if let Ok(Ok(r)) = resp2 {
        if r.status().is_success() {
            let b = r.text().await.unwrap_or_default();
            return Ok(parse_ddg_twitter_results(&b, max));
        }
    }

    Ok(Vec::new())
}

/// Parse DDG HTML results for Twitter/X tweet links + snippets.
fn parse_ddg_twitter_results(html: &str, max: usize) -> Vec<Tweet> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let result_sel = Selector::parse("div.result").expect("valid selector");
    let link_sel = Selector::parse("a.result__a, h2.result__title a").expect("valid selector");
    let snippet_sel =
        Selector::parse("a.result__snippet, .result__snippet").expect("valid selector");

    let mut tweets = Vec::new();

    for result in doc.select(&result_sel) {
        if tweets.len() >= max {
            break;
        }

        // Skip ads
        let classes = result.value().attr("class").unwrap_or("");
        if classes.contains("result--ad") || classes.contains("result--more") {
            continue;
        }

        let href = result
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .unwrap_or("");

        let raw_url = resolve_ddg_url(href);
        if !is_tweet_url(&raw_url) {
            continue;
        }

        let snippet = result
            .select(&snippet_sel)
            .next()
            .map(|e| e.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        // Skip DDG placeholder text (shows when site blocks snippets)
        if snippet.is_empty()
            || snippet.contains("We would like to show you a description")
            || snippet.contains("site won't allow us")
        {
            continue;
        }

        let (username, tweet_id) = extract_twitter_id_and_author(&raw_url);
        let hashtags = extract_hashtags(&snippet);
        let mentions = extract_mentions(&snippet);

        tweets.push(Tweet {
            id: tweet_id,
            url: raw_url,
            author: TwitterUser {
                username: username.clone(),
                display_name: username,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            text: snippet,
            published: String::new(), // DDG snippets don't include dates
            likes: 0,
            retweets: 0,
            replies: 0,
            views: None,
            hashtags,
            mentions,
            media_urls: Vec::new(),
            is_reply: false,
            is_retweet: false,
            quoted_tweet: None,
        });
    }

    tweets
}

/// Tier 2: Try all nitter instances for HTML search in **parallel** with 3s timeout each.
///
/// All instances are queried simultaneously; the first successful response wins.
/// Total wall-clock time ≤ 4s instead of N × 10s sequential.
async fn search_nitter_html_parallel(
    query: &str,
    max: usize,
    config: &TwitterPipelineConfig,
    http: &HttpClient,
) -> Vec<Tweet> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    // Hard cap on total nitter phase — never block the caller more than 5s
    let nitter_timeout = tokio::time::timeout(Duration::from_secs(5), async {
        let mut handles = Vec::new();
        for instance in &config.nitter_instances {
            let search_url = format!("{instance}/search?q={encoded}&f=tweets");
            let http2 = http.clone();
            let instance = instance.clone();
            handles.push(tokio::spawn(async move {
                match tokio::time::timeout(
                    Duration::from_secs(3), // short per-instance timeout
                    http2.fetch_text(&search_url),
                )
                .await
                {
                    Ok(Ok(html)) if !is_error_response(&html) => {
                        parse_nitter_html(&html, &instance, max)
                    }
                    _ => Vec::new(),
                }
            }));
        }
        // Collect results, return first non-empty
        let mut best: Vec<Tweet> = Vec::new();
        for h in handles {
            if let Ok(tweets) = h.await {
                if !tweets.is_empty() && (best.is_empty() || tweets.len() > best.len()) {
                    best = tweets;
                }
            }
        }
        best
    });

    nitter_timeout.await.unwrap_or_default()
}

/// Tier 3: Try nitter RSS in parallel with 3s timeout each.
#[allow(dead_code)]
async fn search_nitter_rss(
    query: &str,
    max: usize,
    config: &TwitterPipelineConfig,
    http: &HttpClient,
) -> Vec<Tweet> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();

    let result = tokio::time::timeout(Duration::from_secs(5), async {
        let mut handles = Vec::new();
        for instance in &config.nitter_instances {
            let rss_url = format!("{instance}/search/rss?q={encoded}&f=tweets");
            let http2 = http.clone();
            let instance = instance.clone();
            handles.push(tokio::spawn(async move {
                match tokio::time::timeout(Duration::from_secs(3), http2.fetch_text(&rss_url)).await
                {
                    Ok(Ok(body)) if !is_error_response(&body) && body.contains("<item>") => {
                        parse_nitter_rss(&body, &instance, max)
                    }
                    _ => Vec::new(),
                }
            }));
        }
        let mut best: Vec<Tweet> = Vec::new();
        for h in handles {
            if let Ok(tweets) = h.await {
                if tweets.len() > best.len() {
                    best = tweets;
                }
            }
        }
        best
    });

    result.await.unwrap_or_default()
}

/// Fetch trending topics from nitter instances.
pub async fn fetch_trends(
    config: &TwitterPipelineConfig,
    http: &HttpClient,
) -> HsxResult<Vec<TwitterTrend>> {
    for instance in &config.nitter_instances {
        // Try /about and / for trending sidebar
        for path in &["/", "/about"] {
            let url = format!("{instance}{path}");
            match tokio::time::timeout(
                Duration::from_secs(config.timeout_secs),
                http.fetch_text(&url),
            )
            .await
            {
                Ok(Ok(html)) if !is_error_response(&html) => {
                    let trends = parse_trends_from_html(&html);
                    if !trends.is_empty() {
                        return Ok(trends);
                    }
                }
                _ => continue,
            }
        }
    }
    Ok(Vec::new())
}

// ─── Nitter HTML Parser ──────────────────────────────────────────

/// Parse Nitter (any variant) HTML search results into Tweet objects.
///
/// Handles multiple nitter UI variants (nitter.net, xcancel, nitter.poast, etc.)
/// by trying multiple CSS selector alternatives.
fn parse_nitter_html(html: &str, instance: &str, max: usize) -> Vec<Tweet> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let mut tweets = Vec::new();

    // Try multiple container selectors (different nitter forks)
    let container_options = [
        ".timeline-item:not(.show-more)",
        ".timeline-item",
        ".tweet-container",
        "div[data-tweet-id]",
    ];
    let containers: Vec<scraper::ElementRef<'_>> = container_options
        .iter()
        .find_map(|sel| Selector::parse(sel).ok().map(|s| doc.select(&s).collect()))
        .unwrap_or_default();

    for card in containers.iter().take(max) {
        // ── Tweet text (try multiple selectors) ──────────────────────
        let text = try_selectors(
            card,
            &[
                ".tweet-content",
                ".tweet-body",
                "div.tweet-content",
                "p.tweet-text",
            ],
        );
        if text.is_empty() {
            continue;
        }

        // ── Author ────────────────────────────────────────────────────
        let username_raw = try_selectors(
            card,
            &[
                "a.username",
                ".username",
                ".tweet-username",
                "a[href*='/status/']",
            ],
        );
        let username = username_raw.trim_start_matches('@').to_string();

        // ── URL from tweet-date link or direct status link ────────────
        let tweet_url = try_attr(
            card,
            &["a.tweet-date", "span.tweet-date a", "a[href*='/status/']"],
            "href",
        )
        .unwrap_or_default();
        let full_url = if tweet_url.starts_with('/') {
            format!("{instance}{tweet_url}")
        } else if tweet_url.is_empty() {
            String::new()
        } else {
            tweet_url.clone()
        };
        let tweet_id = full_url
            .split("/status/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string();

        // ── Published date ────────────────────────────────────────────
        let published = try_attr(
            card,
            &["span.tweet-date a", "a.tweet-date", "time"],
            "title",
        )
        .or_else(|| try_attr(card, &["time"], "datetime"))
        .unwrap_or_default();

        // ── Engagement stats ──────────────────────────────────────────
        let stat_containers =
            Selector::parse(".icon-container, .tweet-stat, .tweet-stats span").ok();
        let stat_vals: Vec<u64> = stat_containers
            .as_ref()
            .map(|s| {
                card.select(s)
                    .map(|e| parse_count_text(e.text().collect::<String>().trim()))
                    .collect()
            })
            .unwrap_or_default();

        let replies = stat_vals.first().copied().unwrap_or(0);
        let retweets = stat_vals.get(1).copied().unwrap_or(0);
        let likes = stat_vals.get(2).copied().unwrap_or(0);

        let hashtags = extract_hashtags(&text);
        let mentions = extract_mentions(&text);

        tweets.push(Tweet {
            id: tweet_id,
            url: full_url,
            author: TwitterUser {
                username: username.clone(),
                display_name: username,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            text,
            published,
            likes,
            retweets,
            replies,
            views: None,
            hashtags,
            mentions,
            media_urls: Vec::new(),
            is_reply: false,
            is_retweet: false,
            quoted_tweet: None,
        });
    }

    tweets
}

// ─── Nitter RSS Parser ────────────────────────────────────────────

fn parse_nitter_rss(xml: &str, instance: &str, max: usize) -> Vec<Tweet> {
    let mut tweets = Vec::new();

    for item in xml.split("<item>").skip(1) {
        if tweets.len() >= max {
            break;
        }

        let link = extract_xml_text(item, "link");
        let pub_date = extract_xml_text(item, "pubDate");
        let title = extract_xml_text(item, "title");
        let desc = extract_xml_text(item, "description");

        if link.is_empty() || is_error_response(&link) {
            continue;
        }

        // Author: strip scheme+host from URL → /username/status/id
        let path = link
            .trim_start_matches("https://")
            .trim_start_matches("http://");
        // Remove host (everything up to first /)
        let path_after_host = path.split_once('/').map(|x| x.1).unwrap_or("");
        let author_name = path_after_host
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let tweet_id = link
            .split("/status/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string();

        let text = if desc.is_empty() {
            title.clone()
        } else {
            strip_html(&desc)
        };
        if text.is_empty() {
            continue;
        }

        let _ = instance; // already embedded in link
        let hashtags = extract_hashtags(&text);
        let mentions = extract_mentions(&text);

        tweets.push(Tweet {
            id: tweet_id,
            url: link,
            author: TwitterUser {
                username: author_name.clone(),
                display_name: author_name,
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            text,
            published: pub_date,
            likes: 0,
            retweets: 0,
            replies: 0,
            views: None,
            hashtags,
            mentions,
            media_urls: Vec::new(),
            is_reply: false,
            is_retweet: false,
            quoted_tweet: None,
        });
    }

    tweets
}

// ─── Trend Parser ─────────────────────────────────────────────────

fn parse_trends_from_html(html: &str) -> Vec<TwitterTrend> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let mut trends = Vec::new();

    // Multiple selectors for different nitter sidebar implementations
    for selector_str in &[
        ".trending-card a",
        ".trend-link",
        "#trending-tags a",
        ".trending a",
        ".sidebar-links a",
    ] {
        if let Ok(sel) = Selector::parse(selector_str) {
            for el in doc.select(&sel).take(20) {
                let topic = el.text().collect::<String>().trim().to_string();
                if topic.is_empty() || topic.len() > 80 {
                    continue;
                }
                let href = el.value().attr("href").unwrap_or("").to_string();
                trends.push(TwitterTrend {
                    topic,
                    tweet_volume: None,
                    region: "Global".into(),
                    url: href,
                });
            }
            if !trends.is_empty() {
                break;
            }
        }
    }

    trends
}

// ─── Helpers ──────────────────────────────────────────────────────

/// Check if a response body is an error/whitelist/block page.
fn is_error_response(body: &str) -> bool {
    // Short bodies are likely errors
    if body.len() < 100 {
        return true;
    }
    let lower = body.to_lowercase();
    lower.contains("whitelist")
        || lower.contains("not yet whitelist")
        || lower.contains("access denied")
        || lower.contains("403 forbidden")
        || lower.contains("blocked")
        || lower.contains("captcha")
        || lower.contains("cloudflare")
        || (lower.contains("error") && body.len() < 500)
}

/// Check if a URL is a Twitter/X tweet or profile URL.
fn is_tweet_url(url: &str) -> bool {
    // Accept nitter-proxied tweet pages (xcancel.com, lightbrd.com, etc.)
    let is_nitter =
        url.contains("xcancel.com/") || url.contains("lightbrd.com/") || url.contains("nitter.");

    // Accept direct Twitter/X tweet status pages only (must have /status/)
    let is_direct_tweet = (url.contains("twitter.com/") || url.contains("x.com/"))
        && url.contains("/status/")
        && !url.contains("developer.twitter.com")
        && !url.contains("developer.x.com")
        && !url.contains("help.twitter.com")
        && !url.contains("business.twitter.com")
        && !url.contains("/search")
        && !url.contains("/home")
        && !url.contains("/i/")
        && !url.contains("duckduckgo.com");

    is_nitter || is_direct_tweet
}

/// Resolve DDG redirect URLs to the actual target URL.
fn resolve_ddg_url(href: &str) -> String {
    if href.is_empty() {
        return String::new();
    }

    // DDG redirect format: //duckduckgo.com/l/?uddg=ENCODED_URL&...
    // Try to parse `uddg` query param using url::form_urlencoded
    if href.contains("uddg=") {
        let qs_start = href.find('?').map(|i| i + 1).unwrap_or(0);
        let qs = &href[qs_start..];
        for (k, v) in url::form_urlencoded::parse(qs.as_bytes()) {
            if k == "uddg" && v.starts_with("http") {
                return v.into_owned();
            }
        }
    }

    // Already a full URL that isn't a DDG redirect
    if (href.starts_with("https://") || href.starts_with("http://"))
        && !href.contains("duckduckgo.com")
    {
        return href.to_string();
    }

    String::new()
}

/// Extract (username, tweet_id) from a Twitter/X or Nitter URL.
///
/// Handles:
/// - `https://twitter.com/username/status/id`
/// - `https://x.com/username/status/id`
/// - `https://xcancel.com/username/status/id` (nitter)
/// - `https://lightbrd.com/username/status/id` (nitter)
fn extract_twitter_id_and_author(url: &str) -> (String, String) {
    let path = url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split_once('/') // skip host
        .map(|x| x.1)
        .unwrap_or("");

    // Nitter format: /username/status/id  (same as Twitter format)
    let parts: Vec<&str> = path.split('/').collect();
    let username = parts.first().copied().unwrap_or("").to_string();
    let tweet_id = if parts.len() >= 3 && parts[1] == "status" {
        parts[2].split('?').next().unwrap_or("").to_string()
    } else if parts.len() >= 2 && !parts[1].is_empty() {
        // Profile URL — no status ID
        String::new()
    } else {
        String::new()
    };

    (username, tweet_id)
}

/// Try multiple CSS selectors and return the first non-empty text match.
fn try_selectors<'a>(el: &scraper::ElementRef<'a>, selectors: &[&str]) -> String {
    for s in selectors {
        if let Ok(sel) = scraper::Selector::parse(s) {
            if let Some(found) = el.select(&sel).next() {
                let text = found.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }
    String::new()
}

/// Try multiple CSS selectors and return the first non-empty attribute match.
fn try_attr<'a>(el: &scraper::ElementRef<'a>, selectors: &[&str], attr: &str) -> Option<String> {
    for s in selectors {
        if let Ok(sel) = scraper::Selector::parse(s) {
            if let Some(found) = el.select(&sel).next() {
                if let Some(val) = found.value().attr(attr) {
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
            }
        }
    }
    None
}

fn extract_xml_text(xml: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    if let Some(start) = xml.find(&open) {
        let after = &xml[start + open.len()..];
        if let Some(end) = after.find(&close) {
            let raw = &after[..end];
            let clean = raw.trim_start_matches("<![CDATA[").trim_end_matches("]]>");
            return clean.trim().to_string();
        }
    }
    String::new()
}

fn strip_html(html: &str) -> String {
    let re = regex::Regex::new(r"<[^>]+>").unwrap_or_else(|_| regex::Regex::new(r"a").unwrap());
    re.replace_all(html, " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_hashtags(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.starts_with('#') && w.len() > 1)
        .map(|w| {
            w.trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .collect()
}

fn extract_mentions(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter(|w| w.starts_with('@') && w.len() > 1)
        .map(|w| {
            w.trim_end_matches(|c: char| !c.is_alphanumeric())
                .to_string()
        })
        .collect()
}

fn parse_count_text(s: &str) -> u64 {
    let clean: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == 'K' || *c == 'M')
        .collect();
    if clean.ends_with('M') {
        (clean.trim_end_matches('M').parse::<f64>().unwrap_or(0.0) * 1_000_000.0) as u64
    } else if clean.ends_with('K') {
        (clean.trim_end_matches('K').parse::<f64>().unwrap_or(0.0) * 1_000.0) as u64
    } else {
        clean.parse().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_hashtags_basic() {
        let tags = extract_hashtags("Hello #Rust #Programming world");
        assert_eq!(tags, vec!["#Rust", "#Programming"]);
    }

    #[test]
    fn extract_mentions_basic() {
        let m = extract_mentions("Hey @alice and @bob!");
        assert_eq!(m[0], "@alice");
        assert_eq!(m[1], "@bob");
    }

    #[test]
    fn parse_count_k() {
        assert_eq!(parse_count_text("12K"), 12_000);
        assert_eq!(parse_count_text("1.5M"), 1_500_000);
        assert_eq!(parse_count_text("42"), 42);
    }

    #[test]
    fn xml_extract_cdata() {
        let xml = "<title><![CDATA[Hello World]]></title>";
        assert_eq!(extract_xml_text(xml, "title"), "Hello World");
    }

    #[test]
    fn is_error_response_detects_whitelist() {
        assert!(is_error_response(
            "RSS reader not yet whitelist! Plain request"
        ));
        assert!(is_error_response("Access denied"));
        assert!(!is_error_response("<html><head><title>Nitter</title></head><body><div class='timeline-item'><div class='tweet-content'>This is a real tweet about Rust programming language</div></div></body></html>"));
    }

    #[test]
    fn is_tweet_url_valid() {
        assert!(is_tweet_url("https://x.com/rustlang/status/123456"));
        assert!(is_tweet_url("https://twitter.com/user/status/789"));
        assert!(!is_tweet_url("https://duckduckgo.com"));
        assert!(!is_tweet_url("https://twitter.com/search?q=rust"));
    }

    #[test]
    fn resolve_ddg_url_passthrough() {
        let url = "https://x.com/rustlang/status/123";
        assert_eq!(resolve_ddg_url(url), url);
    }

    #[test]
    fn extract_twitter_id_and_author_basic() {
        let (user, id) = extract_twitter_id_and_author("https://x.com/rustlang/status/123456");
        assert_eq!(user, "rustlang");
        assert_eq!(id, "123456");
    }

    #[test]
    fn extract_author_no_status() {
        let (user, id) = extract_twitter_id_and_author("https://twitter.com/rustlang");
        assert_eq!(user, "rustlang");
        assert!(id.is_empty());
    }

    #[test]
    fn is_error_response_short_body() {
        assert!(is_error_response("error"));
        assert!(is_error_response(""));
    }

    #[test]
    fn parse_ddg_twitter_results_empty_html() {
        let tweets = parse_ddg_twitter_results("<html><body></body></html>", 10);
        assert!(tweets.is_empty());
    }

    #[test]
    fn parse_nitter_rss_whitelist_skipped() {
        // A whitelist error body contains no <item> tags → empty result
        let xml = "RSS reader not yet whitelist! Plain request with just the ID...";
        let tweets = parse_nitter_rss(xml, "https://xcancel.com", 10);
        assert!(tweets.is_empty());
    }
}
