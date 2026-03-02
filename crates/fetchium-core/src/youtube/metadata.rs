//! Video metadata extraction — chapters, links, channel credibility scoring.
//!
//! All functions are pure (no I/O) except `fetch_metadata` which calls APIs.

use crate::error::{HsxError, HsxResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use chrono::{Duration as ChronoDuration, Utc};
use serde_json::Value;
use std::time::Duration;

// ─── Chapter Extraction ────────────────────────────────────────

/// Extract chapters from a video description using timestamp patterns.
///
/// Matches lines like `0:00 Introduction`, `1:23:45 Deep dive`, etc.
pub fn extract_chapters(description: &str) -> Vec<Chapter> {
    let re = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"(?m)^[\s\-]*(\d{1,2}:)?(\d{1,2}):(\d{2})\s+(.+)$").unwrap()
    });

    let mut chapters: Vec<Chapter> = Vec::new();

    for cap in re.captures_iter(description) {
        let hours: u64 = cap
            .get(1)
            .map(|m| m.as_str().trim_end_matches(':').parse().unwrap_or(0))
            .unwrap_or(0);
        let minutes: u64 = cap[2].parse().unwrap_or(0);
        let seconds: u64 = cap[3].parse().unwrap_or(0);
        let start_secs = hours * 3600 + minutes * 60 + seconds;
        let title = cap[4].trim().to_string();

        chapters.push(Chapter {
            title,
            start_secs,
            end_secs: None,
        });
    }

    // Fill in end times from next chapter's start
    for i in 0..chapters.len().saturating_sub(1) {
        chapters[i].end_secs = Some(chapters[i + 1].start_secs);
    }

    chapters
}

// ─── Link Extraction ───────────────────────────────────────────

/// Extract and classify links from a video description.
pub fn extract_links(description: &str) -> Vec<DescriptionLink> {
    let re = once_cell::sync::Lazy::new(|| regex::Regex::new(r"https?://[^\s<>\]\)]+").unwrap());

    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for m in re.find_iter(description) {
        let url = m.as_str().trim_end_matches(['.', ',', ')']);
        if !seen.insert(url.to_string()) {
            continue;
        }
        let domain = extract_domain(url);
        let link_type = classify_link(&domain, url);
        links.push(DescriptionLink {
            url: url.to_string(),
            domain,
            link_type,
        });
    }

    links
}

/// Extract domain from a URL.
fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| u.host_str().unwrap_or("unknown").to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Classify a link based on its domain.
fn classify_link(domain: &str, _url: &str) -> LinkType {
    let d = domain.to_lowercase();
    if d.contains("twitter.com")
        || d.contains("x.com")
        || d.contains("instagram.com")
        || d.contains("facebook.com")
        || d.contains("tiktok.com")
        || d.contains("linkedin.com")
        || d.contains("discord.gg")
        || d.contains("t.me")
    {
        return LinkType::Social;
    }
    if d.contains("github.com")
        || d.contains("gitlab.com")
        || d.contains("bitbucket.org")
        || d.contains("codepen.io")
        || d.contains("replit.com")
    {
        return LinkType::Code;
    }
    if d.contains("docs.")
        || d.contains("readthedocs")
        || d.contains("docs.rs")
        || d.contains("developer.")
        || d.contains("wiki")
    {
        return LinkType::Documentation;
    }
    if d.contains("amazon.")
        || d.contains("gumroad")
        || d.contains("patreon")
        || d.contains("buymeacoffee")
        || d.contains("shop.")
    {
        return LinkType::Product;
    }
    if d.contains("medium.com")
        || d.contains("substack")
        || d.contains("blog")
        || d.contains("dev.to")
        || d.contains("hashnode")
    {
        return LinkType::Article;
    }
    LinkType::Other
}

// ─── Channel Credibility ───────────────────────────────────────

