//! Universal Video Transcript Protocol (UVTP) — transcribe any public video URL.
//!
//! Supports 1000+ platforms via yt-dlp subtitle extraction:
//! YouTube, Vimeo, TikTok, Twitter/X, Instagram, Dailymotion, Twitch, and more.
//!
//! ## Algorithm
//!
//! 1. **YouTube fast-path**: Innertube API — parallel racing, ~1-2s, no subprocess
//! 2. **Universal path**: `yt-dlp --write-subs --write-auto-subs --skip-download`
//!    downloads only the subtitle file (not the video), typically ~2-5s
//! 3. Parse VTT / SRT / JSON3 subtitle files into structured `TranscriptEntry` vecs
//! 4. Enhance with gap-based speaker detection and key moment tagging
//!
//! ## Supported Platforms (via yt-dlp)
//!
//! YouTube, Vimeo, TikTok, Twitter/X, Facebook, Instagram, Reddit (v.redd.it),
//! Twitch, Dailymotion, Bilibili, Rumble, Odysee, PeerTube, and ~1000 more.

use crate::config::FetchiumConfig;
use crate::error::{FetchiumError, FetchiumResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use std::time::Duration;

/// Hard timeout for yt-dlp subtitle extraction (subtitles only, not video).
const YTDLP_SUBTITLE_TIMEOUT_SECS: u64 = 15;

// ─── Public API ────────────────────────────────────────────────

/// Extract transcript from ANY public video URL.
///
/// Routes automatically to the optimal extraction method:
/// - YouTube URLs → Innertube + timedtext parallel racing (~1-2s)
/// - All other URLs → yt-dlp subtitle extraction (~2-5s if captions exist)
///
/// Returns an `EnhancedTranscript` with speaker detection and key moments,
/// compatible with all downstream YouTube Intelligence analysis.
pub async fn fetch_universal_transcript(
    url: &str,
    http: &HttpClient,
    config: &FetchiumConfig,
) -> FetchiumResult<EnhancedTranscript> {
    if is_youtube_url(url) {
        let video_id = crate::multimodal::video::extract_video_id(url)?;
        return crate::youtube::transcript::fetch_transcript(&video_id, http, config).await;
    }
    fetch_via_ytdlp_universal(url).await
}

/// Returns `true` if the URL points to a YouTube video (any format).
pub fn is_youtube_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("youtube.com/watch")
        || lower.contains("youtu.be/")
        || lower.contains("youtube.com/shorts/")
        || lower.contains("youtube.com/embed/")
        || lower.contains("m.youtube.com/watch")
        || lower.contains("music.youtube.com/watch")
}

/// Detect the platform name from a URL (for display purposes).
pub fn detect_platform(url: &str) -> &'static str {
    let lower_url = url.to_lowercase();
    if lower_url.contains("youtube.com") || lower_url.contains("youtu.be") {
        "YouTube"
    } else if lower_url.contains("vimeo.com") {
        "Vimeo"
    } else if lower_url.contains("tiktok.com") {
        "TikTok"
    } else if lower_url.contains("twitter.com") || lower_url.contains("x.com") {
        "Twitter/X"
    } else if lower_url.contains("instagram.com") {
        "Instagram"
    } else if lower_url.contains("twitch.tv") {
        "Twitch"
    } else if lower_url.contains("dailymotion.com") {
        "Dailymotion"
    } else if lower_url.contains("facebook.com") || lower_url.contains("fb.watch") {
        "Facebook"
    } else if lower_url.contains("reddit.com") || lower_url.contains("v.redd.it") {
        "Reddit"
    } else if lower_url.contains("rumble.com") {
        "Rumble"
    } else if lower_url.contains("odysee.com") || lower_url.contains("lbry.tv") {
        "Odysee"
    } else if lower_url.contains("bilibili.com") {
        "Bilibili"
    } else {
        "Video"
    }
}

// ─── yt-dlp Subtitle Extraction ────────────────────────────────

