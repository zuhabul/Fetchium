//! Multi-source YouTube search — Innertube + Invidious/Piped/DDG racing.
//!
//! Search source priority (all race simultaneously, first wins):
//! 1. **YouTube Innertube API** — YouTube's own internal search, no auth, most reliable
//! 2. Invidious instances (race all)
//! 3. Piped instances (race all)
//! 4. yt-dlp subprocess (if installed)
//! 5. DuckDuckGo `site:youtube.com/watch` scraping (last resort)

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use serde_json::Value;
use std::time::Duration;

/// Search YouTube by racing ALL sources simultaneously.
///
/// Innertube is the highest-priority source — it's YouTube's own internal API
/// (used by the web player), requires no API key or third-party instances, and
/// is the most reliable. Other sources provide redundancy.
pub async fn search_youtube(
    query: &str,
    max_results: usize,
    http: &HttpClient,
    config: &crate::config::FetchiumConfig,
) -> FetchiumResult<(Vec<YouTubeSearchResult>, YouTubeSearchSource)> {
    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<(Vec<YouTubeSearchResult>, YouTubeSearchSource)>(8);

    let per_src = Duration::from_secs(config.youtube.timeout_secs);

    // Source 0: YouTube Innertube — most reliable, no third-party dependency
    {
        let tx = tx.clone();
        let http = http.clone();
        let q = query.to_string();
        tokio::spawn(async move {
            if let Ok(results) = search_innertube(&q, max_results, &http, per_src).await {
                if !results.is_empty() {
                    let _ = tx.send((results, YouTubeSearchSource::Innertube)).await;
                }
            }
        });
    }

    // Source 1: Invidious instances (race all)
    if !config.youtube.invidious_instances.is_empty() {
        let tx = tx.clone();
        let http = http.clone();
        let instances = config.youtube.invidious_instances.clone();
        let q = query.to_string();
        tokio::spawn(async move {
            if let Some(results) =
                search_invidious_parallel(&q, max_results, &instances, &http, per_src).await
            {
                let _ = tx.send((results, YouTubeSearchSource::Invidious)).await;
            }
        });
    }

    // Source 2: Piped instances (race all)
    if !config.youtube.piped_instances.is_empty() {
        let tx = tx.clone();
        let http = http.clone();
        let instances = config.youtube.piped_instances.clone();
        let q = query.to_string();
        tokio::spawn(async move {
            if let Some(results) =
                search_piped_parallel(&q, max_results, &instances, &http, per_src).await
            {
                let _ = tx.send((results, YouTubeSearchSource::Piped)).await;
            }
        });
    }

    // Source 3: yt-dlp subprocess (fast when installed)
    {
        let tx = tx.clone();
        let q = query.to_string();
        tokio::spawn(async move {
            if let Ok(results) = search_ytdlp(&q, max_results).await {
                if !results.is_empty() {
                    let _ = tx.send((results, YouTubeSearchSource::YtDlp)).await;
                }
            }
        });
    }

    // Source 4: DDG site:youtube.com/watch scraping (always attempted)
    {
        let tx = tx.clone();
        let http = http.clone();
        let q = query.to_string();
        tokio::spawn(async move {
            if let Ok(results) = search_via_ddg(&q, max_results, &http).await {
                if !results.is_empty() {
                    let _ = tx.send((results, YouTubeSearchSource::DuckDuckGo)).await;
                }
            }
        });
    }

    drop(tx);

    // Return first successful result; 12s global cap
    match tokio::time::timeout(Duration::from_secs(12), rx.recv()).await {
        Ok(Some((results, source))) => Ok((results, source)),
        _ => Err(FetchiumError::YouTube(format!(
            "YouTube search failed for '{query}' — all sources timed out or returned no results"
        ))),
    }
}

