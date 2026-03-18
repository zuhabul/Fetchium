//! Proxy pool with rotation, health tracking, and automatic failover.

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Proxy protocol type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyProtocol {
    #[default]
    Http,
    Https,
    Socks5,
}

/// Health status of a proxy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyStatus {
    /// Proxy is healthy and available for use.
    Active,
    /// Proxy is temporarily cooling down after failures.
    Cooldown,
    /// Proxy has been manually disabled.
    Disabled,
    /// Proxy is dead (too many consecutive failures).
    Dead,
}

/// Statistics for a single proxy.
#[derive(Debug, Serialize)]
pub struct ProxyStats {
    pub host: String,
    pub port: u16,
    pub status: ProxyStatus,
    pub success_count: u64,
    pub fail_count: u64,
    pub total_requests: u64,
    pub avg_latency_ms: u64,
    pub success_rate: f64,
    pub last_used_secs_ago: Option<u64>,
    pub cooldown_remaining_secs: Option<u64>,
}

/// Internal proxy entry with health tracking.
#[derive(Debug)]
pub struct ProxyEntry {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub protocol: ProxyProtocol,
    // Health tracking
    status: RwLock<ProxyStatus>,
    success_count: AtomicU64,
    fail_count: AtomicU64,
    consecutive_fails: AtomicU64,
    total_latency_ms: AtomicU64,
    last_used: RwLock<Option<Instant>>,
    cooldown_until: RwLock<Option<Instant>>,
}

impl ProxyEntry {
    fn new(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        Self {
            host,
            port,
            username,
            password,
            protocol: ProxyProtocol::Http,
            status: RwLock::new(ProxyStatus::Active),
            success_count: AtomicU64::new(0),
            fail_count: AtomicU64::new(0),
            consecutive_fails: AtomicU64::new(0),
            total_latency_ms: AtomicU64::new(0),
            last_used: RwLock::new(None),
            cooldown_until: RwLock::new(None),
        }
    }

    /// Build the proxy URL for reqwest.
    pub fn url(&self) -> String {
        let scheme = match self.protocol {
            ProxyProtocol::Http => "http",
            ProxyProtocol::Https => "https",
            ProxyProtocol::Socks5 => "socks5",
        };
        match (&self.username, &self.password) {
            (Some(user), Some(pass)) => {
                format!("{scheme}://{user}:{pass}@{}:{}", self.host, self.port)
            }
            _ => format!("{scheme}://{}:{}", self.host, self.port),
        }
    }

    /// Check if proxy is available for use.
    fn is_available(&self) -> bool {
        let status = *self.status.read();
        if status == ProxyStatus::Disabled || status == ProxyStatus::Dead {
            return false;
        }
        if status == ProxyStatus::Cooldown {
            if let Some(until) = *self.cooldown_until.read() {
                if Instant::now() < until {
                    return false;
                }
                // Cooldown expired, restore to active
                *self.status.write() = ProxyStatus::Active;
                *self.cooldown_until.write() = None;
            }
        }
        true
    }

    /// Record a successful request.
    pub fn record_success(&self, latency_ms: u64) {
        self.success_count.fetch_add(1, Ordering::Relaxed);
        self.consecutive_fails.store(0, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency_ms, Ordering::Relaxed);
        *self.last_used.write() = Some(Instant::now());
        // Ensure status is active
        let status = *self.status.read();
        if status == ProxyStatus::Cooldown {
            *self.status.write() = ProxyStatus::Active;
            *self.cooldown_until.write() = None;
        }
    }

    /// Record a failed request with automatic cooldown/death.
    pub fn record_failure(&self) {
        self.fail_count.fetch_add(1, Ordering::Relaxed);
        let consecutive = self.consecutive_fails.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_used.write() = Some(Instant::now());

        if consecutive >= 10 {
            // Too many consecutive failures — mark as dead
            *self.status.write() = ProxyStatus::Dead;
            warn!(
                "Proxy {}:{} marked as dead after {} consecutive failures",
                self.host, self.port, consecutive
            );
        } else if consecutive >= 3 {
            // Enter cooldown with exponential backoff
            let cooldown_secs = 30 * (1u64 << (consecutive - 3)).min(16); // 30s, 60s, 120s, ...
            *self.status.write() = ProxyStatus::Cooldown;
            *self.cooldown_until.write() =
                Some(Instant::now() + Duration::from_secs(cooldown_secs));
            debug!(
                "Proxy {}:{} in cooldown for {}s after {} failures",
                self.host, self.port, cooldown_secs, consecutive
            );
        }
    }

