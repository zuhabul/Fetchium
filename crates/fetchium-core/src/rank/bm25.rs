//! BM25 ranking using tantivy.
//!
//! PRD SS21: BM25 as the lexical precision signal in HyperFusion.
//! Phase 1: standalone BM25 scoring for result re-ranking.
//! Phase 2: integrated into the full 8-signal HyperFusion pipeline.

use crate::error::FetchiumResult;
use crate::types::ResultItem;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, Value, STORED, STRING, TEXT};
use tantivy::{doc, Index, TantivyDocument};
use tracing::{debug, info};

/// In-memory BM25 scorer backed by a RAM-based tantivy index.
///
/// Creates a fresh in-memory index per use. For Phase 1 re-ranking
/// (small document sets, <100 items), the index build cost is <1ms.
pub struct Bm25Scorer {
    schema: Schema,
    title_field: Field,
    body_field: Field,
    url_field: Field,
}

/// Document submitted for BM25 indexing.
#[derive(Debug, Clone)]
pub struct ScoringDocument {
    pub title: String,
    pub body: String,
    pub url: String,
}

/// A result with its computed BM25 score.
#[derive(Debug, Clone)]
pub struct ScoredResult {
    pub url: String,
    pub title: String,
    pub bm25_score: f64,
}

impl Bm25Scorer {
    /// Create a new BM25 scorer.
    pub fn new() -> FetchiumResult<Self> {
        let mut schema_builder = Schema::builder();
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT | STORED);
        let url_field = schema_builder.add_text_field("url", STRING | STORED);
        let schema = schema_builder.build();

        Ok(Self {
            schema,
            title_field,
            body_field,
            url_field,
        })
    }

    /// Index and score documents against a query.
    ///
    /// Returns scored results sorted by BM25 score descending.
    /// Builds a fresh RAM index — appropriate for small document sets (<100).
    pub fn score_documents(
        &self,
        documents: &[ScoringDocument],
        query: &str,
        top_n: usize,
    ) -> FetchiumResult<Vec<ScoredResult>> {
        if documents.is_empty() || query.is_empty() {
            return Ok(Vec::new());
        }

        let index = Index::create_in_ram(self.schema.clone());

        // Build writer with 50MB heap
        let mut writer = index
            .writer(50_000_000)
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        for doc_data in documents {
            writer
                .add_document(doc!(
                    self.title_field => doc_data.title.as_str(),
                    self.body_field => doc_data.body.as_str(),
                    self.url_field => doc_data.url.as_str(),
                ))
                .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;
        }

        writer
            .commit()
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        info!("BM25: indexed {} documents", documents.len());

        let reader = index
            .reader()
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(&index, vec![self.title_field, self.body_field]);

        // Sanitise query: tantivy rejects queries with special characters
        let sanitised = sanitise_query(query);
        if sanitised.is_empty() {
            return Ok(Vec::new());
        }

        let parsed = match query_parser.parse_query(&sanitised) {
            Ok(q) => q,
            Err(e) => {
                debug!("BM25 query parse failed for {:?}: {e}", sanitised);
                return Ok(Vec::new());
            }
        };

        let top_docs = searcher
            .search(&parsed, &TopDocs::with_limit(top_n))
            .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_address)
                .map_err(|e| crate::error::FetchiumError::Extraction(e.to_string()))?;

            let url = doc
                .get_first(self.url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(ScoredResult {
                url,
                title,
                bm25_score: score as f64,
            });
        }

        debug!("BM25: scored {} results for {:?}", results.len(), query);
        Ok(results)
    }

    /// Re-rank a list of `ResultItem`s using tantivy BM25.
    ///
    /// Builds an in-memory index from titles + snippets, scores them,
    /// applies BM25 scores back to the items, and re-sorts.
    pub fn rerank(&self, items: &mut [ResultItem], query: &str) -> FetchiumResult<()> {
        if items.is_empty() || query.is_empty() {
            return Ok(());
        }

        let docs: Vec<ScoringDocument> = items
            .iter()
            .map(|item| ScoringDocument {
                title: item.title.clone(),
                body: item.snippet.clone(),
                url: item.url.clone(),
            })
            .collect();

        let scored = self.score_documents(&docs, query, items.len())?;

        // Apply scores
        for item in items.iter_mut() {
            if let Some(s) = scored.iter().find(|s| s.url == item.url) {
                item.score = Some(s.bm25_score);
            }
        }

        // Sort by score descending, original rank as tiebreaker
        items.sort_by(|a, b| {
            b.score
                .unwrap_or(0.0)
                .partial_cmp(&a.score.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.rank.cmp(&b.rank))
        });

        // Re-assign ranks
        for (i, item) in items.iter_mut().enumerate() {
            item.rank = (i + 1) as u32;
        }

        Ok(())
    }
}

impl Default for Bm25Scorer {
    fn default() -> Self {
        Self::new().expect("failed to create Bm25Scorer")
    }
}

/// Remove special characters that tantivy's query parser rejects.
fn sanitise_query(query: &str) -> String {
    query
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' || c == '-' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    #[test]
    fn bm25_index_and_score() {
        let scorer = Bm25Scorer::new().unwrap();

        let docs = vec![
            ScoringDocument {
                title: "Rust Programming".into(),
                body: "Rust is a systems programming language focused on safety and performance"
                    .into(),
                url: "https://rust-lang.org".into(),
            },
            ScoringDocument {
                title: "Python Tutorial".into(),
                body: "Python is a general purpose programming language for beginners".into(),
                url: "https://python.org".into(),
            },
            ScoringDocument {
                title: "Cooking Recipes".into(),
                body: "How to make pasta with tomato sauce and basil".into(),
                url: "https://cooking.example.com".into(),
            },
        ];

        let results = scorer
            .score_documents(&docs, "Rust programming language", 10)
            .unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].url, "https://rust-lang.org");
        assert!(results[0].bm25_score > 0.0);
    }

    #[test]
    fn bm25_reranking() {
        let scorer = Bm25Scorer::new().unwrap();

        let mut items = vec![
            ResultItem {
                title: "Cooking Recipes".into(),
                url: "https://cooking.example.com".into(),
                snippet: "Pasta with tomato sauce".into(),
                rank: 1,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
            ResultItem {
                title: "Rust Guide".into(),
                url: "https://rust-lang.org".into(),
                snippet: "Systems programming with Rust memory safety".into(),
                rank: 2,
                backend: BackendId::DuckDuckGo,
                score: None,
                published_date: None,
            },
        ];

        scorer.rerank(&mut items, "Rust programming").unwrap();

        assert_eq!(items[0].title, "Rust Guide");
        assert_eq!(items[0].rank, 1);
    }

    #[test]
    fn bm25_empty_query() {
        let scorer = Bm25Scorer::new().unwrap();
        let docs = vec![ScoringDocument {
            title: "test".into(),
            body: "test body".into(),
            url: "https://test.com".into(),
        }];
        let results = scorer.score_documents(&docs, "", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn bm25_empty_docs() {
        let scorer = Bm25Scorer::new().unwrap();
        let results = scorer.score_documents(&[], "rust", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn sanitise_strips_special_chars() {
        let sanitised = sanitise_query("rust + async \"programming\"");
        assert!(!sanitised.contains('"'));
        assert!(!sanitised.contains('+'));
        assert!(sanitised.contains("rust"));
        assert!(sanitised.contains("async"));
        assert!(sanitised.contains("programming"));
    }
}
