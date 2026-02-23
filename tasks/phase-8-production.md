# Phase 8: Testing, Benchmarks & Production Readiness

> **Duration:** Ongoing (parallel with all phases)
> **Priority:** P0-P1
> **Depends On:** Phase 0 (for CI), progressively builds alongside Phases 1-7
> **PRD Sections:** 40, 41, 45, 46, 47, 48

---

## Overview

Phase 8 runs continuously alongside development. It ensures every feature is tested, documented, benchmarked, and production-hardened. This phase is not "do testing at the end" -- it defines the test infrastructure, patterns, and standards that all other phases use from day one.

---

## Epic 8.1: Test Suite

### P8-E1-T1: Unit Tests with cargo test

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T1` |
| **Status** | `TODO` |
| **Priority** | P0 |
| **Description** | Establish unit test patterns and achieve comprehensive coverage across all core modules: ranker, chunker, QATBE, SCS, CEP, HyperFusion, PDS, embeddings, cache, and all intelligence algorithms. Every module should have a `tests` submodule with property-based tests where applicable. |
| **PRD Ref** | 45 (Testing Strategy - Unit level) |
| **Depends On** | `P0-E1` (project scaffolding) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| All `src/**/*.rs` files | Add `#[cfg(test)] mod tests { ... }` blocks |
| `crates/hsx-core/tests/` | Integration-style unit tests |
| `Cargo.toml` (workspace) | Add `proptest` as dev-dependency |

#### Test Patterns for Each Module

**Pattern 1: Pure Function Tests (BM25, Token Counter, SimHash)**

```rust
// crates/hsx-core/src/ranking/bm25.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_exact_match_scores_highest() {
        let query = "rust web framework";
        let exact_match = "rust web framework comparison and benchmarks";
        let partial_match = "web framework for python django";
        let no_match = "cooking recipes for beginners";

        let score_exact = bm25_score(exact_match, query);
        let score_partial = bm25_score(partial_match, query);
        let score_none = bm25_score(no_match, query);

        assert!(score_exact > score_partial, "Exact match should score higher");
        assert!(score_partial > score_none, "Partial match should score higher than no match");
        assert!(score_none >= 0.0, "BM25 scores should be non-negative");
    }

    #[test]
    fn bm25_empty_query_returns_zero() {
        assert_eq!(bm25_score("some content", ""), 0.0);
    }

    #[test]
    fn bm25_empty_content_returns_zero() {
        assert_eq!(bm25_score("", "some query"), 0.0);
    }

    #[test]
    fn bm25_handles_unicode() {
        let score = bm25_score("Rust框架比较", "Rust框架");
        assert!(score > 0.0, "BM25 should handle CJK characters");
    }
}
```

**Pattern 2: Async Tests (HTTP, Search, Extraction)**

```rust
// crates/hsx-core/src/extraction/cep.rs

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::method;

    #[tokio::test]
    async fn cep_layer1_extracts_static_html() {
        // Set up mock server
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(r#"
                    <html>
                    <body>
                        <h1>Test Page</h1>
                        <p>This is a test paragraph with useful content.</p>
                        <nav>Navigation should be removed</nav>
                    </body>
                    </html>
                "#)
                .insert_header("content-type", "text/html"))
            .mount(&mock_server)
            .await;

        let result = extract(&mock_server.uri(), &CepConfig::default()).await.unwrap();

        assert_eq!(result.extraction_layer, 1, "Static HTML should use Layer 1");
        assert!(result.content.contains("test paragraph"), "Content should be extracted");
        assert!(!result.content.contains("Navigation"), "Nav should be stripped");
    }

    #[tokio::test]
    async fn cep_escalates_on_empty_content() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("<html><body><div id='root'></div><script src='app.js'></script></body></html>")
                .insert_header("content-type", "text/html"))
            .mount(&mock_server)
            .await;

        let result = extract(&mock_server.uri(), &CepConfig::default()).await.unwrap();

        assert!(result.extraction_layer >= 2, "Empty body should escalate beyond Layer 1");
    }
}
```

**Pattern 3: Property-Based Tests (Ranking, Token Counting)**

```rust
// crates/hsx-core/src/tokens/counter.rs

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn token_count_never_negative(s in "\\PC{0,1000}") {
            let count = count_tokens(&s);
            prop_assert!(count >= 0);
        }

        #[test]
        fn token_count_monotonic_with_length(
            a in "\\PC{0,100}",
            b in "\\PC{0,100}",
        ) {
            let combined = format!("{} {}", a, b);
            let count_a = count_tokens(&a);
            let count_b = count_tokens(&b);
            let count_combined = count_tokens(&combined);

            // Token count of "A B" should be <= count(A) + count(B) + 1
            prop_assert!(count_combined <= count_a + count_b + 1);
        }

        #[test]
        fn hyperfusion_scores_are_finite(
            bm25 in 0.0f64..100.0,
            semantic in 0.0f64..1.0,
            temporal in 0.0f64..1.0,
        ) {
            let weights = HyperFusionWeights::default();
            let score = weights.bm25 * bm25
                + weights.semantic * semantic
                + weights.temporal * temporal;

            prop_assert!(score.is_finite());
            prop_assert!(score >= 0.0);
        }
    }
}
```