    /// Get stats for this proxy.
    fn stats(&self) -> ProxyStats {
        let success = self.success_count.load(Ordering::Relaxed);
        let fail = self.fail_count.load(Ordering::Relaxed);
        let total = success + fail;
        let total_lat = self.total_latency_ms.load(Ordering::Relaxed);
        let avg_lat = if success > 0 { total_lat / success } else { 0 };
        let rate = if total > 0 {
            success as f64 / total as f64
        } else {
            1.0
        };

        let last_used_secs = self.last_used.read().map(|t| t.elapsed().as_secs());

        let cooldown_remaining = self.cooldown_until.read().and_then(|until| {
            let now = Instant::now();
            if now < until {
                Some((until - now).as_secs())
            } else {
                None
            }
        });

        ProxyStats {
            host: self.host.clone(),
            port: self.port,
            status: *self.status.read(),
            success_count: success,
            fail_count: fail,
            total_requests: total,
            avg_latency_ms: avg_lat,
            success_rate: rate,
            last_used_secs_ago: last_used_secs,
            cooldown_remaining_secs: cooldown_remaining,
        }
    }

    /// Manually set the status.
    pub fn set_status(&self, status: ProxyStatus) {
        *self.status.write() = status;
        if status == ProxyStatus::Active {
            self.consecutive_fails.store(0, Ordering::Relaxed);
            *self.cooldown_until.write() = None;
        }
    }
}

/// Thread-safe proxy pool with rotation and health tracking.
#[derive(Clone)]
pub struct ProxyPool {
    inner: Arc<ProxyPoolInner>,
}

struct ProxyPoolInner {
    proxies: RwLock<Vec<Arc<ProxyEntry>>>,
    /// Round-robin index.
    next_index: AtomicUsize,
    /// Per-domain proxy assignment for sticky sessions.
    domain_assignments: DashMap<String, usize>,
}

