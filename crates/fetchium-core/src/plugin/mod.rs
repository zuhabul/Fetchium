//! Plugin system — manifest-based registry + CLI lifecycle (PRD §29).
//!
//! Supports 6 plugin types: Backend, Extractor, Ranker, Formatter, Validator, AiProvider.
//! Plugins are discovered from `~/.fetchium/plugins/` via `plugin.toml` manifests.
//! Native (.so/.dylib) and WASM loading are planned but currently handled through the
//! manifest/registry layer only.

pub mod loader;
pub mod manifest;
pub mod registry;
pub mod traits;

pub use manifest::PluginManifest;
pub use registry::{PluginEntry, PluginRegistry, PluginStatus};
pub use traits::PluginType;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_starts_empty() {
        let reg = PluginRegistry::new();
        assert!(reg.all().is_empty());
    }

    #[test]
    fn plugin_type_display() {
        assert_eq!(PluginType::Backend.to_string(), "backend");
        assert_eq!(PluginType::AiProvider.to_string(), "ai_provider");
    }
}
