//! Citation types for 6-style citation formatting (PRD §24).

use serde::{Deserialize, Serialize};

/// Which citation style to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CitationStyle {
    #[default]
    Inline,
    Footnote,
    Apa,
    Mla,
    Chicago,
    Ieee,
    Bibtex,
}

/// Metadata about a source needed for citation formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub url: String,
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub accessed_date: String,
}

/// A formatted citation with inline marker and full reference entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedCitation {
    pub inline_marker: String,
    pub reference_entry: String,
    pub url: String,
    pub index: usize,
}