/// Score a channel's credibility based on available info.
pub fn score_channel_credibility(channel: &ChannelInfo) -> ChannelCredibility {
    let sub_score = match channel.subscriber_count {
        Some(s) if s >= 1_000_000 => 1.0,
        Some(s) if s >= 100_000 => 0.8,
        Some(s) if s >= 10_000 => 0.6,
        Some(s) if s >= 1_000 => 0.4,
        Some(_) => 0.2,
        None => 0.3,
    };

    let tier = match channel.subscriber_count {
        Some(s) if s >= 1_000_000 => CredibilityTier::Mega,
        Some(s) if s >= 100_000 => CredibilityTier::Authority,
        Some(s) if s >= 10_000 => CredibilityTier::Established,
        Some(s) if s >= 100 => CredibilityTier::Emerging,
        _ => CredibilityTier::Unknown,
    };

    let verified_bonus = if channel.verified { 0.15 } else { 0.0 };
    let consistency_score = 0.5; // default without history data

    let raw: f64 = sub_score * 0.5 + consistency_score * 0.35 + verified_bonus;
    let score = raw.min(1.0);

    ChannelCredibility {
        score,
        tier,
        factors: CredibilityFactors {
            subscriber_score: sub_score,
            consistency_score,
            verified_bonus,
        },
    }
}

// ─── Metadata Fetching ─────────────────────────────────────────

/// Fetch video metadata by racing ALL sources simultaneously.
///
/// Sources (all start at the same time, first valid result wins):
/// 1. YouTube Innertube player API — rich metadata (title, description, duration,
///    views, likes, channel, keywords). Same API used by YouTube's web player.
/// 2. YouTube oEmbed — title + channel only; always available; ~150ms
/// 3. Invidious API — rich metadata when available
/// 4. Piped API — rich metadata when available
///
/// Innertube and oEmbed always work without auth. Guarantees a result in <4s.
pub async fn fetch_metadata(
    video_id: &str,
    http: &HttpClient,
    config: &crate::config::HsxConfig,
) -> HsxResult<VideoMetadata> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<VideoMetadata>(10);
    let timeout = Duration::from_secs(config.youtube.timeout_secs);

    // Source 0: YouTube Innertube player (rich metadata, no auth, same API as transcript)
    {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok(meta) = fetch_metadata_innertube(&vid, &http, timeout).await {
                let _ = tx.send(meta).await;
            }
        });
    }

    // Source 1: oEmbed (always works, ~150ms, title+channel only)
    {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok(meta) = fetch_metadata_oembed_direct(&vid, &http, timeout).await {
                let _ = tx.send(meta).await;
            }
        });
    }

    // Source 2: Invidious instances (rich data when available)
    // Uses fetch_text_once — connection errors are expected for these third-party
    // instances; a single fast attempt beats 3.5s of retry sleeps when racing.
    for instance in &config.youtube.invidious_instances {
        let tx = tx.clone();
        let http = http.clone();
        let url = format!("{instance}/api/v1/videos/{video_id}");
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok(Ok(body)) = tokio::time::timeout(timeout, http.fetch_text_once(&url)).await {
                if let Ok(meta) = parse_invidious_video(&body, &vid) {
                    let _ = tx.send(meta).await;
                }
            }
        });
    }

    // Source 3: Piped instances (rich data when available)
    // Uses fetch_text_once for same reason as Invidious above.
    for instance in &config.youtube.piped_instances {
        let tx = tx.clone();
        let http = http.clone();
        let url = format!("{instance}/streams/{video_id}");
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok(Ok(body)) = tokio::time::timeout(timeout, http.fetch_text_once(&url)).await {
                if let Ok(meta) = parse_piped_video(&body, &vid) {
                    let _ = tx.send(meta).await;
                }
            }
        });
    }

    // Close our sender — channel ends when all spawns drop their senders
    drop(tx);

    // Quality-aware selection:
    // oEmbed is often first but sparse; allow a short window for richer metadata.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
    let mut best: Option<VideoMetadata> = None;
    let mut best_score: u8 = 0;

    loop {
        let next = tokio::time::timeout_at(deadline, rx.recv()).await;
        let Some(meta) = (match next {
            Ok(Some(m)) => Some(m),
            _ => None,
        }) else {
            break;
        };

        let score = metadata_richness(&meta);
        if score > best_score {
            best_score = score;
            best = Some(meta);
            // Rich enough metadata found; return early.
            if best_score >= 6 {
                break;
            }
        } else if best.is_none() {
            best = Some(meta);
        }
    }

    let mut best = best.ok_or_else(|| {
        HsxError::YouTube(format!("All metadata sources timed out for {video_id}"))
    })?;

    // Final enrichment pass: if important fields are still sparse, try yt-dlp.
    if metadata_needs_enrichment(&best) {
        let enrich_timeout = std::cmp::max(timeout, Duration::from_secs(6));
        if let Ok(extra) = fetch_metadata_ytdlp(video_id, enrich_timeout).await {
            enrich_metadata(&mut best, extra);
        }
    }

    Ok(best)
}

