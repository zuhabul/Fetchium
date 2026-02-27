//! YouTube comment fetching, sentiment analysis, authenticity scoring, topic extraction.

use crate::error::{HsxError, HsxResult};
use crate::http::client::HttpClient;
use crate::youtube::types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

// ─── Comment Fetching ──────────────────────────────────────────

/// Fetch comments for a video (Invidious primary, with fallback).
pub async fn fetch_comments(
    video_id: &str,
    http: &HttpClient,
    config: &crate::config::HsxConfig,
    max_comments: usize,
) -> HsxResult<Vec<VideoComment>> {
    // Source 1: Invidious comments API
    for instance in &config.youtube.invidious_instances {
        let url = format!("{instance}/api/v1/comments/{video_id}");
        match tokio::time::timeout(
            Duration::from_secs(config.youtube.timeout_secs),
            http.fetch_text(&url),
        )
        .await
        {
            Ok(Ok(body)) => {
                if let Ok(comments) = parse_invidious_comments(&body, max_comments) {
                    if !comments.is_empty() {
                        return Ok(comments);
                    }
                }
            }
            _ => continue,
        }
    }

    // Source 2: Piped comments API
    for instance in &config.youtube.piped_instances {
        let url = format!("{instance}/comments/{video_id}");
        match tokio::time::timeout(
            Duration::from_secs(config.youtube.timeout_secs),
            http.fetch_text(&url),
        )
        .await
        {
            Ok(Ok(body)) => {
                if let Ok(comments) = parse_piped_comments(&body, max_comments) {
                    if !comments.is_empty() {
                        return Ok(comments);
                    }
                }
            }
            _ => continue,
        }
    }

    Err(HsxError::YouTube(format!(
        "Could not fetch comments for video {video_id}"
    )))
}

fn parse_invidious_comments(body: &str, max: usize) -> HsxResult<Vec<VideoComment>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| HsxError::YouTube(format!("Comments parse: {e}")))?;

    let comments = v["comments"]
        .as_array()
        .ok_or_else(|| HsxError::YouTube("No comments array".into()))?;

    Ok(comments
        .iter()
        .take(max)
        .map(|c| VideoComment {
            author: c["author"].as_str().unwrap_or("").to_string(),
            text: c["content"].as_str().unwrap_or("").to_string(),
            likes: c["likeCount"].as_u64().unwrap_or(0) as u32,
            published: c["publishedText"].as_str().unwrap_or("").to_string(),
            is_hearted: c["creatorHeart"].as_object().is_some(),
            reply_count: c["replies"]
                .as_object()
                .and_then(|r| r.get("replyCount"))
                .and_then(|c| c.as_u64())
                .unwrap_or(0) as u32,
        })
        .collect())
}

fn parse_piped_comments(body: &str, max: usize) -> HsxResult<Vec<VideoComment>> {
    let v: Value = serde_json::from_str(body)
        .map_err(|e| HsxError::YouTube(format!("Piped comments parse: {e}")))?;

    let comments = v["comments"]
        .as_array()
        .ok_or_else(|| HsxError::YouTube("No comments array".into()))?;

    Ok(comments
        .iter()
        .take(max)
        .map(|c| VideoComment {
            author: c["author"].as_str().unwrap_or("").to_string(),
            text: c["commentText"].as_str().unwrap_or("").to_string(),
            likes: c["likeCount"].as_u64().unwrap_or(0) as u32,
            published: c["commentedTime"].as_str().unwrap_or("").to_string(),
            is_hearted: c["hearted"].as_bool().unwrap_or(false),
            reply_count: c["replyCount"].as_u64().unwrap_or(0) as u32,
        })
        .collect())
}

// ─── Sentiment Analysis ────────────────────────────────────────

/// Analyze comments and produce a full CommentAnalysis report.
pub fn analyze_comments(comments: &[VideoComment]) -> CommentAnalysis {
    let sentiment = compute_overall_sentiment(comments);
    let top_topics = extract_topics(comments);
    let authenticity = assess_authenticity(comments);
    let informative = find_informative_comments(comments);

    CommentAnalysis {
        total_comments: comments.len(),
        analyzed_comments: comments.len(),
        overall_sentiment: sentiment,
        top_topics,
        authenticity,
        informative_comments: informative,
    }
}

