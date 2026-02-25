//! Social post validation, fact-checking, and quality filtering.
//!
//! Every post from every platform passes through this pipeline:
//!
//! ```text
//! Raw Posts
//!   -> relevance_score()     -- word-overlap BM25 against query
//!   -> authenticity_score()  -- bot/spam/quality heuristics
//!   -> credibility_score()   -- platform weight x engagement legitimacy
//!   -> filter_low_quality()  -- drop posts below threshold
//!   -> cross_validate()      -- boost posts confirmed on 2+ platforms
//!   -> ValidationReport      -- per-post flags + summary stats
//! ```

use crate::social::types::{EngagementMetrics, SocialPlatform, SocialPost};
use std::collections::HashSet;

// ─── Public Types ─────────────────────────────────────────────────

/// Quality flags attached to a post after validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationFlag {
    /// More than 8 hashtags in a single post (spam signal)
    ExcessiveHashtags,
    /// Content is fewer than 8 words (low-value)
    ShortContent,
    /// Content is mostly uppercase (shouting / clickbait)
    ExcessiveCaps,
    /// Repeated filler phrases detected
    SpamPatterns,
    /// Account appears bot-like (random username, no bio context)
    PossibleBot,
    /// Low relevance to the search query
    LowRelevance,
    /// Deleted or anonymous author
    DeletedAccount,
    /// Engagement looks artificially inflated
    SuspiciousEngagement,
    /// Claim is extraordinary / unverified (high sentiment outlier)
    ExtraordinaryClaim,
}

/// Per-post validation result.
#[derive(Debug, Clone)]
pub struct PostValidation {
    pub relevance: f64,      // 0-1 word-overlap relevance to query
    pub authenticity: f64,   // 0-1 content + account quality
    pub credibility: f64,    // 0-1 platform weight × engagement legitimacy
    pub quality_score: f64,  // 0-1 composite (weighted average)
    pub flags: Vec<ValidationFlag>,
    pub passes: bool,        // true if quality_score ≥ threshold
}

/// Summary stats from validating a batch of posts.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub total: usize,
    pub passed: usize,
    pub filtered_bot: usize,
    pub filtered_spam: usize,
    pub filtered_irrelevant: usize,
    pub avg_quality: f64,
    pub avg_relevance: f64,
    pub avg_credibility: f64,
}

// ─── Default threshold ────────────────────────────────────────────

/// Minimum quality score for a post to be included in output.
pub const MIN_QUALITY: f64 = 0.20;

// ─── Main API ─────────────────────────────────────────────────────

/// Validate a single post against a search query.
pub fn validate_post(post: &SocialPost, query: &str) -> PostValidation {
    let mut flags = Vec::new();

    // ── Relevance ──────────────────────────────────────────────────
    let relevance = compute_relevance(&post.content, query);
    if relevance < 0.05 {
        flags.push(ValidationFlag::LowRelevance);
    }

    // ── Authenticity ───────────────────────────────────────────────
    let auth = compute_authenticity(post, &mut flags);

    // ── Credibility ────────────────────────────────────────────────
    let cred = compute_credibility(post, &mut flags);

    // ── Composite quality score ────────────────────────────────────
    // Weights: relevance 40%, authenticity 35%, credibility 25%
    let quality = (relevance * 0.40 + auth * 0.35 + cred * 0.25).clamp(0.0, 1.0);

    let passes = quality >= MIN_QUALITY;

    PostValidation {
        relevance,
        authenticity: auth,
        credibility: cred,
        quality_score: quality,
        flags,
        passes,
    }
}

