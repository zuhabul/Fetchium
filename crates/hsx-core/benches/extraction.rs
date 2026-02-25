use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hsx_core::extract::layer1;

fn load_fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        // Fallback inline HTML if fixture missing
        "<html><body><article><h1>Test</h1><p>Content here.</p></article></body></html>".into()
    })
}

fn bench_extract_simple_article(c: &mut Criterion) {
    let html = load_fixture("simple-article.html");
    c.bench_function("extract/layer1/simple_article", |b| {
        b.iter(|| layer1::extract(black_box(&html), black_box("https://example.com/article")))
    });
}

fn bench_extract_table_heavy(c: &mut Criterion) {
    let html = load_fixture("table-heavy.html");
    c.bench_function("extract/layer1/table_heavy", |b| {
        b.iter(|| {
            layer1::extract(
                black_box(&html),
                black_box("https://benchmarks.example.com"),
            )
        })
    });
}

fn bench_extract_scaling(c: &mut Criterion) {
    let base_html = load_fixture("simple-article.html");
    let mut group = c.benchmark_group("extract/layer1/scaling");
    // PRD SS40 target: 250KB HTML in <100ms
    for multiplier in [1usize, 5, 10, 25, 50] {
        let large_html = base_html.repeat(multiplier);
        group.bench_with_input(
            BenchmarkId::new("html_kb", large_html.len() / 1024),
            &large_html,
            |b, html| {
                b.iter(|| {
                    layer1::extract(black_box(html.as_str()), black_box("https://example.com"))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_extract_simple_article,
    bench_extract_table_heavy,
    bench_extract_scaling
);
criterion_main!(benches);