/// Compute overall sentiment from comments using AFINN-style lexicon.
pub fn compute_overall_sentiment(comments: &[VideoComment]) -> SentimentScore {
    if comments.is_empty() {
        return SentimentScore {
            positive: 0.0,
            negative: 0.0,
            neutral: 1.0,
            compound: 0.0,
        };
    }

    let mut total_positive = 0.0f64;
    let mut total_negative = 0.0f64;
    let mut total_neutral = 0.0f64;

    for comment in comments {
        let score = score_text_sentiment(&comment.text);
        if score > 0.0 {
            total_positive += 1.0;
        } else if score < 0.0 {
            total_negative += 1.0;
        } else {
            total_neutral += 1.0;
        }
    }

    let total = comments.len() as f64;
    let positive = total_positive / total;
    let negative = total_negative / total;
    let neutral = total_neutral / total;
    let compound = (total_positive - total_negative) / total;

    SentimentScore {
        positive,
        negative,
        neutral,
        compound,
    }
}

/// Score a single text's sentiment using the lexicon + negation + intensifiers.
pub fn score_text_sentiment(text: &str) -> f64 {
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut score = 0.0f64;
    let mut negated = false;
    let mut intensifier = 1.0f64;

    for word in &words {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());

        // Check negation
        if NEGATION_WORDS.contains(&clean) {
            negated = true;
            continue;
        }

        // Check intensifier
        if let Some((_, mult)) = INTENSIFIERS.iter().find(|(w, _)| *w == clean) {
            intensifier = *mult;
            continue;
        }

        // Lookup sentiment
        if let Some((_, s)) = SENTIMENT_LEXICON.iter().find(|(w, _)| *w == clean) {
            let mut word_score = *s as f64 * intensifier;
            if negated {
                word_score = -word_score;
                negated = false;
            }
            score += word_score;
            intensifier = 1.0;
        }
    }

    score
}

// ─── Authenticity Assessment ───────────────────────────────────

/// Assess the authenticity of a comment section.
pub fn assess_authenticity(comments: &[VideoComment]) -> AuthenticityReport {
    if comments.is_empty() {
        return AuthenticityReport {
            score: 0.5,
            bot_percentage: 0.0,
            spam_percentage: 0.0,
            uniformity: 0.0,
        };
    }

    let mut bot_count = 0usize;
    let mut spam_count = 0usize;

    for comment in comments {
        if is_likely_bot(comment) {
            bot_count += 1;
        }
        if is_likely_spam(&comment.text) {
            spam_count += 1;
        }
    }

    let total = comments.len() as f64;
    let bot_pct = bot_count as f64 / total;
    let spam_pct = spam_count as f64 / total;

    // Uniformity: how similar comments are to each other (1.0 = all identical)
    let uniformity = compute_uniformity(comments);

    let score = (1.0 - bot_pct * 0.4 - spam_pct * 0.4 - uniformity * 0.2).max(0.0);

    AuthenticityReport {
        score,
        bot_percentage: bot_pct,
        spam_percentage: spam_pct,
        uniformity,
    }
}

/// Check if a comment looks like it came from a bot.
fn is_likely_bot(comment: &VideoComment) -> bool {
    let text = &comment.text;
    // Bot indicators: very short + emoji only, exact copy patterns, url spam
    if text.len() < 5 && text.chars().all(|c| !c.is_alphanumeric()) {
        return true;
    }
    // Check for common bot patterns
    let lower = text.to_lowercase();
    if lower.contains("check my channel")
        || lower.contains("sub to me")
        || lower.contains("i'm gifting")
        || lower.contains("promo sm")
    {
        return true;
    }
    false
}

/// Check if a text is likely spam.
fn is_likely_spam(text: &str) -> bool {
    let lower = text.to_lowercase();
    // URL spam
    let url_count = lower.matches("http").count();
    if url_count >= 2 {
        return true;
    }
    // Repeated characters
    if lower
        .chars()
        .take(20)
        .all(|c| c == lower.chars().next().unwrap_or(' '))
        && text.len() > 10
    {
        return true;
    }
    // Common spam phrases
    if lower.contains("free gift")
        || lower.contains("click here")
        || lower.contains("dm me")
        || lower.contains("check my")
        || lower.contains("make money")
    {
        return true;
    }
    false
}

