use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fetchium_core::token::{count_tokens, estimate_tokens_fast};

fn bench_count_tokens(c: &mut Criterion) {
    let mut group = c.benchmark_group("token/count_tokens");
    let texts: Vec<(&str, String)> = vec![
        ("short", "Hello world, this is a test.".into()),
        (
            "medium",
            "The quick brown fox jumps over the lazy dog. ".repeat(100),
        ),
        (
            "long",
            "Rust is a multi-paradigm systems programming language. ".repeat(1000),
        ),
    ];
    for (label, text) in &texts {
        group.bench_with_input(BenchmarkId::new("text", label), text.as_str(), |b, text| {
            b.iter(|| count_tokens(black_box(text)))
        });
    }
    group.finish();
}

fn bench_estimate_tokens_fast(c: &mut Criterion) {
    let mut group = c.benchmark_group("token/estimate_tokens_fast");
    let texts: Vec<(&str, String)> = vec![
        ("short", "Hello world.".into()),
        ("medium", "word ".repeat(10_000)),
        ("large", "word ".repeat(100_000)),
    ];
    for (label, text) in &texts {
        group.bench_with_input(BenchmarkId::new("text", label), text.as_str(), |b, text| {
            b.iter(|| estimate_tokens_fast(black_box(text)))
        });
    }
    group.finish();
}

fn bench_token_estimate_1mb(c: &mut Criterion) {
    // PRD SS40 target: token estimation on 1MB text in <100ms
    let large_text = "word ".repeat(200_000); // ~1MB
    c.bench_function("token/count_tokens_1mb", |b| {
        b.iter(|| count_tokens(black_box(&large_text)))
    });
    c.bench_function("token/estimate_fast_1mb", |b| {
        b.iter(|| estimate_tokens_fast(black_box(&large_text)))
    });
}

criterion_group!(
    benches,
    bench_count_tokens,
    bench_estimate_tokens_fast,
    bench_token_estimate_1mb
);
criterion_main!(benches);
