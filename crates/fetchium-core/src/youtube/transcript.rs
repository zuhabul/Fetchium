//! Fast parallel transcript extraction — watch-page scraping + Innertube + fallbacks.
//!
//! All sources race simultaneously; first valid result wins within 15s.
//!
//! ## Source Priority
//! 1. **Watch-page + ANDROID Innertube** — GET /watch to extract live API key,
//!    then POST Innertube with ANDROID client (bypasses poToken/exp=xpe). Returns
//!    XML transcript from clean baseUrl. Same technique as youtube-transcript-api.
//! 2. **ANDROID Innertube direct** — POST with hardcoded API key (no watch-page
//!    fetch; faster but may break if key changes). Both Sources 1 & 2 race.
//! 3. Invidious captions API (community instance)
//! 4. Piped captions API (community instance)

use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use std::time::Duration;

/// Global timeout for transcript fetching across all sources.
const TRANSCRIPT_TIMEOUT_SECS: u64 = 15;

// ─── Public API ────────────────────────────────────────────────

/// Fetch enhanced transcript by racing all sources simultaneously.
///
/// Sources (all start in parallel, first non-empty result wins):
/// 1. **Watch-page scraping** — extracts pre-signed captionTracks baseUrl from HTML
///    (same technique as youtube-transcript-api; bypasses bot detection)
/// 2. Innertube API POST (fallback — may fail without visitor cookies)
/// 3. Invidious captions API
/// 4. Piped captions API
///
/// Global cap: 15s — the pipeline never blocks longer regardless of source health.
pub async fn fetch_transcript(
    video_id: &str,
    http: &HttpClient,
    config: &crate::config::FetchiumConfig,
) -> FetchiumResult<EnhancedTranscript> {
    let (tx, mut rx) =
        tokio::sync::mpsc::channel::<(Vec<TranscriptEntry>, TranscriptSource, String)>(8);

    let per_src = Duration::from_secs(config.youtube.timeout_secs.max(10));

    // Source 1: Watch-page scraping — MOST RELIABLE.
    // Fetches /watch page, extracts ytInitialPlayerResponse, uses the pre-signed
    // captionTracks baseUrl (contains expire + signature tokens already embedded).
    // This is the approach used by youtube-transcript-api (Python) and works on
    // any public video regardless of YouTube's bot detection changes.
    {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok((entries, lang)) = fetch_via_watch_page(&vid, &http, per_src).await {
                if !entries.is_empty() {
                    let _ = tx
                        .send((entries, TranscriptSource::YouTubeTimedtext, lang))
                        .await;
                }
            }
        });
    }

    // Source 2: ANDROID Innertube direct — no watch-page fetch, uses stable key.
    // Races with Source 1; faster on good connections, same quality.
    {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok((entries, lang)) = fetch_via_android_innertube(&vid, &http, per_src).await {
                if !entries.is_empty() {
                    let _ = tx
                        .send((entries, TranscriptSource::YouTubeTimedtext, lang))
                        .await;
                }
            }
        });
    }

    // Source 3: TVHTML5 (Smart TV) Innertube — different client fingerprint.
    // Succeeds for some videos where ANDROID returns UNPLAYABLE or times out.
    {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        tokio::spawn(async move {
            if let Ok((entries, lang)) = fetch_via_tvhtml5(&vid, &http, per_src).await {
                if !entries.is_empty() {
                    let _ = tx
                        .send((entries, TranscriptSource::YouTubeTimedtext, lang))
                        .await;
                }
            }
        });
    }

    // Source 4+: Invidious captions (race all instances)
    for instance in &config.youtube.invidious_instances {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        let inst = instance.clone();
        tokio::spawn(async move {
            if let Ok(entries) = fetch_invidious_captions_fast(&vid, &inst, &http, per_src).await {
                if !entries.is_empty() {
                    let _ = tx
                        .send((entries, TranscriptSource::Invidious, "en".to_string()))
                        .await;
                }
            }
        });
    }

    // Source 5+: Piped captions (race all instances)
    for instance in &config.youtube.piped_instances {
        let tx = tx.clone();
        let http = http.clone();
        let vid = video_id.to_string();
        let inst = instance.clone();
        tokio::spawn(async move {
            if let Ok(entries) = fetch_piped_captions_fast(&vid, &inst, &http, per_src).await {
                if !entries.is_empty() {
                    let _ = tx
                        .send((entries, TranscriptSource::Piped, "en".to_string()))
                        .await;
                }
            }
        });
    }

    drop(tx);

    // Wait for first result from any parallel source within the global cap.
    let parallel_result =
        tokio::time::timeout(Duration::from_secs(TRANSCRIPT_TIMEOUT_SECS), rx.recv()).await;

    if let Ok(Some((entries, source, lang))) = parallel_result {
        return Ok(enhance_transcript(video_id, entries, source, lang));
    }

    // ── Whisper ASR fallback ──────────────────────────────────────────────────
    // All caption sources failed (video has no captions, or network issues).
    // Try Whisper speech-to-text as a last resort: yt-dlp downloads audio-only,
    // Whisper transcribes it. Works for ANY public video regardless of captions.
    // Requires: `yt-dlp` + `whisper` (pip install yt-dlp openai-whisper).
    if let Ok((entries, lang)) = fetch_via_whisper(video_id).await {
        if !entries.is_empty() {
            return Ok(enhance_transcript(
                video_id,
                entries,
                TranscriptSource::YtDlp,
                lang,
            ));
        }
    }

    Err(FetchiumError::YouTube(format!(
        "No transcript available for video {video_id}. \
         Caption sources returned nothing and Whisper ASR fallback also failed. \
         Ensure yt-dlp and openai-whisper are installed: \
         `pip install yt-dlp openai-whisper`"
    )))
}