/// Compute uniformity (similarity) of comments. 1.0 = all identical.
fn compute_uniformity(comments: &[VideoComment]) -> f64 {
    if comments.len() < 3 {
        return 0.0;
    }

    // Sample up to 50 pairs for efficiency
    let sample_size = comments.len().min(50);
    let mut same_count = 0u32;
    let mut total_pairs = 0u32;

    for i in 0..sample_size.saturating_sub(1) {
        for j in (i + 1)..sample_size {
            total_pairs += 1;
            let a = &comments[i].text.to_lowercase();
            let b = &comments[j].text.to_lowercase();
            if a == b || bigram_jaccard(a, b) > 0.8 {
                same_count += 1;
            }
        }
    }

    if total_pairs == 0 {
        0.0
    } else {
        same_count as f64 / total_pairs as f64
    }
}

/// Bigram Jaccard similarity between two strings.
fn bigram_jaccard(a: &str, b: &str) -> f64 {
    let a_bigrams = make_bigrams(a);
    let b_bigrams = make_bigrams(b);

    if a_bigrams.is_empty() && b_bigrams.is_empty() {
        return 1.0;
    }

    let mut intersection = 0usize;
    let mut union = 0usize;

    let mut all_keys: std::collections::HashSet<&String> = std::collections::HashSet::new();
    all_keys.extend(a_bigrams.keys());
    all_keys.extend(b_bigrams.keys());

    for key in all_keys {
        let ca = a_bigrams.get(key).copied().unwrap_or(0);
        let cb = b_bigrams.get(key).copied().unwrap_or(0);
        intersection += ca.min(cb);
        union += ca.max(cb);
    }

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn make_bigrams(s: &str) -> BigramSet {
    let chars: Vec<char> = s.chars().collect();
    let mut bigrams = HashMap::new();
    for window in chars.windows(2) {
        let key = format!("{}{}", window[0], window[1]);
        *bigrams.entry(key).or_insert(0) += 1;
    }
    bigrams
}

// ─── Topic Extraction ──────────────────────────────────────────

/// Extract top topics from comments using bigram TF-IDF + stopword filtering.
pub fn extract_topics(comments: &[VideoComment]) -> Vec<TopicCluster> {
    if comments.is_empty() {
        return Vec::new();
    }

    let stopwords: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut bigram_counts: HashMap<String, usize> = HashMap::new();
    let mut bigram_examples: HashMap<String, String> = HashMap::new();

    for comment in comments {
        let words: Vec<String> = comment
            .text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| w.len() > 2 && !stopwords.contains(w.as_str()))
            .collect();

        for window in words.windows(2) {
            let bigram = format!("{} {}", window[0], window[1]);
            *bigram_counts.entry(bigram.clone()).or_insert(0) += 1;
            bigram_examples
                .entry(bigram)
                .or_insert_with(|| comment.text.clone());
        }
    }

    // Sort by count, take top 10
    let mut sorted: Vec<(String, usize)> = bigram_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(10)
        .map(|(topic, count)| TopicCluster {
            representative: bigram_examples.get(&topic).cloned().unwrap_or_default(),
            topic,
            count,
        })
        .collect()
}

// ─── Informative Comment Scoring ───────────────────────────────

