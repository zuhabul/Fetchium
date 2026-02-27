//! Lightweight sentiment analysis using AFINN-111 lexicon.
//!
//! Fast (<1ms per text), zero external dependencies, runs on CPU.

use serde::{Deserialize, Serialize};

/// Sentiment label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

impl std::fmt::Display for Sentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sentiment::Positive => write!(f, "Positive"),
            Sentiment::Negative => write!(f, "Negative"),
            Sentiment::Neutral => write!(f, "Neutral"),
            Sentiment::Mixed => write!(f, "Mixed"),
        }
    }
}

/// Result of sentiment analysis on a text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    /// Score from -1.0 (very negative) to 1.0 (very positive)
    pub score: f64,
    /// Categorical label
    pub label: Sentiment,
    /// Confidence 0.0-1.0
    pub confidence: f64,
    /// Number of sentiment-bearing words found
    pub word_count: usize,
}

/// Analyze sentiment of text using AFINN-111 word list.
pub fn analyze_sentiment(text: &str) -> SentimentResult {
    let lower = text.to_lowercase();
    let tokens: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| w.len() > 1)
        .collect();

    let mut total_score: i32 = 0;
    let mut word_count: usize = 0;
    let mut positive_count: usize = 0;
    let mut negative_count: usize = 0;

    for word in &tokens {
        if let Some(score) = afinn_score(word) {
            total_score += score;
            word_count += 1;
            if score > 0 {
                positive_count += 1;
            } else if score < 0 {
                negative_count += 1;
            }
        }
    }

    if word_count == 0 {
        return SentimentResult {
            score: 0.0,
            label: Sentiment::Neutral,
            confidence: 0.5,
            word_count: 0,
        };
    }

    // Normalize to -1.0..1.0 range
    let avg = total_score as f64 / word_count as f64;
    let score = (avg / 5.0).clamp(-1.0, 1.0);

    let label = if positive_count > 0
        && negative_count > 0
        && (positive_count as f64 / negative_count as f64) < 2.0
    {
        Sentiment::Mixed
    } else if score > 0.1 {
        Sentiment::Positive
    } else if score < -0.1 {
        Sentiment::Negative
    } else {
        Sentiment::Neutral
    };

    let confidence = (word_count as f64 / tokens.len().max(1) as f64).min(1.0);

    SentimentResult {
        score,
        label,
        confidence,
        word_count,
    }
}

/// AFINN-111 word scores (subset of most common words).
/// Full list: 2477 words scored -5 to +5.
fn afinn_score(word: &str) -> Option<i32> {
    // Top ~200 most impactful sentiment words from AFINN-111
    Some(match word {
        // Very positive (+4, +5)
        "outstanding" | "superb" | "breathtaking" => 5,
        "excellent" | "amazing" | "awesome" | "fantastic" | "wonderful" | "brilliant" => 4,
        "thrilled" | "magnificent" | "incredible" | "exceptional" => 4,
        // Positive (+2, +3)
        "great" | "love" | "loved" | "loving" | "happy" | "good" | "best" => 3,
        "beautiful" | "perfect" | "exciting" | "impressive" | "inspired" => 3,
        "recommend" | "recommended" | "praise" | "praised" => 3,
        "like" | "liked" | "nice" | "enjoy" | "enjoyed" | "fun" | "cool" => 2,
        "useful" | "helpful" | "interesting" | "easy" | "fast" | "clean" => 2,
        "reliable" | "stable" | "efficient" | "innovative" | "powerful" => 2,
        "improved" | "improve" | "improving" | "pleased" | "glad" => 2,
        "thank" | "thanks" | "grateful" | "appreciate" | "win" | "won" => 2,
        // Mildly positive (+1)
        "ok" | "okay" | "fine" | "decent" | "fair" | "reasonable" => 1,
        "working" | "works" | "supported" | "active" | "popular" => 1,
        // Mildly negative (-1)
        "lacking" | "limited" | "mediocre" | "average" | "meh" => -1,
        "boring" | "old" | "outdated" | "basic" => -1,
        "confusing" | "complicated" | "complex" | "difficult" => -1,
        // Negative (-2, -3)
        "bad" | "poor" | "broken" | "failed" | "fail" | "failing" => -3,
        "hate" | "hated" | "worst" | "terrible" | "horrible" | "awful" => -3,
        "ugly" | "disgusting" | "angry" | "frustrated" | "annoying" => -3,
        "bug" | "bugs" | "buggy" | "crash" | "crashed" | "error" => -2,
        "slow" | "laggy" | "unusable" | "painful" | "waste" => -2,
        "disappointed" | "disappointing" | "problem" | "problems" | "issue" | "issues" => -2,
        "wrong" | "worse" | "sadly" | "unfortunately" | "regret" => -2,
        // Very negative (-4, -5)
        "catastrophic" | "devastating" | "malicious" | "fraud" | "scam" => -5,
        "toxic" | "abusive" | "dangerous" | "disaster" | "nightmare" => -4,
        "critical" | "severe" | "exploit" | "vulnerability" => -4,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_text() {
        let result = analyze_sentiment("This is amazing and wonderful, I love it!");
        assert!(result.score > 0.0);
        assert_eq!(result.label, Sentiment::Positive);
    }

    #[test]
    fn negative_text() {
        let result = analyze_sentiment("This is terrible and horrible, I hate it");
        assert!(result.score < 0.0);
        assert_eq!(result.label, Sentiment::Negative);
    }

    #[test]
    fn neutral_text() {
        let result = analyze_sentiment("The function returns a value");
        assert_eq!(result.label, Sentiment::Neutral);
    }

    #[test]
    fn mixed_text() {
        let result = analyze_sentiment("I love the features but hate the bugs and crashes");
        assert_eq!(result.label, Sentiment::Mixed);
    }
}