fn metadata_richness(meta: &VideoMetadata) -> u8 {
    let mut score = 0u8;
    if !meta.title.trim().is_empty() {
        score += 1;
    }
    if !meta.description.trim().is_empty() {
        score += 1;
    }
    if meta.duration_secs > 0 {
        score += 2;
    }
    if meta.view_count > 0 {
        score += 2;
    }
    if meta.like_count > 0 {
        score += 1;
    }
    if !meta.published.trim().is_empty() {
        score += 1;
    }
    if !meta.channel.id.trim().is_empty() {
        score += 1;
    }
    score
}

fn metadata_needs_enrichment(meta: &VideoMetadata) -> bool {
    meta.like_count == 0
        || meta.published.trim().is_empty()
        || meta.channel.subscriber_count.is_none()
}

fn enrich_metadata(base: &mut VideoMetadata, extra: VideoMetadata) {
    if base.title.trim().is_empty() && !extra.title.trim().is_empty() {
        base.title = extra.title;
    }
    if base.description.trim().is_empty() && !extra.description.trim().is_empty() {
        base.description = extra.description;
    }
    if base.duration_secs == 0 && extra.duration_secs > 0 {
        base.duration_secs = extra.duration_secs;
    }
    if base.view_count == 0 && extra.view_count > 0 {
        base.view_count = extra.view_count;
    }
    if base.like_count == 0 && extra.like_count > 0 {
        base.like_count = extra.like_count;
    }
    if base.published.trim().is_empty() && !extra.published.trim().is_empty() {
        base.published = extra.published;
    }
    if base.channel.name.trim().is_empty() && !extra.channel.name.trim().is_empty() {
        base.channel.name = extra.channel.name;
    }
    if base.channel.id.trim().is_empty() && !extra.channel.id.trim().is_empty() {
        base.channel.id = extra.channel.id;
    }
    if base.channel.subscriber_count.is_none() {
        base.channel.subscriber_count = extra.channel.subscriber_count;
    }
    if !base.channel.verified {
        base.channel.verified = extra.channel.verified;
    }
    if base.thumbnail_url.is_none() {
        base.thumbnail_url = extra.thumbnail_url;
    }
}

