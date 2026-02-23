//! HyperFusion 8-signal implementations (PRD §8.1).
//!
//! Each signal returns f64 in [0.0, 1.0].
//! Used by the fusion module to compute weighted combined scores.

use crate::types::ResultItem;
use chrono::Datelike;
use std::collections::HashSet;
use tracing::trace;

/// Context maintained across scoring all results in a batch.
///
/// Shared mutable state needed by diversity and consensus signals.
pub struct ScoringContext {
    /// Lowercase query terms (2+ chars, alphanumeric).
    pub query_terms: Vec<String>,
    /// Domains already seen in the output (for diversity penalty).
    pub seen_domains: HashSet<String>,
    /// All snippets/titles from the result set (for consensus scoring).
    pub all_text: Vec<String>,
    /// Pre-computed query embedding (populated when `embeddings` feature is enabled).
    #[cfg(feature = "embeddings")]
    pub query_embedding: Option<Vec<f32>>,
    /// Placeholder for non-embedding builds (keeps struct layout consistent).
    #[cfg(not(feature = "embeddings"))]
    pub query_embedding: Option<Vec<f32>>,
    /// Pre-computed result embeddings, keyed by URL.
    ///
    /// Populated via a single `embed_batch()` call in `ScoringContext::new()`, replacing the
    /// previous per-result `embed()` approach (N×15ms → 1×20ms; 7.5× speedup for 10 results).
    pub result_embeddings: std::collections::HashMap<String, Vec<f32>>,
}

impl ScoringContext {
    /// Build context from a query string and the full result set.
    pub fn new(query: &str, results: &[ResultItem]) -> Self {
        let query_terms = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 2)
            .map(|t| t.to_string())
            .collect();

        let all_text = results
            .iter()
            .map(|r| format!("{} {}", r.title, r.snippet).to_lowercase())
            .collect();

        // Lazily compute query embedding when embeddings feature is enabled
        #[cfg(feature = "embeddings")]
        let query_embedding = crate::embeddings::embed(query).ok();

        #[cfg(not(feature = "embeddings"))]
        let query_embedding: Option<Vec<f32>> = None;

        // Batch-embed all results in a single inference pass (N×15ms → 1×20ms).
        #[cfg(feature = "embeddings")]
        let result_embeddings = {
            let full_texts: Vec<String> = results
                .iter()
                .map(|r| format!("{} {}", r.title, r.snippet))
                .collect();
            let text_refs: Vec<&str> = full_texts.iter().map(|s| s.as_str()).collect();
            match crate::embeddings::embed_batch(&text_refs) {
                Ok(embs) => results
                    .iter()
                    .zip(embs)
                    .map(|(r, e)| (r.url.clone(), e))
                    .collect(),
                Err(_) => std::collections::HashMap::new(),
            }
        };

        #[cfg(not(feature = "embeddings"))]
        let result_embeddings: std::collections::HashMap<String, Vec<f32>> =
            std::collections::HashMap::new();

        Self {
            query_terms,
            seen_domains: HashSet::new(),
            all_text,
            query_embedding,
            result_embeddings,
        }
    }
}

// ─── Signal 1: BM25 ──────────────────────────────────────────────

/// BM25 relevance of title+snippet against the query.
/// Uses local TF with IDF=1 (no corpus), k1=1.2, b=0.75, avg_dl=80.
pub fn bm25_score(result: &ResultItem, ctx: &ScoringContext) -> f64 {
    const K1: f64 = 1.2;
    const B: f64 = 0.75;
    const AVG_DL: f64 = 80.0;

    if ctx.query_terms.is_empty() {
        return 0.0;
    }

    let text = format!("{} {}", result.title, result.snippet).to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();
    let doc_len = words.len() as f64;

    let mut score = 0.0f64;
    for term in &ctx.query_terms {
        let tf = words.iter().filter(|w| **w == term.as_str()).count() as f64;
        if tf == 0.0 {
            // Half-weight for substring match
            let partial = words.iter().filter(|w| w.contains(term.as_str())).count() as f64;
            if partial > 0.0 {
                let denom = partial * 0.5 + K1 * (1.0 - B + B * doc_len / AVG_DL);
                score += 0.5 * (partial * 0.5 * (K1 + 1.0)) / denom;
            }
            continue;
        }
        let num = tf * (K1 + 1.0);
        let den = tf + K1 * (1.0 - B + B * doc_len / AVG_DL);
        score += num / den;
    }

    let max_possible = ctx.query_terms.len() as f64 * (K1 + 1.0);
    (score / max_possible).clamp(0.0, 1.0)
}

// ─── Signal 2: Semantic ───────────────────────────────────────────

