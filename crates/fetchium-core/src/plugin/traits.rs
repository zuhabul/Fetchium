//! Plugin trait definitions for all 6 plugin types (PRD §29).

use crate::error::HsxError;
use async_trait::async_trait;

/// Plugin type classification.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    Backend,
    Extractor,
    Ranker,
    Formatter,
    Validator,
    AiProvider,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Backend => write!(f, "backend"),
            PluginType::Extractor => write!(f, "extractor"),
            PluginType::Ranker => write!(f, "ranker"),
            PluginType::Formatter => write!(f, "formatter"),
            PluginType::Validator => write!(f, "validator"),
            PluginType::AiProvider => write!(f, "ai_provider"),
        }
    }
}

/// Base trait every plugin must implement.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn description(&self) -> &str {
        ""
    }
    fn init(&mut self, config: &serde_json::Value) -> Result<(), HsxError>;
    fn shutdown(&mut self) -> Result<(), HsxError>;
}

/// Backend plugin: provides a search data source.
#[async_trait]
pub trait BackendPlugin: Plugin {
    async fn search(
        &self,
        query: &str,
        max_results: usize,
    ) -> Result<Vec<crate::types::ResultItem>, HsxError>;
    fn supported_features(&self) -> BackendFeatures {
        BackendFeatures::default()
    }
}

/// Features a backend plugin supports.
#[derive(Debug, Default, Clone)]
pub struct BackendFeatures {
    pub supports_date_filter: bool,
    pub supports_language_filter: bool,
    pub max_results_per_query: usize,
}

/// Extractor plugin: custom content extraction strategy.
#[async_trait]
pub trait ExtractorPlugin: Plugin {
    async fn extract(
        &self,
        url: &str,
        raw_html: &str,
    ) -> Result<crate::extract::ExtractedContent, HsxError>;
    fn supported_content_types(&self) -> Vec<String>;
}

/// Ranker plugin: custom ranking algorithm.
pub trait RankerPlugin: Plugin {
    fn rank(
        &self,
        query: &str,
        results: &mut Vec<crate::types::ResultItem>,
    ) -> Result<(), HsxError>;
}

/// Formatter plugin: custom output format.
pub trait FormatterPlugin: Plugin {
    fn format(&self, result: &crate::types::SearchResult) -> Result<String, HsxError>;
    fn file_extension(&self) -> &str;
    fn mime_type(&self) -> &str;
}

/// Validator plugin: custom content validation.
pub trait ValidatorPlugin: Plugin {
    /// Returns confidence score in [0.0, 1.0].
    fn validate(&self, text: &str, url: &str) -> Result<f64, HsxError>;
}

/// AI provider plugin: custom model backend.
#[async_trait]
pub trait AiProviderPlugin: Plugin {
    async fn complete(&self, prompt: &str, max_tokens: u32) -> Result<String, HsxError>;
    fn available_models(&self) -> Vec<String>;
}