/// Find the most informative comments (long, liked, substantive).
pub fn find_informative_comments(comments: &[VideoComment]) -> Vec<ScoredComment> {
    let mut scored: Vec<ScoredComment> = comments
        .iter()
        .map(|c| {
            let length_score = (c.text.split_whitespace().count() as f64 / 50.0).min(1.0);
            let like_score = (c.likes as f64).ln_1p() / 10.0;
            let heart_bonus = if c.is_hearted { 0.2 } else { 0.0 };
            let score = length_score * 0.4 + like_score * 0.4 + heart_bonus + 0.0;

            ScoredComment {
                text: c.text.clone(),
                author: c.author.clone(),
                score,
            }
        })
        .collect();

    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(10);
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentiment_positive() {
        let score = score_text_sentiment("This is amazing and wonderful");
        assert!(score > 0.0);
    }

    #[test]
    fn sentiment_negative() {
        let score = score_text_sentiment("This is terrible and awful");
        assert!(score < 0.0);
    }

    #[test]
    fn sentiment_negation() {
        let score = score_text_sentiment("This is not good");
        assert!(score < 0.0);
    }

    #[test]
    fn sentiment_intensifier() {
        let base = score_text_sentiment("This is good");
        let intensified = score_text_sentiment("This is very good");
        assert!(intensified > base);
    }

    #[test]
    fn sentiment_neutral() {
        let score = score_text_sentiment("The cat sat on the mat");
        assert!((score - 0.0).abs() < 0.001);
    }

    #[test]
    fn authenticity_clean() {
        let comments = vec![
            VideoComment {
                author: "Alice".into(),
                text: "Great explanation of the topic, learned a lot!".into(),
                likes: 5,
                published: "1 day ago".into(),
                is_hearted: false,
                reply_count: 0,
            },
            VideoComment {
                author: "Bob".into(),
                text: "This really helped me understand the concept better.".into(),
                likes: 3,
                published: "2 days ago".into(),
                is_hearted: false,
                reply_count: 0,
            },
        ];
        let report = assess_authenticity(&comments);
        assert!(report.score > 0.7);
        assert!(report.bot_percentage < 0.1);
    }

    #[test]
    fn bot_detection() {
        let bot = VideoComment {
            author: "Bot".into(),
            text: "Check my channel for free gifts!".into(),
            likes: 0,
            published: "1h ago".into(),
            is_hearted: false,
            reply_count: 0,
        };
        assert!(is_likely_bot(&bot));
    }

    #[test]
    fn spam_detection() {
        assert!(is_likely_spam("Click here http://spam.com http://scam.com"));
        assert!(is_likely_spam("Check my profile for free gift cards"));
        assert!(!is_likely_spam("Great video, really enjoyed it"));
    }

    #[test]
    fn topic_extraction_basic() {
        let comments = vec![
            VideoComment {
                author: "A".into(),
                text: "Rust async programming is great".into(),
                likes: 1,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
            VideoComment {
                author: "B".into(),
                text: "I love async programming in Rust".into(),
                likes: 2,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
            VideoComment {
                author: "C".into(),
                text: "async programming is the future".into(),
                likes: 0,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
        ];
        let topics = extract_topics(&comments);
        assert!(!topics.is_empty());
        // "async programming" should be a top topic
        assert!(topics.iter().any(|t| t.topic.contains("async")));
    }

    #[test]
    fn informative_comments_sorted() {
        let comments = vec![
            VideoComment { author: "Short".into(), text: "Nice".into(), likes: 0, published: "".into(), is_hearted: false, reply_count: 0 },
            VideoComment { author: "Long".into(), text: "This is a really detailed and informative comment about the subject matter discussed in the video, covering multiple aspects and providing additional context that viewers might find useful".into(), likes: 50, published: "".into(), is_hearted: true, reply_count: 5 },
        ];
        let scored = find_informative_comments(&comments);
        assert_eq!(scored[0].author, "Long");
    }

    #[test]
    fn bigram_jaccard_identical() {
        assert!((bigram_jaccard("hello world", "hello world") - 1.0).abs() < 0.001);
    }

    #[test]
    fn bigram_jaccard_different() {
        assert!(bigram_jaccard("hello world", "foo bar baz") < 0.3);
    }

    #[test]
    fn overall_sentiment_mixed() {
        let comments = vec![
            VideoComment {
                author: "A".into(),
                text: "amazing video".into(),
                likes: 0,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
            VideoComment {
                author: "B".into(),
                text: "terrible content".into(),
                likes: 0,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
            VideoComment {
                author: "C".into(),
                text: "watched the whole thing".into(),
                likes: 0,
                published: "".into(),
                is_hearted: false,
                reply_count: 0,
            },
        ];
        let sentiment = compute_overall_sentiment(&comments);
        assert!(sentiment.positive > 0.0);
        assert!(sentiment.negative > 0.0);
    }

    #[test]
    fn parse_invidious_comments_json() {
        let json = serde_json::json!({
            "comments": [
                {
                    "author": "User1",
                    "content": "Great video!",
                    "likeCount": 10,
                    "publishedText": "2 days ago"
                },
                {
                    "author": "User2",
                    "content": "Very helpful explanation",
                    "likeCount": 5,
                    "publishedText": "1 week ago"
                }
            ]
        });
        let comments = parse_invidious_comments(&json.to_string(), 100).unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].author, "User1");
        assert_eq!(comments[0].likes, 10);
    }
}