/// Semantic similarity between query and result.
///
/// Looks up the pre-computed batch embedding for this result from `ctx.result_embeddings`
/// (populated in `ScoringContext::new()` via a single `embed_batch()` call).
/// Falls back to Jaccard term overlap when embeddings are unavailable.
pub fn semantic_score(result: &ResultItem, ctx: &ScoringContext) -> f64 {
    // Feature-gated: real cosine similarity via pre-computed batch embeddings
    #[cfg(feature = "embeddings")]
    {
        if let (Some(ref query_emb), Some(result_emb)) = (
            &ctx.query_embedding,
            ctx.result_embeddings.get(&result.url),
        ) {
            let cosine = crate::embeddings::cosine_similarity(query_emb, result_emb);
            return (cosine.max(0.0) as f64).min(1.0);
        }
    }

    // Fallback: Jaccard term overlap
    jaccard_semantic(result, ctx)
}

/// Jaccard overlap of query terms vs document terms (lexical proxy).
fn jaccard_semantic(result: &ResultItem, ctx: &ScoringContext) -> f64 {
    if ctx.query_terms.is_empty() {
        return 0.5;
    }

    let text = format!("{} {}", result.title, result.snippet).to_lowercase();
    let doc_terms: HashSet<&str> = text.split_whitespace().collect();
    let query_set: HashSet<&str> = ctx.query_terms.iter().map(|s| s.as_str()).collect();

    let intersection = query_set.intersection(&doc_terms).count();
    let union = query_set.union(&doc_terms).count();

    if union == 0 {
        return 0.0;
    }
    (intersection as f64 / union as f64).min(1.0)
}

// ─── Signal 3: Temporal ──────────────────────────────────────────

/// Temporal freshness score with EDF domain-calibrated exponential decay.
///
/// Formula: `exp(-freshness_need × λ × days_old)` where `λ = ln(2) / half_life`
/// and `half_life` is the domain-appropriate half-life from EDF (e.g. 7 days
/// for social media, 36 500 days for mathematics).
///
/// - freshness_need=0: temporal doesn't matter → 1.0
/// - freshness_need=1, days_old=half_life: score ≈ 0.5
/// - Unknown date → 0.5 (neutral)
pub fn temporal_score(result: &ResultItem, freshness_need: f64) -> f64 {
    let Some(ref date_str) = result.published_date else {
        return 0.5; // Unknown date = neutral
    };

    let days_old = match parse_days_old(date_str) {
        Some(d) => d,
        None => return 0.5,
    };

    // Look up domain-specific half-life via EDF instead of using a hardcoded 365 days.
    let half_life = crate::intelligence::edf::domain_half_life(&result.url);
    let lambda = std::f64::consts::LN_2 / half_life;
    let score = (-freshness_need * lambda * days_old as f64).exp();
    score.clamp(0.0, 1.0)
}

/// Parse a date string and return days since publication.
fn parse_days_old(date_str: &str) -> Option<u64> {
    // Try ISO 8601 formats (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SSZ)
    let date_part = date_str.split('T').next()?;
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() < 3 {
        // Try just year
        if let Ok(year) = parts.first()?.parse::<i32>() {
            let now = chrono::Utc::now().year();
            let years_old = (now - year).max(0) as u64;
            return Some(years_old * 365);
        }
        return None;
    }

    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let day: u32 = parts[2].parse().ok()?;

    let pub_date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let today = chrono::Utc::now().naive_utc().date();
    let duration = today.signed_duration_since(pub_date);
    Some(duration.num_days().max(0) as u64)
}

// ─── Signal 4: Authority ─────────────────────────────────────────

/// Domain authority score based on known authoritative domains + citation count.
///
/// Tier scoring:
/// - High (0.9): wikipedia, github, arxiv, nature, nih.gov, .gov, .edu
/// - Medium (0.7): stackoverflow, medium, bbc, nytimes, reuters
/// - Default (0.5): everything else
pub fn authority_score(result: &ResultItem) -> f64 {
    let domain = extract_domain(&result.url);

    let base = domain_tier_score(&domain);

    // Boost from citation count if encoded in snippet (Scholar results)
    // "Cited by N" → ln(N) / ln(10000) bonus (capped at 0.3)
    let citation_boost = if let Some(n) = extract_citation_count(&result.snippet) {
        (n as f64).ln().min((10000f64).ln()) / (10000f64).ln() * 0.3
    } else {
        0.0
    };

    (base + citation_boost).min(1.0)
}