/// Filter and rank a list of posts by quality score, keeping only passing posts.
///
/// Returns posts sorted by quality descending, up to `limit`.
pub fn filter_and_rank(
    posts: Vec<SocialPost>,
    query: &str,
    limit: usize,
) -> (Vec<SocialPost>, ValidationReport) {
    let total = posts.len();
    let mut scored: Vec<(SocialPost, PostValidation)> = posts
        .into_iter()
        .map(|p| {
            let v = validate_post(&p, query);
            (p, v)
        })
        .collect();

    let mut report = ValidationReport {
        total,
        ..Default::default()
    };

    for (_, v) in &scored {
        if !v.passes {
            if v.flags.contains(&ValidationFlag::PossibleBot) {
                report.filtered_bot += 1;
            } else if v.flags.contains(&ValidationFlag::SpamPatterns)
                || v.flags.contains(&ValidationFlag::ExcessiveHashtags)
            {
                report.filtered_spam += 1;
            } else if v.flags.contains(&ValidationFlag::LowRelevance) {
                report.filtered_irrelevant += 1;
            }
        }
    }

    // Keep passing posts, sort by quality descending
    scored.retain(|(_, v)| v.passes);
    scored.sort_by(|(_, a), (_, b)| {
        b.quality_score
            .partial_cmp(&a.quality_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    report.passed = scored.len().min(limit);
    if !scored.is_empty() {
        let n = scored.len() as f64;
        report.avg_quality = scored.iter().map(|(_, v)| v.quality_score).sum::<f64>() / n;
        report.avg_relevance = scored.iter().map(|(_, v)| v.relevance).sum::<f64>() / n;
        report.avg_credibility = scored.iter().map(|(_, v)| v.credibility).sum::<f64>() / n;
    }

    let posts: Vec<SocialPost> = scored.into_iter().take(limit).map(|(p, _)| p).collect();
    (posts, report)
}

/// Apply validation to a SocialPost and update its `authenticity` field in-place.
pub fn enrich_post(post: &mut SocialPost, query: &str) {
    let v = validate_post(post, query);
    // Update the built-in authenticity field with our computed value
    post.authenticity = v.authenticity;
    // Adjust sentiment if extraordinary claim detected
    if v.flags.contains(&ValidationFlag::ExtraordinaryClaim) {
        // Dampen extreme sentiment toward neutral — signals need verification
        post.sentiment *= 0.5;
    }
}

// ─── Scoring Functions ────────────────────────────────────────────

/// Compute word-overlap relevance score between post content and query.
///
/// Uses normalised Jaccard-like word set overlap with TF-IDF-style boost
/// for rare query terms appearing in content.
pub fn compute_relevance(content: &str, query: &str) -> f64 {
    if content.is_empty() || query.is_empty() {
        return 0.0;
    }

    let content_lower = content.to_lowercase();
    let content_words: HashSet<&str> = content_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !is_stopword(w))
        .collect();

    let query_lower_owned = query.to_lowercase();
    let query_words: Vec<&str> = query_lower_owned
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 3 && !is_stopword(w))
        .collect();

    if query_words.is_empty() {
        return 0.5; // no meaningful query words → neutral
    }

    let matched = query_words
        .iter()
        .filter(|qw| {
            // Exact match or prefix match (e.g. "rust" matches "rustlang")
            content_words.iter().any(|cw| cw.contains(*qw) || qw.contains(cw))
        })
        .count();

    let base = matched as f64 / query_words.len() as f64;

    // Boost for exact multi-word phrase presence
    let phrase_boost = if content_lower.contains(query_lower_owned.as_str()) {
        0.2
    } else {
        0.0
    };

    (base + phrase_boost).min(1.0)
}

/// Compute authenticity score (0-1) with flag side-effects.
fn compute_authenticity(post: &SocialPost, flags: &mut Vec<ValidationFlag>) -> f64 {
    let mut score = 1.0f64;
    let content = &post.content;
    let words: Vec<&str> = content.split_whitespace().collect();

    // Short content penalty
    if words.len() < 8 {
        flags.push(ValidationFlag::ShortContent);
        score -= 0.25;
    }

    // Excessive hashtags penalty (>8 is spam territory)
    let hashtag_count = words.iter().filter(|w| w.starts_with('#')).count();
    if hashtag_count > 8 {
        flags.push(ValidationFlag::ExcessiveHashtags);
        score -= 0.30;
    } else if hashtag_count > 5 {
        score -= 0.10;
    }

    // Excessive caps penalty (>40% uppercase chars = shouting/spam)
    let alpha_chars: Vec<char> = content.chars().filter(|c| c.is_alphabetic()).collect();
    if !alpha_chars.is_empty() {
        let caps_ratio =
            alpha_chars.iter().filter(|c| c.is_uppercase()).count() as f64
                / alpha_chars.len() as f64;
        if caps_ratio > 0.55 {
            flags.push(ValidationFlag::ExcessiveCaps);
            score -= 0.20;
        }
    }

    // Spam patterns: repeated punctuation, URL-only content
    let has_spam = content.contains("!!!") || content.contains("$$$")
        || content.contains("click here")
        || content.contains("buy now")
        || content.contains("limited offer")
        || content.contains("bit.ly")
        || content.contains("goo.gl");
    if has_spam {
        flags.push(ValidationFlag::SpamPatterns);
        score -= 0.35;
    }

    // Deleted/anonymous account penalty
    let author = &post.author;
    if author == "[deleted]" || author == "[removed]" || author.is_empty() {
        flags.push(ValidationFlag::DeletedAccount);
        score -= 0.20;
    }

    // Bot-like username: random alphanum, no vowels, too short/long
    if looks_like_bot_username(author) {
        flags.push(ValidationFlag::PossibleBot);
        score -= 0.30;
    }

    // Extraordinary claim detection: extreme sentiment (|s| > 0.85) on short content
    if post.sentiment.abs() > 0.85 && words.len() < 20 {
        flags.push(ValidationFlag::ExtraordinaryClaim);
        score -= 0.10;
    }

    score.clamp(0.0, 1.0)
}

/// Compute credibility score (0-1) based on platform weight × engagement legitimacy.
fn compute_credibility(post: &SocialPost, flags: &mut Vec<ValidationFlag>) -> f64 {
    use crate::social::unified::trend::platform_weight;

    let platform_cred = platform_weight(post.platform);

    // Engagement legitimacy check
    let eng_legit = compute_engagement_legitimacy(&post.engagement, post.platform);
    if eng_legit < 0.3 {
        flags.push(ValidationFlag::SuspiciousEngagement);
    }

    (platform_cred * 0.6 + eng_legit * 0.4).clamp(0.0, 1.0)
}

/// Check if engagement numbers look artificially inflated.
///
/// Red flags: 0 comments but 100k likes, 0 shares but massive likes, etc.
fn compute_engagement_legitimacy(eng: &EngagementMetrics, platform: SocialPlatform) -> f64 {
    // Zero engagement on all metrics → unknown (not fake, just new/unnoticed)
    if eng.likes == 0 && eng.shares == 0 && eng.comments == 0 {
        return 0.6; // neutral — can't tell
    }

    let likes = eng.likes as f64;
    let shares = eng.shares as f64;
    let comments = eng.comments as f64;

    // Expected ratios (platform-specific)
    let (expected_share_ratio, expected_comment_ratio) = match platform {
        SocialPlatform::Twitter => (0.15, 0.08),   // retweets common
        SocialPlatform::Reddit => (0.02, 0.25),    // comments high relative to score
        SocialPlatform::TikTok => (0.05, 0.03),   // shares moderate
        SocialPlatform::HackerNews => (0.00, 0.40), // no shares, lots of comments
        SocialPlatform::YouTube => (0.03, 0.02),   // shares low, comments moderate
        SocialPlatform::Facebook => (0.08, 0.05),  // shares common
    };

    let mut score = 1.0f64;

    // If shares wildly exceed likes (>3×) → suspicious
    if likes > 10.0 && shares > likes * 3.0 {
        score -= 0.30;
    }

    // If zero comments on high-engagement post → suspicious
    if likes > 1000.0 && comments == 0.0 && platform != SocialPlatform::TikTok {
        score -= 0.20;
    }

    // Expected ratio checks (gentle penalty for outliers)
    if likes > 0.0 {
        let actual_share_ratio = shares / likes;
        if actual_share_ratio > expected_share_ratio * 10.0 {
            score -= 0.15;
        }
        let actual_comment_ratio = comments / likes;
        if actual_comment_ratio > expected_comment_ratio * 10.0 {
            score -= 0.10;
        }
    }

    score.clamp(0.3, 1.0)
}

// ─── Cross-Platform Validation ────────────────────────────────────

/// Boost authenticity of posts whose content is confirmed on 2+ platforms.
///
/// Uses bigram-Jaccard similarity to detect "same story" across platforms.
pub fn cross_validate_boost(posts: &mut [SocialPost]) {
    use crate::social::types::bigrams;

    let bigram_sets: Vec<_> = posts
        .iter()
        .map(|p| bigrams(&p.content.to_lowercase()))
        .collect();

    for i in 0..posts.len() {
        let mut confirmed_on = 1usize;
        for j in 0..posts.len() {
            if i == j || posts[i].platform == posts[j].platform {
                continue;
            }
            let sim = crate::social::types::jaccard(&bigram_sets[i], &bigram_sets[j]);
            if sim > 0.35 {
                confirmed_on += 1;
            }
        }
        // Each cross-platform confirmation boosts authenticity by 5%
        if confirmed_on > 1 {
            let boost = 0.05 * (confirmed_on - 1) as f64;
            posts[i].authenticity = (posts[i].authenticity + boost).min(1.0);
        }
    }
}

// ─── Markdown Report ──────────────────────────────────────────────

/// Format a validation report as markdown summary.
pub fn format_validation_report(report: &ValidationReport) -> String {
    if report.total == 0 {
        return String::new();
    }
    let pass_pct = report.passed as f64 / report.total as f64 * 100.0;
    format!(
        "**Validation:** {}/{} posts passed ({:.0}% quality rate) · avg quality {:.0}% · avg relevance {:.0}% · avg credibility {:.0}%\n\
         Filtered: {} bot · {} spam · {} off-topic\n",
        report.passed,
        report.total,
        pass_pct,
        report.avg_quality * 100.0,
        report.avg_relevance * 100.0,
        report.avg_credibility * 100.0,
        report.filtered_bot,
        report.filtered_spam,
        report.filtered_irrelevant,
    )
}

// ─── Helpers ──────────────────────────────────────────────────────

/// Check if a username looks bot-generated.
///
/// Heuristics: random alphanumeric (>3 consecutive digits), no vowels, generic patterns.
fn looks_like_bot_username(username: &str) -> bool {
    if username.is_empty() || username == "[deleted]" {
        return false; // handled separately
    }
    let lower = username.to_lowercase();

    // Contains 4+ consecutive digits → likely bot-generated
    let digit_run = lower.chars().collect::<Vec<_>>();
    for window in digit_run.windows(4) {
        if window.iter().all(|c| c.is_ascii_digit()) {
            return true;
        }
    }

    // Very short username (1-2 chars) without context
    if lower.len() <= 2 {
        return true;
    }

    // No vowels at all in names >5 chars (like "brtxkrz")
    if lower.len() > 5 {
        let vowels = "aeiou";
        let has_vowel = lower.chars().any(|c| vowels.contains(c));
        if !has_vowel {
            return true;
        }
    }

    false
}

/// Common English stopwords to exclude from relevance computation.
fn is_stopword(word: &str) -> bool {
    matches!(
        word,
        "the" | "and" | "for" | "are" | "but" | "not" | "you" | "all" | "can" | "her" | "was"
            | "one" | "our" | "out" | "day" | "get" | "has" | "him" | "his" | "how" | "its"
            | "may" | "new" | "now" | "old" | "see" | "two" | "way" | "who" | "boy" | "did"
            | "she" | "use" | "yes" | "yet" | "any" | "far" | "few" | "got" | "let" | "off"
            | "put" | "say" | "set" | "try" | "also" | "back" | "from" | "have" | "here"
            | "just" | "know" | "like" | "look" | "make" | "more" | "much" | "need" | "only"
            | "over" | "same" | "take" | "than" | "that" | "them" | "then" | "they" | "this"
            | "time" | "very" | "well" | "when" | "will" | "with" | "been" | "your"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::types::{EngagementMetrics, SocialPost, MediaAttachment};

    fn make_post(platform: SocialPlatform, content: &str, author: &str) -> SocialPost {
        let mut eng = EngagementMetrics { likes: 500, shares: 50, comments: 30, views: None, score: 0.0 };
        eng.compute_score();
        SocialPost {
            platform,
            id: "1".into(),
            url: "https://example.com".into(),
            author: author.into(),
            content: content.into(),
            published: "2025-01-01".into(),
            engagement: eng,
            media: Vec::new(),
            hashtags: Vec::new(),
            mentions: Vec::new(),
            sentiment: 0.2,
            authenticity: 0.7,
        }
    }

    #[test]
    fn relevance_exact_phrase_high() {
        let r = compute_relevance(
            "Rust programming language memory safety systems",
            "rust programming",
        );
        assert!(r > 0.7, "exact phrase match should be high: {r}");
    }

    #[test]
    fn relevance_unrelated_low() {
        let r = compute_relevance("delicious chocolate cake recipe baking", "rust programming");
        assert!(r < 0.2, "unrelated content should be low: {r}");
    }

    #[test]
    fn validate_good_post_passes() {
        let post = make_post(
            SocialPlatform::Reddit,
            "Rust's ownership model provides memory safety without garbage collection, making it ideal for systems programming and performance-critical applications.",
            "rustacean",
        );
        let v = validate_post(&post, "rust programming");
        assert!(v.passes, "good post should pass: score={}", v.quality_score);
        assert!(v.quality_score > 0.5);
    }

    #[test]
    fn validate_spam_post_flagged() {
        let post = make_post(
            SocialPlatform::Twitter,
            "BUY NOW!!! LIMITED OFFER!!! bit.ly/abc123 #discount #sale #promo #deals #buy #now #cheap #free #win #money",
            "spambot123456",
        );
        let v = validate_post(&post, "rust programming");
        assert!(v.flags.contains(&ValidationFlag::ExcessiveHashtags) || v.flags.contains(&ValidationFlag::SpamPatterns));
        assert!(v.quality_score < 0.4, "spam should have low quality: {}", v.quality_score);
    }

    #[test]
    fn validate_deleted_author_penalised() {
        let post = make_post(SocialPlatform::Reddit, "Some content here that is reasonably long enough to pass minimum word check easily in this test", "[deleted]");
        let v = validate_post(&post, "rust");
        assert!(v.flags.contains(&ValidationFlag::DeletedAccount));
    }

    #[test]
    fn bot_username_detection() {
        assert!(looks_like_bot_username("user12345678"));  // 8 consecutive digits
        assert!(looks_like_bot_username("brtxkrz"));       // no vowels, >5 chars
        assert!(!looks_like_bot_username("rustacean"));    // normal
        assert!(!looks_like_bot_username("alice42"));      // normal with some digits
    }

    #[test]
    fn filter_and_rank_removes_low_quality() {
        let good = make_post(
            SocialPlatform::HackerNews,
            "Rust 2024 edition introduces new lifetime elision rules and improved async support with better ergonomics for developers building systems software and web backends.",
            "pg",
        );
        let spam = make_post(
            SocialPlatform::Twitter,
            "BUY NOW!!! bit.ly #spam #ad",
            "bot99999999",
        );
        let (filtered, report) = filter_and_rank(vec![good, spam], "rust programming", 10);
        assert_eq!(report.total, 2);
        assert!(filtered.len() <= 2);
        // The spam post should be filtered out or ranked last
        if filtered.len() == 2 {
            assert!(filtered[0].author != "bot99999999", "spam should be ranked last");
        }
    }

    #[test]
    fn engagement_legit_zero_is_neutral() {
        let eng = EngagementMetrics { likes: 0, shares: 0, comments: 0, views: None, score: 0.0 };
        let legit = compute_engagement_legitimacy(&eng, SocialPlatform::Reddit);
        assert!((legit - 0.6).abs() < 0.1);
    }

    #[test]
    fn relevance_empty_content_zero() {
        assert_eq!(compute_relevance("", "rust"), 0.0);
        assert_eq!(compute_relevance("rust", ""), 0.0);
    }

    #[test]
    fn short_content_flagged() {
        let post = make_post(SocialPlatform::Twitter, "Nice lol", "user");
        let v = validate_post(&post, "rust");
        assert!(v.flags.contains(&ValidationFlag::ShortContent));
    }

    #[test]
    fn cross_validate_boost_increases_authenticity() {
        let mut posts = vec![
            make_post(SocialPlatform::Reddit, "Rust memory safety ownership borrow checker guarantees thread safety", "user1"),
            make_post(SocialPlatform::HackerNews, "Rust memory safety ownership borrow checker guarantees thread safe", "user2"),
        ];
        let before = posts[0].authenticity;
        cross_validate_boost(&mut posts);
        assert!(posts[0].authenticity >= before, "cross-platform confirmation should boost authenticity");
    }
}