async fn fetch_metadata_ytdlp(video_id: &str, timeout: Duration) -> HsxResult<VideoMetadata> {
    use tokio::process::Command;

    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let fut = Command::new("yt-dlp")
        .arg("--dump-single-json")
        .arg("--no-warnings")
        .arg("--no-playlist")
        .arg(&url)
        .output();

    let out = tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| HsxError::YouTube(format!("yt-dlp metadata timeout for {video_id}")))?
        .map_err(|e| HsxError::YouTube(format!("yt-dlp metadata spawn failed: {e}")))?;

    if !out.status.success() {
        return Err(HsxError::YouTube(format!(
            "yt-dlp metadata failed for {video_id}: {}",
            String::from_utf8_lossy(&out.stderr)
        )));
    }

    let v: Value = serde_json::from_slice(&out.stdout)
        .map_err(|e| HsxError::YouTube(format!("yt-dlp metadata JSON parse error: {e}")))?;

    let title = v["title"].as_str().unwrap_or("").to_string();
    if title.is_empty() {
        return Err(HsxError::YouTube(format!(
            "yt-dlp metadata empty title for {video_id}"
        )));
    }

    let upload_date = v["upload_date"].as_str().unwrap_or("");
    let published = if upload_date.len() == 8 && upload_date.chars().all(|c| c.is_ascii_digit()) {
        format!(
            "{}-{}-{}",
            &upload_date[0..4],
            &upload_date[4..6],
            &upload_date[6..8]
        )
    } else {
        normalize_published(upload_date)
    };

    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description: v["description"].as_str().unwrap_or("").to_string(),
        channel: ChannelInfo {
            name: v["channel"].as_str().unwrap_or("").to_string(),
            id: v["channel_id"].as_str().unwrap_or("").to_string(),
            subscriber_count: v["channel_follower_count"].as_u64(),
            verified: v["channel_is_verified"].as_bool().unwrap_or(false),
        },
        duration_secs: v["duration"].as_u64().unwrap_or(0),
        view_count: v["view_count"].as_u64().unwrap_or(0),
        like_count: v["like_count"].as_u64().unwrap_or(0),
        published,
        keywords: v["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        chapters: v["chapters"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|ch| {
                        let title = ch["title"].as_str()?.to_string();
                        let start_secs = ch["start_time"].as_u64()?;
                        let end_secs = ch["end_time"].as_u64();
                        Some(Chapter {
                            title,
                            start_secs,
                            end_secs,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        links: extract_links(v["description"].as_str().unwrap_or("")),
        thumbnail_url: v["thumbnail"].as_str().map(|s| s.to_string()),
        is_live: v["is_live"].as_bool().unwrap_or(false),
    })
}

/// Fetch full video metadata via YouTube Innertube player API.
///
/// This is the same endpoint used for transcript fetching. The player response
/// includes videoDetails: title, description, duration, view/like counts, keywords,
/// channel info — richer than oEmbed and more reliable than Invidious/Piped.
async fn fetch_metadata_innertube(
    video_id: &str,
    http: &HttpClient,
    timeout: Duration,
) -> HsxResult<VideoMetadata> {
    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240101.00.00",
                "hl": "en",
                "gl": "US"
            }
        },
        "videoId": video_id
    })
    .to_string();

    let response_text = tokio::time::timeout(timeout, async {
        http.client()
            .post("https://www.youtube.com/youtubei/v1/player?prettyPrint=false")
            .header("Content-Type", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
            )
            .body(body)
            .send()
            .await
            .map_err(HsxError::Network)?
            .text()
            .await
            .map_err(HsxError::Network)
    })
    .await
    .map_err(|_| HsxError::YouTube(format!("Innertube metadata timeout for {video_id}")))??;

    parse_innertube_player_metadata(&response_text, video_id)
}

