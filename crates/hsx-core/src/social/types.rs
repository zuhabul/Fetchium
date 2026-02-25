//! Shared social media intelligence types.
//!
//! Used across Twitter/X, Reddit, TikTok, and the unified research engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Core Post Type ──────────────────────────────────────────────

/// Platform where a post originates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SocialPlatform {
    Twitter,
    Reddit,
    TikTok,
    HackerNews,
    YouTube,
    Facebook,
}

impl std::fmt::Display for SocialPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Twitter => write!(f, "Twitter/X"),
            Self::Reddit => write!(f, "Reddit"),
            Self::TikTok => write!(f, "TikTok"),
            Self::HackerNews => write!(f, "Hacker News"),
            Self::YouTube => write!(f, "YouTube"),
            Self::Facebook => write!(f, "Facebook"),
        }
    }
}

/// A normalised social media post, usable across all platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPost {
    pub platform: SocialPlatform,
    pub id: String,
    pub url: String,
    pub author: String,
    pub content: String,
    pub published: String,
    pub engagement: EngagementMetrics,
    pub media: Vec<MediaAttachment>,
    pub hashtags: Vec<String>,
    pub mentions: Vec<String>,
    pub sentiment: f64,    // –1.0 (negative) … +1.0 (positive)
    pub authenticity: f64, // 0.0 … 1.0
}

/// Platform-agnostic engagement metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngagementMetrics {
    pub likes: u64,
    pub shares: u64, // retweets / reposts / shares
    pub comments: u64,
    pub views: Option<u64>,
    pub score: f64, // normalised 0.0 … 1.0
}

impl EngagementMetrics {
    /// Compute normalised engagement score (log-scaled).
    pub fn compute_score(&mut self) {
        let total = (self.likes + self.shares * 2 + self.comments * 3) as f64;
        self.score = if total > 0.0 {
            (total.ln() / 20.0).min(1.0)
        } else {
            0.0
        };
    }
}

/// Media attached to a social post.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub media_type: MediaType,
    pub url: String,
    pub alt_text: Option<String>,
}

/// Type of media in a post.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Video,
    Gif,
    Poll,
    Link,
}

// ─── Trend Types ─────────────────────────────────────────────────

/// A trending topic on a social platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendItem {
    pub platform: SocialPlatform,
    pub topic: String,
    pub post_count: u64,
    pub velocity: f64, // posts per hour (estimated)
    pub sentiment: f64,
    pub region: Option<String>,
    pub related_topics: Vec<String>,
    pub sample_posts: Vec<SocialPost>,
}

/// Cross-platform trending topic (merged from all platforms).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformTrend {
    pub topic: String,
    pub platforms: Vec<SocialPlatform>,
    pub total_engagement: u64,
    pub velocity: f64,
    pub sentiment: f64,
    pub is_viral: bool,
    pub content_ideas: Vec<ContentIdea>,
    pub sample_posts: Vec<SocialPost>,
}

/// AI-generated content idea derived from trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentIdea {
    pub platform: SocialPlatform,
    pub format: ContentFormat,
    pub title: String,
    pub hook: String,
    pub key_points: Vec<String>,
    pub viral_potential: f64,
}

/// Content format suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentFormat {
    Thread,     // Twitter/X thread
    ShortVideo, // TikTok / YouTube Shorts
    LongVideo,  // YouTube
    Post,       // Reddit / LinkedIn post
    Thread60s,  // Quick take thread
    Infographic,
    Tutorial,
    Listicle,
    Debate,
}

impl std::fmt::Display for ContentFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Thread => write!(f, "Twitter Thread"),
            Self::ShortVideo => write!(f, "Short Video (TikTok/Shorts)"),
            Self::LongVideo => write!(f, "Long-form Video"),
            Self::Post => write!(f, "Reddit/LinkedIn Post"),
            Self::Thread60s => write!(f, "60-Second Thread"),
            Self::Infographic => write!(f, "Infographic"),
            Self::Tutorial => write!(f, "Tutorial"),
            Self::Listicle => write!(f, "Listicle"),
            Self::Debate => write!(f, "Debate / Hot Take"),
        }
    }
}

// ─── Analysis Types ──────────────────────────────────────────────

/// Sentiment analysis for a set of posts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SentimentBreakdown {
    pub positive_pct: f64,
    pub negative_pct: f64,
    pub neutral_pct: f64,
    pub compound: f64,
}

