// Integration: ranking and deduplication pipeline tests.

use hsx_core::rank::rerank;
use hsx_core::types::{BackendId, ResultItem};

fn make_item(title: &str, url: &str, snippet: &str, rank: u32) -> ResultItem {
    ResultItem {
        title: title.to_string(),
        url: url.to_string(),
        snippet: snippet.to_string(),
        rank,
        backend: BackendId::DuckDuckGo,
        score: None,
        published_date: None,
    }
}

#[test]
fn rerank_sorts_by_relevance_descending() {
    let results = vec![
        make_item("Python Tutorial", "https://python.org/tutorial", "Learn Python basics", 0),
        make_item(
            "Rust Ownership Guide",
            "https://doc.rust-lang.org/book/ch04.html",
            "Rust ownership and borrowing explained",
            1,
        ),
        make_item(
            "JavaScript Closures",
            "https://js.info/closures",
            "Understanding JS closures",
            2,
        ),
    ];

    let ranked = rerank(results, "Rust ownership");

    // The Rust result should rank higher for a "Rust ownership" query
    assert!(!ranked.is_empty());
    // Scores should be in non-increasing order
    for window in ranked.windows(2) {
        let s0 = window[0].score.unwrap_or(0.0);
        let s1 = window[1].score.unwrap_or(0.0);
        assert!(
            s0 >= s1,
            "results must be sorted descending by score, but {s0} < {s1}"
        );
    }
    // The Rust result should be first or second
    let rust_pos = ranked
        .iter()
        .position(|r| r.url.contains("rust-lang"))
        .expect("Rust result must be present");
    assert!(rust_pos < 2, "Rust result should rank in top 2 for Rust query");
}

#[test]
fn rerank_empty_query_returns_original_order() {
    let results = vec![
        make_item("A", "https://a.com", "First", 0),
        make_item("B", "https://b.com", "Second", 1),
    ];
    let original_urls: Vec<_> = results.iter().map(|r| r.url.clone()).collect();
    let ranked = rerank(results, "");
    let ranked_urls: Vec<_> = ranked.iter().map(|r| r.url.clone()).collect();
    assert_eq!(original_urls, ranked_urls, "empty query should preserve order");
}

#[test]
fn rerank_empty_results_returns_empty() {
    let ranked = rerank(vec![], "some query");
    assert!(ranked.is_empty());
}

#[test]
fn rerank_reassigns_ranks_after_sorting() {
    let results = vec![
        make_item("Rust Book", "https://doc.rust-lang.org", "The Rust programming language book", 0),
        make_item("Python Tutorial", "https://python.org", "Learn Python basics", 1),
    ];
    let ranked = rerank(results, "Rust programming language");
    for (i, item) in ranked.iter().enumerate() {
        // rerank() assigns ranks starting from 1 (1-indexed)
        assert_eq!(item.rank as usize, i + 1, "rank should be reassigned 1-indexed after sorting");
    }
}

#[test]
fn rerank_single_item_returns_single() {
    let results = vec![make_item("Test", "https://test.com", "A test page", 0)];
    let ranked = rerank(results, "test");
    assert_eq!(ranked.len(), 1);
}

#[test]
fn rerank_all_results_get_scores() {
    let results = (0..5)
        .map(|i| make_item(&format!("Result {i}"), &format!("https://example.com/{i}"), "content about Rust", i as u32))
        .collect();
    let ranked = rerank(results, "Rust");
    for item in &ranked {
        assert!(item.score.is_some(), "every result should have a score after reranking");
    }
}
