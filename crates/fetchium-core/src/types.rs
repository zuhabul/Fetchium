//! Core data types for Fetchium (PRD §43).
//!
//! All types in this module are serializable and form the foundation
//! of the entire pipeline: search → extract → rank → validate → output.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Search Results ──────────────────────────────────────────────

/// Top-level result for agent-facing commands (`agent-search`, `agent-fetch`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSearchResult {
    pub meta: SearchMeta,
    pub segments: Vec<Segment>,
    pub findings: Vec<Finding>,
    pub evidence: Vec<EvidenceLink>,
    pub contradictions: Vec<Contradiction>,
    pub sources: Vec<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_graph: Option<EvidenceGraph>,
    pub audit_trail: Vec<AuditEntry>,
}

/// Top-level result for human-facing commands (`search`, `fetch`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub meta: SearchMeta,
    pub items: Vec<ResultItem>,
}

/// Metadata for any search/fetch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMeta {
    pub query: String,
    pub mode: SearchMode,
    pub tier: PdsTier,
    pub tokens_used: u32,
    pub tokens_budget: u32,
    pub sources_fetched: u32,
    pub sources_validated: u32,
    pub validation_pass_rate: f64,
    pub duration_ms: u64,
    pub resource_tier: ResourceTier,
    pub timestamp: String,
    pub result_id: String,
    #[serde(default)]
    pub content_hashes: HashMap<String, String>,
}

// ─── Search Items ────────────────────────────────────────────────

/// A single search result item (URL + title + snippet).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub rank: u32,
    pub backend: BackendId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
}

// ─── Content Segments (SCS §18) ─────────────────────────────────

/// Typed content segment from Structured Content Segmentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub seg_type: SegmentType,
    pub relevance: f64,
    pub tokens: u32,
    pub content: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<u32>,
}

/// All 14 segment types per PRD §18.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Heading,
    Paragraph,
    Fact,
    Opinion,
    Table,
    Code,
    List,
    Quote,
    Data,
    Link,
    Image,
    Definition,
    DateEvent,
    Entity,
}

// ─── Evidence & Findings ─────────────────────────────────────────

/// A research finding — claim with supporting evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub claim: String,
    pub confidence: f64,
    pub evidence_ids: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Link between a claim and a source quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLink {
    pub claim: String,
    pub source_id: u32,
    pub quote: String,
    pub quote_hash: String,
    pub confidence: f64,
    pub evidence_type: EvidenceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Supports,
    Contradicts,
    PartiallySupports,
}

/// Contradiction between two or more sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a: String,
    pub claim_b: String,
    pub source_a: u32,
    pub source_b: u32,
    pub severity: ContradictionSeverity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

// ─── Sources ─────────────────────────────────────────────────────

/// A fetched and processed source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub domain: String,
    pub fetch_method: FetchMethod,
    pub content_type: String,
    pub tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
    pub trust_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<Citation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchMethod {
    Http,
    HeadlessChromium,
    Cache,
    WaybackMachine,
}

/// Citation in a specific format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub style: CitationStyle,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationStyle {
    Apa,
    Mla,
    Chicago,
    Harvard,
    Ieee,
    Bibtex,
}

// ─── Evidence Graph (EGP §24) ────────────────────────────────────

/// Evidence graph showing relationships between claims and sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNode {
    pub id: u32,
    pub node_type: EvidenceNodeType,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceNodeType {
    Claim,
    Source,
    Finding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEdge {
    pub from: u32,
    pub to: u32,
    pub relation: EvidenceType,
    pub weight: f64,
}

// ─── Audit Trail ─────────────────────────────────────────────────

/// Entry in the operation audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub step: String,
    pub duration_ms: u64,
    pub detail: String,
}

// ─── Enums ───────────────────────────────────────────────────────

/// Search mode (PRD §10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchMode {
    #[default]
    Search,
    Fetch,
    Research,
    Deep,
    Ai,
    Compare,
    Monitor,
}

/// Progressive Disclosure System tier (PRD §27).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PdsTier {
    KeyFacts,
    #[default]
    Summary,
    Detailed,
    Complete,
}

/// Resource tier based on system capabilities (PRD §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ResourceTier {
    Minimal,
    #[default]
    Standard,
    Performance,
    Server,
}

/// Backend identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendId {
    DuckDuckGo,
    Google,
    Bing,
    GoogleScholar,
    Searxng,
    Wikipedia,
    Brave,
    HackerNews,
    Arxiv,
    Github,
    Reddit,
    StackOverflow,
    YouTube,
    Tavily,
    Serper,
    Exa,
    Firecrawl,
    Custom(String),
}

