//! TikTok video analysis: engagement scoring, viral detection, trend alignment.

use crate::social::tiktok::types::*;
use crate::social::types::{score_sentiment, SentimentBreakdown, ViralScore};
use std::collections::HashMap;

/// Analyse a set of TikTok videos and return TikTokAnalysis.
pub fn analyse_videos(videos: &[TikTokVideo]) -> TikTokAnalysis {
    let total_videos = videos.len();
    if total_videos == 0 {
        return TikTokAnalysis {
            total_videos: 0,
            avg_plays: 0.0,
            avg_engagement_rate: 0.0,
            top_hashtags: Vec::new(),
            top_creators: Vec::new(),
            viral_videos: Vec::new(),
            trending_music: Vec::new(),
            sentiment: SentimentBreakdown::default(),
        };
    }

    let avg_plays = videos.iter().map(|v| v.plays as f64).sum::<f64>() / total_videos as f64;

    // Engagement rate = (likes + comments + shares) / plays
    let avg_engagement_rate = videos
        .iter()
        .map(|v| {
            if v.plays > 0 {
                (v.likes + v.comments + v.shares) as f64 / v.plays as f64
            } else {
                0.0
            }
        })
        .sum::<f64>()
        / total_videos as f64;

    // Top hashtags
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for video in videos {
        for tag in &video.hashtags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut top_hashtags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    top_hashtags.sort_by(|a, b| b.1.cmp(&a.1));
    top_hashtags.truncate(10);

    // Top creators by total engagement
    let mut creator_eng: HashMap<String, u64> = HashMap::new();
    let mut creator_map: HashMap<String, &TikTokUser> = HashMap::new();
    for video in videos {
        let eng = video.likes + video.comments + video.shares;
        *creator_eng
            .entry(video.author.username.clone())
            .or_insert(0) += eng;
        creator_map.insert(video.author.username.clone(), &video.author);
    }
    let mut creator_sorted: Vec<(String, u64)> = creator_eng.into_iter().collect();
    creator_sorted.sort_by(|a, b| b.1.cmp(&a.1));
    let top_creators: Vec<TikTokUser> = creator_sorted
        .iter()
        .take(5)
        .filter_map(|(u, _)| creator_map.get(u).map(|c| (*c).clone()))
        .collect();

    // Viral videos (top 10% by plays)
    let mut sorted = videos.to_vec();
    sorted.sort_by(|a, b| b.plays.cmp(&a.plays));
    let viral_count = (total_videos / 10).max(3).min(total_videos);
    let viral_videos = sorted.into_iter().take(viral_count).collect();

    // Trending music
    let mut music_counts: HashMap<String, usize> = HashMap::new();
    for video in videos {
        if let Some(ref m) = video.music {
            if !m.is_original {
                *music_counts
                    .entry(format!("{} — {}", m.title, m.artist))
                    .or_insert(0) += 1;
            }
        }
    }
    let mut trending_music: Vec<(String, usize)> = music_counts.into_iter().collect();
    trending_music.sort_by(|a, b| b.1.cmp(&a.1));
    trending_music.truncate(5);

    // Sentiment on descriptions
    let scores: Vec<f64> = videos
        .iter()
        .map(|v| score_sentiment(&v.description))
        .collect();
    let n = scores.len() as f64;
    let pos = scores.iter().filter(|&&s| s > 0.1).count() as f64 / n;
    let neg = scores.iter().filter(|&&s| s < -0.1).count() as f64 / n;
    let compound = scores.iter().sum::<f64>() / n;
    let sentiment = SentimentBreakdown {
        positive_pct: pos,
        negative_pct: neg,
        neutral_pct: 1.0 - pos - neg,
        compound,
    };

    TikTokAnalysis {
        total_videos,
        avg_plays,
        avg_engagement_rate,
        top_hashtags,
        top_creators,
        viral_videos,
        trending_music,
        sentiment,
    }
}

/// Score viral potential of a TikTok video.
pub fn viral_score_video(video: &TikTokVideo) -> ViralScore {
    let plays_norm = (video.plays as f64).ln().max(0.0) / 20.0;
    let eng_rate = if video.plays > 0 {
        (video.likes + video.comments + video.shares) as f64 / video.plays as f64
    } else {
        0.0
    };

    let engagement_velocity = (plays_norm * 0.6 + eng_rate * 10.0 * 0.4).min(1.0);
    let emotional_resonance = score_sentiment(&video.description).abs();
    let shareability = ((video.shares as f64).ln().max(0.0) / 15.0).clamp(0.0, 1.0);
    let novelty = if video.duration_secs < 30 {
        0.9
    } else if video.duration_secs < 60 {
        0.7
    } else {
        0.4
    };
    let trend_alignment = if video.hashtags.iter().any(|h| {
        let h = h.to_lowercase();
        h.contains("fyp") || h.contains("foryou") || h.contains("viral") || h.contains("trending")
    }) {
        0.9
    } else if !video.hashtags.is_empty() {
        0.6
    } else {
        0.3
    };

    ViralScore::compute(
        engagement_velocity,
        emotional_resonance,
        shareability,
        novelty,
        trend_alignment,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_video(desc: &str, plays: u64, likes: u64) -> TikTokVideo {
        TikTokVideo {
            id: "v1".into(),
            url: "https://tiktok.com/@u/video/v1".into(),
            author: TikTokUser {
                username: "creator".into(),
                display_name: "Creator".into(),
                followers: None,
                following: None,
                verified: false,
                bio: String::new(),
            },
            description: desc.into(),
            published: "1700000000".into(),
            duration_secs: 30,
            likes,
            comments: 100,
            shares: 50,
            plays,
            hashtags: vec!["#fyp".into(), "#viral".into()],
            music: None,
            is_duet: false,
            is_stitch: false,
        }
    }

    #[test]
    fn analyse_empty() {
        let a = analyse_videos(&[]);
        assert_eq!(a.total_videos, 0);
    }

    #[test]
    fn analyse_basic() {
        let videos = vec![
            make_video("amazing tutorial rust", 1_000_000, 50_000),
            make_video("great programming content", 500_000, 20_000),
        ];
        let a = analyse_videos(&videos);
        assert_eq!(a.total_videos, 2);
        assert!(a.avg_plays > 0.0);
        assert!(a.avg_engagement_rate > 0.0);
    }

    #[test]
    fn viral_score_range() {
        let v = make_video("amazing", 10_000_000, 500_000);
        let vs = viral_score_video(&v);
        assert!(vs.overall >= 0.0 && vs.overall <= 1.0);
    }

    #[test]
    fn viral_score_fyp_alignment_boost() {
        let mut fyp = make_video("great content", 100_000, 10_000);
        fyp.hashtags = vec!["#fyp".into(), "#viral".into()];
        let mut no_tags = make_video("great content", 100_000, 10_000);
        no_tags.hashtags = Vec::new();
        let fyp_score = viral_score_video(&fyp);
        let no_score = viral_score_video(&no_tags);
        assert!(
            fyp_score.trend_alignment > no_score.trend_alignment,
            "#fyp should boost trend_alignment"
        );
    }

    #[test]
    fn viral_score_short_video_novelty() {
        let mut short = make_video("content", 100_000, 5_000);
        short.duration_secs = 15; // < 30s
        let mut long = make_video("content", 100_000, 5_000);
        long.duration_secs = 120; // > 60s
        let short_score = viral_score_video(&short);
        let long_score = viral_score_video(&long);
        assert!(
            short_score.novelty > long_score.novelty,
            "short video novelty ({}) > long ({})",
            short_score.novelty,
            long_score.novelty
        );
    }

    #[test]
    fn analyse_trending_music_counted() {
        use crate::social::tiktok::types::TikTokMusic;
        let mut v1 = make_video("content", 500_000, 30_000);
        v1.music = Some(TikTokMusic {
            title: "Hit Song".into(),
            artist: "Artist".into(),
            is_original: false,
        });
        let mut v2 = make_video("another", 300_000, 20_000);
        v2.music = Some(TikTokMusic {
            title: "Hit Song".into(),
            artist: "Artist".into(),
            is_original: false,
        });
        let a = analyse_videos(&[v1, v2]);
        assert!(!a.trending_music.is_empty());
        assert_eq!(a.trending_music[0].1, 2, "music should appear twice");
    }

    #[test]
    fn analyse_original_music_excluded() {
        use crate::social::tiktok::types::TikTokMusic;
        let mut v = make_video("content", 500_000, 30_000);
        v.music = Some(TikTokMusic {
            title: "My Original".into(),
            artist: "Me".into(),
            is_original: true, // original should not be in trending
        });
        let a = analyse_videos(&[v]);
        assert!(
            a.trending_music.is_empty(),
            "original music should be excluded"
        );
    }

    #[test]
    fn analyse_top_hashtags_sorted() {
        let mut v1 = make_video("v1", 100_000, 5_000);
        v1.hashtags = vec!["#rust".into(), "#coding".into()];
        let mut v2 = make_video("v2", 200_000, 10_000);
        v2.hashtags = vec!["#rust".into()];
        let a = analyse_videos(&[v1, v2]);
        assert_eq!(a.top_hashtags[0].0, "#rust");
        assert_eq!(a.top_hashtags[0].1, 2);
    }

    #[test]
    fn analyse_engagement_rate_calculation() {
        // 1000 likes + 100 comments + 50 shares = 1150 interactions; 10000 plays → 11.5%
        let v = make_video("test", 10_000, 1_000);
        // make_video sets comments=100, shares=50 → (1000+100+50)/10000 = 0.115
        let a = analyse_videos(&[v]);
        assert!(a.avg_engagement_rate > 0.0 && a.avg_engagement_rate < 1.0);
    }
}