// ─── Known Innertube API key (stable, extracted from any YouTube page) ─────

/// YouTube's public Innertube API key — stable since 2020.
/// Used as fallback; watch-page path extracts the live key for reliability.
const INNERTUBE_API_KEY: &str = "AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8";

// ─── Noise Detection & Quality Scoring ────────────────────────

/// Returns `true` if a transcript entry consists entirely of noise annotations.
///
/// YouTube auto-captions emit tags like `[Music]`, `[Applause]`, `[foreign]`,
/// `[Laughter]` for non-speech segments. These are useful for chapter alignment
/// but should be excluded from readable output and quality scoring.
fn is_noise_text(text: &str) -> bool {
    static NOISE_RE: once_cell::sync::Lazy<regex::Regex> =
        once_cell::sync::Lazy::new(|| regex::Regex::new(r"^(\s*\[[^\]]*\]\s*)+$").unwrap());
    let t = text.trim();
    t.is_empty() || NOISE_RE.is_match(t)
}

/// Score the quality of a transcript to detect wrong-language ASR or garbled output.
///
/// Returns 0.0 (garbage) to 1.0 (excellent). Penalises:
/// - High noise ratio (`[Music]` / `[Applause]` etc.)
/// - `[foreign]` markers (YouTube's signal that English ASR detected non-English speech)
/// - Very short average word length (garbled ASR produces truncated tokens like "spee")
fn score_quality(entries: &[TranscriptEntry]) -> f64 {
    if entries.is_empty() {
        return 0.0;
    }

    let total = entries.len() as f64;
    let noise_count = entries.iter().filter(|e| is_noise_text(&e.text)).count() as f64;
    let foreign_count = entries
        .iter()
        .filter(|e| {
            let lower = e.text.to_lowercase();
            lower.contains("[foreign]") || lower.contains("foreign")
        })
        .count() as f64;

    let noise_ratio = noise_count / total;
    let foreign_ratio = foreign_count / total;

    // Average word length on speech-only entries (garbled ASR → very short words).
    let speech_words: Vec<usize> = entries
        .iter()
        .filter(|e| !is_noise_text(&e.text))
        .flat_map(|e| {
            e.text
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).len())
        })
        .filter(|&l| l > 0)
        .collect();

    let avg_word_len = if speech_words.is_empty() {
        0.0_f64
    } else {
        speech_words.iter().sum::<usize>() as f64 / speech_words.len() as f64
    };

    let mut score = (1.0 - noise_ratio).max(0.0);

    // [foreign] = YouTube explicitly detected non-English speech in English ASR — heavy penalty.
    if foreign_ratio > 0.05 {
        score *= 0.25;
    }

    // Garbled ASR word length heuristic.
    match avg_word_len as u32 {
        0 => score = 0.0,
        1..=2 => score *= 0.2,
        3 => score *= 0.5,
        _ => {}
    }

    score
}

// ─── Watch-Page + ANDROID Innertube (Primary Source) ──────────

