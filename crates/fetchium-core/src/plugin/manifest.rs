//! Plugin manifest — `plugin.toml` parser (PRD §29).

use crate::error::FetchiumError;
use serde::{Deserialize, Serialize};

/// Parsed contents of a `plugin.toml` manifest file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    /// One of: backend, extractor, ranker, formatter, validator, ai_provider
    pub plugin_type: String,
    /// One of: native, wasm, builtin
    pub runtime: String,
    pub description: String,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub config_schema: Option<serde_json::Value>,
}

impl PluginManifest {
    /// Load and parse a `plugin.toml` file.
    pub fn load(path: &std::path::Path) -> Result<Self, FetchiumError> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content)
            .map_err(|e| FetchiumError::Config(format!("plugin.toml parse error: {e}")))
    }

    /// Write a scaffold `plugin.toml` to the given path.
    pub fn scaffold(
        name: &str,
        plugin_type: &str,
        path: &std::path::Path,
    ) -> Result<(), FetchiumError> {
        let manifest = PluginManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            plugin_type: plugin_type.to_string(),
            runtime: "native".to_string(),
            description: format!("A Fetchium {plugin_type} plugin"),
            author: None,
            homepage: None,
            config_schema: None,
        };
        let content = toml::to_string_pretty(&manifest)
            .map_err(|e| FetchiumError::Config(format!("serialise error: {e}")))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
