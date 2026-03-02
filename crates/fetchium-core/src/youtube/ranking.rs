//! VideoFusion 8-signal ranking engine + clickbait detection + educational scoring.

use crate::youtube::types::*;

// ─── VideoFusion Ranking ───────────────────────────────────────

/// Rank videos using the VideoFusion 8-signal engine.
pub fn rank_videos(analyses: &[VideoAnalysis], query: &str) -> Vec<VideoRanking> {
    let mut rankings: Vec<VideoRanking> =
        analyses.iter().map(|a| compute_ranking(a, query)).collect();

    let intent = detect_video_query_intent(query);
    // Recompute final score with intent/mismatch adjustments, then sort.
    for r in &mut rankings {
        let base = weighted_score_by_intent(&r.signals, intent);
        let mismatch_penalty = title_query_mismatch_penalty(query, &r.title);
        r.final_score = base * (1.0 - r.clickbait_score * 0.45) * (1.0 - mismatch_penalty);
    }
    rankings.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    rankings
}

fn compute_ranking(analysis: &VideoAnalysis, query: &str) -> VideoRanking {
    let meta = &analysis.metadata;

    let relevance = compute_relevance(query, &meta.title, &meta.description, &meta.keywords);
    let freshness = compute_freshness(&meta.published);
    let authority = analysis.credibility.score;
    let engagement = compute_engagement(
        meta.view_count,
        meta.like_count,
        meta.duration_secs,
        &meta.published,
    );
    let educational = compute_educational_score(analysis);
    let authenticity = compute_authenticity_signal(analysis);
    let comment_quality = compute_comment_quality(analysis);
    let depth = compute_depth(meta.duration_secs, analysis.transcript.as_ref());

    let signals = VideoSignals {
        relevance,
        freshness,
        authority,
        engagement,
        educational,
        authenticity,
        comment_quality,
        depth,
    };

    let clickbait_score = detect_clickbait(&meta.title, &meta.description, analysis);
    let educational_score = educational;
    let intent = detect_video_query_intent(query);
    let final_score = weighted_score_by_intent(&signals, intent)
        * (1.0 - clickbait_score * 0.45)
        * (1.0 - title_query_mismatch_penalty(query, &meta.title));

    VideoRanking {
        video_id: meta.video_id.clone(),
        title: meta.title.clone(),
        final_score,
        signals,
        clickbait_score,
        educational_score,
    }
}

// ─── Signal Computations ───────────────────────────────────────

/// Signal 1: BM25-like relevance scoring on title + description vs query.
fn compute_relevance(query: &str, title: &str, description: &str, keywords: &[String]) -> f64 {
    let query_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    if query_terms.is_empty() {
        return 0.5;
    }

    let title_lower = title.to_lowercase();
    let desc_lower = description.to_lowercase();
    let kw_lower: String = keywords.join(" ").to_lowercase();
    let combined = format!("{title_lower} {desc_lower} {kw_lower}");

    let mut matches = 0usize;
    for term in &query_terms {
        if title_lower.contains(term.as_str()) {
            matches += 3; // Title match is 3x more important
        }
        if desc_lower.contains(term.as_str()) {
            matches += 1;
        }
        if kw_lower.contains(term.as_str()) {
            matches += 2;
        }
    }

    let doc_len = combined.split_whitespace().count() as f64;
    let tf = matches as f64 / (matches as f64 + 1.5 + 1.5 * doc_len / 200.0);
    tf.min(1.0)
}

/// Signal 2: Freshness — EDF temporal decay with 180-day half-life for YouTube.
fn compute_freshness(published: &str) -> f64 {
    // Parse relative time strings like "2 months ago", "3 weeks ago"
    let days = parse_relative_time(published);
    let half_life = 180.0; // YouTube-specific
    let lambda = (2.0_f64).ln() / half_life;
    (-lambda * days).exp()
}

/// Parse relative time strings into days.
fn parse_relative_time(s: &str) -> f64 {
    let lower = s.to_lowercase();
    let num: f64 = lower
        .split_whitespace()
        .find_map(|w| w.parse::<f64>().ok())
        .unwrap_or(30.0); // default to 30 days

    if lower.contains("hour") || lower.contains("minute") || lower.contains("just") {
        num / 24.0
    } else if lower.contains("day") {
        num
    } else if lower.contains("week") {
        num * 7.0
    } else if lower.contains("month") {
        num * 30.0
    } else if lower.contains("year") {
        num * 365.0
    } else {
        30.0 // default fallback
    }
}

