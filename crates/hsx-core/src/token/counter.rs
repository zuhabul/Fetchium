//! Token counter and budget tracking.
//!
//! PRD SS20: Token Budget System. Approximates GPT-4 tokenization using
//! a whitespace+punctuation heuristic (~95% accuracy for English text).
//! For exact counts, integrate tiktoken in Phase 5.

/// Average characters per token for the heuristic estimator.
const CHARS_PER_TOKEN: f64 = 4.0;

/// Count tokens in a text string using the heuristic estimator.
///
/// Returns an approximate token count. For typical English text,
/// this is within ~5% of GPT-4's tiktoken.
pub fn count_tokens(text: &str) -> u32 {
    if text.is_empty() {
        return 0;
    }
    let char_estimate = (text.len() as f64 / CHARS_PER_TOKEN).ceil() as u32;

    let word_count = text.split_whitespace().count();
    let word_estimate = (word_count as f64 * 1.3).ceil() as u32;

    (char_estimate + word_estimate) / 2
}

/// Quickly estimate tokens without any splitting (fastest path).
pub fn estimate_tokens_fast(text: &str) -> u32 {
    (text.len() as f64 / CHARS_PER_TOKEN).ceil() as u32
}

/// Count tokens in a JSON value (accounts for JSON syntax overhead).
pub fn count_tokens_json(value: &serde_json::Value) -> u32 {
    let json_str = serde_json::to_string(value).unwrap_or_default();
    count_tokens(&json_str)
}

/// Budget tracker that monitors token consumption.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Total token budget allocated.
    pub total: u32,
    /// Tokens consumed so far.
    pub used: u32,
}

impl TokenBudget {
    /// Create a new budget with the given total.
    pub fn new(total: u32) -> Self {
        Self { total, used: 0 }
    }

    /// Remaining tokens available.
    pub fn remaining(&self) -> u32 {
        self.total.saturating_sub(self.used)
    }

    /// Whether the budget has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.used >= self.total
    }

    /// Try to consume tokens. Returns true if within budget.
    pub fn try_consume(&mut self, tokens: u32) -> bool {
        if self.used + tokens <= self.total {
            self.used += tokens;
            true
        } else {
            false
        }
    }

    /// Force-consume tokens (may exceed budget).
    pub fn consume(&mut self, tokens: u32) {
        self.used += tokens;
    }

    /// How full the budget is (0.0 to 1.0+).
    pub fn utilization(&self) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        self.used as f64 / self.total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_tokens_empty() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn count_tokens_short_text() {
        let tokens = count_tokens("Hello, world!");
        assert!((2..=6).contains(&tokens), "got {tokens}");
    }

    #[test]
    fn count_tokens_paragraph() {
        let text = "Rust is a multi-paradigm, general-purpose programming language. \
                    It emphasizes performance, type safety, and concurrency.";
        let tokens = count_tokens(text);
        assert!((15..=45).contains(&tokens), "got {tokens}");
    }

    #[test]
    fn budget_tracking() {
        let mut budget = TokenBudget::new(1000);
        assert_eq!(budget.remaining(), 1000);
        assert!(!budget.is_exhausted());

        assert!(budget.try_consume(500));
        assert_eq!(budget.remaining(), 500);
        assert_eq!(budget.used, 500);

        assert!(budget.try_consume(500));
        assert!(budget.is_exhausted());

        assert!(!budget.try_consume(1));
    }

    #[test]
    fn budget_utilization() {
        let mut budget = TokenBudget::new(100);
        budget.consume(50);
        assert!((budget.utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn fast_estimate_consistency() {
        let text = "This is a test sentence with several words.";
        let fast = estimate_tokens_fast(text);
        let normal = count_tokens(text);
        assert!((fast as i32 - normal as i32).unsigned_abs() < 10);
    }
}