**Pattern 4: Data-Driven Tests (SCS Segmentation)**

```rust
// crates/hsx-core/src/segmentation/scs.rs

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        html: &'static str,
        expected_types: Vec<SegmentType>,
        description: &'static str,
    }

    #[test]
    fn scs_segments_content_types_correctly() {
        let cases = vec![
            TestCase {
                html: "<p>Paris is the capital of France.</p>",
                expected_types: vec![SegmentType::Paragraph],
                description: "Simple paragraph",
            },
            TestCase {
                html: "<table><tr><th>Name</th></tr><tr><td>Alice</td></tr></table>",
                expected_types: vec![SegmentType::Table],
                description: "HTML table",
            },
            TestCase {
                html: "<pre><code class='language-rust'>fn main() {}</code></pre>",
                expected_types: vec![SegmentType::Code],
                description: "Code block",
            },
            TestCase {
                html: "<ul><li>Item 1</li><li>Item 2</li></ul>",
                expected_types: vec![SegmentType::List],
                description: "Unordered list",
            },
            TestCase {
                html: "<blockquote>Famous quote here</blockquote>",
                expected_types: vec![SegmentType::Quote],
                description: "Blockquote",
            },
        ];

        for case in cases {
            let segments = segment_html(case.html).unwrap();
            let types: Vec<SegmentType> = segments.iter().map(|s| s.seg_type.clone()).collect();
            assert_eq!(
                types, case.expected_types,
                "Failed for: {} (html: {})", case.description, case.html
            );
        }
    }
}
```

**Pattern 5: QATBE Budget Compliance Tests**

```rust
// crates/hsx-core/src/extraction/qatbe.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qatbe_respects_token_budget() {
        let segments = vec![
            ContentSegment { text: "A ".repeat(500), tokens: 500, relevance: 0.9, .. },
            ContentSegment { text: "B ".repeat(300), tokens: 300, relevance: 0.8, .. },
            ContentSegment { text: "C ".repeat(400), tokens: 400, relevance: 0.7, .. },
            ContentSegment { text: "D ".repeat(200), tokens: 200, relevance: 0.6, .. },
        ];

        let budget = 800;
        let packed = budget_pack(&segments, budget);
        let total_tokens: usize = packed.iter().map(|s| s.tokens).sum();

        assert!(
            total_tokens <= budget,
            "Total tokens {} should not exceed budget {}",
            total_tokens, budget
        );
    }

    #[test]
    fn qatbe_packs_highest_relevance_first() {
        let segments = vec![
            ContentSegment { tokens: 100, relevance: 0.5, .. },
            ContentSegment { tokens: 100, relevance: 0.9, .. },
            ContentSegment { tokens: 100, relevance: 0.7, .. },
        ];

        let packed = budget_pack(&segments, 200);

        assert_eq!(packed.len(), 2);
        assert!(packed[0].relevance >= packed[1].relevance, "Higher relevance should come first");
    }

    #[test]
    fn qatbe_handles_zero_budget() {
        let segments = vec![
            ContentSegment { tokens: 100, relevance: 0.9, .. },
        ];

        let packed = budget_pack(&segments, 0);
        assert!(packed.is_empty(), "Zero budget should return no segments");
    }
}
```

#### Acceptance Criteria

- [ ] Every public function has at least one unit test
- [ ] Test coverage > 80% for `hsx-core` crate (measured by `cargo-tarpaulin`)
- [ ] Property-based tests for: token counting, BM25 scoring, HyperFusion, SimHash
- [ ] Data-driven tests for: SCS segmentation, CEP layer selection, QATBE budget packing
- [ ] Async tests for: HTTP fetching, search backends, extraction pipeline
- [ ] All tests pass with `cargo test --workspace`
- [ ] No `#[ignore]` tests without a documented reason
- [ ] Tests run in < 30 seconds (excluding network tests)

#### Pitfalls

- **Flaky tests**: Tests depending on external services are flaky. Always use `wiremock` or mock servers for HTTP tests.
- **Test isolation**: Each test should be independent. Don't share mutable state between tests.
- **Slow tests**: Property-based tests can be slow with many iterations. Use `proptest::test_runner::Config` to limit iterations in CI.

---

### P8-E1-T2: Integration Tests

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T2` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Build integration tests that verify the full pipeline works end-to-end within the Rust codebase. Tests exercise: fetch -> extract -> rank -> output, and search -> fetch -> QATBE -> SCS -> PDS flows. |
| **PRD Ref** | 45 (Testing Strategy - Integration level) |
| **Depends On** | `P8-E1-T1`, Phase 1 modules |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/tests/integration_pipeline.rs` | Pipeline integration tests |
| `crates/hsx-core/tests/integration_search.rs` | Search flow integration tests |
| `crates/hsx-core/tests/fixtures/` | HTML fixtures for test pages |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/tests/integration_pipeline.rs

use hsx_core::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