/// Signal 4: Engagement — view/like ratio + comment activity.
fn compute_engagement(views: u64, likes: u64, _duration: u64, published: &str) -> f64 {
    if views == 0 {
        return 0.0;
    }
    let like_ratio = likes as f64 / views as f64;
    // YouTube typical like ratio is 2-5%, great is 5%+
    let normalized = (like_ratio / 0.05).min(1.0);
    // View count logarithmic boost
    let view_boost = (views as f64).log10() / 7.0; // 10M views = 1.0
                                                   // View velocity (views/day) rewards currently relevant videos.
    let days = parse_relative_time(published).max(1.0);
    let views_per_day = views as f64 / days;
    let velocity = ((views_per_day + 1.0).log10() / 5.0).min(1.0);
    (normalized * 0.45 + view_boost.min(1.0) * 0.35 + velocity * 0.20).min(1.0)
}

/// Signal 5: Educational value scoring.
fn compute_educational_score(analysis: &VideoAnalysis) -> f64 {
    let mut score = 0.0f64;
    let meta = &analysis.metadata;

    // Chapters indicate structured content
    if !meta.chapters.is_empty() {
        score += 0.25;
    }

    // Description links to docs/code indicate educational
    let doc_links = meta
        .links
        .iter()
        .filter(|l| matches!(l.link_type, LinkType::Code | LinkType::Documentation))
        .count();
    score += (doc_links as f64 * 0.05).min(0.15);

    // Transcript word density (words per minute)
    if let Some(ref transcript) = analysis.transcript {
        let duration_min = meta.duration_secs as f64 / 60.0;
        if duration_min > 0.0 {
            let wpm = transcript.word_count as f64 / duration_min;
            // Good educational content: 120-180 wpm
            if (120.0..=180.0).contains(&wpm) {
                score += 0.2;
            } else if wpm > 80.0 {
                score += 0.1;
            }
        }

        // Key moments (definitions, key points)
        let def_count = transcript
            .key_moments
            .iter()
            .filter(|m| matches!(m.moment_type, MomentType::Definition | MomentType::KeyPoint))
            .count();
        score += (def_count as f64 * 0.03).min(0.2);
    }

    // Duration sweet spot for educational content (8-30 minutes)
    let dur_min = meta.duration_secs as f64 / 60.0;
    if (8.0..=30.0).contains(&dur_min) {
        score += 0.1;
    }

    // Keywords containing educational terms
    let edu_keywords = [
        "tutorial",
        "guide",
        "explained",
        "learn",
        "course",
        "lecture",
        "how to",
    ];
    if meta
        .keywords
        .iter()
        .any(|k| edu_keywords.iter().any(|e| k.to_lowercase().contains(e)))
        || edu_keywords
            .iter()
            .any(|e| meta.title.to_lowercase().contains(e))
    {
        score += 0.1;
    }

    score.min(1.0)
}

/// Signal 6: Authenticity (inverse of clickbait + ACS on transcript).
fn compute_authenticity_signal(analysis: &VideoAnalysis) -> f64 {
    let clickbait = detect_clickbait(
        &analysis.metadata.title,
        &analysis.metadata.description,
        analysis,
    );
    let comment_auth = analysis
        .comments
        .as_ref()
        .map(|c| c.authenticity.score)
        .unwrap_or(0.5);
    (1.0 - clickbait) * 0.6 + comment_auth * 0.4
}

/// Signal 7: Comment quality.
fn compute_comment_quality(analysis: &VideoAnalysis) -> f64 {
    analysis
        .comments
        .as_ref()
        .map(|c| {
            let sentiment_pos = c.overall_sentiment.positive;
            let info_score = if c.informative_comments.is_empty() {
                0.3
            } else {
                c.informative_comments.iter().map(|i| i.score).sum::<f64>()
                    / c.informative_comments.len() as f64
            };
            sentiment_pos * 0.4 + info_score * 0.6
        })
        .unwrap_or(0.5)
}

/// Signal 8: Depth (duration × word density).
fn compute_depth(duration_secs: u64, transcript: Option<&EnhancedTranscript>) -> f64 {
    let dur_min = duration_secs as f64 / 60.0;
    // Duration contribution (diminishing returns past 30 min)
    let dur_score = (dur_min / 30.0).min(1.0);

    let word_density = transcript
        .map(|t| {
            if dur_min > 0.0 {
                t.word_count as f64 / dur_min / 150.0
            } else {
                0.5
            }
        })
        .unwrap_or(0.5);

    (dur_score * 0.5 + word_density.min(1.0) * 0.5).min(1.0)
}