/// Race all Invidious instances simultaneously. Returns first non-empty result.
///
/// Bounded by `timeout + 1s` to ensure termination even if channel receive stalls.
async fn search_invidious_parallel(
    query: &str,
    max_results: usize,
    instances: &[String],
    http: &HttpClient,
    timeout: Duration,
) -> Option<Vec<YouTubeSearchResult>> {
    let cap = instances.len().max(1);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<YouTubeSearchResult>>(cap);
    for instance in instances {
        let tx = tx.clone();
        let http = http.clone();
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!("{instance}/api/v1/search?q={encoded}&type=video&page=1");
        // fetch_text_once: connection errors are expected for third-party instances;
        // single attempt fails fast instead of sleeping 3.5s on retries.
        tokio::spawn(async move {
            if let Ok(Ok(body)) = tokio::time::timeout(timeout, http.fetch_text_once(&url)).await {
                if let Ok(results) = parse_invidious_search_results(&body, max_results) {
                    if !results.is_empty() {
                        let _ = tx.send(results).await;
                    }
                }
            }
        });
    }
    drop(tx);
    // Cap at per-source timeout + 1s buffer so we never block forever
    tokio::time::timeout(timeout + Duration::from_secs(1), rx.recv())
        .await
        .ok()
        .flatten()
}

/// Race all Piped instances simultaneously. Returns first non-empty result.
///
/// Bounded by `timeout + 1s` to ensure termination even if channel receive stalls.
async fn search_piped_parallel(
    query: &str,
    max_results: usize,
    instances: &[String],
    http: &HttpClient,
    timeout: Duration,
) -> Option<Vec<YouTubeSearchResult>> {
    let cap = instances.len().max(1);
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<YouTubeSearchResult>>(cap);
    for instance in instances {
        let tx = tx.clone();
        let http = http.clone();
        let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
        let url = format!("{instance}/search?q={encoded}&filter=videos");
        // fetch_text_once: same rationale as Invidious above.
        tokio::spawn(async move {
            if let Ok(Ok(body)) = tokio::time::timeout(timeout, http.fetch_text_once(&url)).await {
                if let Ok(results) = parse_piped_search_results(&body, max_results) {
                    if !results.is_empty() {
                        let _ = tx.send(results).await;
                    }
                }
            }
        });
    }
    drop(tx);
    tokio::time::timeout(timeout + Duration::from_secs(1), rx.recv())
        .await
        .ok()
        .flatten()
}

// ─── YouTube Innertube Search ──────────────────────────────────

/// Search YouTube via the Innertube API — YouTube's own internal search used
/// by the web player. No API key, no third-party instances, most reliable.
///
/// Response path: contents → twoColumnSearchResultsRenderer →
///   primaryContents → sectionListRenderer → contents[] →
///   itemSectionRenderer → contents[] → videoRenderer
async fn search_innertube(
    query: &str,
    max_results: usize,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240101.00.00",
                "hl": "en",
                "gl": "US"
            }
        },
        "query": query,
        // EgIQAQ== = video filter (base64 protobuf for filter_type=video)
        "params": "EgIQAQ=="
    })
    .to_string();

    let response_text = tokio::time::timeout(timeout, async {
        http.client()
            .post("https://www.youtube.com/youtubei/v1/search?prettyPrint=false")
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            )
            .header("X-YouTube-Client-Name", "1")
            .header("X-YouTube-Client-Version", "2.20240101.00.00")
            .body(body)
            .send()
            .await
            .map_err(|e| FetchiumError::YouTube(format!("Innertube search send: {e}")))?
            .text()
            .await
            .map_err(|e| FetchiumError::YouTube(format!("Innertube search body: {e}")))
    })
    .await
    .map_err(|_| FetchiumError::YouTube("Innertube search timeout".into()))??;

    parse_innertube_search_results(&response_text, max_results)
}

