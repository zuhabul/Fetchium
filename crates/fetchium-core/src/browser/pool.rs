//! Browser pool manager — manages headless Chromium instances (PRD §8.3).
//!
//! Resource tier controls pool size:
//! - Minimal:     1 browser, 2 tabs  (~200 MB)
//! - Standard:    1 browser, 4 tabs  (~500 MB)
//! - Performance: 2 browsers, 8 tabs (~1 GB)
//!
//! ## Chrome binary detection (Ubuntu 24.04 snap)
//! **Snap Chromium (Ubuntu 24.04)**: Uses `.user_data_dir()` per-instance to avoid
//! Chrome's `SingletonLock` error (without it, only the 1st instance can launch).
//!
//! ## Graceful init
//! `init()` logs individual browser failures but only errors if ALL instances fail.
//! This lets snap Chromium work even when only 1 of N instances launches.
//!
//! Only compiled when the `headless` feature is enabled.

#[cfg(not(feature = "headless"))]
/// Stub pool for non-headless builds.
pub struct BrowserPool;

#[cfg(not(feature = "headless"))]
impl BrowserPool {
    pub fn new_stub() -> Self {
        Self
    }
}

#[cfg(feature = "headless")]
pub use headless_impl::{BrowserPool, BrowserTier};

#[cfg(feature = "headless")]
mod headless_impl {
    use crate::browser::tab::ManagedTab;
    use chromiumoxide::{Browser, BrowserConfig};
    use parking_lot::Mutex;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Semaphore;
    use tracing::{debug, info};