#[derive(Debug, Clone, Copy)]
enum VideoQueryIntent {
    Learn,
    News,
    Review,
    Comparison,
    General,
}

fn detect_video_query_intent(query: &str) -> VideoQueryIntent {
    let q = query.to_lowercase();
    if ["learn", "tutorial", "course", "beginner", "how to", "guide"]
        .iter()
        .any(|k| q.contains(k))
    {
        VideoQueryIntent::Learn
    } else if ["latest", "news", "update", "2026", "today", "breaking"]
        .iter()
        .any(|k| q.contains(k))
    {
        VideoQueryIntent::News
    } else if ["review", "best", "top", "vs", "comparison", "compare"]
        .iter()
        .any(|k| q.contains(k))
    {
        if q.contains("vs") || q.contains("compare") || q.contains("comparison") {
            VideoQueryIntent::Comparison
        } else {
            VideoQueryIntent::Review
        }
    } else {
        VideoQueryIntent::General
    }
}

fn weighted_score_by_intent(signals: &VideoSignals, intent: VideoQueryIntent) -> f64 {
    match intent {
        VideoQueryIntent::Learn => {
            signals.relevance * 0.22
                + signals.freshness * 0.06
                + signals.authority * 0.14
                + signals.engagement * 0.08
                + signals.educational * 0.25
                + signals.authenticity * 0.10
                + signals.comment_quality * 0.05
                + signals.depth * 0.10
        }
        VideoQueryIntent::News => {
            signals.relevance * 0.22
                + signals.freshness * 0.20
                + signals.authority * 0.14
                + signals.engagement * 0.14
                + signals.educational * 0.08
                + signals.authenticity * 0.10
                + signals.comment_quality * 0.06
                + signals.depth * 0.06
        }
        VideoQueryIntent::Review | VideoQueryIntent::Comparison => {
            signals.relevance * 0.24
                + signals.freshness * 0.09
                + signals.authority * 0.14
                + signals.engagement * 0.15
                + signals.educational * 0.10
                + signals.authenticity * 0.12
                + signals.comment_quality * 0.08
                + signals.depth * 0.08
        }
        VideoQueryIntent::General => signals.weighted_score(),
    }
}

fn title_query_mismatch_penalty(query: &str, title: &str) -> f64 {
    let stop = [
        "a", "an", "the", "in", "on", "for", "to", "of", "and", "or", "with", "how", "what",
    ];
    let q_terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .filter(|t| t.len() > 2 && !stop.contains(t))
        .map(|t| t.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|t| !t.is_empty())
        .collect();
    if q_terms.is_empty() {
        return 0.0;
    }
    let t = title.to_lowercase();
    let matched = q_terms
        .iter()
        .filter(|term| t.contains(term.as_str()))
        .count();
    if matched == 0 {
        0.35
    } else if matched * 2 <= q_terms.len() {
        0.12
    } else {
        0.0
    }
}

// ─── Clickbait Detection ───────────────────────────────────────

