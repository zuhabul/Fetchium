//! YouTube Intelligence System — data types.
//!
//! All structures for search, metadata, transcripts, comments, ranking,
//! intelligence, and pipeline results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Video Metadata ────────────────────────────────────────────

/// Full metadata for a YouTube video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub channel: ChannelInfo,
    pub duration_secs: u64,
    pub view_count: u64,
    pub like_count: u64,
    pub published: String,
    pub keywords: Vec<String>,
    pub chapters: Vec<Chapter>,
    pub links: Vec<DescriptionLink>,
    pub thumbnail_url: Option<String>,
    pub is_live: bool,
}

/// YouTube channel information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    pub name: String,
    pub id: String,
    pub subscriber_count: Option<u64>,
    pub verified: bool,
}

/// Video chapter (timestamp + title).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_secs: u64,
    pub end_secs: Option<u64>,
}

/// Link extracted from a video description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescriptionLink {
    pub url: String,
    pub domain: String,
    pub link_type: LinkType,
}

/// Classification of a link found in a description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Social,
    Article,
    Product,
    Code,
    Documentation,
    Other,
}

// ─── Channel Credibility ───────────────────────────────────────

/// Channel credibility scoring result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCredibility {
    pub score: f64,
    pub tier: CredibilityTier,
    pub factors: CredibilityFactors,
}

/// Subscriber-based credibility tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredibilityTier {
    Unknown,
    Emerging,
    Established,
    Authority,
    Mega,
}

/// Factors contributing to channel credibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredibilityFactors {
    pub subscriber_score: f64,
    pub consistency_score: f64,
    pub verified_bonus: f64,
}

// ─── Transcript ────────────────────────────────────────────────

/// Enhanced transcript with speaker detection and key moments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTranscript {
    pub video_id: String,
    pub language: String,
    pub entries: Vec<TranscriptEntry>,
    pub speakers: Vec<Speaker>,
    pub key_moments: Vec<KeyMoment>,
    /// Clean speech-only text (noise markers like [Music] removed).
    pub full_text: String,
    pub word_count: usize,
    pub source: TranscriptSource,
    /// Quality 0.0–1.0. <0.4 = garbled/wrong language; ≥0.8 = high quality.
    #[serde(default = "default_quality")]
    pub quality_score: f64,
}

fn default_quality() -> f64 {
    1.0
}

/// A single transcript entry (caption segment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub start_ms: u32,
    pub duration_ms: u32,
    pub text: String,
    pub speaker_id: Option<u32>,
}

/// Source of the transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptSource {
    YouTubeTimedtext,
    Invidious,
    Piped,
    YtDlp,
}

/// Detected speaker in transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub id: u32,
    pub label: String,
    pub segment_count: usize,
}

/// A key moment detected in the transcript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMoment {
    pub timestamp_ms: u32,
    pub moment_type: MomentType,
    pub text: String,
    pub importance: f64,
}

/// Type of key moment in a transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MomentType {
    TopicShift,
    KeyPoint,
    Definition,
    Example,
    Conclusion,
}

// ─── Comments ──────────────────────────────────────────────────

/// Comment from a YouTube video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoComment {
    pub author: String,
    pub text: String,
    pub likes: u32,
    pub published: String,
    pub is_hearted: bool,
    pub reply_count: u32,
}

/// Comment analysis result for a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentAnalysis {
    pub total_comments: usize,
    pub analyzed_comments: usize,
    pub overall_sentiment: SentimentScore,
    pub top_topics: Vec<TopicCluster>,
    pub authenticity: AuthenticityReport,
    pub informative_comments: Vec<ScoredComment>,
}

/// Sentiment score with breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentScore {
    pub positive: f64,
    pub negative: f64,
    pub neutral: f64,
    pub compound: f64,
}

/// A cluster of related comment topics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicCluster {
    pub topic: String,
    pub count: usize,
    pub representative: String,
}

/// Comment authenticity assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticityReport {
    pub score: f64,
    pub bot_percentage: f64,
    pub spam_percentage: f64,
    pub uniformity: f64,
}

/// A comment scored for informativeness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredComment {
    pub text: String,
    pub author: String,
    pub score: f64,
}

