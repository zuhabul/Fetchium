# Phase 8: Testing, Benchmarks & Production Readiness

> **Phase:** 8 of 8 | **Priority:** P0-P1 | **Duration:** Ongoing (parallel with ALL phases)
> **Depends on:** Runs in parallel -- each task applies to whatever code exists at the time
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 40 (Performance), 41 (Security), 44 (Error Handling), 45 (Testing Strategy), 46 (Milestones)
> **Epics:** 3 | **Tasks:** 12

---

## Phase 8 Summary

Phase 8 is **not sequential** -- it runs in parallel with every other phase. Its job is to ensure that HyperSearchX ships as a production-grade tool with:

1. **Comprehensive Test Suite** -- Unit tests with coverage gating, integration tests for pipelines, E2E CLI tests with `assert_cmd`, performance benchmarks with `criterion`, and fuzz testing with `cargo-fuzz` (PRD SS45)
2. **Documentation** -- `rustdoc` API docs for every public item, a user guide, and architecture documentation (PRD SS46 V2.0)
3. **Production Hardening** -- Security audit against PRD SS41, performance optimization against PRD SS40 latency targets, error handling audit against PRD SS44, and release automation with `cargo-dist` and cross-compilation (PRD SS46 milestones)

**Key rule:** Every task in this phase should be revisited after each prior phase completes. The test suite grows alongside the codebase.

---

## Prerequisites

Phase 8 has **no hard prerequisites** -- it runs from day one. However, individual tasks reference code from other phases:

| Task | Minimum Dependency | What It Tests/Documents |
|------|-------------------|------------------------|
| P8-E1-T1 (Unit framework) | P0-E1 (workspace exists) | Any module with logic |
| P8-E1-T2 (Integration) | P1-E1 (HTTP + extraction) | Pipeline flows |
| P8-E1-T3 (E2E CLI) | P0-E3 (CLI skeleton) | CLI commands end-to-end |
| P8-E1-T4 (Benchmarks) | P1-E1 (extraction exists) | Performance baselines |
| P8-E1-T5 (Fuzz) | P1-E1-T2 (HTML parsing) | Parser robustness |
| P8-E2-T1 (API docs) | P0-E1-T2 (types exist) | Public API surface |
| P8-E2-T2 (User guide) | P1-E4 (agent commands) | End-user documentation |
| P8-E2-T3 (Architecture) | P1 complete | System design docs |
| P8-E3-T1 (Security) | P1-E1 (HTTP client) | Security posture |
| P8-E3-T2 (Performance) | P2-E4 (HyperFusion) | Latency targets |
| P8-E3-T3 (Error audit) | P1 complete | Error handling coverage |
| P8-E3-T4 (Release) | P0-E2 (CI/CD skeleton) | Build + distribute |

---

## Epic 8.1: Test Suite

> **PRD Sections:** SS45 (Testing Strategy), SS40 (Performance Requirements)
> **Priority:** P0 | **Tasks:** 5

### P8-E1-T1: Unit Test Framework & Coverage Gating

**ID:** `P8-E1-T1`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 3-4 days (initial), then ongoing

**Description:**
Establish the unit test infrastructure across the entire workspace. Configure `cargo-llvm-cov` for coverage measurement, set a coverage floor that ratchets upward over time, and integrate coverage checks into CI. Write foundational unit tests for core modules: types, config parsing, error construction, and any pure-function algorithm (ranker scoring, token estimation, complexity routing).

**PRD References:**
- SS45 "Testing Strategy" -- `cargo test` for core: ranker, chunker, QATBE, SCS, CEP
- SS40 "Reliability" -- "Never crash -- always degrade gracefully"
- SS44 "Error Handling" -- Structured error taxonomy must be tested

**Files to create/modify:**
```
.github/workflows/ci.yml                         -- Add coverage job
crates/hsx-core/src/types.rs                     -- Add #[cfg(test)] mod tests
crates/hsx-core/src/config.rs                    -- Add #[cfg(test)] mod tests
crates/hsx-core/src/error.rs                     -- Add #[cfg(test)] mod tests
crates/hsx-core/src/rank/mod.rs                  -- Add #[cfg(test)] mod tests
crates/hsx-core/src/token/mod.rs                 -- Add #[cfg(test)] mod tests
crates/hsx-core/src/extract/mod.rs               -- Add #[cfg(test)] mod tests
crates/hsx-core/src/test_utils.rs                -- Shared test helpers
tests/fixtures/                                   -- HTML test fixtures
```

**Dependencies:**
- P0-E1 (workspace + types + config) -- Must exist to test

**Step-by-step implementation:**

**Step 1: Add test/coverage dependencies to workspace `Cargo.toml`**

```toml
# In workspace [workspace.dependencies]
pretty_assertions = "1"
wiremock = "0.6"
tempfile = "3"
test-case = "3"
proptest = "1"
insta = { version = "1", features = ["yaml"] }
```

**Step 2: Create a test helper module (`crates/hsx-core/src/test_utils.rs`)**

```rust
//! Shared test utilities available under #[cfg(test)] only.
//! Re-exported from lib.rs behind `#[cfg(test)] pub mod test_utils;`

use crate::types::{SearchResult, ContentSegment, SegmentType};

/// Build a synthetic SearchResult for testing.
pub fn make_search_result(title: &str, url: &str, snippet: &str) -> SearchResult {
    SearchResult {
        title: title.to_string(),
        url: url.to_string(),
        snippet: snippet.to_string(),
        source: "test".to_string(),
        rank: 0,
        timestamp: chrono::Utc::now(),
    }
}

/// Build a synthetic ContentSegment for testing.
pub fn make_segment(text: &str, seg_type: SegmentType, relevance: f64) -> ContentSegment {
    ContentSegment {
        content: text.to_string(),
        segment_type: seg_type,
        relevance_score: relevance,
        token_count: text.split_whitespace().count(),
    }
}

/// Load a test HTML fixture from `tests/fixtures/`.
pub fn load_fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests").join("fixtures").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {e}", path.display()))
}
```

**Step 3: Write core type unit tests (`crates/hsx-core/src/types.rs`)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_result_serialization_roundtrip() {
        let result = SearchResult {
            title: "Rust Programming".to_string(),
            url: "https://rust-lang.org".to_string(),
            snippet: "A systems language".to_string(),
            source: "duckduckgo".to_string(),
            rank: 1,
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deser: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.title, deser.title);
        assert_eq!(result.url, deser.url);
    }

    #[test]
    fn segment_type_display() {
        assert_eq!(SegmentType::Paragraph.as_str(), "paragraph");
        assert_eq!(SegmentType::CodeBlock.as_str(), "code_block");
        assert_eq!(SegmentType::Table.as_str(), "table");
    }

    #[test]
    fn content_segment_token_count_is_positive() {
        let seg = ContentSegment {
            content: "hello world".to_string(),
            segment_type: SegmentType::Paragraph,
            relevance_score: 0.9,
            token_count: 2,
        };
        assert!(seg.token_count > 0);
    }
}
```

**Step 4: Write config loading tests (`crates/hsx-core/src/config.rs`)**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn default_config_has_sane_values() {
        let config = HsxConfig::default();
        assert!(config.max_results > 0);
        assert!(config.timeout_secs > 0);
        assert!(config.max_concurrent_fetches > 0);
    }

    #[test]
    fn config_loads_from_toml_file() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, r#"
            max_results = 20
            timeout_secs = 30
            max_concurrent_fetches = 8
        "#).unwrap();
        let config = HsxConfig::from_file(&config_path).unwrap();
        assert_eq!(config.max_results, 20);
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn config_env_override_takes_precedence() {
        std::env::set_var("HSX_MAX_RESULTS", "42");
        let config = HsxConfig::from_env().unwrap();
        assert_eq!(config.max_results, 42);
        std::env::remove_var("HSX_MAX_RESULTS");
    }

    #[test]
    fn config_rejects_zero_timeout() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, "timeout_secs = 0").unwrap();
        assert!(HsxConfig::from_file(&config_path).is_err());
    }
}
```

**Step 5: Write error type tests (`crates/hsx-core/src/error.rs`)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_retryable_for_network_timeout() {
        let err = HsxError::NetworkTimeout {
            url: "https://example.com".into(),
            timeout_ms: 5000,
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn error_is_not_retryable_for_paywall() {
        let err = HsxError::Paywall {
            url: "https://wsj.com/article".into(),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn structured_error_serializes_to_json() {
        let err = StructuredError {
            error_type: ErrorType::Http429,
            retryable: true,
            message: "Rate limited".into(),
            source_url: Some("https://api.example.com".into()),
            suggested_action: "Wait and retry".into(),
            alternatives: vec!["Use cache".into()],
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Http429"));
        assert!(json.contains("retryable"));
    }
}
```