/// Fetch transcript via watch-page scraping + ANDROID Innertube API.
///
/// ## Algorithm (mirrors youtube-transcript-api Python library)
/// 1. GET https://www.youtube.com/watch?v={id} with browser User-Agent
/// 2. Extract live `INNERTUBE_API_KEY` from the page HTML
/// 3. POST to Innertube with **ANDROID client context** — bypasses poToken/exp=xpe
/// 4. Auto-quality: if English ASR is garbled, retry with native language track
async fn fetch_via_watch_page(
    video_id: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    let url = format!("https://www.youtube.com/watch?v={video_id}&hl=en");

    let html = tokio::time::timeout(timeout, async {
        http.client()
            .get(&url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
                 AppleWebKit/537.36 (KHTML, like Gecko) \
                 Chrome/120.0.0.0 Safari/537.36",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .send()
            .await
            .map_err(FetchiumError::Network)?
            .text()
            .await
            .map_err(FetchiumError::Network)
    })
    .await
    .map_err(|_| FetchiumError::YouTube("Watch page request timed out".into()))??;

    let api_key = extract_innertube_api_key(&html).unwrap_or_else(|| INNERTUBE_API_KEY.to_string());

    fetch_innertube_with_client(
        video_id,
        http,
        &api_key,
        timeout.min(Duration::from_secs(10)),
        "ANDROID",
        "20.10.38",
        "com.google.android.youtube/20.10.38 (Linux; U; Android 11)",
    )
    .await
}

/// Extract YouTube's Innertube API key from a watch page HTML blob.
fn extract_innertube_api_key(html: &str) -> Option<String> {
    let marker = "\"INNERTUBE_API_KEY\":\"";
    let start = html.find(marker)? + marker.len();
    let end = html[start..].find('"')? + start;
    Some(html[start..end].to_string())
}

// ─── ANDROID Innertube (Fast Fallback) ────────────────────────

/// ANDROID Innertube — hardcoded stable API key, no watch-page fetch required.
/// Races with `fetch_via_watch_page`; whichever responds first wins.
async fn fetch_via_android_innertube(
    video_id: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    fetch_innertube_with_client(
        video_id,
        http,
        INNERTUBE_API_KEY,
        timeout,
        "ANDROID",
        "20.10.38",
        "com.google.android.youtube/20.10.38 (Linux; U; Android 11)",
    )
    .await
}

// ─── TVHTML5 Innertube (Timeout Fallback) ─────────────────────

/// TVHTML5 (Smart TV) Innertube — different client context than ANDROID.
///
/// Some videos that time out or return `UNPLAYABLE` for the ANDROID client
/// succeed with the TV client, which has a different bot-detection surface.
/// Runs in parallel with the other sources.
async fn fetch_via_tvhtml5(
    video_id: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    fetch_innertube_with_client(
        video_id,
        http,
        INNERTUBE_API_KEY,
        timeout,
        "TVHTML5",
        "7.20241201.11.00",
        "Mozilla/5.0 (SMART-TV; LINUX; Tizen 6.0) AppleWebKit/538.1 \
         (KHTML, like Gecko) Version/6.0 TV Safari/538.1",
    )
    .await
}

// ─── Generic Innertube Core ────────────────────────────────────

/// Core Innertube transcript extraction — parameterised by client type.
///
/// Supports any Innertube client (ANDROID, TVHTML5, IOS, …).
///
/// ## Quality-Aware Language Selection
/// After fetching the English track, we score transcript quality. If the score
/// is below 0.4 — indicating garbled ASR (e.g. English ASR on a Bengali video
/// producing "[foreign] spee speee") — we automatically retry with the best
/// available manual caption track in the video's native language.
///
/// Flow: POST /youtubei/v1/player → captionTracks → pick best track →
///       fetch XML timedtext → quality check → optional native-lang retry → parse.
async fn fetch_innertube_with_client(
    video_id: &str,
    http: &HttpClient,
    api_key: &str,
    timeout: Duration,
    client_name: &str,
    client_version: &str,
    user_agent: &str,
) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    let innertube_url =
        format!("https://www.youtube.com/youtubei/v1/player?key={api_key}&prettyPrint=false");
    let body = serde_json::json!({
        "context": {
            "client": {
                "clientName": client_name,
                "clientVersion": client_version,
                "hl": "en",
                "gl": "US"
            }
        },
        "videoId": video_id
    })
    .to_string();

    let response_text = tokio::time::timeout(timeout, async {
        http.client()
            .post(&innertube_url)
            .header("Content-Type", "application/json")
            .header("User-Agent", user_agent)
            .body(body)
            .send()
            .await
            .map_err(FetchiumError::Network)?
            .text()
            .await
            .map_err(FetchiumError::Network)
    })
    .await
    .map_err(|_| {
        FetchiumError::YouTube(format!("Innertube ({client_name}) request timed out"))
    })??;

    let v: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| FetchiumError::YouTube(format!("Innertube JSON parse: {e}")))?;

    let tracks = v["captions"]["playerCaptionsTracklistRenderer"]["captionTracks"]
        .as_array()
        .ok_or_else(|| {
            FetchiumError::YouTube(
                "No caption tracks in Innertube response (video may have no captions)".into(),
            )
        })?;

    // ── Track selection ────────────────────────────────────────────────────────
    // Priority: manual-en → ASR-en → manual-any → ASR-any
    // If we later detect the English track is garbage, we retry with native-any.
    let (language_code, caption_url) = {
        let selected = tracks
            .iter()
            .find(|t| {
                t["languageCode"]
                    .as_str()
                    .map(|c| c.starts_with("en"))
                    .unwrap_or(false)
                    && t["kind"].as_str().map(|k| k != "asr").unwrap_or(true)
            })
            .or_else(|| {
                tracks.iter().find(|t| {
                    t["languageCode"]
                        .as_str()
                        .map(|c| c.starts_with("en"))
                        .unwrap_or(false)
                })
            })
            .or_else(|| {
                tracks.iter().find(|t| {
                    t["kind"].as_str().map(|k| k != "asr").unwrap_or(true)
                        && t["baseUrl"].as_str().is_some()
                })
            })
            .or_else(|| tracks.first())
            .ok_or_else(|| FetchiumError::YouTube("No caption tracks available".into()))?;

        let lang = selected["languageCode"]
            .as_str()
            .unwrap_or("und")
            .to_string();
        let url = selected["baseUrl"]
            .as_str()
            .ok_or_else(|| FetchiumError::YouTube("No baseUrl in caption track".into()))?
            .replace("&fmt=srv3", ""); // ANDROID/TVHTML5 URLs are XML — don't add &fmt=json3
        (lang, url)
    }; // `selected` borrow dropped here so we can iterate `tracks` again below

    let cap_timeout = timeout.min(Duration::from_secs(8));
    let xml_body = fetch_caption_xml(&caption_url, http, cap_timeout).await?;
    let entries = parse_timedtext_xml(&xml_body)?;

    // ── Quality check: auto-retry with native language ──────────────────────
    // Detect garbled ASR (English ASR on non-English video). Hallmarks:
    //   - Many [foreign] markers (YouTube's own signal)
    //   - Very short average word length (truncated/garbage tokens)
    //   - High noise ratio ([Music] drowning out speech)
    let quality = score_quality(&entries);
    if quality < 0.4 && language_code.starts_with("en") {
        // Try to find a manual caption track in the video's native language.
        let native = tracks.iter().find(|t| {
            let is_non_english = t["languageCode"]
                .as_str()
                .map(|c| !c.starts_with("en"))
                .unwrap_or(false);
            let is_manual = t["kind"].as_str().map(|k| k != "asr").unwrap_or(true);
            is_non_english && is_manual && t["baseUrl"].as_str().is_some()
        });

        if let Some(native_track) = native {
            let native_lang = native_track["languageCode"]
                .as_str()
                .unwrap_or("und")
                .to_string();
            let native_url = native_track["baseUrl"]
                .as_str()
                .unwrap_or("")
                .replace("&fmt=srv3", "");

            let retry_timeout = Duration::from_secs(6);
            if let Ok(native_xml) = fetch_caption_xml(&native_url, http, retry_timeout).await {
                if let Ok(native_entries) = parse_timedtext_xml(&native_xml) {
                    let native_quality = score_quality(&native_entries);
                    if native_quality > quality || !native_entries.is_empty() {
                        return Ok((native_entries, native_lang));
                    }
                }
            }
        }

        // If no manual native track, also try ASR in any other language.
        let asr_native = tracks.iter().find(|t| {
            t["languageCode"]
                .as_str()
                .map(|c| !c.starts_with("en"))
                .unwrap_or(false)
                && t["baseUrl"].as_str().is_some()
        });
        if let Some(asr_track) = asr_native {
            let asr_lang = asr_track["languageCode"]
                .as_str()
                .unwrap_or("und")
                .to_string();
            let asr_url = asr_track["baseUrl"]
                .as_str()
                .unwrap_or("")
                .replace("&fmt=srv3", "");
            let retry_timeout = Duration::from_secs(6);
            if let Ok(asr_xml) = fetch_caption_xml(&asr_url, http, retry_timeout).await {
                if let Ok(asr_entries) = parse_timedtext_xml(&asr_xml) {
                    let asr_quality = score_quality(&asr_entries);
                    if asr_quality > quality {
                        return Ok((asr_entries, asr_lang));
                    }
                }
            }
        }
    }

    Ok((entries, language_code))
}