// ─── Ranking (VideoFusion) ─────────────────────────────────────

/// VideoFusion ranking result for a single video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRanking {
    pub video_id: String,
    pub title: String,
    pub final_score: f64,
    pub signals: VideoSignals,
    pub clickbait_score: f64,
    pub educational_score: f64,
}

/// 8-signal breakdown for VideoFusion ranking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSignals {
    pub relevance: f64,
    pub freshness: f64,
    pub authority: f64,
    pub engagement: f64,
    pub educational: f64,
    pub authenticity: f64,
    pub comment_quality: f64,
    pub depth: f64,
}

impl VideoSignals {
    /// Compute weighted final score using VideoFusion weights.
    pub fn weighted_score(&self) -> f64 {
        self.relevance * 0.25
            + self.freshness * 0.10
            + self.authority * 0.15
            + self.engagement * 0.10
            + self.educational * 0.15
            + self.authenticity * 0.10
            + self.comment_quality * 0.05
            + self.depth * 0.10
    }
}

// ─── Intelligence ──────────────────────────────────────────────

/// Fact extracted from a video for cross-checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoFact {
    pub claim: String,
    pub video_id: String,
    pub timestamp_ms: Option<u32>,
    pub confidence: f64,
}

/// Cross-video fact check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactCheckResult {
    pub claim: String,
    pub supporting: Vec<String>,
    pub contradicting: Vec<String>,
    pub consensus: FactConsensus,
}

/// Consensus level across videos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactConsensus {
    Confirmed,
    Disputed,
    Unverified,
    Contradicted,
}

/// A topic timeline entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub date: String,
    pub event: String,
    pub video_id: String,
    pub video_title: String,
}

/// Key concept extracted from videos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyConcept {
    pub term: String,
    pub definition: Option<String>,
    pub frequency: usize,
    pub source_videos: Vec<String>,
}

/// Learning path step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStep {
    pub order: usize,
    pub video_id: String,
    pub title: String,
    pub difficulty: DifficultyLevel,
    pub key_concepts: Vec<String>,
    pub estimated_minutes: u64,
}

/// Difficulty classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DifficultyLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

/// Teaching content generated from videos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingContent {
    pub summary: String,
    pub flashcards: Vec<Flashcard>,
    pub quiz_questions: Vec<QuizQuestion>,
}

/// A flashcard (term + definition).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub front: String,
    pub back: String,
}

/// A quiz question with options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

// ─── Search ────────────────────────────────────────────────────

/// A YouTube search result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeSearchResult {
    pub video_id: String,
    pub title: String,
    pub description: String,
    pub channel: String,
    pub duration_secs: u64,
    pub view_count: u64,
    pub published: String,
    pub thumbnail_url: Option<String>,
}

/// Source of the YouTube search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YouTubeSearchSource {
    Invidious,
    Piped,
    YtDlp,
}

// ─── Pipeline ──────────────────────────────────────────────────

/// Configuration for a YouTube intelligence pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubePipelineConfig {
    pub query: String,
    pub max_videos: usize,
    pub fetch_transcript: bool,
    pub fetch_comments: bool,
    pub fact_check: bool,
    pub build_timeline: bool,
    pub build_learning_path: bool,
    pub generate_teaching: bool,
    pub token_budget: usize,
}

impl Default for YouTubePipelineConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_videos: 5,
            fetch_transcript: true,
            fetch_comments: true,
            fact_check: false,
            build_timeline: false,
            build_learning_path: false,
            generate_teaching: false,
            token_budget: 8000,
        }
    }
}

/// Full pipeline result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubePipelineResult {
    pub query: String,
    pub videos: Vec<VideoAnalysis>,
    pub rankings: Vec<VideoRanking>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_checks: Option<Vec<FactCheckResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Vec<TimelineEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_path: Option<Vec<LearningStep>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teaching: Option<TeachingContent>,
    pub duration_ms: u64,
}

/// Analysis results for a single video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAnalysis {
    pub metadata: VideoMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<EnhancedTranscript>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<CommentAnalysis>,
    pub credibility: ChannelCredibility,
}