/// Parse video metadata from Innertube player API response.
fn parse_innertube_player_metadata(json_str: &str, video_id: &str) -> HsxResult<VideoMetadata> {
    let v: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| HsxError::YouTube(format!("Innertube metadata JSON: {e}")))?;

    let vd = &v["videoDetails"];
    let title = vd["title"].as_str().unwrap_or("").to_string();

    if title.is_empty() {
        return Err(HsxError::YouTube(format!(
            "Innertube returned empty title for {video_id}"
        )));
    }

    let description = vd["shortDescription"].as_str().unwrap_or("").to_string();
    let duration_secs = vd["lengthSeconds"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| vd["lengthSeconds"].as_u64())
        .unwrap_or(0);
    let view_count = vd["viewCount"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .or_else(|| vd["viewCount"].as_u64())
        .unwrap_or(0);

    let channel = ChannelInfo {
        name: vd["author"].as_str().unwrap_or("").to_string(),
        id: vd["channelId"].as_str().unwrap_or("").to_string(),
        subscriber_count: None, // not in videoDetails, would need channel API
        verified: false,
    };

    let keywords: Vec<String> = vd["keywords"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|k| k.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let chapters = extract_chapters(&description);
    let links = extract_links(&description);

    // Best available thumbnail
    let thumbnail_url = v["videoDetails"]["thumbnail"]["thumbnails"]
        .as_array()
        .and_then(|arr| arr.last())
        .and_then(|t| t["url"].as_str())
        .map(String::from)
        .or_else(|| Some(format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg")));

    let is_live =
        vd["isLive"].as_bool().unwrap_or(false) || vd["isLiveContent"].as_bool().unwrap_or(false);

    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description,
        channel,
        duration_secs,
        view_count,
        like_count: 0, // videoDetails doesn't include likes (privacy policy)
        published: normalize_published(vd["publishDate"].as_str().unwrap_or("")),
        keywords,
        chapters,
        links,
        thumbnail_url,
        is_live,
    })
}

/// oEmbed fetch — uses http.client() directly for zero retry overhead.
///
/// oEmbed is a reliable YouTube endpoint (~150ms), so no retries are needed.
/// Using http.client() directly avoids rate limiting that would serialize
/// concurrent oEmbed calls across multiple videos.
async fn fetch_metadata_oembed_direct(
    video_id: &str,
    http: &HttpClient,
    timeout: Duration,
) -> HsxResult<VideoMetadata> {
    let encoded_url: String = url::form_urlencoded::byte_serialize(
        format!("https://www.youtube.com/watch?v={video_id}").as_bytes(),
    )
    .collect();
    let url = format!("https://www.youtube.com/oembed?url={encoded_url}&format=json");

    let body = tokio::time::timeout(timeout, async {
        http.client()
            .get(&url)
            .send()
            .await
            .map_err(HsxError::Network)?
            .text()
            .await
            .map_err(HsxError::Network)
    })
    .await
    .map_err(|_| HsxError::YouTube(format!("oEmbed timeout for {video_id}")))??;

    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| HsxError::YouTube(format!("oEmbed JSON: {e}")))?;

    let title = v["title"]
        .as_str()
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    let author = v["author_name"].as_str().unwrap_or("").to_string();

    if title.is_empty() {
        return Err(HsxError::YouTube(format!(
            "oEmbed empty title for {video_id}"
        )));
    }

    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description: String::new(),
        channel: ChannelInfo {
            name: author,
            id: String::new(),
            subscriber_count: None,
            verified: false,
        },
        duration_secs: 0,
        view_count: 0,
        like_count: 0,
        published: String::new(),
        keywords: vec![],
        chapters: vec![],
        links: vec![],
        thumbnail_url: Some(format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg")),
        is_live: false,
    })
}

/// Parse Invidious `/api/v1/videos/{id}` JSON into VideoMetadata.
fn parse_invidious_video(json_str: &str, video_id: &str) -> HsxResult<VideoMetadata> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| HsxError::YouTube(format!("Invidious JSON parse error: {e}")))?;

    let title = v["title"].as_str().unwrap_or("").to_string();
    let description = v["description"].as_str().unwrap_or("").to_string();
    let duration_secs = v["lengthSeconds"].as_u64().unwrap_or(0);
    let view_count = v["viewCount"].as_u64().unwrap_or(0);
    let like_count = v["likeCount"].as_u64().unwrap_or(0);
    let published = normalize_published(v["publishedText"].as_str().unwrap_or(""));

    let channel = ChannelInfo {
        name: v["author"].as_str().unwrap_or("").to_string(),
        id: v["authorId"].as_str().unwrap_or("").to_string(),
        subscriber_count: v["subCountText"].as_str().and_then(parse_subscriber_count),
        verified: v["authorVerified"].as_bool().unwrap_or(false),
    };

    let keywords: Vec<String> = v["keywords"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|k| k.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let chapters = extract_chapters(&description);
    let links = extract_links(&description);

    let thumbnail_url = v["videoThumbnails"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|t| t["url"].as_str())
        .map(String::from);

    let is_live = v["liveNow"].as_bool().unwrap_or(false);

    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description,
        channel,
        duration_secs,
        view_count,
        like_count,
        published,
        keywords,
        chapters,
        links,
        thumbnail_url,
        is_live,
    })
}