/// Parse YouTube Innertube search response JSON.
fn parse_innertube_search_results(
    body: &str,
    max_results: usize,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| FetchiumError::YouTube(format!("Innertube JSON: {e}")))?;

    let mut results = Vec::new();

    // Navigate: contents → twoColumnSearchResultsRenderer → primaryContents →
    //           sectionListRenderer → contents[] → itemSectionRenderer → contents[]
    let primary = &v["contents"]["twoColumnSearchResultsRenderer"]["primaryContents"]
        ["sectionListRenderer"]["contents"];

    let sections = primary
        .as_array()
        .ok_or_else(|| FetchiumError::YouTube("No search sections in Innertube response".into()))?;

    'outer: for section in sections {
        let items = section["itemSectionRenderer"]["contents"]
            .as_array()
            .unwrap_or(&vec![])
            .to_vec();

        for item in &items {
            if results.len() >= max_results {
                break 'outer;
            }
            let vr = &item["videoRenderer"];
            let video_id = match vr["videoId"].as_str() {
                Some(id) if !id.is_empty() => id.to_string(),
                _ => continue,
            };

            let title = vr["title"]["runs"]
                .as_array()
                .and_then(|r| r.first())
                .and_then(|r| r["text"].as_str())
                .unwrap_or("")
                .to_string();

            if title.is_empty() {
                continue;
            }

            let description = vr["descriptionSnippet"]["runs"]
                .as_array()
                .and_then(|r| r.first())
                .and_then(|r| r["text"].as_str())
                .unwrap_or("")
                .to_string();

            // Channel name: Innertube uses shortBylineText, longBylineText, or ownerText
            let channel = ["shortBylineText", "longBylineText", "ownerText"]
                .iter()
                .find_map(|key| {
                    vr[*key]["runs"]
                        .as_array()
                        .and_then(|r| r.first())
                        .and_then(|r| r["text"].as_str())
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                })
                .unwrap_or_default();

            let duration_secs = vr["lengthText"]["simpleText"]
                .as_str()
                .map(parse_duration_text)
                .unwrap_or(0);

            let view_count = vr["viewCountText"]["simpleText"]
                .as_str()
                .map(parse_view_count_text)
                .unwrap_or(0);

            let published = vr["publishedTimeText"]["simpleText"]
                .as_str()
                .unwrap_or("")
                .to_string();

            let thumbnail_url = vr["thumbnail"]["thumbnails"]
                .as_array()
                .and_then(|arr| arr.last()) // last = highest resolution
                .and_then(|t| t["url"].as_str())
                .map(String::from)
                .or_else(|| Some(format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg")));

            results.push(YouTubeSearchResult {
                video_id,
                title,
                description,
                channel,
                duration_secs,
                view_count,
                published,
                thumbnail_url,
            });
        }
    }

    Ok(results)
}

/// Parse duration text like "10:30" or "1:23:45" into seconds.
fn parse_duration_text(s: &str) -> u64 {
    let parts: Vec<u64> = s.split(':').filter_map(|p| p.parse().ok()).collect();
    match parts.as_slice() {
        [m, s] => m * 60 + s,
        [h, m, s] => h * 3600 + m * 60 + s,
        _ => 0,
    }
}

/// Parse view count text like "1.2M views" or "543,210 views" into u64.
fn parse_view_count_text(s: &str) -> u64 {
    let clean = s
        .replace(" views", "")
        .replace(" view", "")
        .replace(',', "")
        .trim()
        .to_lowercase();
    if clean.ends_with('m') {
        let n: f64 = clean.trim_end_matches('m').trim().parse().unwrap_or(0.0);
        (n * 1_000_000.0) as u64
    } else if clean.ends_with('k') {
        let n: f64 = clean.trim_end_matches('k').trim().parse().unwrap_or(0.0);
        (n * 1_000.0) as u64
    } else if clean.ends_with('b') {
        let n: f64 = clean.trim_end_matches('b').trim().parse().unwrap_or(0.0);
        (n * 1_000_000_000.0) as u64
    } else {
        clean.parse().unwrap_or(0)
    }
}

/// Fetch trending videos.
pub async fn trending_videos(
    http: &HttpClient,
    config: &crate::config::FetchiumConfig,
    max_results: usize,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    for instance in &config.youtube.invidious_instances {
        let url = format!("{instance}/api/v1/trending");
        if let Ok(Ok(body)) = tokio::time::timeout(
            Duration::from_secs(config.youtube.timeout_secs),
            http.fetch_text_once(&url),
        )
        .await
        {
            if let Ok(results) = parse_invidious_search_results(&body, max_results) {
                return Ok(results);
            }
        }
    }

    for instance in &config.youtube.piped_instances {
        let url = format!("{instance}/trending");
        if let Ok(Ok(body)) = tokio::time::timeout(
            Duration::from_secs(config.youtube.timeout_secs),
            http.fetch_text_once(&url),
        )
        .await
        {
            if let Ok(results) = parse_piped_search_results(&body, max_results) {
                return Ok(results);
            }
        }
    }

    Err(FetchiumError::YouTube("Could not fetch trending videos".into()))
}