/// Invidious API instance list (verified February 2026).
///
/// Only 2 public clearnet instances remain due to YouTube's aggressive blocking.
/// inv.nadeko.net has API disabled; all others are defunct.
pub const INVIDIOUS_INSTANCES: &[&str] = &[
    "https://invidious.nerdvpn.de", // Most reliable, uses IP rotation
    "https://yewtu.be",              // Custom fork with YouTube workarounds
];

/// Piped API instance list (verified February 2026).
///
/// Piped is more stable than Invidious since it uses a different proxying architecture.
pub const PIPED_INSTANCES: &[&str] = &[
    "https://pipedapi.kavin.rocks",       // Official instance, most reliable
    "https://pipedapi-libre.kavin.rocks", // Same operator, no CDN
    "https://api.piped.yt",               // Germany, confirmed Feb 2026
    "https://pipedapi.ducks.party",       // Netherlands, confirmed Feb 2026
    "https://api.piped.private.coffee",   // Austria, confirmed Feb 2026
];

/// Agent-friendly YouTube result for JSON output (MCP/API).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentYouTubeResult {
    pub query: String,
    pub videos: Vec<VideoRanking>,
    pub total_analyzed: usize,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fact_checks: Option<Vec<FactCheckResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub learning_path: Option<Vec<LearningStep>>,
}

/// Convenience type for cross-video claim matching.
#[derive(Debug, Clone)]
pub struct ClaimMatch {
    pub claim: String,
    pub video_ids: Vec<String>,
    pub similarity: f64,
    pub negated: bool,
}

/// Stopwords for text analysis (English).
pub static STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "is", "was", "are", "were", "be", "been", "being", "have", "has", "had", "do", "does",
    "did", "will", "would", "could", "should", "may", "might", "can", "shall", "this", "that",
    "these", "those", "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us",
    "them", "my", "your", "his", "its", "our", "their", "not", "no", "so", "if", "then", "than",
    "when", "what", "which", "who", "how", "all", "each", "every", "both", "few", "more", "most",
    "other", "some", "such", "only", "own", "same", "just", "very", "about", "up", "out", "into",
    "over", "after", "before", "between", "under", "also", "there", "here", "as",
];

/// AFINN-style sentiment word scores (subset for common words).
pub static SENTIMENT_LEXICON: &[(&str, i32)] = &[
    ("amazing", 3),
    ("awesome", 3),
    ("beautiful", 3),
    ("best", 3),
    ("brilliant", 3),
    ("excellent", 3),
    ("fantastic", 3),
    ("great", 3),
    ("love", 3),
    ("outstanding", 3),
    ("perfect", 3),
    ("superb", 3),
    ("wonderful", 3),
    ("good", 2),
    ("nice", 2),
    ("helpful", 2),
    ("useful", 2),
    ("informative", 2),
    ("clear", 2),
    ("interesting", 2),
    ("enjoy", 2),
    ("like", 1),
    ("okay", 1),
    ("fine", 1),
    ("decent", 1),
    ("bad", -2),
    ("boring", -2),
    ("confusing", -2),
    ("disappointing", -2),
    ("poor", -2),
    ("terrible", -3),
    ("worst", -3),
    ("awful", -3),
    ("horrible", -3),
    ("waste", -2),
    ("useless", -2),
    ("wrong", -2),
    ("hate", -3),
    ("stupid", -3),
    ("garbage", -3),
    ("trash", -3),
    ("misleading", -2),
    ("clickbait", -2),
    ("scam", -3),
    ("fake", -3),
    ("spam", -3),
];

/// Negation words that flip sentiment.
pub static NEGATION_WORDS: &[&str] = &[
    "not",
    "no",
    "never",
    "neither",
    "nobody",
    "nothing",
    "nowhere",
    "nor",
    "cannot",
    "can't",
    "don't",
    "doesn't",
    "didn't",
    "won't",
    "wouldn't",
    "shouldn't",
    "isn't",
    "aren't",
    "wasn't",
    "weren't",
    "haven't",
    "hasn't",
    "hadn't",
];

/// Intensifier words that amplify sentiment.
pub static INTENSIFIERS: &[(&str, f64)] = &[
    ("very", 1.5),
    ("extremely", 2.0),
    ("absolutely", 2.0),
    ("totally", 1.5),
    ("really", 1.3),
    ("quite", 1.2),
    ("super", 1.5),
    ("incredibly", 1.8),
    ("highly", 1.5),
    ("so", 1.3),
];