/// Parse Piped `/streams/{id}` JSON into VideoMetadata.
fn parse_piped_video(json_str: &str, video_id: &str) -> HsxResult<VideoMetadata> {
    let v: Value = serde_json::from_str(json_str)
        .map_err(|e| HsxError::YouTube(format!("Piped JSON parse error: {e}")))?;

    let title = v["title"].as_str().unwrap_or("").to_string();
    let description = v["description"].as_str().unwrap_or("").to_string();
    let duration_secs = v["duration"].as_u64().unwrap_or(0);
    let view_count = v["views"].as_u64().unwrap_or(0);
    let like_count = v["likes"].as_u64().unwrap_or(0);
    let published = normalize_published(v["uploadDate"].as_str().unwrap_or(""));

    let channel = ChannelInfo {
        name: v["uploader"].as_str().unwrap_or("").to_string(),
        id: v["uploaderUrl"].as_str().unwrap_or("").to_string(),
        subscriber_count: v["uploaderSubscriberCount"].as_u64(),
        verified: v["uploaderVerified"].as_bool().unwrap_or(false),
    };

    let chapters = extract_chapters(&description);
    let links = extract_links(&description);

    let thumbnail_url = v["thumbnailUrl"].as_str().map(String::from);

    Ok(VideoMetadata {
        video_id: video_id.to_string(),
        title,
        description,
        channel,
        duration_secs,
        view_count,
        like_count,
        published,
        keywords: vec![],
        chapters,
        links,
        thumbnail_url,
        is_live: v["livestream"].as_bool().unwrap_or(false),
    })
}

/// Parse subscriber count strings like "1.2M subscribers", "500K", "1,234".
fn parse_subscriber_count(s: &str) -> Option<u64> {
    let cleaned = s
        .replace(',', "")
        .replace("subscribers", "")
        .replace("subscriber", "")
        .trim()
        .to_lowercase();
    if cleaned.ends_with('m') {
        let num: f64 = cleaned.trim_end_matches('m').trim().parse().ok()?;
        Some((num * 1_000_000.0) as u64)
    } else if cleaned.ends_with('k') {
        let num: f64 = cleaned.trim_end_matches('k').trim().parse().ok()?;
        Some((num * 1_000.0) as u64)
    } else if cleaned.ends_with('b') {
        let num: f64 = cleaned.trim_end_matches('b').trim().parse().ok()?;
        Some((num * 1_000_000_000.0) as u64)
    } else {
        cleaned.parse().ok()
    }
}

fn normalize_published(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if s.len() >= 10 && s.chars().nth(4) == Some('-') {
        return s.chars().take(10).collect();
    }
    relative_to_absolute_date(s).unwrap_or_else(|| s.to_string())
}