impl std::fmt::Display for BackendId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuckDuckGo => write!(f, "duckduckgo"),
            Self::Google => write!(f, "google"),
            Self::Bing => write!(f, "bing"),
            Self::GoogleScholar => write!(f, "scholar"),
            Self::Searxng => write!(f, "searxng"),
            Self::Wikipedia => write!(f, "wikipedia"),
            Self::Brave => write!(f, "brave"),
            Self::HackerNews => write!(f, "hackernews"),
            Self::Arxiv => write!(f, "arxiv"),
            Self::Github => write!(f, "github"),
            Self::Reddit => write!(f, "reddit"),
            Self::Tavily => write!(f, "tavily"),
            Self::Serper => write!(f, "serper"),
            Self::Exa => write!(f, "exa"),
            Self::Firecrawl => write!(f, "firecrawl"),
            Self::StackOverflow => write!(f, "stackoverflow"),
            Self::YouTube => write!(f, "youtube"),
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
    Csv,
    Yaml,
    Html,
    Segments,
    Bibtex,
}

/// Content Extraction Protocol layer (PRD §16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CepLayer {
    /// HTTP + CSS selectors (scraper)
    Layer1 = 1,
    /// HTTP + streaming HTML rewriter (lol_html)
    Layer2 = 2,
    /// Headless Chromium — static render
    Layer3 = 3,
    /// Headless Chromium — wait for JS
    Layer4 = 4,
    /// Headless Chromium — interaction required
    Layer5 = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_type_roundtrip() {
        let seg = SegmentType::Code;
        let json = serde_json::to_string(&seg).unwrap();
        assert_eq!(json, "\"code\"");
        let back: SegmentType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, seg);
    }

    #[test]
    fn backend_id_display() {
        assert_eq!(BackendId::DuckDuckGo.to_string(), "duckduckgo");
        assert_eq!(
            BackendId::Custom("my-engine".into()).to_string(),
            "my-engine"
        );
    }

    #[test]
    fn search_meta_serialization() {
        let meta = SearchMeta {
            query: "test".into(),
            mode: SearchMode::Search,
            tier: PdsTier::Summary,
            tokens_used: 100,
            tokens_budget: 4000,
            sources_fetched: 5,
            sources_validated: 4,
            validation_pass_rate: 0.8,
            duration_ms: 1200,
            resource_tier: ResourceTier::Standard,
            timestamp: "2026-02-23T00:00:00Z".into(),
            result_id: "abc123".into(),
            content_hashes: HashMap::new(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: SearchMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.query, "test");
    }

    #[test]
    fn cep_layer_ordering() {
        assert!(CepLayer::Layer1 < CepLayer::Layer3);
        assert!(CepLayer::Layer5 > CepLayer::Layer2);
    }

    #[test]
    fn agent_search_result_roundtrip() {
        let result = AgentSearchResult {
            meta: SearchMeta {
                query: "test query".into(),
                mode: SearchMode::Search,
                tier: PdsTier::Summary,
                tokens_used: 500,
                tokens_budget: 4000,
                sources_fetched: 3,
                sources_validated: 3,
                validation_pass_rate: 1.0,
                duration_ms: 2100,
                resource_tier: ResourceTier::Standard,
                timestamp: "2026-02-23T12:00:00Z".into(),
                result_id: "r-001".into(),
                content_hashes: HashMap::new(),
            },
            segments: vec![],
            findings: vec![],
            evidence: vec![],
            contradictions: vec![],
            sources: vec![],
            evidence_graph: None,
            audit_trail: vec![],
        };
        let json = serde_json::to_string_pretty(&result).unwrap();
        let back: AgentSearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.meta.query, "test query");
        assert_eq!(back.meta.tokens_budget, 4000);
    }

    #[test]
    fn evidence_type_all_variants() {
        for variant in [
            EvidenceType::Supports,
            EvidenceType::Contradicts,
            EvidenceType::PartiallySupports,
        ] {
            let json = serde_json::to_string(&variant).unwrap();
            let back: EvidenceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn output_format_default_is_markdown() {
        assert_eq!(OutputFormat::default(), OutputFormat::Markdown);
    }

    #[test]
    fn pds_tier_default_is_summary() {
        assert_eq!(PdsTier::default(), PdsTier::Summary);
    }
}