impl ProxyPool {
    /// Create an empty proxy pool.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ProxyPoolInner {
                proxies: RwLock::new(Vec::new()),
                next_index: AtomicUsize::new(0),
                domain_assignments: DashMap::new(),
            }),
        }
    }

    /// Load proxies from a file.
    ///
    /// Supported formats:
    /// - `ip:port:username:password` (Webshare format)
    /// - `ip:port` (no auth)
    /// - `protocol://ip:port` (with protocol)
    /// - `protocol://user:pass@ip:port` (full URL)
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let pool = Self::new();
        let mut count = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(entry) = Self::parse_proxy_line(line) {
                pool.add_proxy(entry);
                count += 1;
            } else {
                warn!("Failed to parse proxy line: {line}");
            }
        }

        info!("Loaded {count} proxies from {}", path.display());
        Ok(pool)
    }

    /// Parse a single proxy line into a ProxyEntry.
    fn parse_proxy_line(line: &str) -> Option<ProxyEntry> {
        // URL format first: protocol://[user:pass@]host:port
        if line.contains("://") {
            let url = url::Url::parse(line).ok()?;
            let host = url.host_str()?.to_string();
            let port = url.port().unwrap_or(8080);
            let username = if url.username().is_empty() {
                None
            } else {
                Some(url.username().to_string())
            };
            let password = url.password().map(|s| s.to_string());
            let mut entry = ProxyEntry::new(host, port, username, password);
            entry.protocol = match url.scheme() {
                "socks5" => ProxyProtocol::Socks5,
                "https" => ProxyProtocol::Https,
                _ => ProxyProtocol::Http,
            };
            return Some(entry);
        }

        // Plain format: ip:port or ip:port:username:password
        let parts: Vec<&str> = line.split(':').collect();
        match parts.len() {
            2 => {
                let port = parts[1].parse().ok()?;
                Some(ProxyEntry::new(parts[0].to_string(), port, None, None))
            }
            4 => {
                let port = parts[1].parse().ok()?;
                Some(ProxyEntry::new(
                    parts[0].to_string(),
                    port,
                    Some(parts[2].to_string()),
                    Some(parts[3].to_string()),
                ))
            }
            _ => None,
        }
    }

    /// Add a proxy to the pool.
    pub fn add_proxy(&self, entry: ProxyEntry) {
        self.inner.proxies.write().push(Arc::new(entry));
    }

    /// Get the next available proxy using round-robin rotation.
    pub fn next_proxy(&self) -> Option<Arc<ProxyEntry>> {
        let proxies = self.inner.proxies.read();
        if proxies.is_empty() {
            return None;
        }

        let len = proxies.len();
        // Try up to len times to find an available proxy
        for _ in 0..len {
            let idx = self.inner.next_index.fetch_add(1, Ordering::Relaxed) % len;
            let proxy = &proxies[idx];
            if proxy.is_available() {
                return Some(Arc::clone(proxy));
            }
        }

        // All proxies unavailable — try to find one in cooldown that's closest to expiring
        warn!("All proxies unavailable, attempting cooldown recovery");
        proxies
            .iter()
            .find(|p| {
                *p.status.read() != ProxyStatus::Dead && *p.status.read() != ProxyStatus::Disabled
            })
            .cloned()
    }

    /// Get a proxy assigned to a specific domain (sticky session).
    /// Uses round-robin for initial assignment, then sticks to the same proxy.
    pub fn proxy_for_domain(&self, domain: &str) -> Option<Arc<ProxyEntry>> {
        let proxies = self.inner.proxies.read();
        if proxies.is_empty() {
            return None;
        }

        // Check if we have a sticky assignment
        if let Some(idx) = self.inner.domain_assignments.get(domain) {
            let idx = *idx % proxies.len();
            let proxy = &proxies[idx];
            if proxy.is_available() {
                return Some(Arc::clone(proxy));
            }
            // Assigned proxy unavailable, remove assignment and get new one
            self.inner.domain_assignments.remove(domain);
        }

        // Get next available and assign it
        drop(proxies);
        if let Some(proxy) = self.next_proxy() {
            let proxies = self.inner.proxies.read();
            if let Some(idx) = proxies
                .iter()
                .position(|p| std::ptr::eq(p.as_ref(), proxy.as_ref()))
            {
                self.inner
                    .domain_assignments
                    .insert(domain.to_string(), idx);
            }
            Some(proxy)
        } else {
            None
        }
    }

    /// Force a new proxy assignment for a domain, bypassing sticky reuse.
    pub fn fresh_proxy_for_domain(&self, domain: &str) -> Option<Arc<ProxyEntry>> {
        self.inner.domain_assignments.remove(domain);
        self.proxy_for_domain(domain)
    }

    /// Get stats for all proxies.
    pub fn stats(&self) -> Vec<ProxyStats> {
        self.inner
            .proxies
            .read()
            .iter()
            .map(|p| p.stats())
            .collect()
    }

    /// Get aggregate pool stats.
    pub fn pool_summary(&self) -> serde_json::Value {
        let proxies = self.inner.proxies.read();
        let total = proxies.len();
        let active = proxies.iter().filter(|p| p.is_available()).count();
        let dead = proxies
            .iter()
            .filter(|p| *p.status.read() == ProxyStatus::Dead)
            .count();
        let cooldown = proxies
            .iter()
            .filter(|p| *p.status.read() == ProxyStatus::Cooldown)
            .count();

        let total_success: u64 = proxies
            .iter()
            .map(|p| p.success_count.load(Ordering::Relaxed))
            .sum();
        let total_fail: u64 = proxies
            .iter()
            .map(|p| p.fail_count.load(Ordering::Relaxed))
            .sum();
        let total_reqs = total_success + total_fail;
        let overall_rate = if total_reqs > 0 {
            total_success as f64 / total_reqs as f64
        } else {
            1.0
        };

        serde_json::json!({
            "total_proxies": total,
            "active": active,
            "cooldown": cooldown,
            "dead": dead,
            "disabled": total - active - dead - cooldown,
            "total_requests": total_reqs,
            "total_success": total_success,
            "total_failures": total_fail,
            "overall_success_rate": format!("{:.1}%", overall_rate * 100.0),
        })
    }

    /// Number of proxies in the pool.
    pub fn len(&self) -> usize {
        self.inner.proxies.read().len()
    }

    /// Whether the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.proxies.read().is_empty()
    }

    /// Reset all proxies to active status.
    pub fn reset_all(&self) {
        for proxy in self.inner.proxies.read().iter() {
            proxy.set_status(ProxyStatus::Active);
        }
        self.inner.domain_assignments.clear();
        info!("Reset all proxies to active status");
    }

    /// Remove dead proxies from the pool.
    pub fn purge_dead(&self) -> usize {
        let mut proxies = self.inner.proxies.write();
        let before = proxies.len();
        proxies.retain(|p| *p.status.read() != ProxyStatus::Dead);
        let removed = before - proxies.len();
        if removed > 0 {
            info!("Purged {removed} dead proxies");
            self.inner.domain_assignments.clear();
        }
        removed
    }

    /// Build a reqwest::Client configured with the given proxy.
    pub fn build_client_with_proxy(
        proxy: &ProxyEntry,
        user_agent: &str,
        timeout: Duration,
    ) -> Result<reqwest::Client, reqwest::Error> {
        let proxy_url = proxy.url();
        let reqwest_proxy = reqwest::Proxy::all(&proxy_url)?;

        reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(5))
            .proxy(reqwest_proxy)
            .redirect(reqwest::redirect::Policy::limited(10))
            .gzip(true)
            .brotli(true)
            .build()
    }
}

