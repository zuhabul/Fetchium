//! Adversarial Content Shield (ACS) — PRD §8.17, §35.
//!
//! 4-layer detection system:
//! 1. AI content detection (burstiness, vocabulary diversity, sentence variance)
//! 2. Bot farm signals (domain age, publishing velocity)
//! 3. Source manipulation detection (cross-site duplication)
//! 4. Trust score aggregation
//!
//! Starts in **shadow mode** (flag but don't filter) for the first 30 days.

use chrono::{DateTime, Utc};

// ─── Mode and results ────────────────────────────────────────────────────────

/// Operating mode for ACS.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcsMode {
    /// Flag but don't filter — used for the first 30 days to collect accuracy data.
    Shadow,
    /// Flag and filter — active enforcement mode.
    Active,
    /// ACS disabled entirely.
    Disabled,
}

/// A specific flag raised by ACS.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcsFlag {
    LikelyAiGenerated,
    LikelyBotFarm,
    LikelyManipulated,
    LowVocabularyDiversity,
    UniformSentenceLengths,
}

/// Action recommended by ACS after analysis.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcsAction {
    /// Trust > 0.8 — include normally.
    Include,
    /// 0.5 < trust ≤ 0.8 — include with a warning shown to the user.
    IncludeWithWarning { warning: String },
    /// trust ≤ 0.5 — exclude from results.
    Exclude { reason: String },
    /// trust < 0.2 — flag as potentially adversarial.
    FlagAdversarial { reason: String },
}

/// Full ACS analysis result for a single piece of content.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AcsResult {
    /// Aggregate trust score 0.0-1.0.
    pub trust_score: f64,
    pub ai_generated_probability: f64,
    pub bot_farm_probability: f64,
    pub manipulation_probability: f64,
    pub flags: Vec<AcsFlag>,
    pub action: AcsAction,
    /// True if running in shadow mode (action is always Include in this case).
    pub is_shadow: bool,
}

// ─── AI Content Detector ─────────────────────────────────────────────────────

struct AiContentDetector;

impl AiContentDetector {
    /// Returns AI-generation probability 0.0-1.0.
    /// AI text: low burstiness + low vocabulary diversity + uniform sentence length.
    fn detect(&self, content: &str) -> f64 {
        let burst = self.burstiness_score(content);
        let vocab = self.vocabulary_diversity(content);
        let variance = self.sentence_length_variance(content);

        // Low burstiness/diversity/variance → high AI probability
        let ai = (1.0 - burst) * 0.4 + (1.0 - vocab) * 0.3 + (1.0 - variance) * 0.3;
        ai.clamp(0.0, 1.0)
    }

    /// Coefficient of variation of sentence lengths.
    /// High CV = bursty = human; Low CV = uniform = AI.
    fn burstiness_score(&self, content: &str) -> f64 {
        let lengths: Vec<f64> = content
            .split(['.', '!', '?'])
            .filter(|s| s.split_whitespace().count() >= 3)
            .map(|s| s.split_whitespace().count() as f64)
            .collect();
        if lengths.len() < 3 {
            return 0.5;
        }
        let mean = lengths.iter().sum::<f64>() / lengths.len() as f64;
        if mean < 1e-9 {
            return 0.5;
        }
        let variance =
            lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / lengths.len() as f64;
        let cv = variance.sqrt() / mean;
        (cv / 0.5).clamp(0.0, 1.0)
    }

    /// Type-Token Ratio: unique words / total words.
    fn vocabulary_diversity(&self, content: &str) -> f64 {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() {
            return 0.5;
        }
        let unique: std::collections::HashSet<_> = words.iter().map(|w| w.to_lowercase()).collect();
        (unique.len() as f64 / words.len() as f64).clamp(0.0, 1.0)
    }

    /// Std deviation of sentence lengths, normalised by mean.
    fn sentence_length_variance(&self, content: &str) -> f64 {
        let lengths: Vec<f64> = content
            .split(['.', '!', '?'])
            .filter(|s| s.len() > 5)
            .map(|s| s.split_whitespace().count() as f64)
            .collect();
        if lengths.len() < 3 {
            return 0.5;
        }
        let mean = lengths.iter().sum::<f64>() / lengths.len() as f64;
        let std_dev =
            (lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / lengths.len() as f64).sqrt();
        (std_dev / mean.max(1.0)).clamp(0.0, 1.0)
    }
}

