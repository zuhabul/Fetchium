//! robots.txt parser and per-domain cache (PRD §41).
//!
//! Fetches, caches, and respects robots.txt for all domains.
//! Cache entries expire after 24 hours (configurable).

use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A parsed robots.txt directive set for a single user-agent.
#[derive(Debug, Clone)]
pub struct RobotRules {
    /// Disallowed path prefixes.
    pub disallowed: Vec<String>,
    /// Optional crawl delay in seconds.
    pub crawl_delay: Option<f64>,
    /// When these rules were fetched.
    pub fetched_at: Instant,
}

impl RobotRules {
    /// Return `true` if the given path is allowed by these rules.
    pub fn allows(&self, path: &str) -> bool {
        for prefix in &self.disallowed {
            if path.starts_with(prefix.as_str()) {
                return false;
            }
        }
        true
    }

    fn is_stale(&self, ttl: Duration) -> bool {
        self.fetched_at.elapsed() > ttl
    }
}

/// In-memory cache for robots.txt rules, keyed by domain.
pub struct RobotsCache {
    inner: Mutex<HashMap<String, RobotRules>>,
    ttl: Duration,
}

impl Default for RobotsCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(86400)) // 24h default TTL
    }
}

impl RobotsCache {
    /// Create a new cache with the given TTL for rules.
    pub fn new(ttl: Duration) -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Get cached rules for a domain, or `None` if missing/stale.
    pub fn get(&self, domain: &str) -> Option<RobotRules> {
        let lock = self.inner.lock();
        let rules = lock.get(domain)?;
        if rules.is_stale(self.ttl) {
            None
        } else {
            Some(rules.clone())
        }
    }

    /// Insert or replace rules for a domain.
    pub fn insert(&self, domain: &str, rules: RobotRules) {
        self.inner.lock().insert(domain.to_string(), rules);
    }

    /// Check whether a URL is allowed. Returns `true` if no cached rules
    /// exist (fail-open — don't block on missing data).
    pub fn is_allowed(&self, url: &str) -> bool {
        let Ok(parsed) = url::Url::parse(url) else {
            return true;
        };
        let domain = match parsed.host_str() {
            Some(h) => h.to_string(),
            None => return true,
        };
        let path = parsed.path();
        match self.get(&domain) {
            Some(rules) => rules.allows(path),
            None => true, // fail-open: no cached rules → allow
        }
    }
}

/// Parse the text content of a robots.txt file for `user-agent: *`.
///
/// Returns a [`RobotRules`] with disallowed paths and optional crawl delay.
///
/// # Examples
/// ```rust
/// use hsx_core::http::robots::parse_robots_txt;
/// let rules = parse_robots_txt("User-agent: *\nDisallow: /private/\nDisallow: /admin/");
/// assert!(!rules.allows("/private/secret"));
/// assert!(rules.allows("/public/page"));
/// ```
pub fn parse_robots_txt(content: &str) -> RobotRules {
    let mut disallowed = Vec::new();
    let mut crawl_delay: Option<f64> = None;
    let mut in_wildcard_block = false;

    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(agent) = line.strip_prefix("User-agent:") {
            let agent = agent.trim();
            in_wildcard_block = agent == "*" || agent.eq_ignore_ascii_case("hypersearchx");
        } else if in_wildcard_block {
            if let Some(path) = line.strip_prefix("Disallow:") {
                let path = path.trim();
                if !path.is_empty() {
                    disallowed.push(path.to_string());
                }
            } else if let Some(delay) = line.strip_prefix("Crawl-delay:") {
                crawl_delay = delay.trim().parse::<f64>().ok();
            }
        }
    }

    RobotRules {
        disallowed,
        crawl_delay,
        fetched_at: Instant::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic_disallow() {
        let content = "User-agent: *\nDisallow: /admin/\nDisallow: /private/\n";
        let rules = parse_robots_txt(content);
        assert!(!rules.allows("/admin/panel"));
        assert!(!rules.allows("/private/data"));
        assert!(rules.allows("/public/page"));
        assert!(rules.allows("/"));
    }

    #[test]
    fn parse_crawl_delay() {
        let content = "User-agent: *\nDisallow: /\nCrawl-delay: 2.5\n";
        let rules = parse_robots_txt(content);
        assert_eq!(rules.crawl_delay, Some(2.5));
    }

    #[test]
    fn parse_comments_stripped() {
        let content = "User-agent: * # all bots\nDisallow: /secret # private\n";
        let rules = parse_robots_txt(content);
        assert!(!rules.allows("/secret"));
    }

    #[test]
    fn empty_disallow_means_allow_all() {
        let content = "User-agent: *\nDisallow:\n";
        let rules = parse_robots_txt(content);
        assert!(rules.allows("/anything"));
    }

    #[test]
    fn cache_is_allowed_fail_open_when_missing() {
        let cache = RobotsCache::default();
        // No cached rules → fail-open (allow)
        assert!(cache.is_allowed("https://example.com/page"));
    }

    #[test]
    fn cache_respects_disallow() {
        let cache = RobotsCache::default();
        let rules = parse_robots_txt("User-agent: *\nDisallow: /admin/\n");
        cache.insert("example.com", rules);
        assert!(!cache.is_allowed("https://example.com/admin/panel"));
        assert!(cache.is_allowed("https://example.com/public/page"));
    }
}