/// Clickbait trigger phrases.
pub static CLICKBAIT_PHRASES: &[&str] = &[
    "you won't believe",
    "shocking",
    "mind blowing",
    "insane",
    "must watch",
    "life changing",
    "secret",
    "they don't want you to know",
    "gone wrong",
    "exposed",
    "no one tells you",
    "hack",
    "this one trick",
    "number 1",
    "#1",
];

/// Transitional phrases that indicate key moments in transcripts.
pub static TRANSITION_PHRASES: &[&str] = &[
    "the key takeaway",
    "in summary",
    "to summarize",
    "the main point",
    "let me explain",
    "here's the thing",
    "the important thing",
    "what this means",
    "in conclusion",
    "first of all",
    "secondly",
    "finally",
    "moving on",
    "next up",
    "now let's",
    "the bottom line",
    "the reason",
    "for example",
    "let's look at",
    "as you can see",
    "what we found",
    "the result",
    "interestingly",
];

/// Definition patterns in transcripts.
pub static DEFINITION_PATTERNS: &[&str] = &[
    " is defined as ",
    " means ",
    " refers to ",
    " is essentially ",
    " is basically ",
    " is a ",
    " is when ",
    " is the process of ",
    " can be described as ",
];

/// Bigram set for similarity computation.
pub type BigramSet = HashMap<String, usize>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_signals_weighted_score() {
        let signals = VideoSignals {
            relevance: 1.0,
            freshness: 1.0,
            authority: 1.0,
            engagement: 1.0,
            educational: 1.0,
            authenticity: 1.0,
            comment_quality: 1.0,
            depth: 1.0,
        };
        let score = signals.weighted_score();
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn video_signals_partial() {
        let signals = VideoSignals {
            relevance: 0.8,
            freshness: 0.5,
            authority: 0.7,
            engagement: 0.6,
            educational: 0.9,
            authenticity: 0.4,
            comment_quality: 0.3,
            depth: 0.5,
        };
        let score = signals.weighted_score();
        assert!(score > 0.0 && score < 1.0);
    }

    #[test]
    fn difficulty_ordering() {
        assert!(DifficultyLevel::Beginner < DifficultyLevel::Intermediate);
        assert!(DifficultyLevel::Intermediate < DifficultyLevel::Advanced);
        assert!(DifficultyLevel::Advanced < DifficultyLevel::Expert);
    }

    #[test]
    fn pipeline_config_defaults() {
        let cfg = YouTubePipelineConfig::default();
        assert_eq!(cfg.max_videos, 5);
        assert!(cfg.fetch_transcript);
        assert!(!cfg.fact_check);
    }

    #[test]
    fn metadata_roundtrip() {
        let meta = VideoMetadata {
            video_id: "abc123".into(),
            title: "Test Video".into(),
            description: "A test".into(),
            channel: ChannelInfo {
                name: "TestCh".into(),
                id: "UC123".into(),
                subscriber_count: Some(1000),
                verified: false,
            },
            duration_secs: 600,
            view_count: 50000,
            like_count: 2000,
            published: "2025-01-01".into(),
            keywords: vec!["rust".into()],
            chapters: vec![],
            links: vec![],
            thumbnail_url: None,
            is_live: false,
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: VideoMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.video_id, "abc123");
        assert_eq!(back.view_count, 50000);
    }

    #[test]
    fn sentiment_lexicon_sorted_check() {
        // Verify lexicon has both positive and negative words
        let positive: Vec<_> = SENTIMENT_LEXICON.iter().filter(|(_, s)| *s > 0).collect();
        let negative: Vec<_> = SENTIMENT_LEXICON.iter().filter(|(_, s)| *s < 0).collect();
        assert!(!positive.is_empty());
        assert!(!negative.is_empty());
    }

    #[test]
    fn invidious_instances_not_empty() {
        assert!(!INVIDIOUS_INSTANCES.is_empty());
        assert!(INVIDIOUS_INSTANCES[0].starts_with("https://"));
    }
}