    /// Resource tier controlling pool capacity.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BrowserTier {
        /// 1 browser, 2 tabs (~200 MB)
        Minimal,
        /// 1 browser, 4 tabs (~500 MB)
        Standard,
        /// 2 browsers, 8 tabs (~1 GB)
        Performance,
    }

    impl BrowserTier {
        pub fn tab_limit(&self) -> usize {
            match self {
                Self::Minimal => 2,
                Self::Standard => 4,
                Self::Performance => 8,
            }
        }

        pub fn browser_count(&self) -> usize {
            match self {
                Self::Performance => 2,
                _ => 1,
            }
        }
    }

    /// Managed pool of headless Chromium browser instances.
    pub struct BrowserPool {
        browsers: Mutex<Vec<Arc<Browser>>>,
        tab_semaphore: Arc<Semaphore>,
        tier: BrowserTier,
        /// Optional Chrome binary path override. When `None`, auto-detection is used.
        chrome_path: Option<PathBuf>,
    }

    impl BrowserPool {
        /// Create a new pool (does not launch browsers yet — call `init()`).
        /// Uses automatic Chrome binary detection at init time.
        pub fn new(tier: BrowserTier) -> Self {
            Self::new_with_path(tier, None)
        }

        /// Create a pool with an explicit Chrome binary path.
        pub fn new_with_path(tier: BrowserTier, chrome_path: Option<PathBuf>) -> Self {
            Self {
                browsers: Mutex::new(Vec::new()),
                tab_semaphore: Arc::new(Semaphore::new(tier.tab_limit())),
                tier,
                chrome_path,
            }
        }

        /// Create a pool from `HsxConfig`, using the full priority-chain Chrome resolution.
        ///
        /// This is the preferred constructor — it honours `fetchium setup --headless` downloads,
        /// env var overrides, and config file paths automatically.
        pub fn from_config(tier: BrowserTier, config: &crate::config::HsxConfig) -> Self {
            let chrome_path = crate::setup::chromium::resolve_chrome_path(config);
            Self::new_with_path(tier, chrome_path)
        }

        /// Launch headless Chromium instances according to the resource tier.
        pub async fn init(&self) -> anyhow::Result<()> {
            let count = self.tier.browser_count();
            info!("BrowserPool: launching {} Chromium instance(s)", count);

            // Locate Chrome binary: use explicit override or fall back to system detection
            let chrome_path: std::path::PathBuf = match &self.chrome_path {
                Some(p) => {
                    info!("BrowserPool: using configured Chrome at {}", p.display());
                    p.clone()
                }
                None => [
                    "/usr/bin/chromium-browser",
                    "/usr/bin/chromium",
                    "/usr/bin/google-chrome",
                    "/usr/bin/google-chrome-stable",
                    "/snap/bin/chromium",
                ]
                .iter()
                .map(std::path::Path::new)
                .find(|p| p.exists())
                .map(|p| p.to_path_buf())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No Chrome/Chromium binary found. \
                         Run `fetchium setup --headless` to download Chrome for Testing, \
                         or install with: sudo apt-get install chromium-browser"
                    )
                })?,
            };
            info!("BrowserPool: using Chrome at {}", chrome_path.display());

            for i in 0..count {
                // Each browser instance needs its own user-data-dir to avoid
                // Chrome's SingletonLock preventing multiple simultaneous instances.
                let user_data_dir = std::env::temp_dir().join(format!(
                    "fetchium-chrome-{}-{}",
                    std::process::id(),
                    i
                ));

                let config = BrowserConfig::builder()
                    .chrome_executable(&chrome_path)
                    .user_data_dir(&user_data_dir)
                    .arg("--no-sandbox")
                    .arg("--disable-gpu")
                    .arg("--disable-dev-shm-usage")
                    .arg("--disable-extensions")
                    .arg("--disable-background-networking")
                    .arg("--disable-sync")
                    .arg("--headless=new")
                    .arg("--disable-blink-features=AutomationControlled")
                    .arg("--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
                    .build()
                    .map_err(|e| anyhow::anyhow!("BrowserConfig error: {e}"))?;

                match Browser::launch(config).await {
                    Ok((browser, handler)) => {
                        // Spawn the event handler loop in the background
                        tokio::spawn(async move {
                            use tokio_stream::StreamExt;
                            let mut handler = handler;
                            while handler.next().await.is_some() {}
                        });
                        self.browsers.lock().push(Arc::new(browser));
                        debug!("BrowserPool: browser {} launched", i);
                    }
                    Err(e) => {
                        // Log but keep going — if at least one browser launched, pool is usable
                        tracing::warn!("BrowserPool: browser {} failed to launch: {e}", i);
                    }
                }
            }

            let launched = self.browsers.lock().len();
            if launched == 0 {
                return Err(anyhow::anyhow!(
                    "BrowserPool: all {} browser instance(s) failed to launch",
                    count
                ));
            }

            info!(
                "BrowserPool: ready ({}/{} browsers, {} tab slots)",
                launched,
                count,
                self.tier.tab_limit()
            );
            Ok(())
        }

        /// Acquire a tab from the pool. Blocks if at capacity until one is released.
        pub async fn acquire_tab(&self) -> anyhow::Result<ManagedTab> {
            // Acquire a permit (blocks if tab_limit reached)
            let permit = Arc::clone(&self.tab_semaphore)
                .acquire_owned()
                .await
                .map_err(|e| anyhow::anyhow!("Semaphore closed: {e}"))?;

            // Pick a browser (round-robin)
            let browser = {
                let browsers = self.browsers.lock();
                if browsers.is_empty() {
                    return Err(anyhow::anyhow!(
                        "BrowserPool not initialized — call init() first"
                    ));
                }
                Arc::clone(&browsers[0]) // Simple: always use first browser
            };

            let page = browser
                .new_page("about:blank")
                .await
                .map_err(|e| anyhow::anyhow!("Failed to open new tab: {e}"))?;

            debug!(
                "BrowserPool: tab acquired ({} permits remaining)",
                self.tab_semaphore.available_permits()
            );

            Ok(ManagedTab::new(page, permit))
        }

        /// Gracefully shut down all browser instances.
        pub async fn shutdown(&self) {
            let mut browsers = self.browsers.lock();
            info!("BrowserPool: shutting down {} browser(s)", browsers.len());
            browsers.clear(); // Drops all Arc<Browser>, triggering cleanup
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn tier_limits() {
            assert_eq!(BrowserTier::Minimal.tab_limit(), 2);
            assert_eq!(BrowserTier::Standard.tab_limit(), 4);
            assert_eq!(BrowserTier::Performance.tab_limit(), 8);
            assert_eq!(BrowserTier::Performance.browser_count(), 2);
            assert_eq!(BrowserTier::Minimal.browser_count(), 1);
        }

        #[test]
        fn pool_semaphore_capacity() {
            let pool = BrowserPool::new(BrowserTier::Standard);
            assert_eq!(pool.tab_semaphore.available_permits(), 4);
        }
    }
}
