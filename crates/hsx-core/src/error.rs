//! Error types for HyperSearchX (PRD §44).
//!
//! Structured errors with retry info, suggested actions, and fallback alternatives.

use serde::{Deserialize, Serialize};

/// Categorized error type for structured error handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    NetworkTimeout,
    DnsFailure,
    Http403,
    Http429,
    Http5xx,
    AntiBot,
    Paywall,
    ContentNotFound,
    ExtractionFailed,
    BrowserCrash,
    AiUnavailable,
    ValidationFailed,
    BudgetExceeded,
    ConfigError,
    CacheError,
    IndexError,
    ParseError,
    IoError,
    Unknown,
}

/// Structured error with context for automated fallback handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredError {
    pub kind: ErrorKind,
    pub retryable: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    pub suggested_action: String,
    #[serde(default)]
    pub alternatives: Vec<String>,
}

impl std::fmt::Display for StructuredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)?;
        if let Some(url) = &self.source_url {
            write!(f, " (url: {url})")?;
        }
        Ok(())
    }
}

impl std::error::Error for StructuredError {}

/// Main error type for hsx-core.
#[derive(Debug, thiserror::Error)]
pub enum HsxError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Extraction error: {0}")]
    Extraction(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Token budget exceeded: used {used}, budget {budget}")]
    BudgetExceeded { used: u32, budget: u32 },

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("{0}")]
    Structured(StructuredError),

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Convenience alias.
pub type HsxResult<T> = Result<T, HsxError>;

impl HsxError {
    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::Structured(e) => e.retryable,
            Self::Database(_) => false,
            _ => false,
        }
    }

    /// Convert to a structured error for serialized output.
    pub fn to_structured(&self) -> StructuredError {
        match self {
            Self::Network(e) => StructuredError {
                kind: if e.is_timeout() {
                    ErrorKind::NetworkTimeout
                } else {
                    ErrorKind::Http5xx
                },
                retryable: true,
                message: e.to_string(),
                source_url: e.url().map(|u| u.to_string()),
                suggested_action: "Retry the request or check network connectivity".into(),
                alternatives: vec![],
            },
            Self::BudgetExceeded { used, budget } => StructuredError {
                kind: ErrorKind::BudgetExceeded,
                retryable: false,
                message: format!("Token budget exceeded: {used}/{budget}"),
                source_url: None,
                suggested_action: "Increase --budget or use a lower PDS tier".into(),
                alternatives: vec![],
            },
            Self::Structured(e) => e.clone(),
            other => StructuredError {
                kind: ErrorKind::Unknown,
                retryable: false,
                message: other.to_string(),
                source_url: None,
                suggested_action: "Check logs for details".into(),
                alternatives: vec![],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_error_display() {
        let err = StructuredError {
            kind: ErrorKind::Http429,
            retryable: true,
            message: "Rate limited".into(),
            source_url: Some("https://example.com".into()),
            suggested_action: "Wait and retry".into(),
            alternatives: vec!["Use cache".into()],
        };
        let s = err.to_string();
        assert!(s.contains("Rate limited"));
        assert!(s.contains("example.com"));
    }

    #[test]
    fn budget_exceeded_structured() {
        let err = HsxError::BudgetExceeded {
            used: 5000,
            budget: 4000,
        };
        assert!(!err.is_retryable());
        let structured = err.to_structured();
        assert_eq!(structured.kind, ErrorKind::BudgetExceeded);
    }

    #[test]
    fn error_kind_serialization() {
        let kind = ErrorKind::AntiBot;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"anti_bot\"");
    }
}