/// Test the full pipeline: fetch -> extract -> segment -> rank -> output
#[tokio::test]
async fn full_extraction_pipeline() {
    let mock = MockServer::start().await;

    let html = include_str!("fixtures/sample_article.html");
    Mock::given(method("GET"))
        .and(path("/article"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(html)
            .insert_header("content-type", "text/html"))
        .mount(&mock)
        .await;

    let url = format!("{}/article", mock.uri());
    let query = "Rust performance benchmarks";
    let budget = 1500;

    // Step 1: Fetch and extract via CEP
    let extracted = extraction::cep::extract(&url, &Default::default()).await.unwrap();
    assert!(!extracted.content.is_empty());

    // Step 2: Segment via SCS
    let segments = segmentation::scs::segment(&extracted.content).unwrap();
    assert!(!segments.is_empty());

    // Step 3: Rank via QATBE
    let mut ranked = segments.clone();
    extraction::qatbe::rank_segments(&mut ranked, query).unwrap();
    assert!(ranked[0].relevance >= ranked.last().unwrap().relevance);

    // Step 4: Budget pack
    let packed = extraction::qatbe::budget_pack(&ranked, budget);
    let total_tokens: usize = packed.iter().map(|s| s.tokens).sum();
    assert!(total_tokens <= budget);

    // Step 5: Generate PDS tiers
    let tiers = pds::tiers::generate_all_tiers(&packed, query).unwrap();
    assert!(tiers.key_facts.tokens <= 250);
    assert!(tiers.summary.tokens <= 1200);
}

/// Test search -> rank -> output flow with mock backends.
#[tokio::test]
async fn full_search_flow() {
    let mock = MockServer::start().await;

    // Mock DDG-like response
    Mock::given(method("GET"))
        .and(path("/search"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(include_str!("fixtures/mock_search_results.html")))
        .mount(&mock)
        .await;

    let config = search::SearchConfig {
        query: "Rust web framework".into(),
        max_sources: 5,
        // Override backend URL to point to mock
        ..Default::default()
    };

    let results = search::search(&config.query, &config).await.unwrap();
    assert!(!results.is_empty(), "Search should return results");
    assert!(results[0].fusion_score >= results.last().unwrap().fusion_score,
        "Results should be sorted by score");
}
```

#### Acceptance Criteria

- [ ] Full pipeline test (fetch -> extract -> segment -> rank -> output) passes
- [ ] Full search test (search -> rank -> format) passes
- [ ] Integration tests use `wiremock` mock servers, not live HTTP
- [ ] HTML test fixtures stored in `tests/fixtures/` for reproducibility
- [ ] Tests verify correct data flow between pipeline stages
- [ ] At least 5 different page types tested (article, SPA shell, table-heavy, code-heavy, list-heavy)

#### Pitfalls

- **Fixture maintenance**: HTML fixtures can become outdated. Document what each fixture tests and when it was created.
- **Port conflicts**: Mock servers should use random ports (wiremock default) to avoid conflicts in parallel test runs.

---

### P8-E1-T3: E2E Tests with assert_cmd + predicates

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T3` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Build end-to-end tests that invoke the actual `hsx` binary and verify CLI behavior, exit codes, output format, and error handling using `assert_cmd` and `predicates` crates. |
| **PRD Ref** | 45 (Testing Strategy - E2E level) |
| **Depends On** | `P8-E1-T1`, `P0-E3` (CLI skeleton) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-cli/tests/e2e_search.rs` | Search command E2E tests |
| `crates/hsx-cli/tests/e2e_fetch.rs` | Fetch command E2E tests |
| `crates/hsx-cli/tests/e2e_agent.rs` | Agent command E2E tests |
| `crates/hsx-cli/tests/e2e_doctor.rs` | Doctor command E2E tests |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-cli/tests/e2e_doctor.rs
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn doctor_runs_successfully() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("CPU:"))
        .stdout(predicate::str::contains("RAM:"))
        .stdout(predicate::str::contains("Tier:"));
}

#[test]
fn version_flag_works() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hypersearchx"));
}

#[test]
fn unknown_command_shows_help() {
    Command::cargo_bin("hsx")
        .unwrap()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// crates/hsx-cli/tests/e2e_agent.rs

#[test]
fn agent_search_returns_valid_json() {
    // This test requires a mock server or --offline mode
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["agent-search", "test query", "--format", "json", "--budget", "500"])
        .env("HSX_TEST_MODE", "1") // Use test fixtures
        .assert()
        .success()
        .stdout(predicate::str::is_match(r#"\{.*"tokens".*\}"#).unwrap());
}

#[test]
fn agent_search_respects_tier_flag() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["agent-search", "test", "--tier", "key_facts", "--format", "json"])
        .env("HSX_TEST_MODE", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("key_facts"));
}

#[test]
fn fetch_with_invalid_url_shows_error() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["fetch", "not-a-valid-url"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL")
            .or(predicate::str::contains("error")));
}

#[test]
fn search_quiet_mode_suppresses_output() {
    Command::cargo_bin("hsx")
        .unwrap()
        .args(["search", "test", "--quiet", "--format", "json"])
        .env("HSX_TEST_MODE", "1")
        .assert()
        .success();
    // In quiet mode, only JSON output, no decorations
}
```

#### Acceptance Criteria

- [ ] E2E tests cover: search, fetch, view, doctor, version, agent-search, agent-fetch
- [ ] Tests verify exit codes (0 for success, non-zero for errors)
- [ ] Tests verify output format compliance (JSON is valid, markdown has expected sections)
- [ ] Error cases tested: invalid URL, invalid flags, network failure
- [ ] Tests work in CI without network access (using `HSX_TEST_MODE` or mock servers)
- [ ] E2E tests run in < 60 seconds total

#### Pitfalls

- **Binary availability**: `assert_cmd::Command::cargo_bin` requires the binary to be built. CI should build before testing.
- **Network dependency**: E2E tests that hit real servers will fail in CI. Use environment variable to switch to test mode.

---

### P8-E1-T4: Benchmarks with criterion

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T4` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Create a comprehensive benchmark suite using `criterion` that measures latency and throughput for all critical paths. Benchmarks run in CI to detect performance regressions. |
| **PRD Ref** | 40 (Performance Requirements), 45 (Benchmark level), 47 (Success Metrics) |
| **Depends On** | `P8-E1-T1`, core modules from Phase 1 |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/benches/ranking.rs` | HyperFusion + BM25 benchmarks |
| `crates/hsx-core/benches/extraction.rs` | CEP + QATBE benchmarks |
| `crates/hsx-core/benches/segmentation.rs` | SCS benchmarks |
| `crates/hsx-core/benches/tokens.rs` | Token counting benchmarks |
| `crates/hsx-core/benches/embedding.rs` | Embedding generation benchmarks |
| `crates/hsx-core/benches/cache.rs` | Cache read/write benchmarks |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/benches/ranking.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_bm25_scoring(c: &mut Criterion) {
    let query = "rust web framework performance benchmarks";
    let documents: Vec<String> = (0..100)
        .map(|i| format!("Document {} about Rust web frameworks and their performance in production systems", i))
        .collect();

    c.bench_function("bm25_score_100_docs", |b| {
        b.iter(|| {
            for doc in &documents {
                black_box(hsx_core::ranking::bm25::bm25_score(doc, query));
            }
        })
    });
}

fn bench_hyperfusion_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("hyperfusion");

    for size in [10, 50, 100, 500] {
        let mut results: Vec<SearchResult> = (0..size)
            .map(|i| SearchResult::mock(i))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("rank", size),
            &size,
            |b, _| {
                b.iter(|| {
                    hsx_core::ranking::hyperfusion::rank_results(
                        "test query",
                        black_box(&mut results.clone()),
                        &QueryIntent::default(),
                    ).unwrap()
                })
            },
        );
    }
    group.finish();
}

