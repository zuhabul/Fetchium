//! PII redaction engine (PRD §36.2).
//!
//! Scans output text and replaces recognised PII patterns with redaction markers.

use once_cell::sync::Lazy;
use regex::Regex;

/// Compiled PII patterns: (regex, replacement_label).
static PII_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
    vec![
        // Email addresses
        (
            Regex::new(r"(?i)\b[a-z0-9._%+\-]+@[a-z0-9.\-]+\.[a-z]{2,}\b").unwrap(),
            "[EMAIL]",
        ),
        // US phone numbers
        (
            Regex::new(r"\b\d{3}[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap(),
            "[PHONE]",
        ),
        // US SSNs
        (Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(), "[SSN]"),
        // Credit card numbers (16 digits, optionally grouped)
        (
            Regex::new(r"\b(?:\d{4}[\s\-]?){3}\d{4}\b").unwrap(),
            "[CC]",
        ),
        // IPv4 addresses
        (
            Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
            "[IP]",
        ),
        // API keys / tokens (hex 32+ chars)
        (Regex::new(r"\b[0-9a-fA-F]{32,}\b").unwrap(), "[TOKEN]"),
    ]
});

/// Redact all recognised PII from `text`.
///
/// Returns the redacted string; leaves non-PII text unchanged.
pub fn redact_pii(text: &str) -> String {
    let mut result = text.to_string();
    for (re, label) in PII_PATTERNS.iter() {
        result = re.replace_all(&result, *label).into_owned();
    }
    result
}

/// Report the number of PII matches found (without redacting).
pub fn count_pii(text: &str) -> usize {
    PII_PATTERNS
        .iter()
        .map(|(re, _)| re.find_iter(text).count())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_email() {
        let out = redact_pii("Contact: user@example.com for info.");
        assert!(out.contains("[EMAIL]"), "got: {out}");
        assert!(!out.contains("user@example.com"));
    }

    #[test]
    fn redacts_ssn() {
        let out = redact_pii("SSN: 123-45-6789");
        assert!(out.contains("[SSN]"), "got: {out}");
    }

    #[test]
    fn plain_text_unchanged() {
        let text = "Rust is a systems programming language.";
        assert_eq!(redact_pii(text), text);
    }
}