/// Fetch raw caption XML from a URL with a timeout.
async fn fetch_caption_xml(
    url: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<String> {
    tokio::time::timeout(timeout, async {
        http.client()
            .get(url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            )
            .send()
            .await
            .map_err(FetchiumError::Network)?
            .text()
            .await
            .map_err(FetchiumError::Network)
    })
    .await
    .map_err(|_| FetchiumError::YouTube("Caption XML fetch timed out".into()))?
}

// ─── Fallback Sources ──────────────────────────────────────────

/// Invidious captions — uses fetch_text_once (no retries) for both requests.
///
/// Connection errors on Invidious instances are expected; fail fast rather than
/// burning 3.5s on retry sleeps when Innertube and timedtext are already racing.
async fn fetch_invidious_captions_fast(
    video_id: &str,
    instance: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<Vec<TranscriptEntry>> {
    let url = format!("{instance}/api/v1/captions/{video_id}");
    let body = tokio::time::timeout(timeout, http.fetch_text_once(&url))
        .await
        .map_err(|_| FetchiumError::YouTube("Invidious captions timeout".into()))??;

    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| FetchiumError::YouTube(format!("Invidious captions parse: {e}")))?;

    let captions = v["captions"]
        .as_array()
        .ok_or_else(|| FetchiumError::YouTube("No captions".into()))?;

    // Find English caption URL
    let en_url_str = captions
        .iter()
        .find(|c| {
            c["language_code"]
                .as_str()
                .map(|l| l.starts_with("en"))
                .unwrap_or(false)
        })
        .and_then(|c| c["url"].as_str())
        .ok_or_else(|| FetchiumError::YouTube("No English captions in Invidious".into()))?
        .to_string();

    let caption_url = if en_url_str.starts_with("http") {
        en_url_str
    } else {
        format!("{instance}{en_url_str}")
    };

    let caption_body = tokio::time::timeout(timeout, http.fetch_text_once(&caption_url))
        .await
        .map_err(|_| FetchiumError::YouTube("Invidious caption body timeout".into()))??;

    parse_timedtext_xml(&caption_body)
}

/// Piped captions — uses fetch_text_once (no retries) for both requests.
async fn fetch_piped_captions_fast(
    video_id: &str,
    instance: &str,
    http: &HttpClient,
    timeout: Duration,
) -> FetchiumResult<Vec<TranscriptEntry>> {
    let url = format!("{instance}/streams/{video_id}");
    let body = tokio::time::timeout(timeout, http.fetch_text_once(&url))
        .await
        .map_err(|_| FetchiumError::YouTube("Piped stream timeout".into()))??;

    let v: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| FetchiumError::YouTube(format!("Piped parse: {e}")))?;

    let subtitles = v["subtitles"]
        .as_array()
        .ok_or_else(|| FetchiumError::YouTube("No subtitles in Piped".into()))?;

    let en_url = subtitles
        .iter()
        .find(|s| {
            s["code"]
                .as_str()
                .map(|c| c.starts_with("en"))
                .unwrap_or(false)
        })
        .and_then(|s| s["url"].as_str())
        .ok_or_else(|| FetchiumError::YouTube("No English subtitles in Piped".into()))?
        .to_string();

    let sub_body = tokio::time::timeout(timeout, http.fetch_text_once(&en_url))
        .await
        .map_err(|_| FetchiumError::YouTube("Piped subtitle body timeout".into()))??;

    parse_timedtext_xml(&sub_body)
}

