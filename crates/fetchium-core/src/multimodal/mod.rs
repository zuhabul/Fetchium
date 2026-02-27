//! Multimodal content extraction — video, PDF, OCR, charts (PRD §42).

pub mod chart;
pub mod ocr;
pub mod pdf;
pub mod video;

/// Unified multimodal content representation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultimodalContent {
    pub source_url: String,
    pub content_type: ContentType,
    pub text: String,
    pub segments: Vec<MultimodalSegment>,
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Video {
        duration_secs: u32,
        transcript_source: String,
    },
    Pdf {
        page_count: u32,
    },
    Image {
        width: u32,
        height: u32,
    },
    Chart {
        chart_type: String,
        series_count: usize,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultimodalSegment {
    pub offset_ms: Option<u32>,
    pub page: Option<u32>,
    pub text: String,
}