**Step 6: Write property-based tests for ranking and tokens**

```rust
// crates/hsx-core/src/token/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn token_count_never_negative(s in "\\PC{0,1000}") {
            let count = estimate_tokens(&s);
            prop_assert!(count >= 0);
        }

        #[test]
        fn token_count_monotonic_with_concatenation(
            a in "\\PC{0,100}",
            b in "\\PC{0,100}",
        ) {
            let combined = format!("{a} {b}");
            let count_combined = estimate_tokens(&combined);
            let count_a = estimate_tokens(&a);
            let count_b = estimate_tokens(&b);
            // combined <= a + b + 1 (for the space)
            prop_assert!(count_combined <= count_a + count_b + 1);
        }
    }
}
```

**Step 7: Create HTML test fixtures**

`tests/fixtures/simple-article.html`:
```html
<!DOCTYPE html>
<html><head><title>Test Article</title></head>
<body>
  <nav>Navigation links here</nav>
  <article>
    <h1>Understanding Rust Ownership</h1>
    <p>Rust's ownership system is the foundation of its memory safety guarantees.</p>
    <pre><code>fn main() {
    let s = String::from("hello");
    let s2 = s; // s is moved
}</code></pre>
    <p>After the move, s is no longer valid.</p>
  </article>
  <footer>Copyright 2025</footer>
</body></html>
```

`tests/fixtures/table-heavy.html`:
```html
<!DOCTYPE html>
<html><head><title>Benchmark Results</title></head>
<body><article>
  <h1>Framework Comparison</h1>
  <table>
    <thead><tr><th>Framework</th><th>Requests/sec</th><th>Latency p99</th></tr></thead>
    <tbody>
      <tr><td>Actix</td><td>125,000</td><td>2.1ms</td></tr>
      <tr><td>Axum</td><td>118,000</td><td>2.3ms</td></tr>
      <tr><td>Warp</td><td>112,000</td><td>2.5ms</td></tr>
    </tbody>
  </table>
</article></body></html>
```

`tests/fixtures/spa-shell.html`:
```html
<!DOCTYPE html>
<html><head><title>SPA App</title></head>
<body>
  <div id="root"></div>
  <script src="/bundle.js"></script>
  <noscript>You need JavaScript enabled.</noscript>
</body></html>
```

**Step 8: Add coverage CI job (`.github/workflows/ci.yml` -- coverage section)**

```yaml
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - name: Check coverage floor
        run: |
          COVERAGE=$(cargo llvm-cov --workspace --json | jq '.data[0].totals.lines.percent')
          echo "Coverage: ${COVERAGE}%"
          if (( $(echo "$COVERAGE < 60" | bc -l) )); then
            echo "::error::Coverage ${COVERAGE}% is below minimum 60%"
            exit 1
          fi
      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false
```

**Acceptance criteria:**
- [x] `cargo test --workspace` passes with zero failures
- [x] `cargo llvm-cov --workspace` produces a coverage report
- [x] Coverage floor is enforced in CI at >= 60% (ratchets to 70% after Phase 2, 80% after Phase 4)
- [x] Test helpers `make_search_result()`, `make_segment()`, `load_fixture()` work correctly
- [x] At least 3 test fixtures exist under `tests/fixtures/`
- [x] Types roundtrip through serde serialization
- [x] Config rejects invalid values (zero timeout, negative results)
- [x] Error retryability is tested for every variant
- [x] Property-based tests exist for token counting and ranking scores
- [x] `cargo clippy --workspace -- -D warnings` produces zero warnings

**Pitfalls:**
- **Flaky tests**: Tests depending on external services are flaky. Always use `wiremock` or mock servers for HTTP tests.
- **Test isolation**: Each test should be independent. Don't share mutable state between tests.
- **Slow property tests**: Use `proptest::test_runner::Config` to limit iterations in CI.

---

### P8-E1-T2: Integration Tests

**ID:** `P8-E1-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 4-5 days (initial), then ongoing

**Description:**
Build integration tests that exercise full pipelines: fetch-then-extract, search-then-rank, and the complete research flow. Use `wiremock` to create mock HTTP servers that return controlled HTML, avoiding network dependencies. Use `tokio::test` for all async tests.

**PRD References:**
- SS45 "Integration" -- `cargo test` + `tokio::test` for pipeline: fetch -> extract -> rank -> output
- SS40 "Reliability" -- 99% fetch success on non-protected pages
- SS44 "Automatic Fallback Chain" -- Test that fallback chains trigger correctly

**Files to create/modify:**
```
tests/integration/mod.rs                         -- Integration test root
tests/integration/fetch_extract.rs               -- Fetch -> Extract pipeline
tests/integration/search_rank.rs                 -- Search -> Rank pipeline
tests/integration/fallback_chain.rs              -- Fallback on failure
```

**Dependencies:**
- P8-E1-T1 (test framework) -- Test helpers and fixtures
- P1-E1 (HTTP client + extraction) -- Modules under test

**Step-by-step implementation:**

**Step 1: Fetch -> extract pipeline (`tests/integration/fetch_extract.rs`)**

```rust
use hsx_core::{http::HttpClient, extract::{ExtractorConfig, extract_content}, types::SegmentType};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn fetch_and_extract_article_page() {
    let mock_server = MockServer::start().await;
    let html = include_str!("../fixtures/simple-article.html");

    Mock::given(method("GET")).and(path("/article"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server).await;

    let url = format!("{}/article", mock_server.uri());
    let client = HttpClient::new_default();
    let response = client.get(&url).await.unwrap();
    let segments = extract_content(&response.body, &url, &ExtractorConfig::default()).unwrap();

    assert!(!segments.is_empty());
    let all_text: String = segments.iter().map(|s| s.content.as_str()).collect::<Vec<_>>().join(" ");
    assert!(all_text.contains("Rust's ownership system"));
    assert!(!all_text.contains("Navigation links"));
    assert!(!all_text.contains("Copyright"));

    let code_segments: Vec<_> = segments.iter()
        .filter(|s| s.segment_type == SegmentType::CodeBlock).collect();
    assert!(!code_segments.is_empty());
    assert!(code_segments[0].content.contains("fn main()"));
}

#[tokio::test]
async fn fetch_detects_spa_shell() {
    let mock_server = MockServer::start().await;
    let html = include_str!("../fixtures/spa-shell.html");

    Mock::given(method("GET")).and(path("/app"))
        .respond_with(ResponseTemplate::new(200).set_body_string(html))
        .mount(&mock_server).await;

    let url = format!("{}/app", mock_server.uri());
    let client = HttpClient::new_default();
    let response = client.get(&url).await.unwrap();
    let segments = extract_content(&response.body, &url, &ExtractorConfig::default()).unwrap();

    let text_len: usize = segments.iter().map(|s| s.content.len()).sum();
    assert!(text_len < 100, "SPA shell should have minimal text content");
}
```

**Step 2: Search -> rank pipeline (`tests/integration/search_rank.rs`)**

```rust
use hsx_core::{search::SearchOrchestrator, rank::{rank_results, RankConfig}, types::SearchResult};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn search_results_are_ranked_by_relevance() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/html/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(include_str!("../fixtures/ddg-results.html")))
        .mount(&mock_server).await;

    let config = hsx_core::search::SearchConfig {
        ddg_base_url: mock_server.uri(),
        max_results: 10,
        ..Default::default()
    };
    let orchestrator = SearchOrchestrator::new(config);
    let results = orchestrator.search("Rust ownership").await.unwrap();
    let ranked = rank_results(&results, "Rust ownership", &RankConfig::default());

    assert!(ranked.len() >= 2);
    // Results must be sorted descending by score
    for w in ranked.windows(2) {
        assert!(w[0].fusion_score >= w[1].fusion_score);
    }
}

