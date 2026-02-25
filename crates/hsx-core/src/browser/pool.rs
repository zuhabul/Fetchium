//! Browser pool manager — manages headless Chromium instances (PRD §8.3).
//!
//! Resource tier controls pool size:
//! - Minimal:     1 browser, 2 tabs  (~200 MB)
//! - Standard:    1 browser, 4 tabs  (~500 MB)
//! - Performance: 2 browsers, 8 tabs (~1 GB)
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
    }

    impl BrowserPool {
        /// Create a new pool (does not launch browsers yet — call `init()`).
        pub fn new(tier: BrowserTier) -> Self {
            Self {
                browsers: Mutex::new(Vec::new()),
                tab_semaphore: Arc::new(Semaphore::new(tier.tab_limit())),
                tier,
            }
        }

        /// Launch headless Chromium instances according to the resource tier.
        pub async fn init(&self) -> anyhow::Result<()> {
            let count = self.tier.browser_count();
            info!("BrowserPool: launching {} Chromium instance(s)", count);

            for i in 0..count {
                let config = BrowserConfig::builder()
                    .arg("--no-sandbox")
                    .arg("--disable-gpu")
                    .arg("--disable-dev-shm-usage")
                    .arg("--disable-extensions")
                    .arg("--disable-background-networking")
                    .arg("--disable-sync")
                    .arg("--headless")
                    .build()
                    .map_err(|e| anyhow::anyhow!("BrowserConfig error: {e}"))?;

                let (browser, handler) = Browser::launch(config)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to launch browser {i}: {e}"))?;

                // Spawn the event handler loop in the background
                tokio::spawn(async move {
                    use tokio_stream::StreamExt;
                    let mut handler = handler;
                    while handler.next().await.is_some() {}
                });

                self.browsers.lock().push(Arc::new(browser));
                debug!("BrowserPool: browser {} launched", i);
            }

            info!(
                "BrowserPool: ready ({} tabs available)",
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