/// Detect clickbait score (0.0 = genuine, 1.0 = pure clickbait).
pub fn detect_clickbait(title: &str, description: &str, analysis: &VideoAnalysis) -> f64 {
    let mut score = 0.0f64;

    // ALL CAPS words in title (more than 30% caps words = suspicious)
    let words: Vec<&str> = title.split_whitespace().collect();
    if !words.is_empty() {
        let caps_count = words
            .iter()
            .filter(|w| w.len() > 2 && w.chars().all(|c| c.is_uppercase()))
            .count();
        let caps_ratio = caps_count as f64 / words.len() as f64;
        if caps_ratio > 0.3 {
            score += 0.25;
        }
    }

    // Excessive punctuation (!!!, ???, etc.)
    let exclaim_count = title.matches('!').count() + title.matches('?').count();
    if exclaim_count >= 3 {
        score += 0.15;
    }

    // Clickbait trigger phrases
    let lower_title = title.to_lowercase();
    for phrase in CLICKBAIT_PHRASES {
        if lower_title.contains(phrase) {
            score += 0.2;
            break;
        }
    }

    // Title-description mismatch (title mentions things description doesn't)
    let title_words: std::collections::HashSet<String> = lower_title
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() > 3)
        .collect();
    let desc_lower = description.to_lowercase();
    let desc_words: std::collections::HashSet<String> = desc_lower
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| w.len() > 3)
        .collect();
    if !title_words.is_empty() {
        let overlap = title_words.intersection(&desc_words).count();
        let mismatch_ratio = 1.0 - (overlap as f64 / title_words.len() as f64);
        if mismatch_ratio > 0.7 {
            score += 0.15;
        }
    }

    // Very short video with sensational title
    if analysis.metadata.duration_secs < 60 && score > 0.2 {
        score += 0.1;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_analysis(title: &str, views: u64, likes: u64, subs: Option<u64>) -> VideoAnalysis {
        VideoAnalysis {
            metadata: VideoMetadata {
                video_id: "test123".into(),
                title: title.into(),
                description: "A comprehensive guide to the topic".into(),
                channel: ChannelInfo {
                    name: "TestCh".into(),
                    id: "UC1".into(),
                    subscriber_count: subs,
                    verified: false,
                },
                duration_secs: 600,
                view_count: views,
                like_count: likes,
                published: "2 weeks ago".into(),
                keywords: vec!["tutorial".into()],
                chapters: vec![],
                links: vec![],
                thumbnail_url: None,
                is_live: false,
            },
            transcript: None,
            comments: None,
            credibility: crate::youtube::metadata::score_channel_credibility(&ChannelInfo {
                name: "TestCh".into(),
                id: "UC1".into(),
                subscriber_count: subs,
                verified: false,
            }),
        }
    }

    #[test]
    fn rank_videos_basic() {
        let analyses = vec![
            make_analysis("Rust Programming Tutorial", 100_000, 5000, Some(50_000)),
            make_analysis("Go Programming Guide", 50_000, 2000, Some(10_000)),
        ];
        let rankings = rank_videos(&analyses, "rust programming");
        assert_eq!(rankings.len(), 2);
        assert!(rankings[0].final_score >= rankings[1].final_score);
        assert_eq!(rankings[0].title, "Rust Programming Tutorial");
    }

    #[test]
    fn relevance_title_match() {
        let r = compute_relevance(
            "rust async",
            "Rust Async Tutorial",
            "Learn about async in Rust",
            &["rust".into()],
        );
        assert!(r > 0.5);
    }

    #[test]
    fn relevance_no_match() {
        let r = compute_relevance("python", "Rust Tutorial", "Learn Rust", &[]);
        assert!(r < 0.3);
    }

    #[test]
    fn freshness_recent() {
        let f = compute_freshness("2 days ago");
        assert!(f > 0.95);
    }

    #[test]
    fn freshness_old() {
        let f = compute_freshness("2 years ago");
        assert!(f < 0.2);
    }

    #[test]
    fn engagement_high() {
        let e = compute_engagement(1_000_000, 50_000, 600, "2 days ago");
        assert!(e > 0.5);
    }

    #[test]
    fn engagement_low() {
        let e = compute_engagement(100, 1, 60, "3 years ago");
        assert!(e < 0.3);
    }

    #[test]
    fn clickbait_detection_clean() {
        let analysis = make_analysis("How to Learn Rust in 2025", 50_000, 3000, Some(20_000));
        let score = detect_clickbait(
            "How to Learn Rust in 2025",
            "A comprehensive guide to the topic",
            &analysis,
        );
        assert!(score < 0.3);
    }

    #[test]
    fn clickbait_detection_flagged() {
        let analysis = make_analysis("YOU WON'T BELIEVE THIS!!!", 50_000, 100, Some(500));
        let score = detect_clickbait(
            "YOU WON'T BELIEVE THIS SHOCKING SECRET!!!",
            "Random unrelated stuff",
            &analysis,
        );
        assert!(score > 0.3);
    }

    #[test]
    fn parse_relative_time_variants() {
        assert!(parse_relative_time("3 hours ago") < 1.0);
        assert!((parse_relative_time("5 days ago") - 5.0).abs() < 0.1);
        assert!((parse_relative_time("2 weeks ago") - 14.0).abs() < 0.1);
        assert!((parse_relative_time("3 months ago") - 90.0).abs() < 0.1);
        assert!((parse_relative_time("1 year ago") - 365.0).abs() < 0.1);
    }

    #[test]
    fn depth_long_video() {
        let d = compute_depth(3600, None); // 1 hour
        assert!(d > 0.4);
    }

    #[test]
    fn educational_with_chapters() {
        let mut analysis = make_analysis("Tutorial", 10_000, 500, Some(5_000));
        analysis.metadata.chapters = vec![
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
        let score = compute_educational_score(&analysis);
        assert!(score > 0.2);
    }
}
