//! YouTube intelligence — cross-video fact checking, timelines, concepts, learning, teaching.

use crate::youtube::types::*;
use std::collections::HashMap;

// ─── Cross-Video Fact Checking ─────────────────────────────────

/// Cross-check facts across multiple videos using bigram-Jaccard claim matching.
pub fn cross_check_facts(videos: &[VideoAnalysis], min_similarity: f64) -> Vec<FactCheckResult> {
    let facts = extract_all_facts(videos);
    let mut results = Vec::new();
    let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

    for fact in &facts {
        if processed.contains(&fact.claim) {
            continue;
        }

        let mut supporting = Vec::new();
        let mut contradicting = Vec::new();

        for other in &facts {
            if other.video_id == fact.video_id {
                continue;
            }

            let sim = bigram_jaccard_similarity(&fact.claim, &other.claim);
            if sim >= min_similarity {
                if contains_negation_mismatch(&fact.claim, &other.claim) {
                    contradicting.push(other.video_id.clone());
                } else {
                    supporting.push(other.video_id.clone());
                }
            }
        }

        let consensus = if !contradicting.is_empty() && supporting.is_empty() {
            FactConsensus::Contradicted
        } else if !contradicting.is_empty() {
            FactConsensus::Disputed
        } else if supporting.len() >= 2 {
            FactConsensus::Confirmed
        } else {
            FactConsensus::Unverified
        };

        processed.insert(fact.claim.clone());
        results.push(FactCheckResult {
            claim: fact.claim.clone(),
            supporting,
            contradicting,
            consensus,
        });
    }

    results
}

/// Extract factual claims from all video transcripts.
fn extract_all_facts(videos: &[VideoAnalysis]) -> Vec<VideoFact> {
    let mut facts = Vec::new();

    for video in videos {
        if let Some(ref transcript) = video.transcript {
            // Extract sentences that look like factual claims
            for sentence in transcript.full_text.split(['.', '!', '?']) {
                let trimmed = sentence.trim();
                if trimmed.split_whitespace().count() >= 5 && is_factual_claim(trimmed) {
                    facts.push(VideoFact {
                        claim: trimmed.to_string(),
                        video_id: video.metadata.video_id.clone(),
                        timestamp_ms: None,
                        confidence: 0.6,
                    });
                }
            }
        }
    }

    facts
}

/// Check if a sentence is likely a factual claim (not opinion/question).
fn is_factual_claim(text: &str) -> bool {
    let lower = text.to_lowercase();
    // Skip opinions and hedged statements
    let opinion_markers = [
        "i think",
        "i believe",
        "in my opinion",
        "personally",
        "i feel",
    ];
    if opinion_markers.iter().any(|m| lower.contains(m)) {
        return false;
    }
    // Skip questions
    if lower.ends_with('?') {
        return false;
    }
    // Look for factual markers
    let fact_markers = [
        "is",
        "are",
        "was",
        "were",
        "has",
        "have",
        "can",
        "will",
        "does",
        "according to",
    ];
    fact_markers.iter().any(|m| lower.contains(m))
}

/// Check if two claims have a negation mismatch.
fn contains_negation_mismatch(claim_a: &str, claim_b: &str) -> bool {
    let a_lower = claim_a.to_lowercase();
    let b_lower = claim_b.to_lowercase();

    let a_negated = NEGATION_WORDS.iter().any(|n| a_lower.contains(n));
    let b_negated = NEGATION_WORDS.iter().any(|n| b_lower.contains(n));

    // If one is negated and the other isn't, it's a mismatch
    a_negated != b_negated
}