impl Default for ProxyPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_webshare_format() {
        let entry = ProxyPool::parse_proxy_line("31.59.20.176:6754:nxrendke:2yq1w9nkgstq").unwrap();
        assert_eq!(entry.host, "31.59.20.176");
        assert_eq!(entry.port, 6754);
        assert_eq!(entry.username.as_deref(), Some("nxrendke"));
        assert_eq!(entry.password.as_deref(), Some("2yq1w9nkgstq"));
    }

    #[test]
    fn parse_simple_format() {
        let entry = ProxyPool::parse_proxy_line("1.2.3.4:8080").unwrap();
        assert_eq!(entry.host, "1.2.3.4");
        assert_eq!(entry.port, 8080);
        assert!(entry.username.is_none());
    }

    #[test]
    fn parse_url_format() {
        let entry = ProxyPool::parse_proxy_line("http://user:pass@1.2.3.4:1080").unwrap();
        assert_eq!(entry.host, "1.2.3.4");
        assert_eq!(entry.port, 1080);
        assert_eq!(entry.protocol, ProxyProtocol::Http);
        assert_eq!(entry.username.as_deref(), Some("user"));
    }

    #[test]
    fn round_robin_rotation() {
        let pool = ProxyPool::new();
        pool.add_proxy(ProxyEntry::new("1.1.1.1".into(), 8080, None, None));
        pool.add_proxy(ProxyEntry::new("2.2.2.2".into(), 8080, None, None));
        pool.add_proxy(ProxyEntry::new("3.3.3.3".into(), 8080, None, None));

        let p1 = pool.next_proxy().unwrap();
        let p2 = pool.next_proxy().unwrap();
        let p3 = pool.next_proxy().unwrap();
        let p4 = pool.next_proxy().unwrap();

        assert_eq!(p1.host, "1.1.1.1");
        assert_eq!(p2.host, "2.2.2.2");
        assert_eq!(p3.host, "3.3.3.3");
        assert_eq!(p4.host, "1.1.1.1"); // wraps around
    }

    #[test]
    fn health_tracking() {
        let entry = ProxyEntry::new("1.1.1.1".into(), 8080, None, None);
        assert!(entry.is_available());

        entry.record_success(100);
        entry.record_success(200);
        assert_eq!(entry.success_count.load(Ordering::Relaxed), 2);

        // 3 failures → cooldown
        entry.record_failure();
        entry.record_failure();
        entry.record_failure();
        assert_eq!(*entry.status.read(), ProxyStatus::Cooldown);
    }

    #[test]
    fn dead_after_many_failures() {
        let entry = ProxyEntry::new("1.1.1.1".into(), 8080, None, None);
        for _ in 0..10 {
            entry.record_failure();
        }
        assert_eq!(*entry.status.read(), ProxyStatus::Dead);
        assert!(!entry.is_available());
    }

    #[test]
    fn proxy_url_generation() {
        let entry = ProxyEntry::new(
            "1.2.3.4".into(),
            8080,
            Some("user".into()),
            Some("pass".into()),
        );
        assert_eq!(entry.url(), "http://user:pass@1.2.3.4:8080");

        let entry2 = ProxyEntry::new("5.6.7.8".into(), 3128, None, None);
        assert_eq!(entry2.url(), "http://5.6.7.8:3128");
    }

    #[test]
    fn load_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("proxies.txt");
        std::fs::write(
            &path,
            "# Comment line\n1.1.1.1:8080:user:pass\n2.2.2.2:3128\n\n",
        )
        .unwrap();

        let pool = ProxyPool::load_from_file(&path).unwrap();
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn pool_summary_json() {
        let pool = ProxyPool::new();
        pool.add_proxy(ProxyEntry::new("1.1.1.1".into(), 8080, None, None));
        let summary = pool.pool_summary();
        assert_eq!(summary["total_proxies"], 1);
        assert_eq!(summary["active"], 1);
    }

    #[test]
    fn skip_unavailable_in_rotation() {
        let pool = ProxyPool::new();
        pool.add_proxy(ProxyEntry::new("1.1.1.1".into(), 8080, None, None));
        pool.add_proxy(ProxyEntry::new("2.2.2.2".into(), 8080, None, None));

        // Kill the first proxy
        {
            let proxies = pool.inner.proxies.read();
            proxies[0].set_status(ProxyStatus::Dead);
        }

        // Should skip dead proxy and return the second
        let p = pool.next_proxy().unwrap();
        assert_eq!(p.host, "2.2.2.2");
    }

    #[test]
    fn fresh_proxy_for_domain_rotates_assignment() {
        let pool = ProxyPool::new();
        pool.add_proxy(ProxyEntry::new("1.1.1.1".into(), 8080, None, None));
        pool.add_proxy(ProxyEntry::new("2.2.2.2".into(), 8080, None, None));

        let first = pool.proxy_for_domain("duckduckgo.com").unwrap();
        let second = pool.fresh_proxy_for_domain("duckduckgo.com").unwrap();

        assert_ne!(first.host, second.host);
    }
}
