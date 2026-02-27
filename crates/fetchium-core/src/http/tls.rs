//! TLS enforcement: require HTTPS, allow HTTP only for localhost (PRD §41).

use crate::error::HsxError;
use url::Url;

/// Reject plain HTTP for remote hosts; allow localhost for development.
///
/// PRD §41: "TLS enforcement — always HTTPS for remote hosts."
///
/// # Examples
/// ```rust
/// use fetchium_core::http::tls::enforce_tls;
/// assert!(enforce_tls("https://example.com").is_ok());
/// assert!(enforce_tls("http://localhost:11434").is_ok());
/// assert!(enforce_tls("http://example.com").is_err());
/// ```
pub fn enforce_tls(url: &str) -> Result<(), HsxError> {
    let parsed = Url::parse(url).map_err(|e| HsxError::InvalidUrl(format!("{url}: {e}")))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" if is_localhost(&parsed) => Ok(()),
        "http" => Err(HsxError::InsecureConnection {
            url: url.to_string(),
            suggestion: format!(
                "Use https://{} instead",
                parsed.host_str().unwrap_or("unknown")
            ),
        }),
        scheme => Err(HsxError::InvalidUrl(format!(
            "Unsupported URL scheme: '{scheme}'"
        ))),
    }
}

fn is_localhost(url: &Url) -> bool {
    matches!(
        url.host_str(),
        Some("localhost") | Some("127.0.0.1") | Some("::1")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_is_allowed() {
        assert!(enforce_tls("https://example.com").is_ok());
        assert!(enforce_tls("https://docs.rust-lang.org/book/").is_ok());
    }

    #[test]
    fn http_localhost_is_allowed() {
        assert!(enforce_tls("http://localhost:11434").is_ok());
        assert!(enforce_tls("http://127.0.0.1:8080").is_ok());
        assert!(enforce_tls("http://127.0.0.1:3000/api").is_ok());
    }

    #[test]
    fn http_remote_is_rejected() {
        let err = enforce_tls("http://example.com").unwrap_err();
        match err {
            HsxError::InsecureConnection { url, .. } => {
                assert!(url.contains("example.com"));
            }
            other => panic!("Expected InsecureConnection, got {other:?}"),
        }
    }

    #[test]
    fn invalid_url_rejected() {
        assert!(enforce_tls("not a url").is_err());
        assert!(enforce_tls("").is_err());
    }

    #[test]
    fn unsupported_scheme_rejected() {
        assert!(enforce_tls("ftp://example.com").is_err());
    }
}