/// Bigram Jaccard similarity.
fn bigram_jaccard_similarity(a: &str, b: &str) -> f64 {
    let a_bi = make_bigrams(&a.to_lowercase());
    let b_bi = make_bigrams(&b.to_lowercase());

    if a_bi.is_empty() && b_bi.is_empty() {
        return 1.0;
    }

    let mut intersection = 0usize;
    let mut union = 0usize;

    let mut all_keys: std::collections::HashSet<&String> = std::collections::HashSet::new();
    all_keys.extend(a_bi.keys());
    all_keys.extend(b_bi.keys());

    for key in all_keys {
        let ca = a_bi.get(key).copied().unwrap_or(0);
        let cb = b_bi.get(key).copied().unwrap_or(0);
        intersection += ca.min(cb);
        union += ca.max(cb);
    }

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

fn make_bigrams(s: &str) -> HashMap<String, usize> {
    let chars: Vec<char> = s.chars().collect();
    let mut bigrams = HashMap::new();
    for window in chars.windows(2) {
        let key = format!("{}{}", window[0], window[1]);
        *bigrams.entry(key).or_insert(0) += 1;
    }
    bigrams
}

// ─── Topic Timeline ────────────────────────────────────────────

/// Build a topic timeline from video publications.
pub fn build_topic_timeline(videos: &[VideoAnalysis]) -> Vec<TimelineEntry> {
    let mut entries: Vec<TimelineEntry> = videos
        .iter()
        .map(|v| {
            let event = if let Some(ref transcript) = v.transcript {
                // Extract first key point from transcript
                transcript
                    .key_moments
                    .iter()
                    .find(|m| {
                        matches!(m.moment_type, MomentType::KeyPoint | MomentType::Conclusion)
                    })
                    .map(|m| m.text.clone())
                    .unwrap_or_else(|| v.metadata.title.clone())
            } else {
                v.metadata.title.clone()
            };

            TimelineEntry {
                date: v.metadata.published.clone(),
                event,
                video_id: v.metadata.video_id.clone(),
                video_title: v.metadata.title.clone(),
            }
        })
        .collect();

    // Sort by date (approximate — using the published text)
    entries.sort_by(|a, b| a.date.cmp(&b.date));
    entries
}

// ─── Key Concept Extraction ────────────────────────────────────

/// Extract key concepts from videos using definition pattern detection + TF-IDF.
pub fn extract_key_concepts(videos: &[VideoAnalysis]) -> Vec<KeyConcept> {
    let stopwords: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut term_freq: HashMap<String, usize> = HashMap::new();
    let mut term_videos: HashMap<String, Vec<String>> = HashMap::new();
    let mut definitions: HashMap<String, String> = HashMap::new();

    for video in videos {
        if let Some(ref transcript) = video.transcript {
            // Extract terms
            let words: Vec<String> = transcript
                .full_text
                .to_lowercase()
                .split_whitespace()
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
                .filter(|w| w.len() > 3 && !stopwords.contains(w.as_str()))
                .collect();

            for word in &words {
                *term_freq.entry(word.clone()).or_insert(0) += 1;
                term_videos.entry(word.clone()).or_default();
                if !term_videos[word].contains(&video.metadata.video_id) {
                    term_videos
                        .get_mut(word)
                        .unwrap()
                        .push(video.metadata.video_id.clone());
                }
            }

            // Extract definitions
            for pattern in DEFINITION_PATTERNS {
                for sentence in transcript.full_text.split('.') {
                    if let Some(pos) = sentence.to_lowercase().find(pattern) {
                        let term = sentence[..pos]
                            .split_whitespace()
                            .last()
                            .unwrap_or("")
                            .to_lowercase();
                        if term.len() > 2 && !stopwords.contains(term.as_str()) {
                            definitions
                                .entry(term)
                                .or_insert_with(|| sentence.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    // Sort by frequency, take top 20
    let mut sorted: Vec<(String, usize)> = term_freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(20)
        .map(|(term, freq)| KeyConcept {
            definition: definitions.get(&term).cloned(),
            source_videos: term_videos.get(&term).cloned().unwrap_or_default(),
            term,
            frequency: freq,
        })
        .collect()
}

// ─── Learning Path ─────────────────────────────────────────────

/// Generate a learning path from videos ordered by difficulty.
pub fn generate_learning_path(videos: &[VideoAnalysis]) -> Vec<LearningStep> {
    let mut steps: Vec<LearningStep> = videos
        .iter()
        .map(|v| {
            let difficulty = estimate_difficulty(v);
            let concepts = extract_video_concepts(v);

            LearningStep {
                order: 0, // will be set after sorting
                video_id: v.metadata.video_id.clone(),
                title: v.metadata.title.clone(),
                difficulty,
                key_concepts: concepts,
                estimated_minutes: v.metadata.duration_secs / 60,
            }
        })
        .collect();

    // Sort by difficulty level
    steps.sort_by(|a, b| a.difficulty.cmp(&b.difficulty));

    // Assign order
    for (i, step) in steps.iter_mut().enumerate() {
        step.order = i + 1;
    }

    steps
}

/// Estimate video difficulty based on word complexity and content.
fn estimate_difficulty(video: &VideoAnalysis) -> DifficultyLevel {
    let mut complexity_score = 0.0f64;

    // Title complexity
    let title_words = video.metadata.title.split_whitespace().count();
    if title_words > 10 {
        complexity_score += 0.2;
    }

    // Transcript word complexity
    if let Some(ref transcript) = video.transcript {
        let avg_word_len = if transcript.word_count > 0 {
            transcript.full_text.len() as f64 / transcript.word_count as f64
        } else {
            4.0
        };
        // Higher avg word length = more complex
        complexity_score += (avg_word_len - 4.0).max(0.0) * 0.1;

        // More key moments = more structured = potentially more advanced
        complexity_score += (transcript.key_moments.len() as f64 * 0.02).min(0.3);
    }

    // Duration (longer = often more advanced)
    let dur_min = video.metadata.duration_secs as f64 / 60.0;
    if dur_min > 30.0 {
        complexity_score += 0.2;
    } else if dur_min > 15.0 {
        complexity_score += 0.1;
    }

    // Keywords indicating difficulty
    let lower_title = video.metadata.title.to_lowercase();
    let beginner_words = [
        "beginner",
        "introduction",
        "basics",
        "101",
        "getting started",
        "first",
    ];
    let advanced_words = [
        "advanced",
        "deep dive",
        "internals",
        "architecture",
        "expert",
        "master",
    ];

    if beginner_words.iter().any(|w| lower_title.contains(w)) {
        complexity_score -= 0.3;
    }
    if advanced_words.iter().any(|w| lower_title.contains(w)) {
        complexity_score += 0.3;
    }

    match complexity_score {
        s if s < 0.2 => DifficultyLevel::Beginner,
        s if s < 0.5 => DifficultyLevel::Intermediate,
        s if s < 0.8 => DifficultyLevel::Advanced,
        _ => DifficultyLevel::Expert,
    }
}

/// Extract main concepts from a single video.
fn extract_video_concepts(video: &VideoAnalysis) -> Vec<String> {
    let stopwords: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut freq: HashMap<String, usize> = HashMap::new();

    // From title
    for word in video.metadata.title.to_lowercase().split_whitespace() {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();
        if clean.len() > 3 && !stopwords.contains(clean.as_str()) {
            *freq.entry(clean).or_insert(0) += 3;
        }
    }

    // From keywords
    for kw in &video.metadata.keywords {
        let clean = kw.to_lowercase();
        if !stopwords.contains(clean.as_str()) {
            *freq.entry(clean).or_insert(0) += 2;
        }
    }

    let mut sorted: Vec<(String, usize)> = freq.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(5).map(|(t, _)| t).collect()
}

// ─── Teaching Mode ─────────────────────────────────────────────

/// Generate teaching content (flashcards + quiz) from video analyses.
pub fn generate_teaching(videos: &[VideoAnalysis]) -> TeachingContent {
    let concepts = extract_key_concepts(videos);
    let summary = generate_summary(videos);
    let flashcards = generate_flashcards(&concepts);
    let quiz_questions = generate_quiz(&concepts, videos);

    TeachingContent {
        summary,
        flashcards,
        quiz_questions,
    }
}

/// Generate a summary from video analyses.
fn generate_summary(videos: &[VideoAnalysis]) -> String {
    let mut parts = Vec::new();
    for video in videos {
        if let Some(ref transcript) = video.transcript {
            // Take first key point or first 200 chars of transcript
            let excerpt = transcript
                .key_moments
                .iter()
                .find(|m| matches!(m.moment_type, MomentType::KeyPoint | MomentType::Conclusion))
                .map(|m| m.text.clone())
                .unwrap_or_else(|| transcript.full_text.chars().take(200).collect::<String>());
            parts.push(format!("**{}**: {}", video.metadata.title, excerpt));
        } else {
            parts.push(format!(
                "**{}**: {}",
                video.metadata.title,
                video
                    .metadata
                    .description
                    .chars()
                    .take(100)
                    .collect::<String>()
            ));
        }
    }
    parts.join("\n\n")
}

/// Generate flashcards from key concepts.
fn generate_flashcards(concepts: &[KeyConcept]) -> Vec<Flashcard> {
    concepts
        .iter()
        .filter(|c| c.definition.is_some())
        .take(10)
        .map(|c| Flashcard {
            front: format!("What is {}?", c.term),
            back: c.definition.clone().unwrap_or_default(),
        })
        .collect()
}

/// Generate quiz questions from concepts and video content.
fn generate_quiz(concepts: &[KeyConcept], _videos: &[VideoAnalysis]) -> Vec<QuizQuestion> {
    concepts
        .iter()
        .filter(|c| c.definition.is_some() && c.frequency > 2)
        .take(5)
        .map(|concept| {
            let def = concept.definition.clone().unwrap_or_default();
            QuizQuestion {
                question: format!("Which of the following best describes '{}'?", concept.term),
                options: vec![
                    def.clone(),
                    format!("A type of data structure used in {}", concept.term),
                    format!("An algorithm for processing {}", concept.term),
                    "None of the above".to_string(),
                ],
                correct_index: 0,
                explanation: format!("The correct definition is: {def}"),
            }
        })
        .collect()
}

/// Analyze viral content patterns from videos.
pub fn analyze_viral_patterns(videos: &[VideoAnalysis]) -> Vec<ContentIdea> {
    let mut ideas = Vec::new();

    // Find common themes in high-engagement videos
    let high_engagement: Vec<&VideoAnalysis> = videos
        .iter()
        .filter(|v| {
            let like_ratio = if v.metadata.view_count > 0 {
                v.metadata.like_count as f64 / v.metadata.view_count as f64
            } else {
                0.0
            };
            like_ratio > 0.03 || v.metadata.view_count > 100_000
        })
        .collect();

    if high_engagement.is_empty() {
        return ideas;
    }

    // Extract common keywords from viral videos
    let stopwords: std::collections::HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut keyword_freq: HashMap<String, usize> = HashMap::new();
    for video in &high_engagement {
        for kw in &video.metadata.keywords {
            let lower = kw.to_lowercase();
            if !stopwords.contains(lower.as_str()) {
                *keyword_freq.entry(lower).or_insert(0) += 1;
            }
        }
        // Title words
        for word in video.metadata.title.to_lowercase().split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_string();
            if clean.len() > 3 && !stopwords.contains(clean.as_str()) {
                *keyword_freq.entry(clean).or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<(String, usize)> =
        keyword_freq.into_iter().filter(|(_, v)| *v >= 2).collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    for (topic, freq) in sorted.into_iter().take(5) {
        // Analyze duration sweet spot
        let avg_duration: u64 = high_engagement
            .iter()
            .map(|v| v.metadata.duration_secs)
            .sum::<u64>()
            / high_engagement.len().max(1) as u64;

        ideas.push(ContentIdea {
            topic: topic.clone(),
            angle: format!(
                "Trending topic '{}' appears in {} viral videos",
                topic, freq
            ),
            suggested_duration_secs: avg_duration,
            keyword_density: freq,
            reference_videos: high_engagement
                .iter()
                .filter(|v| {
                    v.metadata.title.to_lowercase().contains(&topic)
                        || v.metadata
                            .keywords
                            .iter()
                            .any(|k| k.to_lowercase() == topic)
                })
                .map(|v| v.metadata.video_id.clone())
                .collect(),
        });
    }

    ideas
}

/// Content idea generated from viral analysis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentIdea {
    pub topic: String,
    pub angle: String,
    pub suggested_duration_secs: u64,
    pub keyword_density: usize,
    pub reference_videos: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::youtube::metadata::score_channel_credibility;

    fn make_video(id: &str, title: &str, transcript_text: Option<&str>) -> VideoAnalysis {
        let channel = ChannelInfo {
            name: "TestCh".into(),
            id: "UC1".into(),
            subscriber_count: Some(10_000),
            verified: false,
        };
        VideoAnalysis {
            metadata: VideoMetadata {
                video_id: id.into(),
                title: title.into(),
                description: "Test description".into(),
                channel: channel.clone(),
                duration_secs: 600,
                view_count: 10_000,
                like_count: 500,
                published: "2 weeks ago".into(),
                keywords: vec!["rust".into(), "programming".into()],
                chapters: vec![],
                links: vec![],
                thumbnail_url: None,
                is_live: false,
            },
            transcript: transcript_text.map(|text| EnhancedTranscript {
                video_id: id.into(),
                language: "en".into(),
                entries: vec![],
                speakers: vec![],
                key_moments: vec![],
                full_text: text.into(),
                word_count: text.split_whitespace().count(),
                source: TranscriptSource::YouTubeTimedtext,
                quality_score: 1.0,
            }),
            comments: None,
            credibility: score_channel_credibility(&channel),
        }
    }

    #[test]
    fn cross_check_basic() {
        let videos = vec![
            make_video(
                "v1",
                "Rust is fast",
                Some("Rust is a systems programming language that is very fast and memory safe"),
            ),
            make_video(
                "v2",
                "Rust performance",
                Some("Rust is a systems programming language known for being fast and safe"),
            ),
            make_video(
                "v3",
                "Python vs Rust",
                Some("Python is not a systems programming language but Rust is"),
            ),
        ];
        let results = cross_check_facts(&videos, 0.4);
        assert!(!results.is_empty());
    }

    #[test]
    fn is_factual_claim_check() {
        assert!(is_factual_claim("Rust is a systems programming language"));
        assert!(!is_factual_claim("I think Rust is great"));
        assert!(!is_factual_claim("Is Rust good?"));
    }

    #[test]
    fn negation_mismatch_detection() {
        assert!(contains_negation_mismatch(
            "Rust is memory safe",
            "Rust is not memory safe"
        ));
        assert!(!contains_negation_mismatch(
            "Rust is fast",
            "Rust is very fast"
        ));
    }

    #[test]
    fn topic_timeline_basic() {
        let videos = vec![
            make_video("v1", "Early Topic", None),
            make_video("v2", "Later Topic", None),
        ];
        let timeline = build_topic_timeline(&videos);
        assert_eq!(timeline.len(), 2);
    }

    #[test]
    fn key_concepts_extraction() {
        let videos = vec![
            make_video("v1", "Rust Tutorial", Some("async programming in rust allows you to write concurrent code async programming is powerful async runtime tokio")),
        ];
        let concepts = extract_key_concepts(&videos);
        assert!(!concepts.is_empty());
        assert!(concepts
            .iter()
            .any(|c| c.term.contains("async") || c.term.contains("rust")));
    }

    #[test]
    fn learning_path_ordering() {
        let videos = vec![
            make_video(
                "v1",
                "Advanced Rust Internals Deep Dive",
                Some("Complex topic about ownership and borrowing"),
            ),
            make_video(
                "v2",
                "Rust for Beginners - Getting Started",
                Some("Hello world in rust"),
            ),
        ];
        let path = generate_learning_path(&videos);
        assert_eq!(path.len(), 2);
        assert!(path[0].difficulty <= path[1].difficulty);
    }

    #[test]
    fn teaching_generation() {
        let videos = vec![make_video(
            "v1",
            "Rust Closures",
            Some("A closure is defined as a function that captures variables from its environment"),
        )];
        let teaching = generate_teaching(&videos);
        assert!(!teaching.summary.is_empty());
    }

    #[test]
    fn difficulty_estimation() {
        let beginner = make_video("v1", "Introduction to Rust - Getting Started Basics", None);
        let advanced = make_video("v2", "Advanced Rust Architecture Deep Dive Internals", None);
        assert!(estimate_difficulty(&beginner) < estimate_difficulty(&advanced));
    }

    #[test]
    fn viral_patterns_empty() {
        let videos: Vec<VideoAnalysis> = vec![];
        let ideas = analyze_viral_patterns(&videos);
        assert!(ideas.is_empty());
    }
}