/// Search for related videos given a video ID.
pub async fn related_videos(
    video_id: &str,
    http: &HttpClient,
    config: &crate::config::FetchiumConfig,
    max_results: usize,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    for instance in &config.youtube.invidious_instances {
        let url = format!("{instance}/api/v1/videos/{video_id}");
        match tokio::time::timeout(
            Duration::from_secs(config.youtube.timeout_secs),
            http.fetch_text_once(&url),
        )
        .await
        {
            Ok(Ok(body)) => {
                let v: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
                if let Some(recommended) = v["recommendedVideos"].as_array() {
                    let results: Vec<YouTubeSearchResult> = recommended
                        .iter()
                        .take(max_results)
                        .filter_map(|r| {
                            Some(YouTubeSearchResult {
                                video_id: r["videoId"].as_str()?.to_string(),
                                title: r["title"].as_str().unwrap_or("").to_string(),
                                description: String::new(),
                                channel: r["author"].as_str().unwrap_or("").to_string(),
                                duration_secs: r["lengthSeconds"].as_u64().unwrap_or(0),
                                view_count: r["viewCount"].as_u64().unwrap_or(0),
                                published: String::new(),
                                thumbnail_url: r["videoThumbnails"]
                                    .as_array()
                                    .and_then(|a| a.first())
                                    .and_then(|t| t["url"].as_str())
                                    .map(String::from),
                            })
                        })
                        .collect();
                    if !results.is_empty() {
                        return Ok(results);
                    }
                }
            }
            _ => continue,
        }
    }
    Err(FetchiumError::YouTube("Could not fetch related videos".into()))
}

// ─── Invidious / Piped Parsers ─────────────────────────────────

fn parse_invidious_search_results(
    body: &str,
    max_results: usize,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    let arr: Vec<Value> = serde_json::from_str(body)
        .map_err(|e| FetchiumError::YouTube(format!("Invidious parse: {e}")))?;

    let results = arr
        .iter()
        .filter(|v| v["type"].as_str() == Some("video"))
        .take(max_results)
        .filter_map(|v| {
            Some(YouTubeSearchResult {
                video_id: v["videoId"].as_str()?.to_string(),
                title: v["title"].as_str().unwrap_or("").to_string(),
                description: v["description"].as_str().unwrap_or("").to_string(),
                channel: v["author"].as_str().unwrap_or("").to_string(),
                duration_secs: v["lengthSeconds"].as_u64().unwrap_or(0),
                view_count: v["viewCount"].as_u64().unwrap_or(0),
                published: v["publishedText"].as_str().unwrap_or("").to_string(),
                thumbnail_url: v["videoThumbnails"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|t| t["url"].as_str())
                    .map(String::from),
            })
        })
        .collect();

    Ok(results)
}

fn parse_piped_search_results(
    body: &str,
    max_results: usize,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    let v: Value =
        serde_json::from_str(body).map_err(|e| FetchiumError::YouTube(format!("Piped parse: {e}")))?;

    let items = v["items"]
        .as_array()
        .or_else(|| v.as_array())
        .ok_or_else(|| FetchiumError::YouTube("No items in Piped response".into()))?;

    let results = items
        .iter()
        .filter(|item| item["type"].as_str() == Some("stream") || item["url"].as_str().is_some())
        .take(max_results)
        .filter_map(|item| {
            let url_path = item["url"].as_str().unwrap_or("");
            let video_id = url_path
                .strip_prefix("/watch?v=")
                .unwrap_or(url_path)
                .to_string();
            if video_id.is_empty() {
                return None;
            }
            Some(YouTubeSearchResult {
                video_id,
                title: item["title"].as_str().unwrap_or("").to_string(),
                description: item["shortDescription"].as_str().unwrap_or("").to_string(),
                channel: item["uploaderName"].as_str().unwrap_or("").to_string(),
                duration_secs: item["duration"].as_u64().unwrap_or(0),
                view_count: item["views"].as_u64().unwrap_or(0),
                published: item["uploadedDate"].as_str().unwrap_or("").to_string(),
                thumbnail_url: item["thumbnail"].as_str().map(String::from),
            })
        })
        .collect();

    Ok(results)
}