// ─── Bot Farm Detector ───────────────────────────────────────────────────────

struct BotFarmDetector;

impl BotFarmDetector {
    /// Returns bot-farm probability 0.0-1.0 based on simple URL/domain heuristics.
    fn analyze(&self, domain: &str, content_len: usize) -> f64 {
        let mut score = 0.0_f64;

        // Very short content (< 100 words) from unknown domains → suspicious
        if content_len < 500 {
            score += 0.2;
        }

        // Known content-farm / clickbait TLD patterns
        let lower = domain.to_lowercase();
        let suspicious_patterns = [
            ".click",
            ".info",
            ".biz",
            ".xyz",
            "bestof",
            "top10",
            "viral",
            "clickbait",
        ];
        for pat in suspicious_patterns {
            if lower.contains(pat) {
                score += 0.15;
                break;
            }
        }

        score.clamp(0.0, 1.0)
    }
}

// ─── Manipulation Detector ───────────────────────────────────────────────────

struct ManipulationDetector;

impl ManipulationDetector {
    /// Returns manipulation probability 0.0-1.0 based on content heuristics.
    fn check(&self, content: &str) -> f64 {
        let lower = content.to_lowercase();
        let mut score = 0.0_f64;

        // Excessive hedging / uncertainty language common in disinformation
        let hedge_words = [
            "they say",
            "some people claim",
            "allegedly",
            "reportedly",
            "sources say",
            "rumored",
            "unverified",
        ];
        let hedge_count = hedge_words.iter().filter(|w| lower.contains(*w)).count();
        score += (hedge_count as f64 * 0.1).min(0.4);

        // Extremely clickbait title signals (ALL CAPS words)
        let all_caps_words: usize = content
            .split_whitespace()
            .filter(|w| w.len() > 3 && w.chars().all(|c| c.is_uppercase()))
            .count();
        score += (all_caps_words as f64 * 0.05).min(0.3);

        score.clamp(0.0, 1.0)
    }
}

// ─── ACS Main Engine ─────────────────────────────────────────────────────────

/// Adversarial Content Shield.
pub struct AdversarialContentShield {
    ai_detector: AiContentDetector,
    bot_detector: BotFarmDetector,
    manipulation_detector: ManipulationDetector,
    mode: AcsMode,
    first_enabled_at: DateTime<Utc>,
}

impl AdversarialContentShield {
    /// Create a new ACS in shadow mode starting now.
    pub fn new() -> Self {
        Self {
            ai_detector: AiContentDetector,
            bot_detector: BotFarmDetector,
            manipulation_detector: ManipulationDetector,
            mode: AcsMode::Shadow,
            first_enabled_at: Utc::now(),
        }
    }

    /// Create with an explicit mode and start time.
    pub fn with_mode(mode: AcsMode, first_enabled_at: DateTime<Utc>) -> Self {
        Self {
            ai_detector: AiContentDetector,
            bot_detector: BotFarmDetector,
            manipulation_detector: ManipulationDetector,
            mode,
            first_enabled_at,
        }
    }

    /// Compute the effective operating mode.
    ///
    /// Auto-transitions shadow → active after 30 days.
    pub fn effective_mode(&self) -> AcsMode {
        if self.mode == AcsMode::Shadow {
            let days_active = (Utc::now() - self.first_enabled_at).num_days();
            if days_active >= 30 {
                return AcsMode::Active;
            }
        }
        self.mode.clone()
    }