fn bench_token_counting(c: &mut Criterion) {
    let texts = vec![
        ("short", "Hello world"),
        ("medium", &"This is a medium length text. ".repeat(50)),
        ("long", &"This is a longer document with many words. ".repeat(500)),
    ];

    let mut group = c.benchmark_group("token_counting");
    for (name, text) in &texts {
        group.bench_with_input(
            BenchmarkId::new("count_tokens", name),
            text,
            |b, text| {
                b.iter(|| hsx_core::tokens::count_tokens(black_box(text)))
            },
        );
    }
    group.finish();
}

// crates/hsx-core/benches/extraction.rs

fn bench_qatbe_pipeline(c: &mut Criterion) {
    let html = include_str!("../tests/fixtures/sample_article.html");
    let query = "performance benchmarks";

    c.bench_function("qatbe_full_pipeline", |b| {
        b.iter(|| {
            let segments = hsx_core::segmentation::scs::segment(black_box(html)).unwrap();
            let mut ranked = segments;
            hsx_core::extraction::qatbe::rank_segments(&mut ranked, query).unwrap();
            hsx_core::extraction::qatbe::budget_pack(&ranked, 1500)
        })
    });
}

fn bench_scs_segmentation(c: &mut Criterion) {
    let html = include_str!("../tests/fixtures/sample_article.html");

    c.bench_function("scs_segment_article", |b| {
        b.iter(|| {
            hsx_core::segmentation::scs::segment(black_box(html)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_bm25_scoring,
    bench_hyperfusion_ranking,
    bench_token_counting,
    bench_qatbe_pipeline,
    bench_scs_segmentation,
);
criterion_main!(benches);
```

#### Performance Targets (from PRD 40)

| Benchmark | Target |
|-----------|--------|
| BM25 score 100 docs | < 1ms |
| HyperFusion rank 50 results | < 10ms |
| Token count 10KB text | < 1ms |
| SCS segment full article | < 5ms |
| QATBE full pipeline | < 50ms |
| Cache read (SQLite) | < 5ms |
| Embedding 1 text (cached) | < 1ms |
| Embedding 1 text (uncached) | < 100ms |

#### Acceptance Criteria

- [ ] Benchmarks exist for all critical paths listed above
- [ ] All benchmarks meet their performance targets
- [ ] `cargo bench` produces HTML reports in `target/criterion/`
- [ ] CI runs benchmarks and compares to previous run (criterion baseline comparison)
- [ ] Performance regressions > 10% trigger CI warnings
- [ ] Benchmarks use realistic data (not toy examples)

#### Pitfalls

- **Benchmark variance**: CPU throttling, background processes, and thermal throttling cause variance. Use `--warm-up-time 5` and `--measurement-time 10` for stable results.
- **Criterion baselines**: Save baselines in CI artifacts for regression detection. Use `--save-baseline main` on the main branch.

---

### P8-E1-T5: Snapshot Tests with insta

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T5` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Use `insta` for snapshot testing of extraction output, segmentation results, and CLI output formatting. Snapshots catch unintended output changes. |
| **PRD Ref** | 45 (Extraction snapshot tests) |
| **Depends On** | `P8-E1-T1`, `P1-E1` (extraction) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/extraction/mod.rs` | Snapshot tests for extraction |
| `crates/hsx-core/src/segmentation/mod.rs` | Snapshot tests for SCS |
| `crates/hsx-core/tests/snapshots/` | Snapshot files (auto-generated by insta) |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/extraction/mod.rs

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn extraction_snapshot_simple_article() {
        let html = include_str!("../../tests/fixtures/simple_article.html");
        let result = extract_from_html(html, &ExtractConfig::default()).unwrap();

        // Snapshot the extraction result
        assert_json_snapshot!("simple_article_extraction", result, {
            ".fetched_at" => "[timestamp]",
            ".content_hash" => "[hash]",
        });
    }

    #[test]
    fn scs_snapshot_table_page() {
        let html = include_str!("../../tests/fixtures/table_heavy_page.html");
        let segments = scs::segment(html).unwrap();

        assert_json_snapshot!("table_page_segments", segments, {
            "[].tokens" => insta::rounded_redaction(1),
        });
    }

    #[test]
    fn pds_snapshot_key_facts() {
        let segments = create_test_segments();
        let tiers = pds::tiers::generate_all_tiers(&segments, "test query").unwrap();

        assert_json_snapshot!("key_facts_tier", tiers.key_facts);
    }
}
```

#### Acceptance Criteria

- [ ] Snapshot tests exist for: CEP extraction output, SCS segments, PDS tiers, CLI formatted output
- [ ] Snapshots are reviewed on every PR that changes them
- [ ] `cargo insta review` used to accept or reject snapshot changes
- [ ] Redactions applied for non-deterministic fields (timestamps, hashes, UUIDs)
- [ ] At least 10 HTML fixtures with corresponding snapshots

#### Pitfalls

- **Non-deterministic output**: Timestamps, UUIDs, and hashes differ between runs. Use `insta` redactions to mask them.
- **Snapshot bloat**: Large snapshots are hard to review. Keep snapshot scope narrow (individual segment, not full page).

---

### P8-E1-T6: Fuzz Testing with cargo-fuzz

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T6` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Set up fuzz testing for security-sensitive parsers: HTML parsing, URL handling, JSON deserialization, and configuration parsing. Fuzz targets ensure no panics or undefined behavior on malformed input. |
| **PRD Ref** | 45 (Fuzz level), 41 (Security & Compliance) |
| **Depends On** | `P8-E1-T1`, `P1-E1` (HTML parsing) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `fuzz/Cargo.toml` | Fuzz harness project |
| `fuzz/fuzz_targets/html_parse.rs` | HTML parser fuzz target |
| `fuzz/fuzz_targets/url_handling.rs` | URL parser fuzz target |
| `fuzz/fuzz_targets/json_deser.rs` | JSON deserialization fuzz target |
| `fuzz/fuzz_targets/config_parse.rs` | Config file parser fuzz target |

#### Step-by-Step Implementation Guide

```rust
// fuzz/fuzz_targets/html_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(html) = std::str::from_utf8(data) {
        // These should NEVER panic, regardless of input
        let _ = hsx_core::extraction::extract_from_html(html, &Default::default());
        let _ = hsx_core::segmentation::scs::segment(html);
        let _ = hsx_core::extraction::qadd::structural_pruning_from_html(html);
    }
});

// fuzz/fuzz_targets/url_handling.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(url_str) = std::str::from_utf8(data) {
        let _ = hsx_core::util::normalize_url(url_str);
        let _ = hsx_core::util::extract_domain(url_str);
        let _ = hsx_core::extraction::cep_features::CepFeatures::from_url(url_str);
    }
});

