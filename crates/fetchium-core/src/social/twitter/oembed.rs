//! Twitter oEmbed API — fetch tweet data via X's public endpoint (no auth needed).

use crate::error::FetchiumError;
use crate::http::client::HttpClient;
use crate::social::twitter::types::{Tweet, TwitterUser};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct OEmbedResponse {
    html: String,
    author_name: String,
    author_url: String,
    url: String,
}

/// Fetch tweet data via X's public oEmbed endpoint.
/// GET https://publish.twitter.com/oembed?url=https://x.com/user/status/123
pub async fn fetch_oembed(tweet_url: &str, http: &HttpClient) -> Result<Tweet, FetchiumError> {
    let oembed_url = format!(
        "https://publish.twitter.com/oembed?url={}&omit_script=true",
        urlencoding_encode(tweet_url)
    );

    let response = http.fetch_text(&oembed_url).await?;
    let oembed: OEmbedResponse = serde_json::from_str(&response)
        .map_err(|e| FetchiumError::Extraction(format!("oEmbed parse error: {e}")))?;

    // Parse tweet text from the HTML response
    let text = extract_text_from_oembed_html(&oembed.html);
    let username = oembed
        .author_url
        .rsplit('/')
        .next()
        .unwrap_or("unknown")
        .to_string();

    Ok(Tweet {
        id: extract_tweet_id(tweet_url).unwrap_or_default(),
        url: oembed.url,
        author: TwitterUser {
            username,
            display_name: oembed.author_name,
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
        hashtags: Vec::new(),
        mentions: Vec::new(),
        media_urls: Vec::new(),
        is_reply: false,
        is_retweet: false,
        quoted_tweet: None,
    })
}

fn extract_text_from_oembed_html(html: &str) -> String {
    // Simple HTML tag stripping for oEmbed response
    let mut text = html.to_string();
    // Remove script/style blocks
    while let (Some(start), Some(end)) = (text.find("<script"), text.find("</script>")) {
        if start < end {
            text = format!("{}{}", &text[..start], &text[end + 9..]);
        } else {
            break;
        }
    }
    // Remove HTML tags
    let mut result = String::new();
    let mut in_tag = false;
    for ch in text.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

fn extract_tweet_id(url: &str) -> Option<String> {
    // Extract tweet ID from URLs like https://x.com/user/status/123456
    url.rsplit('/').next().map(|s| s.to_string())
}

/// Percent-encode a string for use in URLs.
fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::with_capacity(s.len() * 2);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_tweet_id_from_url() {
        assert_eq!(
            extract_tweet_id("https://x.com/user/status/123456"),
            Some("123456".into())
        );
    }

    #[test]
    fn strip_html_tags() {
        let html = "<p>Hello <b>world</b></p>";
        assert_eq!(extract_text_from_oembed_html(html), "Hello world");
    }

    #[test]
    fn urlencoding_encodes_special_chars() {
        let encoded = urlencoding_encode("https://x.com/user/status/123");
        assert!(encoded.contains("%3A")); // colon
        assert!(encoded.contains("%2F")); // slash
    }
}
