use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hsx_core::rank::rerank;
use hsx_core::types::{BackendId, ResultItem};

fn make_results(count: usize) -> Vec<ResultItem> {
    (0..count)
        .map(|i| ResultItem {
            title: format!("Result {i}: Understanding Rust Ownership and Borrowing"),
            url: format!("https://example.com/rust-article-{i}"),
            snippet: format!(
                "A detailed explanation of Rust's ownership model for programmers. \
                 Part {i} covers the borrowing rules and lifetimes."
            ),
            rank: i as u32,
            backend: if i % 2 == 0 {
                BackendId::DuckDuckGo
            } else {
                BackendId::Google
            },
            score: None,
            published_date: None,
        })
        .collect()
}

fn bench_rerank(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank/bm25/rerank");
    // PRD SS40: BM25 rank 100 docs in <10ms
    for count in [10usize, 50, 100, 500] {
        let results = make_results(count);
        group.bench_with_input(
            BenchmarkId::new("result_count", count),
            &results,
            |b, results| {
                b.iter(|| {
                    rerank(
                        black_box(results.clone()),
                        black_box("Rust ownership borrowing"),
                    )
                })
            },
        );
    }
    group.finish();
}

fn bench_rerank_with_duplicates(c: &mut Criterion) {
    let mut results = make_results(80);
    // Add 20 near-duplicates (different query params on same URLs)
    for i in 0..20 {
        results.push(ResultItem {
            title: format!("Result {i}: Understanding Rust Ownership and Borrowing"),
            url: format!("https://example.com/rust-article-{i}?utm_source=hn"),
            snippet: format!("Duplicate of result {i}."),
            rank: (80 + i) as u32,
            backend: BackendId::DuckDuckGo,
            score: None,
            published_date: None,
        });
    }
    c.bench_function("rank/bm25/rerank_100_with_dupes", |b| {
        b.iter(|| {
            rerank(
                black_box(results.clone()),
                black_box("Rust ownership"),
            )
        })
    });
}

criterion_group!(benches, bench_rerank, bench_rerank_with_duplicates);
criterion_main!(benches);