// ─── Whisper ASR Fallback (videos with no captions) ───────────

/// Transcribe a YouTube video using Whisper speech-to-text via yt-dlp audio extraction.
///
/// ## Pipeline
/// 1. `yt-dlp -x --audio-format wav -q` — download audio-only (~30s for a 10-min video)
/// 2. Select Whisper model based on available disk RAM:
///    - `tiny`  (~39 MB):  40× realtime, ~5.7% WER  — fast machines/short videos
///    - `base`  (~74 MB):  30× realtime, ~4.9% WER  — default
///    - `small` (~244 MB): 15× realtime, ~3.4% WER  — better accuracy
/// 3. `whisper --model base --output_format json --fp16 False` — transcribe
/// 4. Parse JSON output → `Vec<TranscriptEntry>` with timestamps
///
/// Requires: `pip install yt-dlp openai-whisper` (or `faster-whisper` for 2× speed).
/// Returns `Err` if yt-dlp or whisper are not installed — caller falls back gracefully.
async fn fetch_via_whisper(video_id: &str) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    use tokio::process::Command;

    // Check that both tools are available before starting heavy work.
    let ytdlp_ok = Command::new("yt-dlp")
        .arg("--version")
        .output()
        .await
        .is_ok();
    let whisper_ok = Command::new("whisper").arg("--help").output().await.is_ok();

    if !ytdlp_ok || !whisper_ok {
        return Err(FetchiumError::YouTube(
            "Whisper ASR requires yt-dlp and openai-whisper: \
             `pip install yt-dlp openai-whisper`"
                .into(),
        ));
    }

    let tmp_dir = std::env::temp_dir().join(format!("fetchium_whisper_{video_id}"));
    let _ = tokio::fs::create_dir_all(&tmp_dir).await;
    let audio_template = format!("{}/audio.%(ext)s", tmp_dir.display());
    // Step 1: Download best audio only (no video) without forced WAV conversion.
    // Avoiding WAV post-processing significantly reduces fallback latency.
    let yt_out = tokio::time::timeout(
        Duration::from_secs(120),
        Command::new("yt-dlp")
            .args([
                "-f",
                "bestaudio/best",
                "--no-playlist",
                "--quiet",
                "--output",
                &audio_template,
                &format!("https://www.youtube.com/watch?v={video_id}"),
            ])
            .output(),
    )
    .await
    .map_err(|_| FetchiumError::YouTube("yt-dlp audio download timed out (120s)".into()))?
    .map_err(|e| FetchiumError::YouTube(format!("yt-dlp failed: {e}")))?;

    if !yt_out.status.success() {
        let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
        return Err(FetchiumError::YouTube(format!(
            "yt-dlp audio download failed: {}",
            String::from_utf8_lossy(&yt_out.stderr)
        )));
    }

    // Resolve downloaded audio file path (`audio.m4a`, `audio.webm`, ...).
    let mut audio_path: Option<std::path::PathBuf> = None;
    let mut rd = tokio::fs::read_dir(&tmp_dir)
        .await
        .map_err(|e| FetchiumError::YouTube(format!("failed to read ASR temp dir: {e}")))?;
    while let Ok(Some(entry)) = rd.next_entry().await {
        let p = entry.path();
        if p.is_file() {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if matches!(
                ext.as_str(),
                "m4a" | "webm" | "opus" | "mp3" | "aac" | "wav" | "ogg" | "flac"
            ) {
                audio_path = Some(p);
                break;
            }
        }
    }
    let audio_path = audio_path.ok_or_else(|| {
        FetchiumError::YouTube("yt-dlp did not produce an audio file for Whisper".into())
    })?;

    // Step 2: Run Whisper transcription.
    // Default to `tiny` for low-latency fallback. Override with FETCHIUM_WHISPER_MODEL.
    // `--fp16 False` ensures CPU compatibility (avoids GPU-only FP16 error).
    let model = whisper_model_from_env();
    let whisper_timeout_secs = match model.as_str() {
        "tiny" | "base" | "turbo" => 240,
        "small" => 420,
        _ => 600,
    };
    let mut whisper_cmd = Command::new("whisper");
    whisper_cmd
        .arg(audio_path.to_str().unwrap_or("audio"))
        .arg("--model")
        .arg(&model)
        .arg("--output_format")
        .arg("json")
        .arg("--output_dir")
        .arg(tmp_dir.to_str().unwrap_or("/tmp"))
        .arg("--fp16")
        .arg("False")
        .arg("--temperature")
        .arg("0")
        .arg("--best_of")
        .arg("1")
        .arg("--beam_size")
        .arg("1");
    if let Ok(threads) = std::env::var("FETCHIUM_WHISPER_THREADS") {
        let valid_threads = threads.parse::<u32>().ok().filter(|t| *t > 0).is_some();
        if valid_threads {
            whisper_cmd.arg("--threads").arg(threads);
        }
    }
    let whisper_out = tokio::time::timeout(
        Duration::from_secs(whisper_timeout_secs),
        whisper_cmd.output(),
    )
    .await
    .map_err(|_| FetchiumError::YouTube(format!("Whisper timed out ({whisper_timeout_secs}s)")))?
    .map_err(|e| FetchiumError::YouTube(format!("Whisper failed: {e}")))?;

    let _ = tokio::fs::remove_file(&audio_path).await;

    if !whisper_out.status.success() {
        let _ = tokio::fs::remove_dir_all(&tmp_dir).await;
        return Err(FetchiumError::YouTube(format!(
            "Whisper failed: {}",
            String::from_utf8_lossy(&whisper_out.stderr)
        )));
    }

    // Step 3: Find the JSON output file and parse it.
    let json_path = tmp_dir.join("audio.json");
    let json_content = tokio::fs::read_to_string(&json_path)
        .await
        .map_err(|e| FetchiumError::YouTube(format!("Whisper JSON output not found: {e}")))?;

    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    parse_whisper_json(&json_content)
}