// fuzz/fuzz_targets/json_deser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Ensure deserialization of untrusted JSON never panics
    let _ = serde_json::from_slice::<hsx_core::types::AgentSearchResult>(data);
    let _ = serde_json::from_slice::<hsx_core::config::Config>(data);
    let _ = serde_json::from_slice::<hsx_core::types::Segment>(data);
});
```

#### Acceptance Criteria

- [ ] Fuzz targets exist for: HTML parsing, URL handling, JSON deserialization, YAML config parsing
- [ ] Zero panics found after 1 hour of fuzzing per target
- [ ] Fuzz corpus committed to repo for reproducibility
- [ ] CI runs fuzz tests for 5 minutes per target on each PR
- [ ] Any panic found by fuzzing is fixed and a regression test added

#### Pitfalls

- **Fuzzing time**: Fuzzing is open-ended. Set time limits in CI (5 minutes per target) and run longer sessions locally/nightly.
- **libFuzzer vs AFL**: `cargo-fuzz` uses libFuzzer by default. Consider also running with AFL++ for different mutation strategies.

---

### P8-E1-T7: Concurrency Testing with loom

| Field | Value |
|-------|-------|
| **ID** | `P8-E1-T7` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Use `loom` to verify correctness of concurrent data structures: embedding cache, PIE SQLite access, browser pool, and worker pool. Loom exhaustively explores thread interleavings to find data races and deadlocks. |
| **PRD Ref** | 45 (Concurrency testing with loom) |
| **Depends On** | `P8-E1-T1`, concurrent modules |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/cache/tests_loom.rs` | Loom tests for cache |
| `crates/hsx-core/src/intelligence/tests_loom.rs` | Loom tests for PIE |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/cache/tests_loom.rs

