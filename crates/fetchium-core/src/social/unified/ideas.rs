//! Content idea generation engine.
//!
//! Analyses viral content patterns across platforms and generates
//! optimised content ideas for each platform format.

use crate::social::types::{
    ContentFormat, ContentIdea, CrossPlatformTrend, SocialPlatform, SocialPost, ViralScore,
};

/// Generate content ideas from cross-platform trends.
///
/// Uses the top viral patterns to produce platform-optimised content ideas
/// with hooks, key points, and viral potential scores.
pub fn generate_ideas(trends: &[CrossPlatformTrend], top_posts: &[SocialPost]) -> Vec<ContentIdea> {
    let mut ideas = Vec::new();

    for trend in trends.iter().take(10) {
        // Generate ideas for each platform the trend appears on
        for &platform in &trend.platforms {
            let formats = best_formats_for_platform(platform, trend.is_viral);
            for format in formats {
                let idea = build_idea(&trend.topic, platform, format, trend, top_posts);
                ideas.push(idea);
            }
        }
    }

    // Sort by viral potential descending
    ideas.sort_by(|a, b| {
        b.viral_potential
            .partial_cmp(&a.viral_potential)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Deduplicate by title similarity
    let mut kept: Vec<ContentIdea> = Vec::new();
    for idea in ideas {
        let similar = kept.iter().any(|k| k.title == idea.title);
        if !similar {
            kept.push(idea);
        }
    }

    kept.truncate(20);
    kept
}

/// Build a single content idea for a given trend + platform + format.
fn build_idea(
    topic: &str,
    platform: SocialPlatform,
    format: ContentFormat,
    trend: &CrossPlatformTrend,
    top_posts: &[SocialPost],
) -> ContentIdea {
    let viral_potential = compute_viral_potential(trend, format, top_posts);
    let (title, hook, key_points) = generate_content_elements(topic, platform, format, trend);

    ContentIdea {
        platform,
        format,
        title,
        hook,
        key_points,
        viral_potential,
    }
}

/// Generate title, hook, and key points for a content idea.
fn generate_content_elements(
    topic: &str,
    platform: SocialPlatform,
    format: ContentFormat,
    trend: &CrossPlatformTrend,
) -> (String, String, Vec<String>) {
    let title = match format {
        ContentFormat::Thread => {
            format!("🧵 Thread: Everything about {topic} (and why it matters)")
        }
        ContentFormat::ShortVideo => format!("60s explainer: {topic} 🔥"),
        ContentFormat::LongVideo => format!("Deep Dive: {topic} — Full Analysis"),
        ContentFormat::Post => format!("The definitive guide to {topic} [Discussion]"),
        ContentFormat::Thread60s => format!("THREAD: {topic} explained in 5 tweets"),
        ContentFormat::Infographic => format!("{topic}: The visual breakdown"),
        ContentFormat::Tutorial => format!("How to master {topic} (step-by-step)"),
        ContentFormat::Listicle => format!("10 things you need to know about {topic}"),
        ContentFormat::Debate => format!("Hot take: {topic} is changing everything"),
    };

    let hook = match platform {
        SocialPlatform::Twitter => {
            format!("Everyone's talking about {topic}. Here's what you're missing 👇")
        }
        SocialPlatform::TikTok => {
            format!("POV: You just discovered {topic} and your mind is blown")
        }
        SocialPlatform::Reddit => {
            format!("I spent 3 days researching {topic}. Here's the complete breakdown.")
        }
        SocialPlatform::YouTube => format!(
            "In this video, I break down {topic} so clearly that even beginners will get it."
        ),
        SocialPlatform::HackerNews => {
            format!("Ask HN: What's the best resource for understanding {topic}?")
        }
        SocialPlatform::Facebook => {
            format!("Join the conversation: What does {topic} mean for you and your community?")
        }
    };

    // Build key points from trend data
    let mut key_points = vec![
        format!(
            "Why {topic} is trending across {} platforms",
            trend.platforms.len()
        ),
        format!(
            "Key insight from {} posts analysed",
            trend.sample_posts.len()
        ),
    ];

    if trend.is_viral {
        key_points.push(format!(
            "Viral signal: velocity={:.0} posts/hour",
            trend.velocity
        ));
    }

    if trend.sentiment > 0.3 {
        key_points
            .push("Audience sentiment: highly positive — great for educational content".into());
    } else if trend.sentiment < -0.3 {
        key_points.push("Audience sentiment: controversial — high debate potential".into());
    }

    // Extract talking points from top posts
    for post in trend.sample_posts.iter().take(2) {
        let snippet: String = post.content.chars().take(80).collect();
        key_points.push(format!("From the community: \"{snippet}\""));
    }

    (title, hook, key_points)
}

/// Compute viral potential score for a given format and trend.
fn compute_viral_potential(
    trend: &CrossPlatformTrend,
    format: ContentFormat,
    top_posts: &[SocialPost],
) -> f64 {
    // Base: cross-platform presence
    let platform_score = (trend.platforms.len() as f64 / 5.0).min(1.0);

    // Engagement velocity
    let velocity_score = (trend.velocity / 500.0).min(1.0);

    // Format fit multiplier
    let format_multiplier = match format {
        ContentFormat::ShortVideo if trend.is_viral => 1.0,
        ContentFormat::Thread if trend.sentiment.abs() > 0.4 => 0.9,
        ContentFormat::Debate if trend.sentiment < -0.2 => 0.95,
        ContentFormat::Tutorial => 0.85,
        ContentFormat::Listicle => 0.80,
        _ => 0.70,
    };

    // Top post engagement boost
    let engagement_boost = top_posts
        .iter()
        .filter(|p| {
            let content_lower = p.content.to_lowercase();
            let topic_lower = trend.topic.to_lowercase();
            content_lower.contains(&topic_lower)
        })
        .map(|p| p.engagement.score)
        .fold(0.0f64, f64::max)
        * 0.2;

    let viral_score = ViralScore::compute(
        velocity_score,
        trend.sentiment.abs(),
        platform_score,
        0.7,
        format_multiplier,
    );

    (viral_score.overall + engagement_boost).min(1.0)
}

/// Determine best content formats for a platform.
fn best_formats_for_platform(platform: SocialPlatform, is_viral: bool) -> Vec<ContentFormat> {
    match platform {
        SocialPlatform::Twitter => {
            if is_viral {
                vec![
                    ContentFormat::Thread,
                    ContentFormat::Debate,
                    ContentFormat::Thread60s,
                ]
            } else {
                vec![ContentFormat::Thread, ContentFormat::Thread60s]
            }
        }
        SocialPlatform::TikTok => vec![ContentFormat::ShortVideo],
        SocialPlatform::YouTube => {
            vec![ContentFormat::LongVideo, ContentFormat::Tutorial]
        }
        SocialPlatform::Reddit => {
            vec![
                ContentFormat::Post,
                ContentFormat::Listicle,
                ContentFormat::Tutorial,
            ]
        }
        SocialPlatform::HackerNews => {
            vec![ContentFormat::Post, ContentFormat::Tutorial]
        }
        SocialPlatform::Facebook => {
            vec![
                ContentFormat::Post,
                ContentFormat::Infographic,
                ContentFormat::LongVideo,
            ]
        }
    }
}

/// Format content ideas as a markdown report.
pub fn format_ideas_markdown(ideas: &[ContentIdea]) -> String {
    if ideas.is_empty() {
        return "No content ideas generated.\n".to_string();
    }

    let mut out = String::from("## Content Ideas\n\n");

    for (i, idea) in ideas.iter().enumerate().take(10) {
        out.push_str(&format!(
            "### {}. {} `[{:.0}% viral potential]`\n",
            i + 1,
            idea.title,
            idea.viral_potential * 100.0
        ));
        out.push_str(&format!(
            "**Platform:** {} | **Format:** {}\n\n",
            idea.platform, idea.format
        ));
        out.push_str(&format!("**Hook:** {}\n\n", idea.hook));
        out.push_str("**Key Points:**\n");
        for point in &idea.key_points {
            out.push_str(&format!("- {point}\n"));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trend(topic: &str, platforms: &[SocialPlatform]) -> CrossPlatformTrend {
        CrossPlatformTrend {
            topic: topic.into(),
            platforms: platforms.to_vec(),
            total_engagement: 50_000,
            velocity: 120.0,
            sentiment: 0.4,
            is_viral: true,
            content_ideas: Vec::new(),
            sample_posts: Vec::new(),
        }
    }

    #[test]
    fn ideas_empty_trends() {
        let ideas = generate_ideas(&[], &[]);
        assert!(ideas.is_empty());
    }

    #[test]
    fn ideas_single_trend() {
        let trend = make_trend(
            "Rust programming",
            &[SocialPlatform::Reddit, SocialPlatform::HackerNews],
        );
        let ideas = generate_ideas(&[trend], &[]);
        assert!(!ideas.is_empty());
        // All ideas should be within [0, 1]
        for idea in &ideas {
            assert!(idea.viral_potential >= 0.0 && idea.viral_potential <= 1.0);
        }
    }

    #[test]
    fn best_formats_tiktok() {
        let formats = best_formats_for_platform(SocialPlatform::TikTok, true);
        assert!(formats.contains(&ContentFormat::ShortVideo));
    }

    #[test]
    fn format_ideas_output_not_empty() {
        let trend = make_trend(
            "AI tools",
            &[SocialPlatform::Twitter, SocialPlatform::Reddit],
        );
        let ideas = generate_ideas(&[trend], &[]);
        let md = format_ideas_markdown(&ideas);
        assert!(md.contains("Content Ideas"));
    }

    #[test]
    fn format_ideas_empty_message() {
        let md = format_ideas_markdown(&[]);
        assert!(md.contains("No content ideas"));
    }

    #[test]
    fn best_formats_youtube_has_long_video() {
        let formats = best_formats_for_platform(SocialPlatform::YouTube, false);
        assert!(formats.contains(&ContentFormat::LongVideo));
        assert!(formats.contains(&ContentFormat::Tutorial));
    }

    #[test]
    fn best_formats_reddit_has_post() {
        let formats = best_formats_for_platform(SocialPlatform::Reddit, false);
        assert!(formats.contains(&ContentFormat::Post));
        assert!(formats.contains(&ContentFormat::Listicle));
    }

    #[test]
    fn best_formats_twitter_viral_includes_debate() {
        let formats = best_formats_for_platform(SocialPlatform::Twitter, true);
        assert!(formats.contains(&ContentFormat::Debate));
        assert!(formats.contains(&ContentFormat::Thread));
    }

    #[test]
    fn best_formats_twitter_nonviral_no_debate() {
        let formats = best_formats_for_platform(SocialPlatform::Twitter, false);
        assert!(!formats.contains(&ContentFormat::Debate));
    }

    #[test]
    fn best_formats_hn_has_post() {
        let formats = best_formats_for_platform(SocialPlatform::HackerNews, false);
        assert!(formats.contains(&ContentFormat::Post));
    }

    #[test]
    fn ideas_sorted_by_viral_potential() {
        let trend1 = make_trend(
            "AI revolution",
            &[
                SocialPlatform::Twitter,
                SocialPlatform::Reddit,
                SocialPlatform::TikTok,
                SocialPlatform::HackerNews,
            ],
        );
        let trend2 = make_trend(
            "Rust",
            &[SocialPlatform::Reddit, SocialPlatform::HackerNews],
        );
        let ideas = generate_ideas(&[trend1, trend2], &[]);
        // Verify sorted descending
        for pair in ideas.windows(2) {
            assert!(
                pair[0].viral_potential >= pair[1].viral_potential,
                "ideas should be sorted by viral potential"
            );
        }
    }

    #[test]
    fn ideas_all_have_non_empty_hook() {
        let trend = make_trend(
            "quantum computing",
            &[SocialPlatform::YouTube, SocialPlatform::Reddit],
        );
        let ideas = generate_ideas(&[trend], &[]);
        for idea in &ideas {
            assert!(!idea.hook.is_empty(), "hook should not be empty");
            assert!(!idea.title.is_empty(), "title should not be empty");
        }
    }

    #[test]
    fn ideas_capped_at_twenty() {
        // Create many trends with many platforms to generate lots of ideas
        let trends: Vec<_> = (0..10)
            .map(|i| {
                make_trend(
                    &format!("topic {i}"),
                    &[
                        SocialPlatform::Twitter,
                        SocialPlatform::Reddit,
                        SocialPlatform::TikTok,
                        SocialPlatform::YouTube,
                        SocialPlatform::HackerNews,
                    ],
                )
            })
            .collect();
        let ideas = generate_ideas(&trends, &[]);
        assert!(
            ideas.len() <= 20,
            "ideas should be capped at 20: got {}",
            ideas.len()
        );
    }
}