/// Extract transcript via yt-dlp for any yt-dlp-supported platform.
///
/// Races manual subtitles against auto-generated captions simultaneously;
/// returns the first non-empty result. Cleans up temp files afterward.
async fn fetch_via_ytdlp_universal(url: &str) -> FetchiumResult<EnhancedTranscript> {
    let hash = url_hash(url);
    let tmp_dir = std::env::temp_dir().join(format!("fetchium_subs_{hash}"));
    let _ = tokio::fs::create_dir_all(&tmp_dir).await;
    let output_template = format!("{}/%(id)s", tmp_dir.display());

    // Race manual subs against auto-generated captions in parallel.
    // Most platforms with captions respond in 2-5s; timeout caps at 15s.
    let (manual_result, auto_result) = tokio::join!(
        run_ytdlp_subs(url, &output_template, false),
        run_ytdlp_subs(url, &output_template, true),
    );

    let _ = tokio::fs::remove_dir_all(&tmp_dir).await;

    // Prefer manual subtitles (higher quality), fall back to auto-generated.
    let entries = manual_result
        .ok()
        .filter(|e: &Vec<TranscriptEntry>| !e.is_empty())
        .or_else(|| auto_result.ok().filter(|e| !e.is_empty()))
        .ok_or_else(|| {
            FetchiumError::YouTube(format!(
                "No captions/subtitles found for '{url}'. \
                 Make sure yt-dlp is installed (`pip install yt-dlp`) \
                 and the video has captions enabled."
            ))
        })?;

    Ok(build_enhanced_transcript(
        url,
        entries,
        TranscriptSource::YtDlp,
    ))
}

/// Invoke yt-dlp to download subtitle files (no video download).
///
/// With `auto_subs = false` downloads manual/community captions.
/// With `auto_subs = true` downloads auto-generated captions (ASR).
async fn run_ytdlp_subs(
    url: &str,
    output_template: &str,
    auto_subs: bool,
) -> FetchiumResult<Vec<TranscriptEntry>> {
    let sub_flag = if auto_subs {
        "--write-auto-subs"
    } else {
        "--write-subs"
    };

    // Extract parent directory from the template for file scanning.
    let tmp_dir = std::path::Path::new(output_template)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(std::env::temp_dir);

    let output = tokio::time::timeout(
        Duration::from_secs(YTDLP_SUBTITLE_TIMEOUT_SECS),
        tokio::process::Command::new("yt-dlp")
            .args([
                sub_flag,
                "--sub-lang",
                "en.*,bn.*,hi.*,*",
                "--sub-format",
                "json3/vtt/srt/best",
                "--skip-download",
                "--no-playlist",
                "--no-warnings",
                "--quiet",
                "--output",
                output_template,
                url,
            ])
            .output(),
    )
    .await
    .map_err(|_| FetchiumError::YouTube("yt-dlp subtitle extraction timed out (15s)".into()))?
    .map_err(|e| {
        FetchiumError::YouTube(format!(
            "yt-dlp not found — install via `pip install yt-dlp`: {e}"
        ))
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FetchiumError::YouTube(format!("yt-dlp error: {stderr}")));
    }

    // Scan temp dir for subtitle file written by yt-dlp.
    // Files are named: `{id}.en.vtt`, `{id}.en-orig.vtt`, `{id}.en.srt`, etc.
    let mut read_dir = tokio::fs::read_dir(&tmp_dir)
        .await
        .map_err(|e| FetchiumError::YouTube(format!("temp dir read error: {e}")))?;

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let is_subtitle = matches!(ext.as_str(), "vtt" | "srt" | "json3");
        if !is_subtitle {
            continue;
        }

        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            let _ = tokio::fs::remove_file(&path).await;

            let entries = match ext.as_str() {
                "vtt" => parse_vtt(&content),
                "srt" => parse_srt(&content),
                "json3" => parse_json3_subtitle(&content),
                _ => continue,
            };

            if !entries.is_empty() {
                return Ok(entries);
            }
        }
    }

    Err(FetchiumError::YouTube(
        "yt-dlp wrote no subtitle file".into(),
    ))
}

// ─── Subtitle Format Parsers ───────────────────────────────────