fn relative_to_absolute_date(s: &str) -> Option<String> {
    let lower = s.to_lowercase();
    let n: i64 = lower
        .split_whitespace()
        .find_map(|w| w.parse::<i64>().ok())?;
    let now = Utc::now().date_naive();
    let d = if lower.contains("minute") || lower.contains("hour") {
        now
    } else if lower.contains("day") {
        now.checked_sub_signed(ChronoDuration::days(n))?
    } else if lower.contains("week") {
        now.checked_sub_signed(ChronoDuration::days(n * 7))?
    } else if lower.contains("month") {
        now.checked_sub_signed(ChronoDuration::days(n * 30))?
    } else if lower.contains("year") {
        now.checked_sub_signed(ChronoDuration::days(n * 365))?
    } else {
        return None;
    };
    Some(d.format("%Y-%m-%d").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_chapters_basic() {
        let desc = "0:00 Introduction\n2:30 Main Topic\n10:15 Conclusion";
        let chapters = extract_chapters(desc);
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters[0].title, "Introduction");
        assert_eq!(chapters[0].start_secs, 0);
        assert_eq!(chapters[1].start_secs, 150);
        assert_eq!(chapters[2].start_secs, 615);
        assert_eq!(chapters[0].end_secs, Some(150));
        assert_eq!(chapters[2].end_secs, None);
    }

    #[test]
    fn extract_chapters_with_hours() {
        let desc = "1:00:00 Part 1\n1:30:00 Part 2";
        let chapters = extract_chapters(desc);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].start_secs, 3600);
        assert_eq!(chapters[1].start_secs, 5400);
    }

    #[test]
    fn extract_chapters_with_dashes() {
        let desc = "- 0:00 Intro\n- 5:30 Main";
        let chapters = extract_chapters(desc);
        assert_eq!(chapters.len(), 2);
    }

    #[test]
    fn extract_links_basic() {
        let desc = "Check out https://github.com/test/repo and https://twitter.com/user";
        let links = extract_links(desc);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].link_type, LinkType::Code);
        assert_eq!(links[1].link_type, LinkType::Social);
    }

    #[test]
    fn extract_links_dedup() {
        let desc = "https://example.com and again https://example.com";
        let links = extract_links(desc);
        assert_eq!(links.len(), 1);
    }

    #[test]
    fn classify_link_types() {
        assert_eq!(classify_link("github.com", ""), LinkType::Code);
        assert_eq!(classify_link("twitter.com", ""), LinkType::Social);
        assert_eq!(classify_link("x.com", ""), LinkType::Social);
        assert_eq!(classify_link("docs.rs", ""), LinkType::Documentation);
        assert_eq!(classify_link("amazon.com", ""), LinkType::Product);
        assert_eq!(classify_link("medium.com", ""), LinkType::Article);
        assert_eq!(classify_link("random.org", ""), LinkType::Other);
    }

    #[test]
    fn channel_credibility_scoring() {
        let channel = ChannelInfo {
            name: "Test".into(),
            id: "UC123".into(),
            subscriber_count: Some(500_000),
            verified: true,
        };
        let cred = score_channel_credibility(&channel);
        assert!(cred.score > 0.5);
        assert_eq!(cred.tier, CredibilityTier::Authority);
    }

    #[test]
    fn channel_credibility_unknown() {
        let channel = ChannelInfo {
            name: "New".into(),
            id: "UC000".into(),
            subscriber_count: None,
            verified: false,
        };
        let cred = score_channel_credibility(&channel);
        assert_eq!(cred.tier, CredibilityTier::Unknown);
    }

    #[test]
    fn parse_subscriber_count_formats() {
        assert_eq!(parse_subscriber_count("1.2M subscribers"), Some(1_200_000));
        assert_eq!(parse_subscriber_count("500K"), Some(500_000));
        assert_eq!(parse_subscriber_count("1,234"), Some(1234));
        assert_eq!(parse_subscriber_count("2B"), Some(2_000_000_000));
    }

    #[test]
    fn parse_invidious_json() {
        let json = serde_json::json!({
            "title": "Test Video",
            "description": "0:00 Intro\nhttps://github.com/test",
            "lengthSeconds": 600,
            "viewCount": 50000,
            "likeCount": 2000,
            "publishedText": "2 months ago",
            "author": "TestChannel",
            "authorId": "UC123",
            "subCountText": "100K subscribers",
            "authorVerified": true,
            "keywords": ["rust", "programming"],
            "videoThumbnails": [{"url": "https://img.youtube.com/thumb.jpg"}],
            "liveNow": false,
        });
        let meta = parse_invidious_video(&json.to_string(), "test123").unwrap();
        assert_eq!(meta.title, "Test Video");
        assert_eq!(meta.duration_secs, 600);
        assert_eq!(meta.channel.name, "TestChannel");
        assert!(meta.channel.verified);
        assert_eq!(meta.chapters.len(), 1);
        assert_eq!(meta.links.len(), 1);
    }
}
