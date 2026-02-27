//! Tor SOCKS5 proxy helpers (PRD §36).

use crate::error::HsxError;

/// Tor SOCKS5 default endpoint.
pub const TOR_SOCKS5: &str = "socks5://127.0.0.1:9050";

/// Check whether Tor's SOCKS5 port is reachable.
///
/// Returns `true` if the TCP connection succeeds, `false` otherwise.
pub fn is_tor_available() -> bool {
    use std::net::TcpStream;
    use std::time::Duration;
    TcpStream::connect_timeout(&"127.0.0.1:9050".parse().unwrap(), Duration::from_secs(2)).is_ok()
}

/// Returns a Tor-spoofed user-agent string (Tor Browser).
pub fn tor_user_agent() -> &'static str {
    "Mozilla/5.0 (Windows NT 10.0; rv:128.0) Gecko/20100101 Firefox/128.0"
}

/// Build an install hint message for the current OS.
pub fn install_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "Install Tor: `brew install tor && brew services start tor`"
    } else if cfg!(target_os = "linux") {
        "Install Tor: `sudo apt-get install tor && sudo systemctl start tor`"
    } else {
        "Download Tor Browser Bundle from https://www.torproject.org/"
    }
}

/// Validate Tor availability and return a useful error if not reachable.
pub fn require_tor() -> Result<(), HsxError> {
    if is_tor_available() {
        Ok(())
    } else {
        Err(HsxError::Internal(format!(
            "Tor SOCKS5 proxy not reachable at 127.0.0.1:9050. {}",
            install_hint()
        )))
    }
}
