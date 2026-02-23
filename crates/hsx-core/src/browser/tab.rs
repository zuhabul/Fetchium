//! Managed browser tab with auto-cleanup via semaphore permit (PRD §8.3).
//!
//! When the `headless` feature is disabled, this module compiles as stubs.

#[cfg(feature = "headless")]
pub use headless_impl::ManagedTab;

#[cfg(not(feature = "headless"))]
/// Stub type when headless feature is disabled.
pub struct ManagedTab;

#[cfg(feature = "headless")]
mod headless_impl {
    use chromiumoxide::Page;
    use std::time::Duration;
    use anyhow::Context as _;

    /// A managed browser tab that returns its semaphore permit on drop.
    ///
    /// The permit is held for the lifetime of the tab; dropping the tab
    /// automatically releases capacity in the pool.
    pub struct ManagedTab {
        pub(crate) page: Page,
        _permit: tokio::sync::OwnedSemaphorePermit,
    }

    impl ManagedTab {
        pub(crate) fn new(page: Page, permit: tokio::sync::OwnedSemaphorePermit) -> Self {
            Self { page, _permit: permit }
        }

        /// Navigate to a URL, waiting up to `timeout_ms` milliseconds for load.
        pub async fn navigate(&self, url: &str, timeout_ms: u64) -> anyhow::Result<()> {
            tokio::time::timeout(
                Duration::from_millis(timeout_ms),
                self.page.goto(url),
            )
            .await
            .context("Navigation timed out")?
            .context("Navigation failed")?;
            Ok(())
        }

        /// Get the fully rendered HTML content of the current page.
        pub async fn content(&self) -> anyhow::Result<String> {
            self.page
                .content()
                .await
                .context("Failed to get page content")
        }

        /// Evaluate JavaScript in the page context.
        pub async fn evaluate(&self, expression: &str) -> anyhow::Result<String> {
            let result = self.page
                .evaluate(expression)
                .await
                .context("JS evaluation failed")?;
            Ok(format!("{:?}", result.value()))
        }

        /// Access the underlying chromiumoxide Page.
        pub fn page(&self) -> &Page {
            &self.page
        }
    }
}