fn domain_tier_score(domain: &str) -> f64 {
    const HIGH_AUTHORITY: &[&str] = &[
        "wikipedia.org", "github.com", "arxiv.org", "nature.com",
        "nih.gov", "pubmed.ncbi.nlm.nih.gov", "scholar.google.com",
        "docs.rs", "crates.io", "rust-lang.org",
    ];
    const MEDIUM_AUTHORITY: &[&str] = &[
        "stackoverflow.com", "medium.com", "bbc.com", "nytimes.com",
        "reuters.com", "theguardian.com", "techcrunch.com",
        "arstechnica.com", "wired.com", "ieee.org",
    ];

    if HIGH_AUTHORITY.iter().any(|h| domain == *h || domain.ends_with(&format!(".{h}"))) {
        return 0.9;
    }
    if domain.ends_with(".gov") || domain.ends_with(".edu") {
        return 0.9;
    }
    if MEDIUM_AUTHORITY.iter().any(|m| domain == *m || domain.ends_with(&format!(".{m}"))) {
        return 0.7;
    }
    0.5
}

fn extract_domain(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
        .unwrap_or_default()
}

fn extract_citation_count(snippet: &str) -> Option<u64> {
    // Match "Cited by 1234" pattern from Google Scholar
    let lower = snippet.to_lowercase();
    let cited_pos = lower.find("cited by ")?;
    let after = &snippet[cited_pos + 9..];
    let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

// ─── Signal 5: Evidence ───────────────────────────────────────────

/// Evidence quality score: detects statistical data, citations, measurements.
///
/// Higher score = more evidence-dense content.
pub fn evidence_score(result: &ResultItem) -> f64 {
    let text = format!("{} {}", result.title, result.snippet);
    let text_lower = text.to_lowercase();

    let mut score = 0.0f64;
    let word_count = text.split_whitespace().count().max(1) as f64;

    // Count evidence indicators
    let indicators: &[(&str, f64)] = &[
        ("%", 0.1),
        ("study", 0.08),
        ("research", 0.08),
        ("according to", 0.1),
        ("et al", 0.15),
        ("doi:", 0.15),
        ("abstract", 0.08),
        ("published", 0.05),
        ("journal", 0.1),
        ("experiment", 0.1),
        ("data shows", 0.1),
        ("findings", 0.08),
    ];

    for (indicator, weight) in indicators {
        if text_lower.contains(indicator) {
            score += weight;
        }
    }

    // Count numbers/statistics (words containing digits)
    let stat_count = text.split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_ascii_digit()))
        .count() as f64;
    let stat_density = (stat_count / word_count).min(0.3);
    score += stat_density;

    score.clamp(0.0, 1.0)
}

// ─── Signal 6: Diversity ─────────────────────────────────────────

/// Diversity score: 1.0 for new domain, 0.2 for already-seen domain.
///
/// Context tracks seen domains; call this in result order.
pub fn diversity_score(result: &ResultItem, ctx: &mut ScoringContext) -> f64 {
    let domain = extract_domain(&result.url);
    if domain.is_empty() {
        return 0.5;
    }
    if ctx.seen_domains.contains(&domain) {
        0.2
    } else {
        ctx.seen_domains.insert(domain);
        1.0
    }
}

// ─── Signal 7: Depth ─────────────────────────────────────────────

/// Content depth score: min(word_count / 100, 1.0).
///
/// A snippet of 100+ words scores 1.0.
pub fn depth_score(result: &ResultItem) -> f64 {
    let word_count = result.snippet.split_whitespace().count() as f64;
    (word_count / 100.0).min(1.0)
}

// ─── Signal 8: Consensus ─────────────────────────────────────────