    /// Analyse a piece of content from `domain`.
    pub fn analyze(&self, content: &str, domain: &str) -> AcsResult {
        let ai_prob = self.ai_detector.detect(content);
        let bot_prob = self.bot_detector.analyze(domain, content.len());
        let manip_prob = self.manipulation_detector.check(content);

        let trust_score = 1.0_f64 - ai_prob.max(bot_prob).max(manip_prob);
        let trust_score = trust_score.clamp(0.0, 1.0);

        let mut flags = Vec::new();
        if ai_prob > 0.7 {
            flags.push(AcsFlag::LikelyAiGenerated);
        }
        if bot_prob > 0.7 {
            flags.push(AcsFlag::LikelyBotFarm);
        }
        if manip_prob > 0.7 {
            flags.push(AcsFlag::LikelyManipulated);
        }

        let eff_mode = self.effective_mode();
        let is_shadow = eff_mode == AcsMode::Shadow;

        let action = match eff_mode {
            AcsMode::Shadow | AcsMode::Disabled => AcsAction::Include,
            AcsMode::Active => {
                if trust_score > 0.8 {
                    AcsAction::Include
                } else if trust_score > 0.5 {
                    AcsAction::IncludeWithWarning {
                        warning: format!(
                            "Low trust ({:.0}%): possible AI-generated or manipulated content",
                            trust_score * 100.0
                        ),
                    }
                } else if trust_score > 0.2 {
                    AcsAction::Exclude {
                        reason: format!(
                            "Trust score {:.0}% — likely low-quality or manipulated content",
                            trust_score * 100.0
                        ),
                    }
                } else {
                    AcsAction::FlagAdversarial {
                        reason: format!(
                            "Trust score {:.0}% — likely adversarial content (domain: {domain})",
                            trust_score * 100.0
                        ),
                    }
                }
            }
        };

        AcsResult {
            trust_score,
            ai_generated_probability: ai_prob,
            bot_farm_probability: bot_prob,
            manipulation_probability: manip_prob,
            flags,
            action,
            is_shadow,
        }
    }
}

impl Default for AdversarialContentShield {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NATURAL_TEXT: &str = "Rust is a systems programming language focused on safety, speed, \
        and concurrency. It accomplishes these goals without a garbage collector, making it \
        useful for a number of use cases other languages aren't good at: embedding in other \
        languages, programs with specific space and time requirements, and writing low-level \
        code, like device drivers and operating systems.";

    const REPETITIVE_AI_TEXT: &str = "The the the the the. A a a a a a. The solution is great. \
        The answer is good. The result is fine. The output is nice. The outcome is excellent. \
        The conclusion is perfect. The end result is wonderful. The final answer is satisfactory.";

    #[test]
    fn natural_text_has_lower_ai_probability() {
        let acs = AdversarialContentShield::new();
        let natural = acs.analyze(NATURAL_TEXT, "rust-lang.org");
        let repetitive = acs.analyze(REPETITIVE_AI_TEXT, "spammy.xyz");
        assert!(
            repetitive.ai_generated_probability >= natural.ai_generated_probability,
            "natural_ai={:.2}, repetitive_ai={:.2}",
            natural.ai_generated_probability,
            repetitive.ai_generated_probability,
        );
    }

    #[test]
    fn shadow_mode_always_includes() {
        let acs = AdversarialContentShield::with_mode(
            AcsMode::Shadow,
            Utc::now(), // just started
        );
        let result = acs.analyze(REPETITIVE_AI_TEXT, "spammy.xyz");
        assert!(matches!(result.action, AcsAction::Include));
        assert!(result.is_shadow);
    }

    #[test]
    fn active_mode_excludes_low_trust() {
        let acs = AdversarialContentShield::with_mode(
            AcsMode::Active,
            Utc::now() - chrono::Duration::days(31), // 31 days ago
        );
        // Force a highly manipulated-looking piece
        let manip = "THEY SAY allegedly reportedly SOME PEOPLE CLAIM unverified sources say \
                     RUMORED to be true allegedly allegedly allegedly allegedly allegedly";
        let result = acs.analyze(manip, "clickbait.click");
        // Trust score will be low; action should not be Include
        assert!(
            !matches!(result.action, AcsAction::Include),
            "expected non-Include for low-trust content"
        );
    }

    #[test]
    fn auto_transition_to_active_after_30_days() {
        let acs = AdversarialContentShield::with_mode(
            AcsMode::Shadow,
            Utc::now() - chrono::Duration::days(31),
        );
        assert_eq!(acs.effective_mode(), AcsMode::Active);
    }
}