/// Author profile (platform-agnostic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorProfile {
    pub username: String,
    pub display_name: String,
    pub follower_count: Option<u64>,
    pub verified: bool,
    pub credibility_score: f64,
    pub platform: SocialPlatform,
}

/// Platform-level statistics for a search/trend run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStats {
    pub platform: SocialPlatform,
    pub posts_analyzed: usize,
    pub top_authors: Vec<AuthorProfile>,
    pub top_hashtags: Vec<(String, usize)>,
    pub sentiment: SentimentBreakdown,
    pub avg_engagement: f64,
}

// ─── Pipeline Config ─────────────────────────────────────────────

/// Configuration for the unified social research pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialPipelineConfig {
    pub query: String,
    pub platforms: Vec<SocialPlatform>,
    pub max_posts_per_platform: usize,
    pub include_trends: bool,
    pub generate_ideas: bool,
    pub deep_analysis: bool,
    pub timeout_secs: u64,
}

impl Default for SocialPipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            platforms: vec![
                SocialPlatform::Twitter,
                SocialPlatform::Reddit,
                SocialPlatform::TikTok,
                SocialPlatform::HackerNews,
                SocialPlatform::YouTube,
                SocialPlatform::Facebook,
            ],
            max_posts_per_platform: 20,
            include_trends: true,
            generate_ideas: true,
            deep_analysis: false,
            timeout_secs: 30,
        }
    }
}

/// Full result from the unified social research pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialResearchResult {
    pub query: String,
    pub platform_results: HashMap<String, PlatformResult>,
    pub cross_platform_trends: Vec<CrossPlatformTrend>,
    pub top_posts: Vec<SocialPost>,
    pub content_ideas: Vec<ContentIdea>,
    pub summary: String,
    pub duration_ms: u64,
}

/// Per-platform results in a unified research run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformResult {
    pub platform: SocialPlatform,
    pub posts: Vec<SocialPost>,
    pub trends: Vec<TrendItem>,
    pub stats: PlatformStats,
}

// ─── Viral Content Analysis ───────────────────────────────────────

/// Viral content scoring factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViralScore {
    pub overall: f64,
    pub engagement_velocity: f64,
    pub emotional_resonance: f64,
    pub shareability: f64,
    pub novelty: f64,
    pub trend_alignment: f64,
}

impl ViralScore {
    /// Compute overall viral score from factors.
    pub fn compute(
        engagement_velocity: f64,
        emotional_resonance: f64,
        shareability: f64,
        novelty: f64,
        trend_alignment: f64,
    ) -> Self {
        let overall = engagement_velocity * 0.30
            + emotional_resonance * 0.25
            + shareability * 0.20
            + novelty * 0.15
            + trend_alignment * 0.10;
        Self {
            overall,
            engagement_velocity,
            emotional_resonance,
            shareability,
            novelty,
            trend_alignment,
        }
    }
}

// ─── Shared Sentiment ────────────────────────────────────────────

/// Score sentiment of arbitrary text using AFINN-style lexicon + negation.
pub fn score_sentiment(text: &str) -> f64 {
    use crate::youtube::types::{NEGATION_WORDS, SENTIMENT_LEXICON};
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let mut score = 0.0f64;
    let mut negated = false;

    for (i, word) in words.iter().enumerate() {
        if NEGATION_WORDS.contains(word) {
            negated = true;
            continue;
        }
        // reset negation after 3 words
        if i > 0 && negated {
            let prev = i.saturating_sub(4);
            let neg_window = &words[prev..i];
            if !neg_window.iter().any(|w| NEGATION_WORDS.contains(w)) {
                negated = false;
            }
        }
        if let Some(&(_, s)) = SENTIMENT_LEXICON.iter().find(|(w, _)| *w == *word) {
            let raw = s as f64 / 3.0; // normalise to –1..+1
            score += if negated { -raw } else { raw };
        }
    }
    score.clamp(-1.0, 1.0)
}

/// Build bigram set from text for similarity computation.
pub fn bigrams(text: &str) -> HashMap<String, usize> {
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    let mut map = HashMap::new();
    for w in chars.windows(2) {
        let bg: String = w.iter().collect();
        *map.entry(bg).or_insert(0) += 1;
    }
    map
}