/// Consensus score: how much this result's content overlaps with other results.
///
/// High consensus = multiple sources confirm the same information.
/// Uses Jaccard similarity against concatenated text of all other results.
pub fn consensus_score(result: &ResultItem, ctx: &ScoringContext) -> f64 {
    if ctx.all_text.len() <= 1 {
        return 0.5;
    }

    let my_text = format!("{} {}", result.title, result.snippet).to_lowercase();
    let my_terms: HashSet<&str> = my_text.split_whitespace().collect();

    if my_terms.is_empty() {
        return 0.0;
    }

    // Count how many other results share at least 30% of query terms
    let query_set: HashSet<&str> = ctx.query_terms.iter().map(|s| s.as_str()).collect();

    let mut agreement_count = 0usize;
    let others = ctx.all_text.iter()
        .filter(|t| **t != my_text.as_str())
        .count();

    if others == 0 {
        return 0.5;
    }

    for other_text in &ctx.all_text {
        if other_text.as_str() == my_text.as_str() {
            continue;
        }
        let other_terms: HashSet<&str> = other_text.split_whitespace().collect();
        // Overlap with query terms
        let shared_query_terms = query_set.intersection(&other_terms).count();
        let my_query_terms = query_set.intersection(&my_terms).count();
        if my_query_terms > 0 && shared_query_terms as f64 / my_query_terms as f64 >= 0.3 {
            agreement_count += 1;
        }
    }

    trace!("consensus: {}/{} agreement for {:?}", agreement_count, others, result.title);
    (agreement_count as f64 / others as f64).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::BackendId;

    fn make_result(title: &str, url: &str, snippet: &str) -> ResultItem {
        ResultItem {
            title: title.into(),
            url: url.into(),
            snippet: snippet.into(),
            rank: 1,
            backend: BackendId::DuckDuckGo,
            score: None,
            published_date: None,
        }
    }

    fn make_ctx(query: &str, results: &[ResultItem]) -> ScoringContext {
        ScoringContext::new(query, results)
    }

    #[test]
    fn bm25_scores_relevant_higher() {
        let results = vec![
            make_result("Rust Language", "https://rust-lang.org", "systems programming language"),
            make_result("Python", "https://python.org", "dynamic scripting language"),
        ];
        let ctx = make_ctx("rust programming", &results);
        let rust_score = bm25_score(&results[0], &ctx);
        let py_score = bm25_score(&results[1], &ctx);
        assert!(rust_score > py_score, "rust={rust_score}, python={py_score}");
    }

    #[test]
    fn bm25_in_range() {
        let results = vec![make_result("test", "https://example.com", "foo bar baz")];
        let ctx = make_ctx("foo bar", &results);
        let s = bm25_score(&results[0], &ctx);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn temporal_unknown_date_is_neutral() {
        let r = make_result("test", "https://x.com", "content");
        assert_eq!(temporal_score(&r, 1.0), 0.5);
    }

    #[test]
    fn temporal_recent_higher_than_old() {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let old = "2000-01-01".to_string();

        let mut recent = make_result("recent", "https://x.com", "x");
        recent.published_date = Some(today);
        let mut aged = make_result("old", "https://x.com", "x");
        aged.published_date = Some(old);

        let recent_score = temporal_score(&recent, 1.0);
        let old_score = temporal_score(&aged, 1.0);
        assert!(recent_score > old_score, "recent={recent_score}, old={old_score}");
    }

    #[test]
    fn authority_wikipedia_high() {
        let r = make_result("Rust", "https://en.wikipedia.org/wiki/Rust", "A systems language");
        let score = authority_score(&r);
        assert!(score >= 0.9, "score={score}");
    }

    #[test]
    fn authority_random_blog_default() {
        let r = make_result("Blog Post", "https://my-random-blog.xyz/post", "some content");
        let score = authority_score(&r);
        assert!((0.4..=0.6).contains(&score), "score={score}");
    }

    #[test]
    fn authority_in_range() {
        let r = make_result("test", "https://stackoverflow.com/q/1", "How to use Rust?");
        let s = authority_score(&r);
        assert!((0.0..=1.0).contains(&s));
    }

    #[test]
    fn evidence_statistical_content() {
        let r = make_result(
            "Study",
            "https://nature.com/study",
            "According to research, 47% of users showed improvement. Study published in Nature et al. with doi:10.1234.",
        );
        let score = evidence_score(&r);
        assert!(score > 0.3, "score={score}");
    }

    #[test]
    fn evidence_plain_content_low() {
        let r = make_result("Home", "https://example.com", "Welcome to our website.");
        let score = evidence_score(&r);
        assert!(score < 0.5, "score={score}");
    }

    #[test]
    fn diversity_penalizes_same_domain() {
        let results = vec![
            make_result("A", "https://example.com/1", "content"),
            make_result("B", "https://example.com/2", "content"),
            make_result("C", "https://other.com/1", "content"),
        ];
        let mut ctx = make_ctx("test", &results);
        let s1 = diversity_score(&results[0], &mut ctx);
        let s2 = diversity_score(&results[1], &mut ctx);
        let s3 = diversity_score(&results[2], &mut ctx);
        assert_eq!(s1, 1.0);
        assert_eq!(s2, 0.2);
        assert_eq!(s3, 1.0);
    }

    #[test]
    fn depth_long_snippet_max() {
        let long_snippet = "word ".repeat(200);
        let r = make_result("test", "https://x.com", &long_snippet);
        assert_eq!(depth_score(&r), 1.0);
    }

    #[test]
    fn depth_empty_snippet_zero() {
        let r = make_result("test", "https://x.com", "");
        assert_eq!(depth_score(&r), 0.0);
    }

    #[test]
    fn consensus_scores_in_range() {
        let results = vec![
            make_result("Rust safety", "https://a.com", "rust is safe and fast systems language"),
            make_result("Rust performance", "https://b.com", "rust systems language with zero cost"),
            make_result("Python basics", "https://c.com", "python is dynamic and easy scripting"),
        ];
        let ctx = make_ctx("rust systems", &results);
        for r in &results {
            let s = consensus_score(r, &ctx);
            assert!((0.0..=1.0).contains(&s), "consensus out of range: {s}");
        }
    }
}