fn whisper_model_from_env() -> String {
    let raw = std::env::var("FETCHIUM_WHISPER_MODEL")
        .unwrap_or_else(|_| "tiny".to_string())
        .to_lowercase();
    match raw.as_str() {
        "tiny" | "base" | "small" | "medium" | "large" | "turbo" => raw,
        _ => "tiny".to_string(),
    }
}

/// Parse Whisper's JSON output into transcript entries plus detected language.
///
/// Whisper JSON format: `{"language": "english", "segments": [{"start": 0.0, "end": 2.5, "text": "..."}]}`
/// Returns `(entries, language_code)` where language_code is an ISO 639-1 code (e.g. "en", "bn").
fn parse_whisper_json(json: &str) -> FetchiumResult<(Vec<TranscriptEntry>, String)> {
    let v: serde_json::Value = serde_json::from_str(json)
        .map_err(|e| FetchiumError::YouTube(format!("Whisper JSON parse: {e}")))?;

    // Whisper outputs the language name in English (e.g. "english", "bengali").
    // Map common names to ISO 639-1 codes; default to "und" (undetermined) if unknown.
    let lang_name = v["language"].as_str().unwrap_or("").to_lowercase();
    let language_code = whisper_lang_to_iso(&lang_name).to_string();

    let segments = v["segments"]
        .as_array()
        .ok_or_else(|| FetchiumError::YouTube("No segments in Whisper output".into()))?;

    let entries: Vec<TranscriptEntry> = segments
        .iter()
        .filter_map(|seg| {
            let start_secs = seg["start"].as_f64()?;
            let end_secs = seg["end"].as_f64().unwrap_or(start_secs + 2.0);
            let text = seg["text"].as_str()?.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(TranscriptEntry {
                start_ms: (start_secs * 1000.0) as u32,
                duration_ms: ((end_secs - start_secs) * 1000.0) as u32,
                text: decode_html_entities(&text),
                speaker_id: None,
            })
        })
        .collect();

    if entries.is_empty() {
        return Err(FetchiumError::YouTube(
            "Whisper produced no transcript segments".into(),
        ));
    }

    Ok((entries, language_code))
}

/// Map Whisper's English language name to ISO 639-1 code.
fn whisper_lang_to_iso(name: &str) -> &'static str {
    match name {
        "english" => "en",
        "bengali" => "bn",
        "hindi" => "hi",
        "spanish" => "es",
        "french" => "fr",
        "german" => "de",
        "chinese" => "zh",
        "japanese" => "ja",
        "korean" => "ko",
        "arabic" => "ar",
        "portuguese" => "pt",
        "russian" => "ru",
        "italian" => "it",
        "dutch" => "nl",
        "turkish" => "tr",
        "polish" => "pl",
        "swedish" => "sv",
        "urdu" => "ur",
        "indonesian" => "id",
        "thai" => "th",
        "vietnamese" => "vi",
        _ => "und",
    }
}

// ─── XML Parser (fallback for non-json3 sources) ───────────────

/// Parse YouTube timedtext XML into transcript entries.
fn parse_timedtext_xml(xml: &str) -> FetchiumResult<Vec<TranscriptEntry>> {
    let re = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(
            r#"<text[^>]*start="([0-9.]+)"[^>]*(?:dur="([0-9.]+)")?[^>]*>([^<]*)</text>"#,
        )
        .unwrap()
    });

    let mut entries = Vec::new();
    for cap in re.captures_iter(xml) {
        let start_secs: f64 = cap[1].parse().unwrap_or(0.0);
        let dur_secs: f64 = cap
            .get(2)
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(2.0);
        let raw_text = cap[3].trim().to_string();
        let text = decode_html_entities(&raw_text);

        if !text.is_empty() {
            entries.push(TranscriptEntry {
                start_ms: (start_secs * 1000.0) as u32,
                duration_ms: (dur_secs * 1000.0) as u32,
                text,
                speaker_id: None,
            });
        }
    }

    if entries.is_empty() {
        return Err(FetchiumError::YouTube(
            "No transcript entries parsed from XML".into(),
        ));
    }

    Ok(entries)
}

