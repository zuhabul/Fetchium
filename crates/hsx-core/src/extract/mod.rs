//! Content Extraction Protocol (CEP) — 5-layer extraction system (PRD §16).

use crate::types::CepLayer;

/// Extracted content from a web page.
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    pub title: String,
    pub text: String,
    pub layer_used: CepLayer,
    pub tokens: u32,
    pub metadata: ContentMetadata,
}

/// Metadata extracted alongside content.
#[derive(Debug, Clone, Default)]
pub struct ContentMetadata {
    pub description: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<String>,
    pub language: Option<String>,
    pub content_type: String,
}
