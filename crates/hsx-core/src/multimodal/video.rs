//! Video transcript extraction — YouTube timedtext API (no API key required).

use crate::error::{HsxError, HsxResult};
use super::{ContentType, MultimodalContent, MultimodalSegment};

/// Extract YouTube video transcript via the timedtext API.
///
/// Parses the `videoId` from the URL, fetches the timed-text XML, and
/// returns a `MultimodalContent` with one segment per caption entry.
pub async fn extract_youtube_transcript(
    url: &str,
    http: &crate::http::client::HttpClient,
) -> HsxResult<MultimodalContent> {
    let video_id = extract_video_id(url)?;
    let api_url = format!(
        "https://video.google.com/timedtext?lang=en&v={video_id}"
    );

    let resp = http.fetch_text(&api_url).await?;
    if resp.is_empty() {
        return Err(HsxError::Extraction(
            "No transcript available for this video".into(),
        ));
    }

    let segments = parse_timedtext_xml(&resp)?;
    let full_text: String = segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let duration = segments.last().and_then(|s| s.offset_ms).unwrap_or(0);

    Ok(MultimodalContent {
        source_url: url.to_string(),
        content_type: ContentType::Video {
            duration_secs: duration / 1000,
            transcript_source: "youtube_timedtext".into(),
        },
        text: full_text,
        segments,
        extracted_at: chrono::Utc::now(),
    })
}

/// Extract YouTube video ID from various URL formats.
pub fn extract_video_id(url: &str) -> HsxResult<String> {
    // Standard: https://www.youtube.com/watch?v=VIDEO_ID
    if let Some(pos) = url.find("v=") {
        let rest = &url[pos + 2..];
        let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() >= 8 {
            return Ok(id);
        }
    }
    // Short: https://youtu.be/VIDEO_ID
    if let Some(pos) = url.find("youtu.be/") {
        let rest = &url[pos + 9..];
        let id: String = rest.chars().take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_').collect();
        if id.len() >= 8 {
            return Ok(id);
        }
    }
    Err(HsxError::Extraction("Could not extract YouTube video ID from URL".into()))
}

/// Parse YouTube timedtext XML into segments.
///
/// Format: `<transcript><text start="0.5" dur="2.1">Hello world</text>...</transcript>`
fn parse_timedtext_xml(xml: &str) -> HsxResult<Vec<MultimodalSegment>> {
    let mut segments = Vec::new();
    let text_re = once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r#"<text[^>]*start="([0-9.]+)"[^>]*>([^<]*)</text>"#).unwrap()
    });

    for cap in text_re.captures_iter(xml) {
        let start_secs: f64 = cap[1].parse().unwrap_or(0.0);
        let raw_text = cap[2].trim().to_string();
        // Decode basic HTML entities
        let text = raw_text
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        if !text.is_empty() {
            segments.push(MultimodalSegment {
                offset_ms: Some((start_secs * 1000.0) as u32),
                page: None,
                text,
            });
        }
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_video_id_standard() {
        let id = extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_short() {
        let id = extract_video_id("https://youtu.be/dQw4w9WgXcQ").unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_invalid() {
        assert!(extract_video_id("https://example.com/video").is_err());
    }

    #[test]
    fn test_parse_timedtext_xml() {
        let xml = r#"<transcript>
            <text start="0.5" dur="2.0">Hello world</text>
            <text start="2.5" dur="1.5">How are you</text>
        </transcript>"#;
        let segments = parse_timedtext_xml(xml).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[0].offset_ms, Some(500));
    }
}