/// Decode basic HTML entities in transcript text.
fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace('\n', " ")
}

// ─── Enhancement (speaker detection + key moments) ─────────────

/// Enhance raw transcript entries with speaker detection and key moments.
///
/// Noise entries (`[Music]`, `[Applause]` etc.) are preserved in `entries`
/// for chapter alignment purposes but excluded from `full_text` / `word_count`.
fn enhance_transcript(
    video_id: &str,
    mut entries: Vec<TranscriptEntry>,
    source: TranscriptSource,
    language: String,
) -> EnhancedTranscript {
    let quality_score = score_quality(&entries);
    let speakers = detect_speakers(&mut entries);
    let key_moments = detect_key_moments(&entries);

    // Build full_text from speech-only entries — skip pure [Music]/[Applause] etc.
    let full_text: String = entries
        .iter()
        .filter(|e| !is_noise_text(&e.text))
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let word_count = full_text.split_whitespace().count();

    EnhancedTranscript {
        video_id: video_id.to_string(),
        language,
        entries,
        speakers,
        key_moments,
        full_text,
        word_count,
        source,
        quality_score,
    }
}

// ─── Speaker Detection ─────────────────────────────────────────

/// Detect speakers using caption cues, pause gaps, and style shifts.
fn detect_speakers(entries: &mut [TranscriptEntry]) -> Vec<Speaker> {
    let speaker_re = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^(?:\[([^\]]+)\]|>>?\s*([A-Z][A-Z\s]+):)\s*").unwrap()
    });

    let mut speaker_map: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut speaker_counts: std::collections::HashMap<u32, usize> =
        std::collections::HashMap::new();
    let mut next_id = 0u32;

    for entry in entries.iter_mut() {
        if let Some(cap) = speaker_re.captures(&entry.text) {
            let name = cap
                .get(1)
                .or(cap.get(2))
                .map(|m| m.as_str().trim().to_string());
            if let Some(name) = name {
                let id = *speaker_map.entry(name).or_insert_with(|| {
                    let id = next_id;
                    next_id += 1;
                    id
                });
                entry.speaker_id = Some(id);
                *speaker_counts.entry(id).or_insert(0) += 1;
            }
        }
    }

    // Detect speaker changes via long pause gaps (>3s) when no explicit labels
    if speaker_map.is_empty() {
        let mut current_speaker = 0u32;
        let mut last_end_ms = 0u32;
        for entry in entries.iter_mut() {
            let gap = entry.start_ms.saturating_sub(last_end_ms);
            if gap > 3000 {
                current_speaker = (current_speaker + 1) % 2;
            }
            entry.speaker_id = Some(current_speaker);
            *speaker_counts.entry(current_speaker).or_insert(0) += 1;
            last_end_ms = entry.start_ms + entry.duration_ms;
        }
        speaker_map.insert("Speaker A".into(), 0);
        speaker_map.insert("Speaker B".into(), 1);
    }

    speaker_map
        .into_iter()
        .map(|(name, id)| Speaker {
            id,
            label: name,
            segment_count: speaker_counts.get(&id).copied().unwrap_or(0),
        })
        .collect()
}

// ─── Key Moment Detection ──────────────────────────────────────

/// Detect key moments in a transcript using transitional phrases.
fn detect_key_moments(entries: &[TranscriptEntry]) -> Vec<KeyMoment> {
    let mut moments = Vec::new();

    for entry in entries {
        let lower = entry.text.to_lowercase();

        for phrase in TRANSITION_PHRASES {
            if lower.contains(phrase) {
                let moment_type = classify_moment(phrase);
                moments.push(KeyMoment {
                    timestamp_ms: entry.start_ms,
                    moment_type,
                    text: entry.text.clone(),
                    importance: moment_importance(&moment_type),
                });
                break;
            }
        }

        for pattern in DEFINITION_PATTERNS {
            if lower.contains(pattern) {
                moments.push(KeyMoment {
                    timestamp_ms: entry.start_ms,
                    moment_type: MomentType::Definition,
                    text: entry.text.clone(),
                    importance: 0.8,
                });
                break;
            }
        }
    }

    moments.dedup_by(|a, b| {
        a.timestamp_ms.abs_diff(b.timestamp_ms) < 5000 && a.moment_type == b.moment_type
    });

    moments
}

fn classify_moment(phrase: &str) -> MomentType {
    if phrase.contains("summary") || phrase.contains("conclusion") || phrase.contains("bottom line")
    {
        MomentType::Conclusion
    } else if phrase.contains("example") || phrase.contains("look at") {
        MomentType::Example
    } else if phrase.contains("explain") || phrase.contains("key") || phrase.contains("important") {
        MomentType::KeyPoint
    } else if phrase.contains("moving on")
        || phrase.contains("next")
        || phrase.contains("first")
        || phrase.contains("secondly")
        || phrase.contains("finally")
    {
        MomentType::TopicShift
    } else {
        MomentType::KeyPoint
    }
}