#[cfg(loom)]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;

    #[test]
    fn concurrent_cache_read_write() {
        loom::model(|| {
            let cache = Arc::new(super::MemoryCache::new(100));

            let cache_w = cache.clone();
            let writer = thread::spawn(move || {
                cache_w.put("key1", "value1");
                cache_w.put("key2", "value2");
            });

            let cache_r = cache.clone();
            let reader = thread::spawn(move || {
                // Reader should see either None or a valid value, never partial
                let val = cache_r.get("key1");
                if let Some(v) = val {
                    assert_eq!(v, "value1");
                }
            });

            writer.join().unwrap();
            reader.join().unwrap();
        });
    }

    #[test]
    fn concurrent_pie_trust_updates() {
        loom::model(|| {
            let stm = Arc::new(super::SourceTrustMemory::new_in_memory().unwrap());

            let stm1 = stm.clone();
            let t1 = thread::spawn(move || {
                stm1.update_trust("example.com", true, 0.9).unwrap();
            });

            let stm2 = stm.clone();
            let t2 = thread::spawn(move || {
                stm2.update_trust("example.com", false, 0.0).unwrap();
            });

            t1.join().unwrap();
            t2.join().unwrap();

            // Trust score should be valid (between 0 and 1)
            let trust = stm.get_trust("example.com").unwrap();
            assert!(trust >= 0.0 && trust <= 1.0);
        });
    }
}
```

#### Acceptance Criteria

- [ ] Loom tests cover: in-memory cache, SQLite cache, PIE trust memory, embedding cache
- [ ] No deadlocks or data races found by loom
- [ ] Loom tests run in CI (with `RUSTFLAGS="--cfg loom"`)
- [ ] Any issues found are fixed with proper synchronization primitives

#### Pitfalls

- **Loom state space**: Loom explores all interleavings, which can be exponentially large. Keep test scenarios small (2-3 threads, few operations).
- **Loom compatibility**: Not all crates work with loom. Use loom's own `Arc`, `Mutex`, `thread` types within loom tests.

---

## Epic 8.2: Documentation

### P8-E2-T1: cargo doc, User Guide, MCP Setup Guide

| Field | Value |
|-------|-------|
| **ID** | `P8-E2-T1` |
| **Status** | `TODO` |
| **Priority** | P1 |
| **Description** | Ensure all public APIs have `///` doc comments that render correctly with `cargo doc`. Write a user guide covering installation, configuration, and all commands. Write an MCP setup guide for Claude/Claude Code integration. |
| **PRD Ref** | 46 (Documentation site), 30 (MCP server mode) |
| **Depends On** | All core modules implemented |

#### Files to Create/Modify

| File | Action |
|------|--------|
| All `src/lib.rs` and `src/*/mod.rs` files | Crate-level and module-level `//!` docs |
| All public structs/functions | `///` doc comments with examples |
| `docs/user-guide.md` | User guide (only when explicitly requested) |
| `docs/mcp-setup.md` | MCP setup guide |

#### Documentation Standards

```rust
/// Rank search results using the HyperFusion 8-signal algorithm.
///
/// HyperFusion combines BM25, semantic similarity, temporal decay, authority,
/// evidence density, diversity, depth, and consensus signals with intent-adaptive
/// weights (see PRD 8.1).
///
/// # Arguments
///
/// * `query` - The search query string
/// * `results` - Mutable slice of results to rank in-place
/// * `intent` - Query intent classification for weight adaptation
///
/// # Errors
///
/// Returns `Error::Embedding` if semantic signal computation fails and the
/// `embeddings` feature is enabled.
///
/// # Examples
///
/// ```rust
/// use hsx_core::ranking::hyperfusion;
/// use hsx_core::types::{SearchResult, QueryIntent};
///
/// let mut results = vec![SearchResult::default(); 10];
/// let intent = QueryIntent::classify("best rust framework");
/// hyperfusion::rank_results("best rust framework", &mut results, &intent)?;
///
/// // Results are now sorted by fusion score descending
/// assert!(results[0].fusion_score >= results[9].fusion_score);
/// # Ok::<(), hsx_core::Error>(())
/// ```
pub fn rank_results(
    query: &str,
    results: &mut [SearchResult],
    intent: &QueryIntent,
) -> Result<(), crate::Error> { /* ... */ }
```

#### MCP Setup Guide Content

```markdown
# MCP Setup Guide for HyperSearchX

## Claude Desktop Integration

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"]
    }
  }
}
```

## Claude Code Integration

Add to your project's `.mcp.json`:

```json
{
  "servers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"],
      "env": {}
    }
  }
}
```

## Available MCP Tools

- `hypersearch_search` - Token-budgeted web search
- `hypersearch_fetch` - Query-aware content extraction
- `hypersearch_research` - Multi-source research with citations
- `hypersearch_estimate` - Pre-fetch token estimation
- `hypersearch_expand` - Tier expansion without re-fetching
```

