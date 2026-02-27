//! Plugin registry — in-process store for loaded plugins.

use crate::error::HsxError;
use crate::plugin::manifest::PluginManifest;
use std::collections::HashMap;

/// Metadata about an installed plugin (no dynamic dispatch — pure registry).
#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginEntry {
    pub manifest: PluginManifest,
    /// Absolute path to the plugin directory.
    pub path: std::path::PathBuf,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    Loaded,
    Disabled,
    Error { message: String },
}

/// Central registry of all installed plugins.
#[derive(Default)]
pub struct PluginRegistry {
    entries: HashMap<String, PluginEntry>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan the plugin directory and register all found manifests.
    pub fn discover(&mut self, plugin_dir: &std::path::Path) -> Result<usize, HsxError> {
        if !plugin_dir.exists() {
            return Ok(0);
        }
        let mut count = 0;
        for entry in std::fs::read_dir(plugin_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match PluginManifest::load(&manifest_path) {
                        Ok(manifest) => {
                            let name = manifest.name.clone();
                            self.entries.insert(
                                name,
                                PluginEntry {
                                    manifest,
                                    path,
                                    status: PluginStatus::Loaded,
                                },
                            );
                            count += 1;
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to load plugin manifest at {:?}: {e}",
                                manifest_path
                            );
                        }
                    }
                }
            }
        }
        Ok(count)
    }

    pub fn get(&self, name: &str) -> Option<&PluginEntry> {
        self.entries.get(name)
    }

    pub fn all(&self) -> Vec<&PluginEntry> {
        let mut v: Vec<&PluginEntry> = self.entries.values().collect();
        v.sort_by_key(|e| &e.manifest.name);
        v
    }

    pub fn remove(&mut self, name: &str) -> Option<PluginEntry> {
        self.entries.remove(name)
    }

    /// Default plugin directory: `~/.hypersearchx/plugins/`.
    pub fn default_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".hypersearchx")
            .join("plugins")
    }
}
