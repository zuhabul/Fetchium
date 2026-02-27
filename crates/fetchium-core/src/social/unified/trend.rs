//! Cross-platform trend detection and viral burst identification.

use crate::social::types::{
    bigrams, jaccard, CrossPlatformTrend, SocialPlatform, SocialPost, TrendItem,
};
use std::collections::HashMap;

/// Detect cross-platform trends by clustering topics with bigram-Jaccard similarity.
///
/// Topics with jaccard similarity ≥ threshold across ≥2 platforms are merged.
pub fn detect_cross_platform_trends(
    platform_trends: &HashMap<SocialPlatform, Vec<TrendItem>>,
    threshold: f64,
) -> Vec<CrossPlatformTrend> {
    // Collect all trend topics
    let mut all_trends: Vec<(SocialPlatform, &TrendItem)> = Vec::new();
    for (platform, trends) in platform_trends {
        for trend in trends {
            all_trends.push((*platform, trend));
        }
    }

    // Cluster by bigram similarity
    let mut clusters: Vec<Vec<(SocialPlatform, &TrendItem)>> = Vec::new();

    'outer: for (platform, trend) in &all_trends {
        let a = bigrams(&trend.topic.to_lowercase());
        for cluster in &mut clusters {
            let rep = bigrams(&cluster[0].1.topic.to_lowercase());
            if jaccard(&a, &rep) >= threshold {
                cluster.push((*platform, trend));
                continue 'outer;
            }
        }
        clusters.push(vec![(*platform, trend)]);
    }

    // Build CrossPlatformTrend from clusters with ≥2 platforms
    let mut result: Vec<CrossPlatformTrend> = clusters
        .iter()
        .filter(|c| {
            let unique_platforms: std::collections::HashSet<SocialPlatform> =
                c.iter().map(|(p, _)| *p).collect();
            unique_platforms.len() >= 2
        })
        .map(|cluster| {
            let platforms: Vec<SocialPlatform> = {
                let mut seen = std::collections::HashSet::new();
                cluster
                    .iter()
                    .filter_map(|(p, _)| if seen.insert(*p) { Some(*p) } else { None })
                    .collect()
            };

            let topic = cluster[0].1.topic.clone();
            let total_engagement: u64 = cluster.iter().map(|(_, t)| t.post_count).sum();
            let velocity: f64 =
                cluster.iter().map(|(_, t)| t.velocity).sum::<f64>() / cluster.len() as f64;
            let sentiment: f64 =
                cluster.iter().map(|(_, t)| t.sentiment).sum::<f64>() / cluster.len() as f64;

            // Collect sample posts across platforms
            let sample_posts: Vec<SocialPost> = cluster
                .iter()
                .flat_map(|(_, t)| t.sample_posts.iter().cloned())
                .take(6)
                .collect();

            let is_viral = velocity > 100.0 || platforms.len() >= 3;

            CrossPlatformTrend {
                topic,
                platforms,
                total_engagement,
                velocity,
                sentiment,
                is_viral,
                content_ideas: Vec::new(), // filled by ideas engine
                sample_posts,
            }
        })
        .collect();

    // Sort by total engagement descending
    result.sort_by(|a, b| b.total_engagement.cmp(&a.total_engagement));
    result
}

/// Detect viral bursts: topics with high velocity growth across platforms.
///
/// A "burst" is defined as velocity > 3× the median velocity of all topics.
pub fn detect_viral_bursts(trends: &[CrossPlatformTrend]) -> Vec<&CrossPlatformTrend> {
    if trends.is_empty() {
        return Vec::new();
    }

    let mut velocities: Vec<f64> = trends.iter().map(|t| t.velocity).collect();
    velocities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = velocities[velocities.len() / 2];
    let threshold = median * 3.0;

    trends.iter().filter(|t| t.velocity > threshold).collect()
}