/// Parse WebVTT subtitle format into `TranscriptEntry` vec.
///
/// Handles all standard VTT features:
/// - `HH:MM:SS.mmm --> HH:MM:SS.mmm` timestamps
/// - Positioning tags stripped (`align:start`, `position:0%`)
/// - Inline tags stripped (`<c>`, `<b>`, timestamp cues like `<00:01.234>`)
/// - HTML entities decoded
pub fn parse_vtt(content: &str) -> Vec<TranscriptEntry> {
    let mut entries = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Skip WEBVTT header and metadata blocks
    while i < lines.len() && !lines[i].contains("-->") {
        i += 1;
    }

    while i < lines.len() {
        let line = lines[i];

        if !line.contains("-->") {
            i += 1;
            continue;
        }

        // Timestamp line: "00:00:01.000 --> 00:00:04.000 [optional positioning]"
        // Split on "-->" and take only the timestamp portions
        let parts: Vec<&str> = line.splitn(2, "-->").collect();
        if parts.len() < 2 {
            i += 1;
            continue;
        }

        let start_ms = match parse_vtt_timestamp(parts[0].trim()) {
            Some(ms) => ms,
            None => {
                i += 1;
                continue;
            }
        };
        // End timestamp: strip any trailing positioning info
        let end_str = parts[1].split_whitespace().next().unwrap_or("");
        let end_ms = parse_vtt_timestamp(end_str);

        i += 1;

        // Collect text lines until blank line or next cue header
        let mut text_parts: Vec<String> = Vec::new();
        while i < lines.len() {
            let tl = lines[i];
            if tl.trim().is_empty() {
                i += 1;
                break;
            }
            // Skip cue identifier lines (pure digits like "1", "2")
            if tl.trim().chars().all(|c| c.is_ascii_digit()) {
                i += 1;
                continue;
            }
            let cleaned = strip_vtt_tags(tl);
            if !cleaned.trim().is_empty() {
                text_parts.push(cleaned);
            }
            i += 1;
        }

        let text = text_parts.join(" ");
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let dur_ms = end_ms.map(|e| e.saturating_sub(start_ms)).unwrap_or(2000);

        entries.push(TranscriptEntry {
            start_ms: start_ms as u32,
            duration_ms: dur_ms as u32,
            text: decode_subtitle_text(&text),
            speaker_id: None,
        });
    }

    // Deduplicate consecutive identical entries (common in yt-dlp VTT output)
    entries.dedup_by(|a, b| a.text == b.text && a.start_ms.abs_diff(b.start_ms) < 500);

    entries
}

/// Parse SRT subtitle format into `TranscriptEntry` vec.
///
/// SRT uses commas as millisecond separators: `00:00:01,000 --> 00:00:04,000`.
pub fn parse_srt(content: &str) -> Vec<TranscriptEntry> {
    let mut entries = Vec::new();
    let mut lines = content.lines().peekable();

    loop {
        // Find next timestamp line
        let mut ts_line = String::new();
        let mut found = false;
        for line in lines.by_ref() {
            if line.contains("-->") {
                ts_line = line.to_string();
                found = true;
                break;
            }
        }
        if !found {
            break;
        }

        // SRT uses comma for ms: "00:00:01,000 --> 00:00:04,000"
        let normalized = ts_line.replace(',', ".");
        let parts: Vec<&str> = normalized.splitn(2, "-->").collect();
        if parts.len() < 2 {
            continue;
        }

        let start_ms = match parse_vtt_timestamp(parts[0].trim()) {
            Some(ms) => ms,
            None => continue,
        };
        let end_str = parts[1].split_whitespace().next().unwrap_or("");
        let end_ms = parse_vtt_timestamp(end_str);

        // Collect text until blank line
        let mut text_parts = Vec::new();
        for line in lines.by_ref() {
            if line.trim().is_empty() {
                break;
            }
            let cleaned = strip_vtt_tags(line);
            if !cleaned.trim().is_empty() {
                text_parts.push(cleaned);
            }
        }

        let text = text_parts.join(" ");
        let text = text.trim().to_string();
        if text.is_empty() {
            continue;
        }

        let dur_ms = end_ms.map(|e| e.saturating_sub(start_ms)).unwrap_or(2000);

        entries.push(TranscriptEntry {
            start_ms: start_ms as u32,
            duration_ms: dur_ms as u32,
            text: decode_subtitle_text(&text),
            speaker_id: None,
        });
    }

    entries
}