#### Acceptance Criteria

- [ ] `cargo doc --workspace --no-deps` generates without warnings
- [ ] Every public type, trait, function, and method has `///` documentation
- [ ] All doc examples compile and pass (`cargo test --doc`)
- [ ] Crate-level docs (`//!`) explain the crate's purpose and architecture
- [ ] Module-level docs explain the module's role in the pipeline
- [ ] MCP setup guide covers Claude Desktop, Claude Code, and custom MCP clients
- [ ] User guide covers installation via npm/cargo, all CLI commands, and configuration

#### Pitfalls

- **Doc test failures**: Doc examples that use `?` need a function signature returning `Result`. Use `# Ok::<(), Error>(())` at the end.
- **Broken links**: `cargo doc` will warn about broken intra-doc links. Fix all of them.
- **Feature-gated docs**: Items behind feature flags need `#[cfg_attr(docsrs, doc(cfg(feature = "embeddings")))]` to show the feature requirement in docs.

---

## Epic 8.3: Production Hardening

### P8-E3-T1: Security Audit, Performance Optimization, Error Handling, Release Automation

| Field | Value |
|-------|-------|
| **ID** | `P8-E3-T1` |
| **Status** | `TODO` |
| **Priority** | P0 |
| **Description** | Comprehensive production hardening: security audit per PRD 41, performance optimization to meet PRD 40 targets, graceful error handling audit, and release automation with cross-compilation and npm publish. |
| **PRD Ref** | 40 (Performance), 41 (Security), 44 (Error Handling), 46 (CI/CD) |
| **Depends On** | All phases, `P8-E1-*` (test suite), `P8-E2-T1` (docs) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `.github/workflows/release.yml` | Release automation workflow |
| `.github/workflows/security.yml` | Security audit workflow |
| `npm/` | npm wrapper package for publishing |
| `npm/package.json` | npm package manifest |
| `npm/scripts/install-binary.js` | Platform-specific binary installer |
| `crates/hsx-core/src/error.rs` | Error handling audit |

#### Security Audit Checklist (per PRD 41)

```rust
// Security audit implementation patterns:

// 1. No credential storage
// VERIFY: grep for "password", "secret", "token", "key" in stored data
// ENSURE: No third-party credentials written to disk

// 2. Sanitized output
pub fn sanitize_html(html: &str) -> String {
    ammonia::clean(html) // Use ammonia crate for HTML sanitization
}

// 3. TLS enforcement
pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .https_only(true)           // Enforce HTTPS
        .min_tls_version(reqwest::tls::Version::TLS_1_2)
        .danger_accept_invalid_certs(false) // Never skip cert validation
        .build()
        .unwrap()
}

// 4. Input validation for all user-provided URLs
pub fn validate_url(url: &str) -> Result<url::Url, crate::Error> {
    let parsed = url::Url::parse(url)?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed),
        scheme => Err(crate::Error::Validation(
            format!("Unsupported URL scheme: {}. Only http/https allowed.", scheme)
        )),
    }
}

// 5. Never bypass CAPTCHA or paywalls
// VERIFY: No CAPTCHA solving code exists
// VERIFY: No paywall detection bypass

// 6. Rate limiting respected
pub struct DomainRateLimiter {
    limits: DashMap<String, Instant>,
    delay: Duration,
}

impl DomainRateLimiter {
    pub async fn wait(&self, domain: &str) {
        if let Some(last) = self.limits.get(domain) {
            let elapsed = last.elapsed();
            if elapsed < self.delay {
                tokio::time::sleep(self.delay - elapsed).await;
            }
        }
        self.limits.insert(domain.to_string(), Instant::now());
    }
}
```

#### Performance Optimization Targets (PRD 40)

```rust
// Performance optimization patterns:

// 1. Connection pooling
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .tcp_keepalive(Duration::from_secs(60))
    .build()?;

// 2. Streaming HTML parsing (zero-copy where possible)
// Use lol_html for streaming rewriting instead of full DOM parsing
use lol_html::{element, HtmlRewriter, Settings};

// 3. SIMD-accelerated JSON parsing
#[cfg(feature = "simd-json")]
fn parse_json(bytes: &mut [u8]) -> Result<serde_json::Value, crate::Error> {
    simd_json::from_slice(bytes).map_err(Into::into)
}

// 4. Lazy initialization of expensive resources
use std::sync::OnceLock;
static ONNX_SESSION: OnceLock<Session> = OnceLock::new();
static TANTIVY_INDEX: OnceLock<tantivy::Index> = OnceLock::new();

// 5. Memory-mapped file I/O for large indexes
use memmap2::MmapOptions;
```

#### Error Handling Audit

