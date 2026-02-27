//! Content Extraction Protocol (CEP) — 5-layer extraction system (PRD §16).
//!
//! Phase 1: Layers 1-2 (static HTML — CSS selectors + streaming rewriter).
//! Phase 2: Layers 3-5 (headless Chromium, PDF, Screenshot OCR).
//!
//! Layer selection is automatic and progressive:
//! - Layer 1: HTML + CSS selectors (scraper) — ~85% of pages
//! - Layer 2: HTML + streaming rewriter (lol_html) — enhanced boilerplate removal
//! - Layer 3: Headless JS rendering — SPAs, dynamic content [`headless` feature]
//! - Layer 4: PDF/document extraction — PDF, DOCX, RTF
//! - Layer 5: Screenshot OCR — last resort for image-heavy/canvas content [`headless` feature]

pub mod boilerplate;
pub mod cep_predictor;
pub mod layer1;
pub mod layer2;
pub mod layer3;
pub mod layer4;
pub mod layer5;
pub mod pipeline;

use crate::types::CepLayer;

/// Extracted content from a web page.
#[derive(Debug, Clone)]
pub struct ExtractedContent {
    /// Page title.
    pub title: String,
    /// Extracted text content (cleaned, boilerplate removed).
    pub text: String,
    /// Which CEP layer produced this result.
    pub layer_used: CepLayer,
    /// Estimated token count (heuristic: ~4 chars/token).
    pub tokens: u32,
    /// Additional metadata (description, author, date, language, content-type).
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