// ─── DDG Fallback ──────────────────────────────────────────────

/// Search YouTube by scraping DuckDuckGo `site:youtube.com/watch` results.
///
/// Wraps the ENTIRE request — headers AND body — in a single 10s timeout to
/// prevent `.text().await` from hanging after headers arrive.
async fn search_via_ddg(
    query: &str,
    max_results: usize,
    http: &HttpClient,
) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    let ddg_query = format!("site:youtube.com/watch {query}");
    let encoded: String = url::form_urlencoded::byte_serialize(ddg_query.as_bytes()).collect();

    let body = tokio::time::timeout(Duration::from_secs(10), async {
        http.client()
            .post("https://html.duckduckgo.com/html/")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept-Language", "en-US,en;q=0.9")
            .body(format!("q={encoded}&kl=us-en"))
            .send()
            .await
            .map_err(|e| FetchiumError::YouTube(format!("DDG send: {e}")))?
            .text()
            .await
            .map_err(|e| FetchiumError::YouTube(format!("DDG body: {e}")))
    })
    .await
    .map_err(|_| FetchiumError::YouTube("DDG YouTube search timed out (10s)".into()))??;

    Ok(parse_ddg_youtube_results(&body, max_results))
}

fn parse_ddg_youtube_results(html: &str, max_results: usize) -> Vec<YouTubeSearchResult> {
    use scraper::{Html, Selector};

    let doc = Html::parse_document(html);
    let result_sel =
        Selector::parse("div.result").unwrap_or_else(|_| Selector::parse("div").unwrap());
    let title_sel = Selector::parse("a.result__a").ok();
    let snippet_sel = Selector::parse("a.result__snippet").ok();

    let mut results = Vec::new();

    for result in doc.select(&result_sel) {
        if results.len() >= max_results {
            break;
        }

        // Get title + URL from the link
        let (title, href) = if let Some(ref sel) = title_sel {
            match result.select(sel).next() {
                Some(el) => {
                    let title = el.text().collect::<String>().trim().to_string();
                    let href = el.value().attr("href").unwrap_or("").to_string();
                    (title, href)
                }
                None => continue,
            }
        } else {
            continue;
        };

        // Extract YouTube video ID from URL
        let video_id = extract_yt_video_id(&href);
        if video_id.is_empty() {
            continue;
        }

        let snippet = snippet_sel
            .as_ref()
            .and_then(|sel| result.select(sel).next())
            .map(|el| el.text().collect::<String>().trim().to_string())
            .unwrap_or_default();

        let thumbnail = Some(format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg"));

        results.push(YouTubeSearchResult {
            video_id,
            title,
            description: snippet,
            channel: String::new(), // filled in by oEmbed metadata later
            duration_secs: 0,
            view_count: 0,
            published: String::new(),
            thumbnail_url: thumbnail,
        });
    }

    results
}

/// Extract YouTube video ID from a URL string.
pub fn extract_yt_video_id(url: &str) -> String {
    // DDG redirects: /l/?kh=-1&uddg=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DVIDEO_ID
    // DDG redirects: /l/?uddg=https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3DVIDEO_ID
    // Manual percent-decode: replace %XX with the corresponding char
    let decoded = if url.contains("uddg=") {
        let raw = url.split("uddg=").nth(1).unwrap_or("");
        percent_decode_simple(raw)
    } else {
        url.to_string()
    };

    // Standard: youtube.com/watch?v=VIDEO_ID or youtu.be/VIDEO_ID
    if let Ok(parsed) = url::Url::parse(&decoded) {
        if let Some(host) = parsed.host_str() {
            if host.contains("youtube.com") {
                for (k, v) in parsed.query_pairs() {
                    if k == "v" && v.len() == 11 {
                        return v.into_owned();
                    }
                }
            }
            if host == "youtu.be" {
                let path = parsed.path().trim_start_matches('/');
                if path.len() == 11 {
                    return path.to_string();
                }
            }
        }
    }

    // Regex-free: find v= in raw URL
    for part in decoded.split("v=") {
        let id: String = part.chars().take(11).collect::<String>();
        if id.len() == 11
            && id
                .chars()
                .all(|c: char| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return id;
        }
    }

    String::new()
}

/// Minimal percent-decoding for ASCII URLs (handles %XX sequences).
fn percent_decode_simple(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ─── yt-dlp Fallback ───────────────────────────────────────────

async fn search_ytdlp(query: &str, max_results: usize) -> FetchiumResult<Vec<YouTubeSearchResult>> {
    // 10s cap so yt-dlp doesn't outlive the search phase when the network is slow.
    let output = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::process::Command::new("yt-dlp")
            .args([
                "--dump-single-json",
                "--flat-playlist",
                "--no-download",
                &format!("ytsearch{max_results}:{query}"),
            ])
            .output(),
    )
    .await
    .map_err(|_| FetchiumError::YouTube("yt-dlp timed out".into()))?
    .map_err(|e| FetchiumError::YouTube(format!("yt-dlp not available: {e}")))?;

    if !output.status.success() {
        return Err(FetchiumError::YouTube("yt-dlp search failed".into()));
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let v: Value =
        serde_json::from_str(&body).map_err(|e| FetchiumError::YouTube(format!("yt-dlp JSON: {e}")))?;

    let entries = v["entries"]
        .as_array()
        .ok_or_else(|| FetchiumError::YouTube("No entries in yt-dlp output".into()))?;

    let results = entries
        .iter()
        .take(max_results)
        .filter_map(|e| {
            Some(YouTubeSearchResult {
                video_id: e["id"].as_str()?.to_string(),
                title: e["title"].as_str().unwrap_or("").to_string(),
                description: e["description"].as_str().unwrap_or("").to_string(),
                channel: e["channel"]
                    .as_str()
                    .or(e["uploader"].as_str())
                    .unwrap_or("")
                    .to_string(),
                duration_secs: e["duration"].as_u64().unwrap_or(0),
                view_count: e["view_count"].as_u64().unwrap_or(0),
                published: e["upload_date"].as_str().unwrap_or("").to_string(),
                thumbnail_url: e["thumbnail"].as_str().map(String::from),
            })
        })
        .collect();

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_invidious_results() {
        let json = serde_json::json!([
            {
                "type": "video",
                "videoId": "abc123456",
                "title": "Rust Tutorial",
                "description": "Learn Rust",
                "author": "RustChannel",
                "lengthSeconds": 1200,
                "viewCount": 50000,
                "publishedText": "2 months ago",
                "videoThumbnails": [{"url": "https://img.youtube.com/vi/abc123456/0.jpg"}]
            },
            {
                "type": "video",
                "videoId": "def789012",
                "title": "Go Tutorial",
                "description": "Learn Go",
                "author": "GoChannel",
                "lengthSeconds": 900,
                "viewCount": 30000,
                "publishedText": "1 month ago",
                "videoThumbnails": []
            }
        ]);
        let results = parse_invidious_search_results(&json.to_string(), 10).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].video_id, "abc123456");
        assert_eq!(results[0].title, "Rust Tutorial");
        assert_eq!(results[1].channel, "GoChannel");
    }

    #[test]
    fn parse_piped_results() {
        let json = serde_json::json!({
            "items": [
                {
                    "type": "stream",
                    "url": "/watch?v=abc123456",
                    "title": "Async Rust",
                    "shortDescription": "Deep dive into async",
                    "uploaderName": "RustCh",
                    "duration": 1800,
                    "views": 25000,
                    "uploadedDate": "3 weeks ago",
                    "thumbnail": "https://thumb.example.com/1.jpg"
                }
            ]
        });
        let results = parse_piped_search_results(&json.to_string(), 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].video_id, "abc123456");
        assert_eq!(results[0].title, "Async Rust");
    }

    #[test]
    fn parse_piped_empty() {
        let json = serde_json::json!({ "items": [] });
        let results = parse_piped_search_results(&json.to_string(), 10).unwrap();
        assert!(results.is_empty());
    }
}