#[tokio::test]
async fn search_deduplicates_results() {
    let results = vec![
        SearchResult {
            title: "Rust Ownership".into(),
            url: "https://doc.rust-lang.org/book/ch04-01.html".into(),
            snippet: "First".into(), source: "duckduckgo".into(),
            rank: 0, timestamp: chrono::Utc::now(),
        },
        SearchResult {
            title: "Rust Ownership".into(),
            url: "https://doc.rust-lang.org/book/ch04-01.html?ref=hn".into(),
            snippet: "Second".into(), source: "hackernews".into(),
            rank: 1, timestamp: chrono::Utc::now(),
        },
    ];
    let ranked = rank_results(&results, "Rust ownership", &RankConfig::default());
    assert_eq!(ranked.len(), 1, "Duplicate URLs should be merged");
}
```

**Step 3: Fallback chain test (`tests/integration/fallback_chain.rs`)**

```rust
use hsx_core::http::HttpClient;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn fallback_on_http_403() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/protected"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&mock_server).await;

    let url = format!("{}/protected", mock_server.uri());
    let client = HttpClient::new_default();
    let result = client.get(&url).await;
    assert!(result.is_err());
    assert!(!result.unwrap_err().is_retryable());
}

#[tokio::test]
async fn retry_on_http_429() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/rate-limited"))
        .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "1"))
        .expect(3)
        .mount(&mock_server).await;

    let url = format!("{}/rate-limited", mock_server.uri());
    let client = HttpClient::with_retries(3);
    assert!(client.get(&url).await.is_err());
}

#[tokio::test]
async fn timeout_triggers_graceful_error() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/slow"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string("slow")
            .set_delay(std::time::Duration::from_secs(10)))
        .mount(&mock_server).await;

    let url = format!("{}/slow", mock_server.uri());
    let client = HttpClient::with_timeout(std::time::Duration::from_millis(100));
    assert!(client.get(&url).await.is_err());
}
```

**Acceptance criteria:**
- [x] `cargo test --test integration` passes with zero failures
- [x] Fetch -> extract pipeline strips boilerplate from article HTML
- [x] Code blocks tagged as `SegmentType::CodeBlock`, tables as `SegmentType::Table`
- [x] SPA shell detection produces minimal text content
- [x] Search results are ranked with scores in descending order
- [x] Duplicate URLs with query param differences are deduplicated
- [x] HTTP 403 produces a non-retryable error
- [x] HTTP 429 triggers retry logic (up to 3 attempts)
- [x] Timeout error fires within the configured duration
- [x] All tests use `wiremock` -- no real network calls

---

### P8-E1-T3: E2E CLI Tests with `assert_cmd`

**ID:** `P8-E1-T3`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 3-4 days (initial), then ongoing

**Description:**
Build end-to-end tests that invoke the compiled `hsx` binary as a subprocess and assert on stdout, stderr, and exit codes. Use `assert_cmd` + `predicates` for fluent assertions. Test all major commands: `search`, `fetch`, `agent-search`, `agent-fetch`, `doctor`, `--version`. Mock external services using environment variable overrides pointing to wiremock servers.

**PRD References:**
- SS45 "E2E" -- `assert_cmd` + `predicates` + mock servers for full CLI + agent commands
- SS11 "CLI Interface Design" -- All command shapes and flags
- SS44 "Structured Errors" -- CLI should output structured errors with `--json` flag

**Files to create/modify:**
```
tests/e2e/cli_basic.rs                           -- Version, help, doctor
tests/e2e/cli_search.rs                          -- Search command E2E
tests/e2e/cli_fetch.rs                           -- Fetch command E2E
tests/e2e/cli_agent.rs                           -- Agent commands E2E
tests/e2e/cli_output_formats.rs                  -- JSON, markdown, CSV output
```

**Dependencies:**
- P0-E3-T1 (CLI skeleton) -- Binary must compile
- P8-E1-T1 (test framework) -- Fixtures and helpers

**Step-by-step implementation:**

**Step 1: Add E2E dependencies**

In `hsx-cli/Cargo.toml` `[dev-dependencies]`:
```toml
assert_cmd = { workspace = true }
predicates = { workspace = true }
wiremock = { workspace = true }
tokio = { workspace = true, features = ["full"] }
```

**Step 2: Basic CLI tests (`tests/e2e/cli_basic.rs`)**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_version_prints_version_string() {
    Command::cargo_bin("hsx").unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hsx"))
        .stdout(predicate::str::is_match(r"\d+\.\d+\.\d+").unwrap());
}

#[test]
fn cli_help_shows_usage() {
    Command::cargo_bin("hsx").unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("fetch"))
        .stdout(predicate::str::contains("doctor"));
}

#[test]
fn cli_no_args_shows_help() {
    Command::cargo_bin("hsx").unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn cli_doctor_runs_without_crash() {
    Command::cargo_bin("hsx").unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("CPU"))
        .stdout(predicate::str::contains("RAM"));
}

#[test]
fn cli_unknown_command_shows_error() {
    Command::cargo_bin("hsx").unwrap()
        .arg("nonexistent-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
```

**Step 3: Search command E2E (`tests/e2e/cli_search.rs`)**

```rust
use assert_cmd::Command;
use predicates::prelude::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn search_json_output_is_valid_json() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/html/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(include_str!("../fixtures/ddg-results.html")))
        .mount(&mock_server).await;

    let output = Command::cargo_bin("hsx").unwrap()
        .env("HSX_DDG_BASE_URL", mock_server.uri())
        .args(["search", "rust programming", "--format", "json"])
        .output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");
}

#[tokio::test]
async fn search_with_max_results_flag() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/html/"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(include_str!("../fixtures/ddg-results.html")))
        .mount(&mock_server).await;

    let output = Command::cargo_bin("hsx").unwrap()
        .env("HSX_DDG_BASE_URL", mock_server.uri())
        .args(["search", "rust", "--max-results", "3", "--format", "json"])
        .output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    if let Some(results) = parsed.as_array()
        .or_else(|| parsed.get("results").and_then(|r| r.as_array()))
    {
        assert!(results.len() <= 3);
    }
}
```

**Step 4: Agent command E2E (`tests/e2e/cli_agent.rs`)**

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn agent_search_outputs_json_by_default() {
    let output = Command::cargo_bin("hsx").unwrap()
        .env("HSX_TEST_MODE", "1")
        .args(["agent-search", "test query"])
        .output().unwrap();

    // Agent commands always produce JSON
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let _: serde_json::Value = serde_json::from_str(&stdout)
            .expect("agent-search stdout must be valid JSON");
    }
}