```rust
// Ensure ALL error paths follow the structured error taxonomy (PRD 44):

// crates/hsx-core/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network timeout after {timeout_ms}ms for {url}")]
    NetworkTimeout { url: String, timeout_ms: u64 },

    #[error("DNS resolution failed for {domain}")]
    DnsFailure { domain: String },

    #[error("HTTP {status} from {url}")]
    HttpError { status: u16, url: String },

    #[error("Anti-bot detection on {domain}: {details}")]
    AntiBot { domain: String, details: String },

    #[error("Paywall detected on {url}")]
    Paywall { url: String },

    #[error("Extraction failed for {url}: {reason}")]
    ExtractionFailed { url: String, reason: String },

    #[error("AI model unavailable: {reason}")]
    AiUnavailable { reason: String },

    #[error("Token budget exceeded: {used} > {budget}")]
    BudgetExceeded { used: usize, budget: usize },

    // ... etc for all error types
}

impl Error {
    /// Is this error retryable?
    pub fn retryable(&self) -> bool {
        matches!(self,
            Error::NetworkTimeout { .. } |
            Error::HttpError { status, .. } if *status >= 500 |
            Error::DnsFailure { .. }
        )
    }

    /// Suggested action for the user/agent.
    pub fn suggested_action(&self) -> &str {
        match self {
            Error::NetworkTimeout { .. } => "Retry with longer timeout or check network connection",
            Error::AntiBot { .. } => "Try a different backend or wait before retrying",
            Error::Paywall { .. } => "Content is behind a paywall. Try finding the same information on a free source",
            Error::AiUnavailable { .. } => "Start Ollama with: ollama serve",
            _ => "Check the error details and try again",
        }
    }
}
```

#### Release Automation

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Build release binary
        run: cargo build --release --target ${{ matrix.target }}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: hsx-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/hsx*

  publish-npm:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
      - name: Publish to npm
        run: |
          cd npm
          npm version ${{ github.ref_name }}
          npm publish
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-crates:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Publish to crates.io
        run: cargo publish --workspace
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_TOKEN }}
```

#### npm Wrapper Package

```javascript
// npm/scripts/install-binary.js
const os = require('os');
const path = require('path');
const fs = require('fs');

const PLATFORM_MAP = {
  'darwin-x64': '@hypersearchx/darwin-x64',
  'darwin-arm64': '@hypersearchx/darwin-arm64',
  'linux-x64': '@hypersearchx/linux-x64',
  'linux-arm64': '@hypersearchx/linux-arm64',
  'win32-x64': '@hypersearchx/win-x64',
};

const key = `${os.platform()}-${os.arch()}`;
const pkg = PLATFORM_MAP[key];

if (!pkg) {
  console.error(`Unsupported platform: ${key}`);
  console.error('Supported: ' + Object.keys(PLATFORM_MAP).join(', '));
  process.exit(1);
}

try {
  const binaryPath = require.resolve(`${pkg}/bin/hsx`);
  const targetDir = path.join(__dirname, '..', 'bin');
  fs.mkdirSync(targetDir, { recursive: true });
  fs.copyFileSync(binaryPath, path.join(targetDir, 'hsx'));
  fs.chmodSync(path.join(targetDir, 'hsx'), 0o755);
} catch (e) {
  console.error(`Failed to install binary for ${key}: ${e.message}`);
  console.error('You can also install via: cargo install hypersearchx');
  process.exit(1);
}
```

#### Acceptance Criteria

- [ ] **Security**: `cargo audit` passes with zero known vulnerabilities
- [ ] **Security**: No credential storage in any persistent data
- [ ] **Security**: All HTTP output sanitized before display
- [ ] **Security**: TLS 1.2+ enforced for all connections
- [ ] **Security**: robots.txt respected by default
- [ ] **Security**: No CAPTCHA bypass or paywall circumvention code
- [ ] **Performance**: `hsx search` cached < 1s (benchmark verified)
- [ ] **Performance**: `hsx search` uncached < 3s (benchmark verified)
- [ ] **Performance**: `hsx agent-fetch` QATBE < 2s (benchmark verified)
- [ ] **Performance**: Token efficiency > 97% vs raw HTML (benchmark verified)
- [ ] **Error Handling**: Every error path produces a structured `Error` with `retryable()` and `suggested_action()`
- [ ] **Error Handling**: No panics in release builds (verified by fuzz testing)
- [ ] **Error Handling**: Graceful degradation: partial results > no results
- [ ] **Release**: Cross-compilation produces binaries for linux-x64, linux-arm64, darwin-x64, darwin-arm64, win-x64
- [ ] **Release**: `npm install -g hypersearchx` installs and works on all 5 platforms
- [ ] **Release**: `cargo install hypersearchx` works
- [ ] **Release**: GitHub Release includes pre-built binaries and checksums
- [ ] **Release**: `cargo publish` publishes all workspace crates to crates.io
- [ ] **Release**: Version numbers synchronized across Cargo.toml and package.json

#### Pitfalls

- **Cross-compilation**: `aarch64-unknown-linux-gnu` requires a cross-linker. Use `cross` tool or Docker-based cross-compilation.
- **npm optional dependencies**: The platform-specific packages pattern (used by esbuild, turbo, SWC) requires publishing 5 packages per release. Automate this completely.
- **Security audit false positives**: `cargo audit` may flag transitive dependencies. If a fix is not available upstream, document the risk assessment.
- **Windows path handling**: Windows uses `\` in paths. Test all path operations on Windows CI.
- **Binary size**: Rust release binaries can be large (50MB+). Use `strip`, LTO, and `opt-level = "z"` for size-optimized builds. Target < 20MB.