fn moment_importance(mt: &MomentType) -> f64 {
    match mt {
        MomentType::Conclusion => 0.9,
        MomentType::KeyPoint => 0.8,
        MomentType::Definition => 0.8,
        MomentType::Example => 0.6,
        MomentType::TopicShift => 0.5,
    }
}

/// Align transcript entries to video chapters.
pub fn align_to_chapters(
    entries: &[TranscriptEntry],
    chapters: &[crate::youtube::types::Chapter],
) -> Vec<(String, Vec<TranscriptEntry>)> {
    let mut result = Vec::new();

    for chapter in chapters {
        let start_ms = (chapter.start_secs * 1000) as u32;
        let end_ms = chapter
            .end_secs
            .map(|e| (e * 1000) as u32)
            .unwrap_or(u32::MAX);

        let chapter_entries: Vec<TranscriptEntry> = entries
            .iter()
            .filter(|e| e.start_ms >= start_ms && e.start_ms < end_ms)
            .cloned()
            .collect();

        result.push((chapter.title.clone(), chapter_entries));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_timedtext_basic() {
        let xml = r#"<transcript>
            <text start="0.5" dur="2.0">Hello world</text>
            <text start="2.5" dur="1.5">How are you</text>
        </transcript>"#;
        let entries = parse_timedtext_xml(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello world");
        assert_eq!(entries[0].start_ms, 500);
        assert_eq!(entries[0].duration_ms, 2000);
    }

    #[test]
    fn parse_timedtext_entities() {
        let xml = r#"<transcript><text start="0" dur="1">It&apos;s &amp; good &lt;test&gt;</text></transcript>"#;
        let entries = parse_timedtext_xml(xml).unwrap();
        assert_eq!(entries[0].text, "It's & good <test>");
    }

    #[test]
    fn extract_innertube_api_key_basic() {
        let html = r#"<script>var _yt_cfg = {"INNERTUBE_API_KEY":"AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8"};</script>"#;
        let key = extract_innertube_api_key(html);
        assert_eq!(
            key.as_deref(),
            Some("AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8")
        );
    }

    #[test]
    fn extract_innertube_api_key_not_found() {
        let html = "<html>no key here</html>";
        assert!(extract_innertube_api_key(html).is_none());
    }

    #[test]
    fn parse_whisper_json_basic() {
        let json = serde_json::json!({
            "language": "english",
            "segments": [
                {"start": 0.0, "end": 2.5, "text": " Hello world"},
                {"start": 3.0, "end": 6.0, "text": " Rust is fast"}
            ]
        });
        let (entries, lang) = parse_whisper_json(&json.to_string()).unwrap();
        assert_eq!(lang, "en");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello world");
        assert_eq!(entries[0].start_ms, 0);
        assert_eq!(entries[0].duration_ms, 2500);
        assert_eq!(entries[1].text, "Rust is fast");
        assert_eq!(entries[1].start_ms, 3000);
    }

    #[test]
    fn parse_whisper_json_bengali() {
        let json = serde_json::json!({
            "language": "bengali",
            "segments": [
                {"start": 0.0, "end": 3.0, "text": " আমার সোনার বাংলা"}
            ]
        });
        let (entries, lang) = parse_whisper_json(&json.to_string()).unwrap();
        assert_eq!(lang, "bn");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn detect_key_moments_basic() {
        let entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 2000,
                text: "Hello everyone".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 5000,
                duration_ms: 3000,
                text: "The key takeaway is this".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 20000,
                duration_ms: 2000,
                text: "In conclusion we learned".into(),
                speaker_id: None,
            },
        ];
        let moments = detect_key_moments(&entries);
        assert!(moments.len() >= 2);
    }

    #[test]
    fn enhance_transcript_basic() {
        let entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 2000,
                text: "Hello".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 5000,
                duration_ms: 2000,
                text: "World".into(),
                speaker_id: None,
            },
        ];
        let transcript = enhance_transcript(
            "test123",
            entries,
            TranscriptSource::YouTubeTimedtext,
            "en".to_string(),
        );
        assert_eq!(transcript.video_id, "test123");
        assert_eq!(transcript.language, "en");
        assert_eq!(transcript.word_count, 2);
        assert_eq!(transcript.full_text, "Hello World");
    }

    #[test]
    fn align_to_chapters_basic() {
        let entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 1000,
                text: "Intro text".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 60000,
                duration_ms: 1000,
                text: "Main text".into(),
                speaker_id: None,
            },
        ];
        let chapters = vec![
            Chapter {
                title: "Intro".into(),
                start_secs: 0,
                end_secs: Some(60),
            },
            Chapter {
                title: "Main".into(),
                start_secs: 60,
                end_secs: None,
            },
        ];
        let aligned = align_to_chapters(&entries, &chapters);
        assert_eq!(aligned.len(), 2);
        assert_eq!(aligned[0].0, "Intro");
        assert_eq!(aligned[1].1[0].text, "Main text");
    }

    #[test]
    fn detect_speakers_with_labels() {
        let mut entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 2000,
                text: "[HOST] Welcome everyone".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 5000,
                duration_ms: 2000,
                text: "[GUEST] Thanks for having me".into(),
                speaker_id: None,
            },
        ];
        let speakers = detect_speakers(&mut entries);
        assert!(speakers.len() >= 2);
        assert!(entries[0].speaker_id.is_some());
    }
}