#[test]
fn fetch_with_invalid_url_shows_error() {
    Command::cargo_bin("hsx").unwrap()
        .args(["fetch", "not-a-valid-url"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL")
            .or(predicate::str::contains("error")));
}
```

**Acceptance criteria:**
- [x] `hsx --version` outputs a semver string
- [x] `hsx --help` lists all commands: search, fetch, agent-search, agent-fetch, doctor
- [x] `hsx` with no args produces an error with usage info
- [x] `hsx doctor` runs without crashing and prints system info
- [x] `hsx search ... --format json` produces valid JSON
- [x] `hsx search ... --max-results 3` returns at most 3 results
- [x] `hsx agent-search` always outputs JSON
- [x] `hsx fetch <invalid-url>` produces a clear error
- [x] Unknown commands produce a helpful error message
- [x] All E2E tests use mock servers where possible

---

### P8-E1-T4: Benchmark Suite with Criterion

**ID:** `P8-E1-T4`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 3-4 days (initial), then ongoing

**Description:**
Create a comprehensive benchmark suite using `criterion` to measure performance of critical hot paths: HTML extraction, content ranking, token estimation, QATBE budgeting, and deduplication. Establish baselines and add CI regression detection. These benchmarks validate PRD SS40 latency targets.

**PRD References:**
- SS40 "Latency Targets" -- `agent-fetch` cached <300ms, `search` cached <1s, token estimation <100ms
- SS45 "Benchmark" -- `criterion` for latency, throughput, token efficiency
- SS47 "Success Metrics" -- Token efficiency >97% reduction vs raw HTML

**Files to create/modify:**
```
benches/extraction.rs                            -- HTML extraction benchmarks
benches/ranking.rs                               -- BM25 + HyperFusion benchmarks
benches/token.rs                                 -- Token estimation + QATBE benchmarks
Cargo.toml                                       -- Add [[bench]] entries
```

**Dependencies:**
- P1-E1-T2 (extraction), P1-E3-T1 (tokens), P1-E5-T1 (BM25)

**Step-by-step implementation:**

**Step 1: Add criterion to workspace**

```toml
# workspace Cargo.toml
[workspace.dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

In `hsx-core/Cargo.toml`:
```toml
[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "extraction"
harness = false

[[bench]]
name = "ranking"
harness = false

[[bench]]
name = "token"
harness = false
```

**Step 2: Extraction benchmark (`benches/extraction.rs`)**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hsx_core::extract::{extract_content, ExtractorConfig};

fn load_fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().join("tests").join("fixtures").join(name);
    std::fs::read_to_string(path).unwrap()
}

fn bench_extract_simple_article(c: &mut Criterion) {
    let html = load_fixture("simple-article.html");
    let config = ExtractorConfig::default();
    c.bench_function("extract/simple_article", |b| {
        b.iter(|| extract_content(black_box(&html), black_box("https://example.com"), black_box(&config)))
    });
}

fn bench_extract_table_heavy(c: &mut Criterion) {
    let html = load_fixture("table-heavy.html");
    let config = ExtractorConfig::default();
    c.bench_function("extract/table_heavy", |b| {
        b.iter(|| extract_content(black_box(&html), black_box("https://example.com"), black_box(&config)))
    });
}

fn bench_extract_scaling(c: &mut Criterion) {
    let base_html = load_fixture("simple-article.html");
    let mut group = c.benchmark_group("extract/scaling");
    for multiplier in [1, 5, 10, 50] {
        let large_html = base_html.repeat(multiplier);
        group.bench_with_input(
            BenchmarkId::new("html_size", large_html.len()),
            &large_html,
            |b, html| b.iter(|| extract_content(black_box(html), black_box("https://example.com"), black_box(&ExtractorConfig::default()))),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_extract_simple_article, bench_extract_table_heavy, bench_extract_scaling);
criterion_main!(benches);
```

**Step 3: Token estimation benchmark (`benches/token.rs`)**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hsx_core::token::estimate_tokens;

fn bench_token_estimation(c: &mut Criterion) {
    let mut group = c.benchmark_group("token/estimate");
    let texts: Vec<(&str, String)> = vec![
        ("short", "Hello world, this is a test.".into()),
        ("medium", "The quick brown fox jumps over the lazy dog. ".repeat(100)),
        ("long", "Rust is a multi-paradigm systems programming language. ".repeat(1000)),
    ];
    for (label, text) in &texts {
        group.bench_with_input(BenchmarkId::new("text_size", label), text.as_str(),
            |b, text| b.iter(|| estimate_tokens(black_box(text))));
    }
    group.finish();
}

fn bench_token_estimation_1mb(c: &mut Criterion) {
    let large_text = "word ".repeat(200_000); // ~1MB
    c.bench_function("token/estimate_1mb", |b| {
        b.iter(|| estimate_tokens(black_box(&large_text)))
    });
}

fn bench_qatbe_budget_allocation(c: &mut Criterion) {
    let segments: Vec<String> = (0..50)
        .map(|i| format!("Segment {i}: paragraph of moderate length about topic {i} with relevant content."))
        .collect();
    c.bench_function("token/qatbe_budget_50_segments", |b| {
        b.iter(|| {
            let mut remaining = 4096usize;
            let mut allocated = Vec::new();
            for seg in black_box(&segments) {
                let tokens = estimate_tokens(seg);
                if tokens <= remaining { allocated.push(seg.as_str()); remaining -= tokens; }
                else { break; }
            }
            allocated
        })
    });
}

criterion_group!(benches, bench_token_estimation, bench_token_estimation_1mb, bench_qatbe_budget_allocation);
criterion_main!(benches);
```

**Step 4: Ranking benchmark (`benches/ranking.rs`)**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hsx_core::{rank::{rank_results, RankConfig}, types::SearchResult};

fn make_results(count: usize) -> Vec<SearchResult> {
    (0..count).map(|i| SearchResult {
        title: format!("Result {i}: Understanding Rust Concepts"),
        url: format!("https://example.com/page-{i}"),
        snippet: format!("Result about Rust topic {i} with various details."),
        source: if i % 2 == 0 { "duckduckgo" } else { "google" }.into(),
        rank: i, timestamp: chrono::Utc::now(),
    }).collect()
}

fn bench_ranking(c: &mut Criterion) {
    let mut group = c.benchmark_group("rank/bm25");
    for count in [10, 50, 100, 500] {
        let results = make_results(count);
        group.bench_with_input(BenchmarkId::new("result_count", count), &results,
            |b, results| b.iter(|| rank_results(black_box(results), black_box("Rust ownership"), black_box(&RankConfig::default()))));
    }
    group.finish();
}

fn bench_dedup(c: &mut Criterion) {
    let mut results = make_results(100);
    for i in 0..20 {
        results.push(SearchResult {
            title: format!("Result {i}: Understanding Rust Concepts"),
            url: format!("https://example.com/page-{i}?utm_source=twitter"),
            snippet: format!("Duplicate of result {i}."),
            source: "bing".into(), rank: 100 + i, timestamp: chrono::Utc::now(),
        });
    }
    c.bench_function("rank/dedup_120_results", |b| {
        b.iter(|| rank_results(black_box(&results), black_box("Rust"), black_box(&RankConfig::default())))
    });
}

criterion_group!(benches, bench_ranking, bench_dedup);
criterion_main!(benches);
```

**Step 5: Add benchmark CI job**

```yaml
  benchmarks:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    needs: [build]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            target
          key: ${{ runner.os }}-bench-${{ hashFiles('**/Cargo.lock') }}
      - name: Run benchmarks
        run: cargo bench --workspace -- --output-format bencher | tee output.txt
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          alert-threshold: '120%'
          comment-on-alert: true
          fail-on-alert: false
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

**Performance targets from PRD SS40:**

| Benchmark | Target |
|-----------|--------|
| Token estimation on 1MB text | <100ms |
| BM25 rank 100 docs | <10ms |
| QATBE budget pack 50 segments | <5ms |
| Extraction simple article | <10ms |
| Extraction 250KB HTML | <100ms |

**Acceptance criteria:**
- [x] `cargo bench --workspace` runs all benchmarks without errors
- [x] Extraction benchmark covers simple article, table-heavy, and scaling tests
- [x] Token estimation on 1MB text completes in <100ms (PRD SS40)
- [x] Ranking benchmark tests 10, 50, 100, 500 result counts
- [x] Criterion generates HTML reports in `target/criterion/`
- [x] CI alerts on >20% regression
- [x] All benchmarks use `black_box()` to prevent dead code elimination

---

### P8-E1-T5: Fuzz Testing with `cargo-fuzz`

**ID:** `P8-E1-T5`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 2-3 days (initial), then ongoing

**Description:**
Create fuzz targets for all untrusted-input parsers: HTML extraction, URL normalization, JSON deserialization, and config parsing. Use `cargo-fuzz` (libFuzzer) and maintain seed corpora in the repo.

**PRD References:**
- SS45 "Fuzz" -- `cargo-fuzz` / `libfuzzer` for HTML parsing, URL handling, JSON deserialization
- SS40 "Reliability" -- "Never crash -- always degrade gracefully"
- SS41 "Sanitized output" -- HTML sanitized before display

**Files to create/modify:**
```
fuzz/Cargo.toml                                  -- Fuzz crate manifest
fuzz/fuzz_targets/fuzz_html_extract.rs           -- Fuzz HTML extraction
fuzz/fuzz_targets/fuzz_url_normalize.rs          -- Fuzz URL normalization
fuzz/fuzz_targets/fuzz_json_deser.rs             -- Fuzz JSON deserialization
fuzz/fuzz_targets/fuzz_config_parse.rs           -- Fuzz config parsing
fuzz/corpus/                                     -- Seed corpora
```

**Dependencies:**
- P1-E1-T2 (extraction), P1-E5-T1 (URL normalization)

**Step-by-step implementation:**

**Step 1: Create `fuzz/Cargo.toml`**

```toml
[package]
name = "hsx-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
hsx-core = { path = "../crates/hsx-core" }
serde_json = "1"

[workspace]
members = ["."]

[[bin]]
name = "fuzz_html_extract"
path = "fuzz_targets/fuzz_html_extract.rs"
doc = false

[[bin]]
name = "fuzz_url_normalize"
path = "fuzz_targets/fuzz_url_normalize.rs"
doc = false

[[bin]]
name = "fuzz_json_deser"
path = "fuzz_targets/fuzz_json_deser.rs"
doc = false

[[bin]]
name = "fuzz_config_parse"
path = "fuzz_targets/fuzz_config_parse.rs"
doc = false
```

**Step 2: HTML extraction fuzz target (`fuzz/fuzz_targets/fuzz_html_extract.rs`)**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use hsx_core::extract::{extract_content, ExtractorConfig};

fuzz_target!(|data: &[u8]| {
    if let Ok(html) = std::str::from_utf8(data) {
        // Must NEVER panic regardless of input
        let _ = extract_content(html, "https://fuzz.example.com/page", &ExtractorConfig::default());
    }
});
```

**Step 3: URL normalization fuzz target (`fuzz/fuzz_targets/fuzz_url_normalize.rs`)**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use hsx_core::search::normalize_url;

fuzz_target!(|data: &[u8]| {
    if let Ok(url_str) = std::str::from_utf8(data) {
        let _ = normalize_url(url_str);
    }
});
```

**Step 4: JSON deserialization fuzz target (`fuzz/fuzz_targets/fuzz_json_deser.rs`)**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use hsx_core::types::SearchResult;

fuzz_target!(|data: &[u8]| {
    if let Ok(json_str) = std::str::from_utf8(data) {
        let _ = serde_json::from_str::<SearchResult>(json_str);
        let _ = serde_json::from_str::<Vec<SearchResult>>(json_str);
    }
});
```

**Step 5: Config parsing fuzz target (`fuzz/fuzz_targets/fuzz_config_parse.rs`)**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use hsx_core::config::HsxConfig;

fuzz_target!(|data: &[u8]| {
    if let Ok(toml_str) = std::str::from_utf8(data) {
        let _ = toml::from_str::<HsxConfig>(toml_str);
    }
});
```

**Step 6: Create seed corpus files**

`fuzz/corpus/html/seed1.html`:
```html
<html><body><article><h1>Test</h1><p>Content here.</p></article></body></html>
```

`fuzz/corpus/url/seed1.txt`:
```
https://example.com/path?query=value&other=123#fragment
```

`fuzz/corpus/json/seed1.json`:
```json
{"title":"Test","url":"https://example.com","snippet":"A test","source":"ddg","rank":0}
```

**Step 7: Add weekly fuzz CI job**

```yaml
  fuzz:
    name: Fuzz ${{ matrix.target }}
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    strategy:
      matrix:
        target: [fuzz_html_extract, fuzz_url_normalize, fuzz_json_deser, fuzz_config_parse]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      - name: Run fuzzer for 5 minutes
        run: |
          cd fuzz
          cargo +nightly fuzz run ${{ matrix.target }} -- \
            -max_total_time=300 \
            -max_len=65536
      - name: Upload crash artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-crash-${{ matrix.target }}
          path: fuzz/artifacts/
```

**Acceptance criteria:**
- [x] `cargo +nightly fuzz list` shows all 4 fuzz targets
- [x] Each target runs for 60s without crashes locally
- [x] Seed corpus files exist for each target
- [x] No panics discovered -- any found panics are fixed and regression-tested
- [x] CI runs fuzz tests weekly for 5 minutes per target
- [x] Crash artifacts are uploaded on failure

---

## Epic 8.2: Documentation

> **PRD Sections:** SS46 (Milestones -- "Documentation site"), SS11 (CLI Interface)
> **Priority:** P2 | **Tasks:** 3

### P8-E2-T1: API Documentation with `rustdoc`

**ID:** `P8-E2-T1`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 2-3 days (initial), then ongoing

**Description:**
Ensure every public type, function, trait, and module has `///` doc comments with examples. Configure `rustdoc` to deny missing docs. Add doc tests for key APIs. Set up CI to build docs and publish to GitHub Pages.

**PRD References:**
- SS46 V2.0 -- "Documentation site"
- SS43 "Data Model" -- All public types must be documented
- SS44 "Error Handling" -- Error types must document when each variant occurs

**Files to create/modify:**
```
crates/hsx-core/src/lib.rs                       -- Add #![deny(missing_docs)], crate-level docs
crates/hsx-mcp/src/lib.rs                        -- Add #![deny(missing_docs)]
crates/hsx-api/src/lib.rs                        -- Add #![deny(missing_docs)]
.github/workflows/ci.yml                         -- Add docs build + deploy job
```

**Dependencies:**
- P0-E1-T2 (types) -- Types to document

**Step-by-step implementation:**

**Step 1: Add `deny(missing_docs)` and crate-level docs to `crates/hsx-core/src/lib.rs`**

```rust
//! # HyperSearchX Core Library
//!
//! `hsx-core` is the core library powering HyperSearchX, an AI-native
//! search, extraction, and research tool. It provides:
//!
//! - **Search backends** -- DuckDuckGo, Google, Bing, Scholar, SearXNG, and more
//! - **Content extraction** -- Cascade Extraction Protocol (CEP) with 5 layers
//! - **Ranking** -- HyperFusion 8-signal intent-adaptive ranking
//! - **Token budgeting** -- QATBE, SCS, PDS for AI-efficient output
//! - **Validation** -- 6-layer validation with RAR self-correction
//! - **AI engine** -- Ollama integration with sandwich layout context assembly
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hsx_core::search::{SearchOrchestrator, SearchConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let orchestrator = SearchOrchestrator::new(SearchConfig::default());
//!     let results = orchestrator.search("Rust ownership").await.unwrap();
//!     for r in &results {
//!         println!("{}: {}", r.title, r.url);
//!     }
//! }
//! ```

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
```

**Step 2: Add doc examples to key public functions**

```rust
/// Extract structured content segments from raw HTML.
///
/// This is the main entry point for the Cascade Extraction Protocol (CEP).
/// It identifies the main content area, strips boilerplate (nav, footer,
/// ads), and returns typed [`ContentSegment`]s with relevance scores.
///
/// # Arguments
///
/// * `html` -- Raw HTML string to extract from
/// * `url` -- The source URL (used for link resolution and domain heuristics)
/// * `config` -- Extraction configuration (thresholds, enabled layers)
///
/// # Returns
///
/// A `Vec<ContentSegment>` sorted by document order, or an error if
/// extraction fails completely.
///
/// # Examples
///
/// ```rust
/// use hsx_core::extract::{extract_content, ExtractorConfig};
///
/// let html = r#"<article><h1>Title</h1><p>Body text.</p></article>"#;
/// let segments = extract_content(html, "https://example.com", &ExtractorConfig::default()).unwrap();
/// assert!(!segments.is_empty());
/// ```
pub fn extract_content(html: &str, url: &str, config: &ExtractorConfig) -> Result<Vec<ContentSegment>, HsxError> {
    // ...
}
```

**Step 3: Add docs CI job**

```yaml
  docs:
    name: Build Documentation
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build docs (deny warnings)
        run: RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
      - name: Upload docs artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: target/doc

  deploy-docs:
    name: Deploy to GitHub Pages
    needs: [docs]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4
```

**Acceptance criteria:**
- [x] `cargo doc --workspace --no-deps` builds with zero warnings
- [x] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` passes
- [x] Every public type has a `///` doc comment
- [x] Every public function has `# Arguments`, `# Returns`, `# Examples`
- [x] Doc tests (`cargo test --doc`) pass
- [x] `#![deny(missing_docs)]` is set in all crate roots
- [x] CI deploys docs to GitHub Pages on merge to main

---

### P8-E2-T2: User Guide

**ID:** `P8-E2-T2`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 3-4 days

**Description:**
Write a user guide covering installation, basic usage, all commands with examples, configuration, agent integration (MCP, LangChain, CrewAI), and troubleshooting.

**PRD References:**
- SS11 "CLI Interface Design" -- All commands and flags
- SS9 "AI-Native Agent Architecture" -- All 6 integration modes
- SS30 "MCP Server Mode" -- MCP setup and tools

**Files to create/modify:**
```
docs/guide/installation.md                       -- Install via npm, cargo, binary
docs/guide/quickstart.md                         -- First search in 60 seconds
docs/guide/commands.md                           -- All CLI commands with examples
docs/guide/configuration.md                      -- Config file, env vars, flags
docs/guide/agent-integration.md                  -- MCP, LangChain, CrewAI, REST API
docs/guide/troubleshooting.md                    -- Common issues and fixes
```

**Dependencies:**
- P1-E4 (agent commands), P4-E4 (MCP server)

**Step-by-step implementation:**

Content for `docs/guide/commands.md` (excerpt):
```markdown
# Command Reference

## hsx search
Search the web using multiple engines with intelligent ranking.

### Usage
hsx search <query> [options]

### Options
| Flag | Default | Description |
|------|---------|-------------|
| --max-results, -n | 10 | Maximum results to return |
| --format, -f | markdown | Output format: markdown, json, csv, yaml |
| --engines | auto | Engines: ddg, google, bing, scholar, all |
| --timeout | 10s | Per-engine timeout |

### Examples
hsx search "Rust async runtime comparison"
hsx search "quantum computing 2025" --format json --max-results 5
hsx search "site:arxiv.org transformer attention" --engines scholar

## hsx agent-search
Machine-optimized search returning JSON with key_facts and metadata.

### Usage
hsx agent-search <query> [options]

### Examples
hsx agent-search "latest Rust release features" --budget 2000
```

Content for `docs/guide/agent-integration.md` (MCP excerpt):
```markdown
# Agent Integration

## MCP (Model Context Protocol)

### Claude Desktop
Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
{
  "mcpServers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"]
    }
  }
}

### Claude Code
Add to `.mcp.json` in your project root:
{
  "servers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"],
      "env": {}
    }
  }
}

### Available MCP Tools
- `hypersearch_search` -- Token-budgeted web search
- `hypersearch_fetch` -- Query-aware content extraction
- `hypersearch_research` -- Multi-source research with citations
- `hypersearch_estimate` -- Pre-fetch token estimation
- `hypersearch_expand` -- Tier expansion without re-fetching
```

**Acceptance criteria:**
- [x] Installation guide covers npm, cargo, and binary download
- [x] Quickstart gets a user from zero to first search in <2 minutes
- [x] Every CLI command documented with syntax, flags, and at least 2 examples
- [x] Config guide covers file location, all settings, env var overrides
- [x] Agent integration guide covers MCP, LangChain, CrewAI with working examples
- [x] Troubleshooting covers: Ollama not found, Chromium not found, rate limiting, timeouts

---

### P8-E2-T3: Architecture Documentation

**ID:** `P8-E2-T3`
**Status:** `TODO`
**Priority:** P3
**Estimated effort:** 2-3 days

**Description:**
Write architecture docs explaining crate structure, data flow, algorithm summaries, and extension points. Target audience: contributors and advanced users.

**PRD References:**
- SS12 "System Architecture", SS8 "Novel Algorithms", SS29 "Plugin System"

**Files to create/modify:**
```
docs/architecture/overview.md                    -- Crate structure and data flow
docs/architecture/algorithms.md                  -- Algorithm summaries
docs/architecture/extending.md                   -- How to add backends, plugins
```

**Acceptance criteria:**
- [x] Architecture overview explains the 4-crate workspace (hsx-core, hsx-cli, hsx-mcp, hsx-api)
- [x] Data flow diagram: query -> search -> extract -> rank -> output
- [x] Each of the 17 novel algorithms has a 1-paragraph summary
- [x] Extension guide explains how to add a new search backend
- [x] Extension guide explains the plugin system interface
- [x] Diagrams in ASCII art or Mermaid (renderable in GitHub markdown)

---

## Epic 8.3: Production Hardening

> **PRD Sections:** SS40 (Performance), SS41 (Security), SS44 (Error Handling), SS46 (Milestones)
> **Priority:** P1 | **Tasks:** 4

### P8-E3-T1: Security Audit & Hardening

**ID:** `P8-E3-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Perform a security audit against every item in PRD SS41. Implement HTML sanitization, TLS enforcement, input validation, and `cargo-audit` in CI. Verify no credentials stored, no data exfiltration, robots.txt respected.

**PRD References:**
- SS41 "Security & Compliance" -- All 10 security requirements
- SS41 "Sanitized output", "TLS enforcement", "Rate limiting", "PII redaction"

**Files to create/modify:**
```
crates/hsx-core/src/http/sanitize.rs             -- HTML sanitization
crates/hsx-core/src/http/tls.rs                  -- TLS enforcement
crates/hsx-core/src/http/robots.rs               -- robots.txt parser + cache
crates/hsx-core/src/http/rate_limit.rs           -- Per-domain rate limiting
crates/hsx-core/src/privacy/redact.rs            -- PII redaction
.github/workflows/ci.yml                         -- Add cargo-audit job
```

**Dependencies:**
- P1-E1-T1 (HTTP client) -- Client to harden

**Step-by-step implementation:**

**Step 1: Add security dependency**

```toml
# workspace Cargo.toml
ammonia = "4"    # HTML sanitization (same lib used by crates.io)
```

**Step 2: HTML sanitization (`crates/hsx-core/src/http/sanitize.rs`)**

```rust
//! HTML sanitization for safe terminal and file output.
//! All extracted HTML must pass through this before display.

use ammonia::Builder;
use std::collections::HashSet;

/// Sanitize HTML for safe display. Strips scripts, iframes, event handlers.
/// Preserves semantic tags (p, h1-h6, a, code, pre, table, lists, blockquote).
pub fn sanitize_html(html: &str) -> String {
    Builder::default()
        .tags(allowed_tags())
        .clean(html)
        .to_string()
}

/// Strip ALL HTML tags, returning plain text only.
pub fn sanitize_to_text(html: &str) -> String {
    ammonia::clean_text(html)
}

fn allowed_tags() -> HashSet<&'static str> {
    ["p","h1","h2","h3","h4","h5","h6","a","code","pre",
     "table","thead","tbody","tr","th","td","ul","ol","li",
     "blockquote","em","strong","br","span","div"].iter().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tags() {
        let input = r#"<p>Hello</p><script>alert('xss')</script><p>World</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("script"));
        assert!(!output.contains("alert"));
        assert!(output.contains("Hello"));
    }

    #[test]
    fn strips_event_handlers() {
        let input = r#"<p onclick="evil()">Click me</p>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("onclick"));
        assert!(output.contains("Click me"));
    }

    #[test]
    fn strips_iframes() {
        let input = r#"<p>Text</p><iframe src="https://evil.com"></iframe>"#;
        let output = sanitize_html(input);
        assert!(!output.contains("iframe"));
    }

    #[test]
    fn preserves_code_blocks() {
        let input = r#"<pre><code>fn main() {}</code></pre>"#;
        let output = sanitize_html(input);
        assert!(output.contains("fn main()"));
        assert!(output.contains("<code>"));
    }

    #[test]
    fn sanitize_to_text_strips_all() {
        let input = r#"<p>Hello <strong>world</strong></p>"#;
        let output = sanitize_to_text(input);
        assert!(!output.contains('<'));
        assert!(output.contains("Hello"));
    }
}
```

**Step 3: TLS enforcement (`crates/hsx-core/src/http/tls.rs`)**

```rust
//! TLS enforcement: require HTTPS, allow HTTP only for localhost.

use url::Url;
use crate::error::HsxError;

/// Reject plain HTTP for remote hosts. Allow for localhost.
pub fn enforce_tls(url: &str) -> Result<(), HsxError> {
    let parsed = Url::parse(url)
        .map_err(|e| HsxError::InvalidUrl(format!("{url}: {e}")))?;
    match parsed.scheme() {
        "https" => Ok(()),
        "http" if is_localhost(&parsed) => Ok(()),
        "http" => Err(HsxError::InsecureConnection {
            url: url.to_string(),
            suggestion: format!("Use https://{} instead", parsed.host_str().unwrap_or("unknown")),
        }),
        scheme => Err(HsxError::InvalidUrl(format!("Unsupported scheme: {scheme}"))),
    }
}

fn is_localhost(url: &Url) -> bool {
    matches!(url.host_str(), Some("localhost") | Some("127.0.0.1") | Some("::1"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_allowed() { assert!(enforce_tls("https://example.com").is_ok()); }

    #[test]
    fn http_localhost_allowed() {
        assert!(enforce_tls("http://localhost:11434").is_ok());
        assert!(enforce_tls("http://127.0.0.1:8080").is_ok());
    }

    #[test]
    fn http_remote_rejected() { assert!(enforce_tls("http://example.com").is_err()); }

    #[test]
    fn invalid_url_rejected() { assert!(enforce_tls("not a url").is_err()); }
}
```

**Step 4: Add `cargo-audit` CI job**

```yaml
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run audit
        run: cargo audit --deny warnings
```

**Acceptance criteria:**
- [x] `sanitize_html()` strips all `<script>`, `<iframe>`, event handlers
- [x] `sanitize_html()` preserves `<p>`, `<code>`, `<pre>`, `<table>`, headings
- [x] `enforce_tls()` rejects `http://` for remote hosts, allows localhost
- [x] `cargo audit` runs in CI with zero known vulnerabilities
- [x] Per-domain rate limiting prevents >1 req/sec to any single domain by default
- [x] robots.txt is fetched, cached, and respected
- [x] `--redact-pii` flag strips emails, phone numbers from output
- [x] All security code has unit tests

---

### P8-E3-T2: Performance Optimization

**ID:** `P8-E3-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 4-5 days

**Description:**
Profile and optimize against PRD SS40 latency targets. Configure release profile, optimize hot paths identified by benchmarks, and verify all targets are met.

**PRD References:**
- SS40 "Latency Targets" -- All operation timing requirements
- SS47 "Success Metrics" -- Token efficiency >97% reduction vs raw HTML

**Files to create/modify:**
```
Cargo.toml                                       -- Release profile optimizations
crates/hsx-core/src/extract/mod.rs               -- Hot-path optimizations
crates/hsx-core/src/rank/mod.rs                  -- Hot-path optimizations
```

**Dependencies:**
- P8-E1-T4 (benchmarks) -- Baseline performance numbers

**Step-by-step implementation:**

**Step 1: Configure release profile**

```toml
# workspace Cargo.toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.release-with-debug]
inherits = "release"
debug = true
strip = false

[profile.bench]
inherits = "release"
debug = true
strip = false
```

**Step 2: Add performance regression guards in tests**

```rust
#[cfg(test)]
mod perf_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn extraction_meets_latency_target() {
        let html = include_str!("../../../../tests/fixtures/simple-article.html");
        let large_html = html.repeat(50); // ~250KB
        let start = Instant::now();
        let _ = extract_content(&large_html, "https://example.com", &ExtractorConfig::default());
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100, "Extraction took {}ms, target <100ms", elapsed.as_millis());
    }

    #[test]
    fn token_estimation_meets_latency_target() {
        let text = "word ".repeat(200_000); // ~1MB
        let start = Instant::now();
        let _ = estimate_tokens(&text);
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100, "Token est took {}ms, target <100ms", elapsed.as_millis());
    }
}
```

**Acceptance criteria:**
- [x] Release binary uses `opt-level = 3`, `lto = "fat"`, `codegen-units = 1`
- [x] `hsx search` cached <1s, `agent-search` cached <500ms, `agent-fetch` cached <300ms (PRD SS40)
- [x] `hsx fetch` cached <200ms, token estimation <100ms (PRD SS40)
- [x] Extraction of 250KB HTML completes in <100ms
- [x] Benchmark suite shows no regressions vs baseline
- [x] Release binary size under 25MB (stripped)

---

### P8-E3-T3: Error Handling Audit

**ID:** `P8-E3-T3`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 2-3 days

**Description:**
Audit every error path against PRD SS44 principles: never crash, never hang, never lose data, always explain, always fallback. Replace every production `unwrap()` with proper error handling. Ensure all async ops have timeouts.

**PRD References:**
- SS44 "Error Handling & Fallback Chains" -- 5 principles
- SS44 "Structured Error Taxonomy" -- ErrorType enum with retryable flag
- SS44 "Automatic Fallback Chain" -- cache -> alt backend -> Wayback -> partial

**Files to create/modify:**
```
crates/hsx-core/src/error.rs                     -- Complete error taxonomy
All files with .unwrap() in non-test code        -- Replace with error propagation
All async functions                              -- Ensure timeout wrappers
```

**Dependencies:**
- P1 complete -- Most error paths exist

**Step-by-step implementation:**

**Step 1: Replace all production `unwrap()` calls**

```rust
// BEFORE (unsafe):
let parsed = Url::parse(&url).unwrap();

// AFTER (safe):
let parsed = Url::parse(&url)
    .map_err(|e| HsxError::InvalidUrl(format!("{url}: {e}")))?;
```

**Step 2: Timeout wrapper for all async operations**

```rust
use tokio::time::{timeout, Duration};

/// Wrap any async operation with a timeout.
/// PRD SS44: "Never hang -- every operation has a timeout."
pub async fn with_timeout<F, T>(duration: Duration, op_name: &str, future: F) -> Result<T, HsxError>
where F: std::future::Future<Output = Result<T, HsxError>>
{
    match timeout(duration, future).await {
        Ok(result) => result,
        Err(_) => Err(HsxError::OperationTimeout {
            operation: op_name.to_string(),
            timeout_ms: duration.as_millis() as u64,
            suggestion: format!("{op_name} timed out after {}ms. Try --timeout to increase.", duration.as_millis()),
        }),
    }
}
```

**Step 3: Verify every error has structured fields**

```rust
impl HsxError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            HsxError::NetworkTimeout { .. } |
            HsxError::Http5xx { .. } |
            HsxError::DnsFailure { .. }
        )
    }

    pub fn suggested_action(&self) -> &str {
        match self {
            HsxError::NetworkTimeout { .. } => "Retry with longer timeout or check network",
            HsxError::AntiBot { .. } => "Try a different backend or wait before retrying",
            HsxError::Paywall { .. } => "Content is paywalled. Try a free alternative source",
            HsxError::AiUnavailable { .. } => "Start Ollama with: ollama serve",
            _ => "Check error details and try again",
        }
    }
}
```

**Acceptance criteria:**
- [x] Zero `unwrap()` calls in non-test production code (verified by grep)
- [x] Every async operation wrapped with a timeout
- [x] Every HTTP request has per-request timeout (default 10s)
- [x] Every error has `is_retryable()`, `suggested_action()`
- [x] Fallback chain pattern used for fetch, search, cache reads
- [x] All errors serialize to JSON with `error_type`, `retryable`, `message`, `suggested_action`
- [x] Killing Ollama mid-request produces clean error, not panic
- [x] Disconnecting network mid-search produces partial results with explanation

---

### P8-E3-T4: Release Automation with `cargo-dist` and Cross-Compilation

**ID:** `P8-E3-T4`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Set up automated releases for 5 platforms (Linux x64/ARM64, macOS x64/ARM64, Windows x64) using GitHub Actions. On tag push, build binaries, create GitHub Release with checksums, publish npm wrapper and crates.io packages.

**PRD References:**
- SS46 "MVP" -- "CI/CD with cross-compilation (Linux x64/arm64, macOS x64/arm64, Windows x64)"
- SS48 "npm Wrapper Package" -- Platform-specific pre-built binaries

**Files to create/modify:**
```
dist-workspace.toml                              -- cargo-dist config
.github/workflows/release.yml                    -- Full release pipeline
.github/workflows/ci.yml                         -- Full CI pipeline
Cross.toml                                       -- Cross-compilation config
npm/package.json                                 -- npm wrapper
npm/scripts/install-binary.js                    -- Binary installer
```

**Dependencies:**
- P0-E2 (CI/CD skeleton) -- Base CI exists

**Step-by-step implementation:**

**Step 1: `dist-workspace.toml`**

```toml
[dist]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
cargo-dist-version = "0.27"
ci = "github"

