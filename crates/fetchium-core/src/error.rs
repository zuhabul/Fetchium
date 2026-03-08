//! Error types for Fetchium (PRD §44).
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

/// Main error type for fetchium-core.
#[derive(Debug, thiserror::Error)]
pub enum FetchiumError {
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

    #[error("AI engine unavailable: {0}")]
    AiUnavailable(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("External tool error: {0}")]
    ExternalTool(String),

    #[error("Insecure connection to {url}: {suggestion}")]
    InsecureConnection { url: String, suggestion: String },

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("YouTube error: {0}")]
    YouTube(String),

    #[error("Operation '{operation}' timed out after {timeout_ms}ms: {suggestion}")]
    OperationTimeout {
        operation: String,
        timeout_ms: u64,
        suggestion: String,
    },

    #[error("{0}")]
    Other(#[from] anyhow::Error),
}

/// Convenience alias.
pub type FetchiumResult<T> = Result<T, FetchiumError>;

/// Wrap any async operation with a timeout (PRD SS44: "Never hang").
///
/// # Example
/// ```rust,no_run
/// use fetchium_core::error::with_timeout;
/// use tokio::time::Duration;
/// # async fn example() -> fetchium_core::error::FetchiumResult<()> {
/// let result = with_timeout(Duration::from_secs(10), "fetch", async {
///     Ok(())
/// }).await;
/// # result
/// # }
/// ```
pub async fn with_timeout<F, T>(
    duration: tokio::time::Duration,
    op_name: &str,
    future: F,
) -> FetchiumResult<T>
where
    F: std::future::Future<Output = FetchiumResult<T>>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(FetchiumError::OperationTimeout {
            operation: op_name.to_string(),
            timeout_ms: duration.as_millis() as u64,
            suggestion: format!(
                "{op_name} timed out after {}ms. Try increasing the timeout.",
                duration.as_millis()
            ),
        }),
    }
}

impl FetchiumError {
    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) => true,
            Self::Structured(e) => e.retryable,
            Self::Database(_) => false,
            _ => false,
        }
    }

    /// A human-readable suggestion for how to resolve this error.
    pub fn suggested_action(&self) -> &str {
        match self {
            Self::Network(_) => "Retry the request or check network connectivity",
            Self::InsecureConnection { .. } => "Use HTTPS instead of HTTP for remote hosts",
            Self::InvalidUrl(_) => "Check the URL format (must start with https:// or http://)",
            Self::OperationTimeout { suggestion, .. } => suggestion.as_str(),
            Self::AiUnavailable(_) => "Start Ollama with: ollama serve",
            Self::BudgetExceeded { .. } => "Increase --budget or use a lower PDS tier",
            _ => "Check error details and try again",
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
        let err = FetchiumError::BudgetExceeded {
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