/// Jaccard similarity between two bigram sets.
pub fn jaccard(a: &HashMap<String, usize>, b: &HashMap<String, usize>) -> f64 {
    let mut intersection = 0usize;
    let mut union_size = 0usize;
    for (k, av) in a {
        let bv = b.get(k).copied().unwrap_or(0);
        intersection += av.min(&bv);
        union_size += av.max(&bv);
    }
    for (k, bv) in b {
        if !a.contains_key(k) {
            union_size += bv;
        }
    }
    if union_size == 0 {
        return 0.0;
    }
    intersection as f64 / union_size as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engagement_score_positive() {
        let mut e = EngagementMetrics {
            likes: 1000,
            shares: 500,
            comments: 200,
            views: Some(50000),
            score: 0.0,
        };
        e.compute_score();
        assert!(e.score > 0.0 && e.score <= 1.0);
    }

    #[test]
    fn engagement_score_zero() {
        let mut e = EngagementMetrics::default();
        e.compute_score();
        assert_eq!(e.score, 0.0);
    }

    #[test]
    fn viral_score_sums_to_one() {
        let v = ViralScore::compute(1.0, 1.0, 1.0, 1.0, 1.0);
        assert!((v.overall - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sentiment_positive_text() {
        let s = score_sentiment("amazing excellent brilliant fantastic");
        assert!(s > 0.0);
    }

    #[test]
    fn sentiment_negative_text() {
        let s = score_sentiment("terrible awful horrible garbage");
        assert!(s < 0.0);
    }

    #[test]
    fn jaccard_identical() {
        let a = bigrams("hello");
        let b = bigrams("hello");
        assert!((jaccard(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn jaccard_disjoint() {
        let a = bigrams("abc");
        let b = bigrams("xyz");
        assert_eq!(jaccard(&a, &b), 0.0);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let a = bigrams("hello world");
        let b = bigrams("hello there");
        let sim = jaccard(&a, &b);
        assert!(sim > 0.0 && sim < 1.0, "partial overlap sim={sim}");
    }

    #[test]
    fn jaccard_empty_both_zero() {
        let a = bigrams("");
        let b = bigrams("");
        assert_eq!(jaccard(&a, &b), 0.0);
    }

    #[test]
    fn viral_score_weights_sum_to_one() {
        let v = ViralScore::compute(1.0, 1.0, 1.0, 1.0, 1.0);
        assert!((v.overall - 1.0).abs() < 1e-10, "overall={}", v.overall);
    }

    #[test]
    fn viral_score_zero_inputs() {
        let v = ViralScore::compute(0.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(v.overall, 0.0);
    }

    #[test]
    fn viral_score_partial_velocity_only() {
        let v = ViralScore::compute(1.0, 0.0, 0.0, 0.0, 0.0);
        // velocity weight is 0.30
        assert!(
            (v.overall - 0.30).abs() < 1e-10,
            "expected 0.30, got {}",
            v.overall
        );
    }

    #[test]
    fn sentiment_negation_reduces_score() {
        let good = score_sentiment("good excellent");
        let not_good = score_sentiment("not good");
        assert!(
            not_good < good,
            "negation should reduce: {not_good} vs {good}"
        );
    }

    #[test]
    fn sentiment_clamped_range() {
        let s = score_sentiment(
            "amazing brilliant excellent wonderful perfect best greatest fantastic outstanding",
        );
        assert!(s >= -1.0 && s <= 1.0, "out of range: {s}");
    }

    #[test]
    fn social_platform_display() {
        assert_eq!(SocialPlatform::Twitter.to_string(), "Twitter/X");
        assert_eq!(SocialPlatform::HackerNews.to_string(), "Hacker News");
        assert_eq!(SocialPlatform::Reddit.to_string(), "Reddit");
        assert_eq!(SocialPlatform::TikTok.to_string(), "TikTok");
        assert_eq!(SocialPlatform::YouTube.to_string(), "YouTube");
    }

    #[test]
    fn engagement_score_capped_at_one() {
        let mut e = EngagementMetrics {
            likes: u64::MAX / 4,
            shares: u64::MAX / 8,
            comments: 0,
            views: None,
            score: 0.0,
        };
        e.compute_score();
        assert!(e.score <= 1.0, "score must not exceed 1.0: {}", e.score);
    }

    #[test]
    fn content_format_display_variants() {
        assert!(ContentFormat::Thread.to_string().contains("Thread"));
        assert!(ContentFormat::ShortVideo.to_string().contains("Short"));
        assert!(ContentFormat::Tutorial.to_string().contains("Tutorial"));
        assert!(ContentFormat::Listicle.to_string().contains("Listicle"));
    }
}