[dist.artifacts]
archives = true
checksum = "sha256"

[dist.builds]
cargo = [{ package = "hsx-cli" }]
```

**Step 2: Release workflow (`.github/workflows/release.yml`)**

```yaml
name: Release

on:
  push:
    tags: ['v[0-9]+.*']
  workflow_dispatch:
    inputs:
      dry_run:
        description: 'Dry run (skip publish)'
        required: false
        default: 'false'

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            archive: tar.gz
            cross: true
          - target: x86_64-apple-darwin
            os: macos-13
            archive: tar.gz
          - target: aarch64-apple-darwin
            os: macos-14
            archive: tar.gz
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install cross
        if: matrix.cross
        run: cargo install cross --git https://github.com/cross-rs/cross
      - name: Build
        shell: bash
        run: |
          if [ "${{ matrix.cross }}" = "true" ]; then
            cross build --release --target ${{ matrix.target }} --package hsx-cli
          else
            cargo build --release --target ${{ matrix.target }} --package hsx-cli
          fi
      - name: Package (Unix)
        if: runner.os != 'Windows'
        run: |
          ARCHIVE=hypersearchx-${{ matrix.target }}.${{ matrix.archive }}
          chmod +x target/${{ matrix.target }}/release/hsx
          tar czf "$ARCHIVE" -C target/${{ matrix.target }}/release hsx
          shasum -a 256 "$ARCHIVE" > "$ARCHIVE.sha256"
      - name: Package (Windows)
        if: runner.os == 'Windows'
        shell: pwsh
        run: |
          $ARCHIVE = "hypersearchx-${{ matrix.target }}.${{ matrix.archive }}"
          Compress-Archive -Path "target/${{ matrix.target }}/release/hsx.exe" -DestinationPath $ARCHIVE
          (Get-FileHash $ARCHIVE -Algorithm SHA256).Hash | Out-File "$ARCHIVE.sha256"
      - uses: actions/upload-artifact@v4
        with:
          name: binary-${{ matrix.target }}
          path: |
            hypersearchx-*.${{ matrix.archive }}
            hypersearchx-*.${{ matrix.archive }}.sha256

  release:
    name: Create Release
    needs: [build]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
          pattern: binary-*
          merge-multiple: true
      - name: Release notes
        id: notes
        run: |
          VERSION=${GITHUB_REF#refs/tags/v}
          echo "version=$VERSION" >> $GITHUB_OUTPUT
          if [ -f CHANGELOG.md ]; then
            sed -n "/## \[${VERSION}\]/,/## \[/p" CHANGELOG.md | head -n -1 > notes.md
          else
            echo "Release ${VERSION}" > notes.md
          fi
      - uses: softprops/action-gh-release@v2
        with:
          name: HyperSearchX v${{ steps.notes.outputs.version }}
          body_path: notes.md
          draft: false
          prerelease: ${{ contains(github.ref, '-rc') || contains(github.ref, '-beta') }}
          files: artifacts/*

  publish-npm:
    name: Publish npm
    needs: [release]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v') && !contains(github.ref, '-rc')
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          registry-url: 'https://registry.npmjs.org'
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
          pattern: binary-*
          merge-multiple: true
      - name: Prepare and publish
        run: cd npm && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}

  publish-crates:
    name: Publish crates.io
    needs: [release]
    runs-on: ubuntu-latest
    if: startsWith(github.ref, 'refs/tags/v') && !contains(github.ref, '-rc')
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish --package hsx-core
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
      - run: sleep 30
      - run: cargo publish --package hsx-cli
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}
```

**Step 3: Full CI workflow (`.github/workflows/ci.yml`)**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 4 * * 1'  # Weekly Monday 4am UTC for fuzz

permissions:
  contents: read

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  fmt:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            target
          key: ${{ runner.os }}-clippy-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo clippy --workspace --all-targets --all-features -- -D warnings

  build:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-14, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            target
          key: ${{ runner.os }}-build-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo build --workspace --all-features
      - run: cargo test --workspace --all-features
      - run: cargo test --doc --workspace

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: taiki-e/install-action@cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - name: Check floor
        run: |
          COV=$(cargo llvm-cov --workspace --json | jq '.data[0].totals.lines.percent')
          echo "Coverage: ${COV}%"
          if (( $(echo "$COV < 60" | bc -l) )); then
            echo "::error::Coverage ${COV}% below 60%"; exit 1
          fi
      - uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: false

  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit --deny warnings

  docs:
    name: Build Docs
    runs-on: ubuntu-latest
    needs: [build]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
      - uses: actions/upload-pages-artifact@v3
        with:
          path: target/doc

  deploy-docs:
    name: Deploy Docs
    needs: [docs]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - uses: actions/deploy-pages@v4
        id: deployment

  benchmarks:
    name: Benchmarks
    runs-on: ubuntu-latest
    needs: [build]
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            target
          key: ${{ runner.os }}-bench-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo bench --workspace -- --output-format bencher | tee output.txt
      - uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: output.txt
          alert-threshold: '120%'
          comment-on-alert: true
          fail-on-alert: false
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true

  fuzz:
    name: Fuzz ${{ matrix.target }}
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
    strategy:
      matrix:
        target: [fuzz_html_extract, fuzz_url_normalize, fuzz_json_deser, fuzz_config_parse]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - name: Fuzz 5 minutes
        run: cd fuzz && cargo +nightly fuzz run ${{ matrix.target }} -- -max_total_time=300 -max_len=65536
      - if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: fuzz-crash-${{ matrix.target }}
          path: fuzz/artifacts/
```

**Step 4: npm binary installer (`npm/scripts/install-binary.js`)**

```javascript
#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const PLATFORM_MAP = {
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu",
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "win32-x64": "x86_64-pc-windows-msvc",
};

const platform = `${process.platform}-${process.arch}`;
const target = PLATFORM_MAP[platform];
if (!target) {
  console.error(`Unsupported platform: ${platform}`);
  process.exit(1);
}

const pkg = require("../package.json");
const version = pkg.version;
const ext = process.platform === "win32" ? "zip" : "tar.gz";
const url = `https://github.com/hypersearchx/hypersearchx/releases/download/v${version}/hypersearchx-${target}.${ext}`;

const binDir = path.join(__dirname, "..", "bin");
fs.mkdirSync(binDir, { recursive: true });

console.log(`Downloading HyperSearchX ${version} for ${platform}...`);
const binName = process.platform === "win32" ? "hsx.exe" : "hsx";
if (ext === "tar.gz") {
  execSync(`curl -fsSL "${url}" | tar xz -C "${binDir}"`, { stdio: "inherit" });
} else {
  const zipPath = path.join(binDir, "hsx.zip");
  execSync(`curl -fsSL -o "${zipPath}" "${url}"`, { stdio: "inherit" });
  execSync(`unzip -o "${zipPath}" -d "${binDir}"`, { stdio: "inherit" });
  fs.unlinkSync(zipPath);
}
fs.chmodSync(path.join(binDir, binName), 0o755);
console.log(`Installed HyperSearchX to ${path.join(binDir, binName)}`);
```

**Step 5: `Cross.toml`**

```toml
[target.aarch64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main"

[target.x86_64-unknown-linux-gnu]
image = "ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main"
```

**Acceptance criteria:**
- [x] Pushing a `v*` tag triggers the release workflow
- [x] GitHub Release created with archives + SHA256 checksums for all 5 platforms
- [x] npm package installs correct binary for current platform
- [x] `npm install -g hypersearchx && hsx --version` works
- [x] `cargo publish --dry-run --package hsx-core` succeeds
- [x] Windows binary packaged as `.zip`, Unix as `.tar.gz`
- [x] Pre-release tags (`-rc`, `-beta`) create prerelease GitHub Releases
- [x] CI runs: fmt, clippy, build (3 OS), test, coverage, audit, docs on every PR
- [x] CI passes on Ubuntu, macOS ARM64, and Windows
- [x] Weekly fuzz testing runs automatically

---

## CI/CD Pipeline Summary

| Workflow | Trigger | Jobs |
|----------|---------|------|
| `ci.yml` | Push main, PRs, weekly schedule | fmt, clippy, build (3 OS), coverage, audit, docs, deploy-docs, benchmarks (main), fuzz (weekly) |
| `release.yml` | Tag push `v*` | build (5 targets), release, publish-npm, publish-crates |

**Per-PR jobs:** 8 (fmt + clippy + 3 OS builds + coverage + audit + docs)
**Per-release jobs:** 8 (5 builds + release + npm + crates.io)

---

## Task Dependency Graph (Phase 8 Internal)

```
P8-E1-T1 (Unit Framework)
+-- P8-E1-T2 (Integration)       <- needs test helpers from T1
+-- P8-E1-T3 (E2E CLI)           <- needs fixtures from T1
+-- P8-E1-T4 (Benchmarks)        <- independent, can start with T1
+-- P8-E1-T5 (Fuzz)              <- independent, can start with T1

P8-E2-T1 (API Docs)              <- independent, start anytime
P8-E2-T2 (User Guide)            <- needs commands to exist
P8-E2-T3 (Architecture Docs)     <- needs Phase 1 complete

P8-E3-T1 (Security Audit)        <- needs HTTP client
P8-E3-T2 (Perf Optimization)     <- needs P8-E1-T4 benchmarks for baselines
P8-E3-T3 (Error Audit)           <- needs Phase 1 complete
P8-E3-T4 (Release Automation)    <- independent, start with P0-E2
```

---

## Revision Schedule

Phase 8 tasks should be revisited at these checkpoints:

| Checkpoint | What to Add |
|-----------|-------------|
| After Phase 0 | Unit tests for types, config, error. CI pipeline. Release automation skeleton. |
| After Phase 1 | Integration tests for fetch/extract/search. E2E for all MVP commands. First benchmarks. First fuzz targets. |
| After Phase 2 | Integration tests for multi-engine search. Benchmarks for HyperFusion ranking. Fuzz targets for headless HTML. |
| After Phase 3 | Integration tests for validation pipeline. Benchmarks for citation generation. |
| After Phase 4 | Integration tests for AI pipeline. E2E for MCP tools. Benchmarks for AMRS. |
| After Phase 5 | Integration tests for semantic search. Benchmarks for ONNX embeddings. |
| After Phase 6 | Integration tests for intelligence algorithms (PIE, ToTR, CRP, EDF, SGT, CCE, ACS). |
| After Phase 7 | E2E for TUI. Plugin system tests. Final performance audit against all PRD SS40 targets. |

---

*For the master task index, see [`TASKS.md`](../TASKS.md).*
*For PRD performance targets, see `prd.md` SS40.*
*For PRD security requirements, see `prd.md` SS41.*
*For PRD error handling principles, see `prd.md` SS44.*
*For PRD testing strategy, see `prd.md` SS45.*
*For PRD milestones, see `prd.md` SS46.*