/// Parse YouTube's JSON3 subtitle format.
///
/// Used when yt-dlp requests `--sub-format json3`.
fn parse_json3_subtitle(content: &str) -> Vec<TranscriptEntry> {
    let v: serde_json::Value = match serde_json::from_str(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    v["events"]
        .as_array()
        .map(|events| {
            events
                .iter()
                .filter_map(|ev| {
                    let start_ms = ev["tStartMs"].as_u64()? as u32;
                    let dur_ms = ev["dDurationMs"].as_u64().unwrap_or(2000) as u32;
                    let segs = ev["segs"].as_array()?;
                    let text: String = segs
                        .iter()
                        .filter_map(|s| s["utf8"].as_str())
                        .collect::<Vec<_>>()
                        .join("")
                        .trim()
                        .to_string();
                    if text.is_empty() {
                        return None;
                    }
                    Some(TranscriptEntry {
                        start_ms,
                        duration_ms: dur_ms,
                        text: decode_subtitle_text(&text),
                        speaker_id: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

// ─── Timestamp Parsing ─────────────────────────────────────────

/// Parse a VTT/SRT timestamp into milliseconds.
///
/// Accepts: `HH:MM:SS.mmm`, `MM:SS.mmm`, `HH:MM:SS,mmm` (SRT with comma).
pub fn parse_vtt_timestamp(s: &str) -> Option<u64> {
    // Strip any trailing positioning info
    let s = s.split_whitespace().next()?;
    // Normalise comma (SRT) to dot
    let s = &s.replace(',', ".");
    let parts: Vec<&str> = s.split(':').collect();

    let (hours, minutes, seconds_str) = match parts.as_slice() {
        [m, s] => (0u64, m.parse::<u64>().ok()?, *s),
        [h, m, s] => (h.parse::<u64>().ok()?, m.parse::<u64>().ok()?, *s),
        _ => return None,
    };

    let sec_parts: Vec<&str> = seconds_str.split('.').collect();
    let secs: u64 = sec_parts.first()?.parse().ok()?;
    let millis: u64 = sec_parts
        .get(1)
        .map(|ms| {
            let padded = format!("{:0<3}", &ms[..ms.len().min(3)]);
            padded.parse().unwrap_or(0)
        })
        .unwrap_or(0);

    Some((hours * 3600 + minutes * 60 + secs) * 1000 + millis)
}

// ─── Text Cleaning ─────────────────────────────────────────────

/// Strip VTT/SRT inline tags: `<c>`, `</c>`, timestamp cues, `<b>`, `<i>`, etc.
fn strip_vtt_tags(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Decode HTML entities and normalise whitespace for subtitle text.
fn decode_subtitle_text(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace('\n', " ")
}

// ─── Enhancement ───────────────────────────────────────────────

/// Build `EnhancedTranscript` from raw subtitle entries.
///
/// Applies gap-based speaker turn detection and key moment tagging —
/// same quality as the YouTube-specific path.
fn build_enhanced_transcript(
    url: &str,
    mut entries: Vec<TranscriptEntry>,
    source: TranscriptSource,
) -> EnhancedTranscript {
    // Gap-based speaker turn detection (>3s gap = new speaker turn)
    let mut current_speaker = 0u32;
    let mut last_end_ms = 0u32;
    let mut speaker_counts = [0usize; 2];

    for entry in &mut entries {
        let gap = entry.start_ms.saturating_sub(last_end_ms);
        if gap > 3000 {
            current_speaker = 1 - current_speaker;
        }
        entry.speaker_id = Some(current_speaker);
        speaker_counts[current_speaker as usize] += 1;
        last_end_ms = entry.start_ms + entry.duration_ms;
    }

    let speakers: Vec<Speaker> = ["Speaker A", "Speaker B"]
        .iter()
        .enumerate()
        .filter(|(i, _)| speaker_counts[*i] > 0)
        .map(|(i, label)| Speaker {
            id: i as u32,
            label: label.to_string(),
            segment_count: speaker_counts[i],
        })
        .collect();

    let key_moments = detect_key_moments_uvtp(&entries);

    let full_text: String = entries
        .iter()
        .map(|e| e.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let word_count = full_text.split_whitespace().count();

    // Use a stable ID derived from the URL
    let video_id = url_hash(url);

    EnhancedTranscript {
        video_id,
        language: "en".to_string(),
        entries,
        speakers,
        key_moments,
        full_text,
        word_count,
        source,
        quality_score: 1.0,
    }
}

/// Key moment detection for universal transcripts.
fn detect_key_moments_uvtp(entries: &[TranscriptEntry]) -> Vec<KeyMoment> {
    const TRANSITION_PHRASES: &[(&str, MomentType, f64)] = &[
        ("in summary", MomentType::Conclusion, 0.9),
        ("in conclusion", MomentType::Conclusion, 0.9),
        ("to summarize", MomentType::Conclusion, 0.9),
        ("the key point", MomentType::KeyPoint, 0.85),
        ("most importantly", MomentType::KeyPoint, 0.85),
        ("for example", MomentType::Example, 0.7),
        ("such as", MomentType::Example, 0.6),
        ("moving on", MomentType::TopicShift, 0.6),
        ("next", MomentType::TopicShift, 0.5),
        ("first", MomentType::TopicShift, 0.5),
        ("finally", MomentType::TopicShift, 0.7),
        ("that means", MomentType::Definition, 0.75),
        ("which means", MomentType::Definition, 0.75),
        ("is defined as", MomentType::Definition, 0.8),
    ];

    let mut moments = Vec::new();
    let mut last_ts = 0u32;

    for entry in entries {
        // Debounce: don't add two moments within 10s
        if entry.start_ms < last_ts + 10_000 {
            continue;
        }

        let lower = entry.text.to_lowercase();
        for (phrase, moment_type, importance) in TRANSITION_PHRASES {
            if lower.contains(phrase) {
                moments.push(KeyMoment {
                    timestamp_ms: entry.start_ms,
                    moment_type: *moment_type,
                    text: entry.text.clone(),
                    importance: *importance,
                });
                last_ts = entry.start_ms;
                break;
            }
        }
    }

    moments
}

// ─── Utilities ─────────────────────────────────────────────────

/// Stable 16-hex-char hash of a URL for temp directory naming.
fn url_hash(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    format!("{:016x}", h.finish())
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_youtube_url_variants() {
        assert!(is_youtube_url("https://www.youtube.com/watch?v=abc123"));
        assert!(is_youtube_url("https://youtu.be/abc123"));
        assert!(is_youtube_url("https://m.youtube.com/watch?v=abc123"));
        assert!(is_youtube_url("https://youtube.com/shorts/abc123"));
        assert!(!is_youtube_url("https://vimeo.com/123456789"));
        assert!(!is_youtube_url("https://tiktok.com/@user/video/123"));
    }

    #[test]
    fn detect_platform_variants() {
        assert_eq!(detect_platform("https://vimeo.com/123"), "Vimeo");
        assert_eq!(
            detect_platform("https://tiktok.com/@user/video/123"),
            "TikTok"
        );
        assert_eq!(
            detect_platform("https://twitter.com/user/status/123"),
            "Twitter/X"
        );
        assert_eq!(detect_platform("https://twitch.tv/videos/123"), "Twitch");
        assert_eq!(detect_platform("https://example.com/video.mp4"), "Video");
    }

    #[test]
    fn parse_vtt_timestamp_variants() {
        assert_eq!(parse_vtt_timestamp("00:00:01.000"), Some(1000));
        assert_eq!(parse_vtt_timestamp("01:23:45.678"), Some(5025678));
        assert_eq!(parse_vtt_timestamp("01:30.500"), Some(90500));
        assert_eq!(parse_vtt_timestamp("00:00:00.000"), Some(0));
        // SRT comma format
        assert_eq!(parse_vtt_timestamp("00:00:01,500"), Some(1500));
        assert_eq!(parse_vtt_timestamp("invalid"), None);
    }

    #[test]
    fn parse_vtt_basic() {
        let vtt = "WEBVTT\n\n\
                   00:00:01.000 --> 00:00:04.000\n\
                   Hello, welcome to the video.\n\n\
                   00:00:04.500 --> 00:00:08.000\n\
                   Today we cover Rust.\n\n";
        let entries = parse_vtt(vtt);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello, welcome to the video.");
        assert_eq!(entries[0].start_ms, 1000);
        assert_eq!(entries[0].duration_ms, 3000);
        assert_eq!(entries[1].text, "Today we cover Rust.");
        assert_eq!(entries[1].start_ms, 4500);
    }

    #[test]
    fn parse_vtt_with_inline_tags() {
        let vtt = "WEBVTT\n\n\
                   00:00:01.000 --> 00:00:04.000\n\
                   <c>Hello</c> <00:00:01.234><b>world</b>\n\n";
        let entries = parse_vtt(vtt);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Hello world");
    }

    #[test]
    fn parse_vtt_with_positioning() {
        let vtt = "WEBVTT\n\n\
                   00:00:01.000 --> 00:00:04.000 align:start position:0%\n\
                   Caption with positioning.\n\n";
        let entries = parse_vtt(vtt);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "Caption with positioning.");
    }

    #[test]
    fn parse_srt_basic() {
        let srt = "1\n\
                   00:00:01,000 --> 00:00:04,000\n\
                   Hello, welcome to the video.\n\n\
                   2\n\
                   00:00:04,500 --> 00:00:08,000\n\
                   Today we cover Rust.\n\n";
        let entries = parse_srt(srt);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello, welcome to the video.");
        assert_eq!(entries[0].start_ms, 1000);
        assert_eq!(entries[1].text, "Today we cover Rust.");
    }

    #[test]
    fn parse_srt_multiline() {
        let srt = "1\n\
                   00:00:01,000 --> 00:00:05,000\n\
                   Line one of the caption.\n\
                   Line two continues here.\n\n";
        let entries = parse_srt(srt);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].text.contains("Line one"));
        assert!(entries[0].text.contains("Line two"));
    }

    #[test]
    fn parse_vtt_dedup() {
        // yt-dlp sometimes outputs duplicate consecutive lines
        let vtt = "WEBVTT\n\n\
                   00:00:01.000 --> 00:00:03.000\n\
                   Duplicate line.\n\n\
                   00:00:01.200 --> 00:00:03.000\n\
                   Duplicate line.\n\n\
                   00:00:05.000 --> 00:00:07.000\n\
                   Different line.\n\n";
        let entries = parse_vtt(vtt);
        // Should deduplicate the close-together identical entries
        let dup_count = entries
            .iter()
            .filter(|e| e.text == "Duplicate line.")
            .count();
        assert_eq!(dup_count, 1);
    }

    #[test]
    fn parse_json3_subtitle_basic() {
        let json = serde_json::json!({
            "events": [
                { "tStartMs": 1000, "dDurationMs": 3000, "segs": [{"utf8": "Hello world"}] },
                { "tStartMs": 5000, "dDurationMs": 2000, "segs": [{"utf8": "Rust is fast"}] }
            ]
        });
        let entries = parse_json3_subtitle(&json.to_string());
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello world");
        assert_eq!(entries[1].text, "Rust is fast");
    }

    #[test]
    fn strip_vtt_tags_variants() {
        assert_eq!(strip_vtt_tags("<c>Hello</c> <b>world</b>"), "Hello world");
        assert_eq!(strip_vtt_tags("plain text"), "plain text");
        assert_eq!(strip_vtt_tags("<00:01.234>text"), "text");
    }

    #[test]
    fn decode_subtitle_entities() {
        assert_eq!(decode_subtitle_text("It&apos;s &amp; good"), "It's & good");
        assert_eq!(decode_subtitle_text("&lt;tag&gt;"), "<tag>");
        assert_eq!(decode_subtitle_text("line1\nline2"), "line1 line2");
    }

    #[test]
    fn key_moments_detection() {
        let entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 2000,
                text: "Hello everyone".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 15000,
                duration_ms: 3000,
                text: "In conclusion, we learned a lot today".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 30000,
                duration_ms: 2000,
                text: "For example, consider this case".into(),
                speaker_id: None,
            },
        ];
        let moments = detect_key_moments_uvtp(&entries);
        assert!(moments.len() >= 2);
        assert!(moments
            .iter()
            .any(|m| m.moment_type == MomentType::Conclusion));
    }

    #[test]
    fn build_enhanced_transcript_speakers() {
        let entries = vec![
            TranscriptEntry {
                start_ms: 0,
                duration_ms: 2000,
                text: "First speaker".into(),
                speaker_id: None,
            },
            TranscriptEntry {
                start_ms: 10000, // 10s gap → speaker change
                duration_ms: 2000,
                text: "Second speaker".into(),
                speaker_id: None,
            },
        ];
        let t =
            build_enhanced_transcript("https://vimeo.com/123", entries, TranscriptSource::YtDlp);
        assert_eq!(t.word_count, 4);
        assert_eq!(t.source, TranscriptSource::YtDlp);
        assert_eq!(t.speakers.len(), 2);
    }

    #[test]
    fn url_hash_stable() {
        let h1 = url_hash("https://vimeo.com/123456");
        let h2 = url_hash("https://vimeo.com/123456");
        let h3 = url_hash("https://vimeo.com/789012");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 16);
    }
}