/// Merge normalised posts from all platforms, deduplicating by content similarity.
pub fn merge_and_dedup_posts(
    platform_posts: &HashMap<SocialPlatform, Vec<SocialPost>>,
    dedup_threshold: f64,
) -> Vec<SocialPost> {
    let mut all_posts: Vec<SocialPost> = platform_posts
        .values()
        .flat_map(|posts| posts.iter().cloned())
        .collect();

    // Dedup by bigram-Jaccard similarity on content
    let mut kept: Vec<SocialPost> = Vec::new();
    for post in all_posts.drain(..) {
        let a = bigrams(&post.content.to_lowercase());
        let is_dup = kept.iter().any(|k| {
            let b = bigrams(&k.content.to_lowercase());
            jaccard(&a, &b) >= dedup_threshold
        });
        if !is_dup {
            kept.push(post);
        }
    }

    // Sort by engagement score descending
    kept.sort_by(|a, b| {
        b.engagement
            .score
            .partial_cmp(&a.engagement.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    kept
}

/// Compute platform credibility weights for cross-signal fusion.
///
/// HN has highest content quality, TikTok highest reach velocity.
pub fn platform_weight(platform: SocialPlatform) -> f64 {
    match platform {
        SocialPlatform::HackerNews => 1.0, // expert community, high signal/noise
        SocialPlatform::Reddit => 0.85,    // topic-focused, moderated
        SocialPlatform::YouTube => 0.80,   // long-form, researched
        SocialPlatform::Facebook => 0.70,  // large reach, moderate signal quality
        SocialPlatform::Twitter => 0.65,   // fast but noisy
        SocialPlatform::TikTok => 0.55,    // viral reach but lower depth
    }
}

/// Weighted engagement score across platforms.
pub fn cross_signal_score(posts: &[&SocialPost]) -> f64 {
    if posts.is_empty() {
        return 0.0;
    }
    let total: f64 = posts
        .iter()
        .map(|p| p.engagement.score * platform_weight(p.platform))
        .sum();
    (total / posts.len() as f64).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::social::types::EngagementMetrics;

    #[test]
    fn detect_empty_trends() {
        let map = HashMap::new();
        let result = detect_cross_platform_trends(&map, 0.3);
        assert!(result.is_empty());
    }

    #[test]
    fn platform_weight_ordering() {
        assert!(
            platform_weight(SocialPlatform::HackerNews) > platform_weight(SocialPlatform::TikTok)
        );
        assert!(platform_weight(SocialPlatform::Reddit) > platform_weight(SocialPlatform::Twitter));
    }

    #[test]
    fn cross_signal_score_empty() {
        assert_eq!(cross_signal_score(&[]), 0.0);
    }

    #[test]
    fn dedup_identical_posts() {
        let mut platform_posts = HashMap::new();
        let post = SocialPost {
            platform: SocialPlatform::Reddit,
            id: "1".into(),
            url: "https://example.com".into(),
            author: "user".into(),
            content: "Rust is an amazing programming language for systems development".into(),
            published: "2025-01-01".into(),
            engagement: EngagementMetrics::default(),
            media: Vec::new(),
            hashtags: Vec::new(),
            mentions: Vec::new(),
            sentiment: 0.5,
            authenticity: 0.9,
        };
        let mut post2 = post.clone();
        post2.id = "2".into();
        post2.platform = SocialPlatform::Twitter;

        platform_posts.insert(SocialPlatform::Reddit, vec![post]);
        platform_posts.insert(SocialPlatform::Twitter, vec![post2]);

        let merged = merge_and_dedup_posts(&platform_posts, 0.5);
        // Near-identical content should be deduped to 1
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn dedup_distinct_posts_both_kept() {
        let mut platform_posts = HashMap::new();
        let make_post = |platform: SocialPlatform, content: &str| SocialPost {
            platform,
            id: "1".into(),
            url: "https://example.com".into(),
            author: "user".into(),
            content: content.into(),
            published: "2025-01-01".into(),
            engagement: EngagementMetrics::default(),
            media: Vec::new(),
            hashtags: Vec::new(),
            mentions: Vec::new(),
            sentiment: 0.0,
            authenticity: 0.8,
        };
        platform_posts.insert(
            SocialPlatform::Reddit,
            vec![make_post(
                SocialPlatform::Reddit,
                "Rust systems programming memory safety",
            )],
        );
        platform_posts.insert(
            SocialPlatform::HackerNews,
            vec![make_post(
                SocialPlatform::HackerNews,
                "Python machine learning neural networks pytorch",
            )],
        );
        let merged = merge_and_dedup_posts(&platform_posts, 0.7);
        assert_eq!(merged.len(), 2, "distinct posts should both be kept");
    }

    #[test]
    fn detect_viral_bursts_empty() {
        let bursts = detect_viral_bursts(&[]);
        assert!(bursts.is_empty());
    }

    #[test]
    fn detect_viral_bursts_no_burst_single() {
        let trends = vec![CrossPlatformTrend {
            topic: "Rust".into(),
            platforms: vec![SocialPlatform::Reddit, SocialPlatform::HackerNews],
            total_engagement: 10_000,
            velocity: 50.0,
            sentiment: 0.3,
            is_viral: false,
            content_ideas: Vec::new(),
            sample_posts: Vec::new(),
        }];
        // With one trend, median = 50, threshold = 150 → no burst
        let bursts = detect_viral_bursts(&trends);
        assert!(bursts.is_empty());
    }

    #[test]
    fn detect_viral_bursts_with_burst() {
        // Need ≥3 trends so median is taken from the lower half
        // sorted velocities: [5, 10, 12] → median at idx 1 = 10 → threshold = 30
        // The "AI" trend with velocity 200 > 30 → burst
        let make_trend = |topic: &str, velocity: f64| CrossPlatformTrend {
            topic: topic.into(),
            platforms: vec![SocialPlatform::Reddit, SocialPlatform::HackerNews],
            total_engagement: 1_000,
            velocity,
            sentiment: 0.0,
            is_viral: false,
            content_ideas: Vec::new(),
            sample_posts: Vec::new(),
        };
        let trends = vec![
            make_trend("Rust", 5.0),
            make_trend("Python", 10.0),
            make_trend("AI", 200.0), // 200 > 3 × median(10) = 30 → burst
        ];
        let bursts = detect_viral_bursts(&trends);
        assert_eq!(bursts.len(), 1);
        assert_eq!(bursts[0].topic, "AI");
    }

    #[test]
    fn cross_signal_score_hn_higher_than_tiktok() {
        let make_post = |platform: SocialPlatform| {
            let mut e = EngagementMetrics {
                likes: 1000,
                shares: 200,
                comments: 50,
                views: None,
                score: 0.0,
            };
            e.compute_score();
            SocialPost {
                platform,
                id: "x".into(),
                url: "https://example.com".into(),
                author: "u".into(),
                content: "test".into(),
                published: String::new(),
                engagement: e,
                media: Vec::new(),
                hashtags: Vec::new(),
                mentions: Vec::new(),
                sentiment: 0.0,
                authenticity: 1.0,
            }
        };
        let hn = make_post(SocialPlatform::HackerNews);
        let tt = make_post(SocialPlatform::TikTok);
        let hn_score = cross_signal_score(&[&hn]);
        let tt_score = cross_signal_score(&[&tt]);
        assert!(hn_score > tt_score, "HN > TikTok: {hn_score} vs {tt_score}");
    }

    #[test]
    fn detect_cross_platform_same_topic_two_platforms() {
        let mut map = HashMap::new();
        let make_trend = |platform: SocialPlatform, topic: &str| TrendItem {
            platform,
            topic: topic.into(),
            post_count: 500,
            velocity: 20.0,
            sentiment: 0.1,
            region: None,
            related_topics: Vec::new(),
            sample_posts: Vec::new(),
        };
        map.insert(
            SocialPlatform::Reddit,
            vec![make_trend(
                SocialPlatform::Reddit,
                "rust programming language",
            )],
        );
        map.insert(
            SocialPlatform::HackerNews,
            vec![make_trend(
                SocialPlatform::HackerNews,
                "rust programming language",
            )],
        );
        let trends = detect_cross_platform_trends(&map, 0.3);
        assert_eq!(trends.len(), 1, "same topic across 2 platforms → 1 trend");
        assert_eq!(trends[0].platforms.len(), 2);
    }

    #[test]
    fn detect_cross_platform_single_platform_ignored() {
        let mut map = HashMap::new();
        let trend = TrendItem {
            platform: SocialPlatform::Reddit,
            topic: "unique topic xyz zyxwvu".into(),
            post_count: 100,
            velocity: 5.0,
            sentiment: 0.0,
            region: None,
            related_topics: Vec::new(),
            sample_posts: Vec::new(),
        };
        map.insert(SocialPlatform::Reddit, vec![trend]);
        // Only 1 platform → not cross-platform
        let trends = detect_cross_platform_trends(&map, 0.3);
        assert!(
            trends.is_empty(),
            "single-platform trends should be filtered out"
        );
    }
}
