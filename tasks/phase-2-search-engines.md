# Phase 2: Multi-Engine Search

> **Phase:** 2 of 8 | **Priority:** P1 | **Duration:** Weeks 5-8
> **Depends on:** Phase 1 (MVP Core -- Search & Fetch) fully complete
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 8.1, 8.3 (Layers 3-5), 8.10, 14, 21
> **Epics:** 5 | **Tasks:** 16

---

## Phase 2 Summary

Phase 2 upgrades HyperSearchX from single-backend DuckDuckGo search to **full multi-engine parallel search** with advanced ranking. After this phase, users can:

1. **Search 10+ backends simultaneously** -- Google, Bing, Scholar, SearXNG, Wikipedia, Brave, HN, ArXiv, GitHub, Reddit, StackOverflow
2. **Headless Chromium extraction** -- CEP layers 3-5 for JavaScript-heavy sites
3. **HyperFusion 8-signal ranking** -- BM25, semantic, temporal, authority, evidence, diversity, depth, consensus
4. **QADD** -- Query-Aware DOM Distillation for massive token savings on JS-rendered pages
5. **Cross-source deduplication** -- SimHash + URL dedup across all backends

---

## Prerequisites

All of the following must be `DONE` before starting any Phase 2 task:

| Dependency | Phase | What It Provides |
|-----------|-------|-----------------|
| P1-E1-T1 (HTTP client) | Phase 1 | Pooled reqwest client with retries |
| P1-E1-T2 (CEP layers 1-2) | Phase 1 | Static HTML extraction pipeline |
| P1-E2-T1 (DuckDuckGo backend) | Phase 1 | `SearchBackend` trait definition |
| P1-E3-T1 (BM25 ranking) | Phase 1 | Tantivy index, `RankingSignal` trait |
| P1-E4-T1 (SCS) | Phase 1 | Semantic segmentation pipeline |
| P1-E5-T1 (QATBE) | Phase 1 | Token-budgeted extraction |

---

## Epic 2.1: Headless Chromium Pool + Google + Bing + Scholar

> **PRD Sections:** SS8.3 (CEP layers 3-5), SS14 (Parallel Execution)
> **Crate:** `hsx-core` -- `src/browser/`, `src/search/`
> **Priority:** P0 | **Tasks:** 4

### P2-E1-T1: Chromiumoxide Pool Manager

**ID:** `P2-E1-T1`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 3-4 days
**Dependencies:** P1-E1-T1 (HTTP client), P0-E1-T3 (config)

**Description:**
Implement a managed pool of headless Chromium browser instances using `chromiumoxide`. The pool manages browser lifecycle, tab recycling, memory limits per resource tier, and graceful shutdown.

**PRD References:**
- SS8.3 "CEP" -- Layers 3-5 require headless browser
- SS14 "Parallel Execution" -- Resource-aware concurrency

**Files to create/modify:**
```
crates/hsx-core/src/browser/
  mod.rs              -- Module root with re-exports
  pool.rs             -- Browser pool manager
  tab.rs              -- Tab wrapper with auto-cleanup
```

**Step-by-step implementation:**

**Step 1: Add dependency to `hsx-core/Cargo.toml`**

```toml
chromiumoxide = { version = "0.7", features = ["tokio-runtime"] }
```

**Step 2: Tab wrapper (`browser/tab.rs`)**

```rust
use chromiumoxide::{Browser, Page};

/// A managed tab that returns its permit to the pool on drop.
pub struct ManagedTab {
    page: Page,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl ManagedTab {
    pub fn new(page: Page, permit: tokio::sync::OwnedSemaphorePermit) -> Self {
        Self { page, _permit: permit }
    }

    pub async fn navigate(&self, url: &str, timeout_ms: u64) -> anyhow::Result<()> { /* ... */ }
    pub async fn content(&self) -> anyhow::Result<String> { /* ... */ }
    pub async fn evaluate(&self, expression: &str) -> anyhow::Result<String> { /* ... */ }
    pub fn page(&self) -> &Page { &self.page }
}
```

**Step 3: Browser pool (`browser/pool.rs`)**

Resource tier enum controls pool size:

```rust
pub enum ResourceTier {
    Minimal,   // 1 browser, 2 tabs (~200MB)
    Standard,  // 1 browser, 4 tabs (~500MB)
    Performance, // 2 browsers, 8 tabs (~1GB)
}

pub struct BrowserPool {
    browsers: Mutex<Vec<Arc<Browser>>>,
    tab_semaphore: Arc<Semaphore>,
    tier: ResourceTier,
    config: Arc<HsxConfig>,
}

impl BrowserPool {
    pub fn new(config: &HsxConfig, tier: ResourceTier) -> Self { /* ... */ }
    pub async fn init(&self) -> anyhow::Result<()> { /* launch browsers */ }
    pub async fn acquire_tab(&self) -> anyhow::Result<ManagedTab> { /* semaphore-gated */ }
    pub async fn shutdown(&self) { /* drop all browsers */ }
}
```

Key implementation details:
- `init()` launches headless Chromium with `--no-sandbox --disable-gpu --disable-dev-shm-usage`
- `acquire_tab()` uses `Semaphore::acquire_owned()` so the permit is returned when `ManagedTab` drops
- Handler loop spawned in background to process browser events

**Acceptance criteria:**
- [ ] `BrowserPool::new()` creates pool with correct tier limits
- [ ] `BrowserPool::init()` launches headless Chromium instances
- [ ] `acquire_tab()` returns `ManagedTab` with semaphore-backed concurrency
- [ ] Tab concurrency enforced: beyond-limit requests block until release
- [ ] `ManagedTab::navigate()` respects timeout
- [ ] `BrowserPool::shutdown()` cleans up all resources
- [ ] All tests pass: `cargo test -p hsx-core browser`

**Testing instructions:**
```bash
cargo test -p hsx-core browser::pool::tests
# Integration: launch pool, acquire tab, navigate to httpbin.org, verify content
# Concurrency: acquire max tabs, verify next blocks, release one, verify unblock
```

---

### P2-E1-T2: Google Search Scraper

**ID:** `P2-E1-T2`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 2-3 days
**Dependencies:** P2-E1-T1 (browser pool), P1-E2-T1 (SearchBackend trait)

**Description:**
Google search backend using headless Chromium. Scrapes SERPs, parses titles/snippets/URLs/pagination. Implements `SearchBackend` trait. Includes anti-detection (random delays, viewport variation).

**Files to create/modify:**
```
crates/hsx-core/src/search/
  google.rs           -- Google SERP scraper
  mod.rs              -- Update with new exports
```

**Step-by-step implementation:**

**Step 1: Google scraper (`search/google.rs`)**

```rust
pub struct GoogleBackend {
    pool: Arc<BrowserPool>,
}

impl GoogleBackend {
    pub fn new(pool: Arc<BrowserPool>) -> Self { Self { pool } }

    fn build_url(query: &str, page: usize) -> String {
        let start = page * 10;
        format!("https://www.google.com/search?q={}&start={start}&hl=en&num=10",
                urlencoding::encode(query))
    }

    fn parse_serp(html: &str, page: usize) -> Vec<SearchResult> {
        let document = Html::parse_document(html);
        // Selectors: div.g for results, h3 for title, a[href] for link, div.VwiC3b for snippet
        // Filter: only keep results with valid http(s) URLs
        // Rank: page * 10 + position
        /* ... */
    }
}

#[async_trait]
impl SearchBackend for GoogleBackend {
    fn name(&self) -> &str { "google" }

    async fn search(&self, query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        // Paginate up to 3 pages (30 results max)
        // Random anti-detection delay (200-800ms) between pages
        // Acquire tab per page, navigate, parse SERP
        /* ... */
    }
}
```

CSS selectors for Google SERP: `div.g` (result container), `h3` (title), `a[href]` (link), `div.VwiC3b` / `span.aCOpRe` (snippet).

**Acceptance criteria:**
- [ ] `GoogleBackend` implements `SearchBackend` trait
- [ ] `parse_serp()` extracts title, URL, snippet from Google SERP HTML
- [ ] Pagination up to 3 pages (30 results max)
- [ ] Anti-detection delay between page loads
- [ ] Graceful handling of CAPTCHA/blocks
- [ ] All tests pass: `cargo test -p hsx-core search::google`

**Testing instructions:**
```bash
cargo test -p hsx-core search::google::tests
# Unit: parse_serp with saved SERP HTML fixture
# Integration: search for known query, verify results have title+url+snippet
```

---

### P2-E1-T3: Bing Search Scraper

**ID:** `P2-E1-T3`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P2-E1-T1 (browser pool), P1-E2-T1 (SearchBackend trait)

**Description:**
Bing search backend using headless Chromium. Same pattern as Google with Bing-specific selectors. Bing is less aggressive with bot detection, making it a reliable fallback.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  bing.rs             -- Bing SERP scraper
```

**Step-by-step implementation:**

```rust
pub struct BingBackend { pool: Arc<BrowserPool> }

impl BingBackend {
    fn build_url(query: &str, page: usize) -> String {
        format!("https://www.bing.com/search?q={}&first={}",
                urlencoding::encode(query), page * 10 + 1)
    }

    fn parse_serp(html: &str, page: usize) -> Vec<SearchResult> {
        // Selectors: li.b_algo (result), h2 a (title+link), p/.b_caption p (snippet)
        /* ... */
    }
}
```

**Acceptance criteria:**
- [ ] `BingBackend` implements `SearchBackend` trait
- [ ] Parses Bing SERP correctly (title, URL, snippet)
- [ ] Pagination up to 3 pages
- [ ] All tests pass: `cargo test -p hsx-core search::bing`

**Testing instructions:**
```bash
cargo test -p hsx-core search::bing::tests
# Unit: parse saved Bing SERP HTML fixture
```

---

### P2-E1-T4: Google Scholar Scraper

**ID:** `P2-E1-T4`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 2 days
**Dependencies:** P2-E1-T1 (browser pool), P1-E2-T1 (SearchBackend trait)

**Description:**
Google Scholar backend with extra metadata: authors, publication year, citation count, venue. Citation counts feed into the authority signal of HyperFusion.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  scholar.rs          -- Google Scholar scraper
```

**Step-by-step implementation:**

```rust
pub struct ScholarBackend { pool: Arc<BrowserPool> }

#[derive(Debug, Clone, Default)]
pub struct ScholarMetadata {
    pub authors: Vec<String>,
    pub year: Option<u16>,
    pub citation_count: Option<u32>,
    pub venue: Option<String>,
    pub pdf_url: Option<String>,
}

impl ScholarBackend {
    fn build_url(query: &str, page: usize) -> String {
        format!("https://scholar.google.com/scholar?q={}&start={}&hl=en",
                urlencoding::encode(query), page * 10)
    }

    fn parse_serp(html: &str, page: usize) -> Vec<SearchResult> {
        // Selectors: div.gs_r.gs_or.gs_scl (result), h3.gs_rt a (title+link),
        // div.gs_rs (snippet), div.gs_a (author/year/venue info line),
        // div.gs_fl a (citation count from "Cited by N")
        /* ... */
    }
}
```

Key: parse `div.gs_a` text line "A Smith, B Jones - Nature, 2024" for metadata. Extract citation count from "Cited by N" link text in `div.gs_fl`.

**Acceptance criteria:**
- [ ] `ScholarBackend` implements `SearchBackend` trait
- [ ] Parses title, URL, snippet, citation count
- [ ] Citation count extracted from "Cited by N" links
- [ ] Pagination up to 2 pages (20 results)
- [ ] All tests pass: `cargo test -p hsx-core search::scholar`

**Testing instructions:**
```bash
cargo test -p hsx-core search::scholar::tests
# Unit: parse saved Scholar SERP HTML fixture, verify citation_count
```

---

## Epic 2.2: HTTP-Based Search Backends

> **PRD Sections:** SS14, SS21 -- Multi-backend search without browser overhead
> **Crate:** `hsx-core` -- `src/search/`
> **Priority:** P1 | **Tasks:** 3

### P2-E2-T1: SearXNG Backend

**ID:** `P2-E2-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P1-E1-T1 (HTTP client), P1-E2-T1 (SearchBackend trait)

**Description:**
SearXNG meta-search backend using its JSON API. Free, self-hostable, aggregates 70+ sources. Configurable instance URL with public instance defaults.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  searxng.rs          -- SearXNG API backend
```

**Step-by-step implementation:**

```rust
const DEFAULT_INSTANCES: &[&str] = &["https://searx.be", "https://search.ononoki.org"];

#[derive(Deserialize)]
struct SearxResponse { results: Vec<SearxResult> }
#[derive(Deserialize)]
struct SearxResult {
    url: String, title: String, content: Option<String>,
    engine: Option<String>, score: Option<f64>,
    #[serde(rename = "publishedDate")] published_date: Option<String>,
}

pub struct SearxngBackend { http: Arc<HttpClient>, instance_url: String }

#[async_trait]
impl SearchBackend for SearxngBackend {
    fn name(&self) -> &str { "searxng" }
    async fn search(&self, query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        // GET {instance}/search?q={query}&format=json&pageno=1
        // Parse JSON, map to SearchResult with source="searxng/{engine}"
        /* ... */
    }
}
```

**Acceptance criteria:**
- [ ] Uses JSON API (no browser required)
- [ ] Configurable instance URL
- [ ] Parses title, URL, snippet, engine, published date
- [ ] Graceful fallback if primary instance is down
- [ ] All tests pass: `cargo test -p hsx-core search::searxng`

**Testing instructions:**
```bash
cargo test -p hsx-core search::searxng::tests
# Unit: parse saved SearXNG JSON response fixture
```

---

### P2-E2-T2: Wikipedia API Backend

**ID:** `P2-E2-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1 day
**Dependencies:** P1-E1-T1 (HTTP client), P1-E2-T1 (SearchBackend trait)

**Description:**
Wikipedia search via MediaWiki API. High authority signal for factual queries. Returns article summaries and metadata.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  wikipedia.rs        -- Wikipedia API backend
```

**Step-by-step implementation:**

```rust
const WIKI_API: &str = "https://en.wikipedia.org/w/api.php";

pub struct WikipediaBackend { http: Arc<HttpClient> }

#[async_trait]
impl SearchBackend for WikipediaBackend {
    fn name(&self) -> &str { "wikipedia" }
    async fn search(&self, query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        // GET {WIKI_API}?action=query&list=search&srsearch={q}
        //   &srlimit={n}&srprop=snippet|wordcount|timestamp&format=json&origin=*
        // Parse JSON, strip HTML from snippets (<span class="searchmatch"> tags)
        // Build URL: https://en.wikipedia.org/wiki/{title.replace(' ', '_')}
        /* ... */
    }
}
```

**Acceptance criteria:**
- [ ] Uses MediaWiki JSON API (no browser)
- [ ] Clean snippets with HTML tags stripped
- [ ] Correct Wikipedia article URLs from titles
- [ ] Respects `max_results` (capped at 20)
- [ ] All tests pass: `cargo test -p hsx-core search::wikipedia`

**Testing instructions:**
```bash
cargo test -p hsx-core search::wikipedia::tests
# Unit: parse saved Wikipedia API response, verify HTML stripped from snippets
```

---

### P2-E2-T3: Additional HTTP Backends (Brave, HN, ArXiv, GitHub, Reddit, SO)

**ID:** `P2-E2-T3`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 3-4 days
**Dependencies:** P1-E1-T1 (HTTP client), P1-E2-T1 (SearchBackend trait)

**Description:**
Six additional HTTP-based search backends, each implementing `SearchBackend`.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  brave.rs            -- Brave Search API (free tier: 2000 req/month)
  hackernews.rs       -- HackerNews Algolia API (hn.algolia.com/api/v1/search)
  arxiv.rs            -- ArXiv API (Atom XML, parse with quick-xml)
  github.rs           -- GitHub search REST API (/search/repositories, /search/code)
  reddit.rs           -- Reddit JSON API (append .json to search URL)
  stackoverflow.rs    -- StackExchange API v2.3
  mod.rs              -- Update with all new exports
```

**API details per backend:**

| Backend | API URL | Format | Notes |
|---------|---------|--------|-------|
| Brave | `api.search.brave.com/res/v1/web/search` | JSON | Free tier, optional API key |
| HackerNews | `hn.algolia.com/api/v1/search` | JSON | No auth, `hitsPerPage` param |
| ArXiv | `export.arxiv.org/api/query` | Atom XML | `search_query` param, parse `<entry>` |
| GitHub | `api.github.com/search/repositories` | JSON | Unauthenticated: 10 req/min |
| Reddit | `www.reddit.com/search.json` | JSON | `q` param, parse `data.children` |
| StackOverflow | `api.stackexchange.com/2.3/search` | JSON | `intitle` param, gzip response |

Each backend follows the same pattern:

```rust
pub struct XxxBackend { http: Arc<HttpClient> }

#[async_trait]
impl SearchBackend for XxxBackend {
    fn name(&self) -> &str { "xxx" }
    async fn search(&self, query: &SearchQuery) -> anyhow::Result<Vec<SearchResult>> {
        // 1. Build API URL with query params
        // 2. HTTP GET via self.http.fetch_text()
        // 3. Parse response (JSON or XML)
        // 4. Map to Vec<SearchResult>
    }
}
```

**Step: Update module exports (`search/mod.rs`)**

```rust
pub mod google; pub mod bing; pub mod scholar;
pub mod searxng; pub mod wikipedia;
pub mod brave; pub mod hackernews; pub mod arxiv;
pub mod github; pub mod reddit; pub mod stackoverflow;
```

**Acceptance criteria:**
- [ ] All 6 backends implement `SearchBackend` trait
- [ ] Each parses its API's response format correctly
- [ ] ArXiv handles Atom XML parsing
- [ ] Reddit appends `.json` and parses listing format
- [ ] GitHub uses unauthenticated search (respects rate limit)
- [ ] StackOverflow handles gzip-compressed responses
- [ ] All tests pass: `cargo test -p hsx-core search`

**Testing instructions:**
```bash
cargo test -p hsx-core search::brave::tests
cargo test -p hsx-core search::hackernews::tests
# ... etc. Each with saved API response fixtures
```

---

## Epic 2.3: Full Parallel Search Orchestrator

> **PRD Sections:** SS14 (Parallel Execution Engine), SS8.1 (HyperFusion)
> **Crate:** `hsx-core` -- `src/search/orchestrator.rs`
> **Priority:** P0 | **Tasks:** 3

### P2-E3-T1: Parallel Execution Across All Backends

**ID:** `P2-E3-T1`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 2-3 days
**Dependencies:** P2-E1-T2, P2-E1-T3, P2-E1-T4, P2-E2-T1, P2-E2-T2, P2-E2-T3

**Description:**
Parallel search orchestrator dispatching to all backends via `tokio::JoinSet`. Per-backend timeout. Failed backends logged but don't fail overall search.

**Files to create/modify:**
```
crates/hsx-core/src/search/
  orchestrator.rs     -- Parallel search orchestrator
```

**Step-by-step implementation:**

```rust
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    pub backend_timeout_secs: u64,   // Default: 15
    pub max_total_results: usize,    // Default: 100
    pub enabled_backends: Vec<String>, // Empty = all
}

#[derive(Debug)]
pub struct BackendResult {
    pub backend_name: String,
    pub results: Vec<SearchResult>,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub struct SearchOrchestrator {
    backends: Vec<Arc<dyn SearchBackend + Send + Sync>>,
    config: OrchestratorConfig,
}

impl SearchOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self { /* ... */ }
    pub fn add_backend(&mut self, backend: Arc<dyn SearchBackend + Send + Sync>) { /* ... */ }

    pub async fn search(&self, query: &SearchQuery) -> Vec<BackendResult> {
        let mut join_set = JoinSet::new();
        for backend in &self.backends {
            // Skip if not in enabled_backends (when list is non-empty)
            let backend = Arc::clone(backend);
            let query = query.clone();
            let timeout = Duration::from_secs(self.config.backend_timeout_secs);
            join_set.spawn(async move {
                let start = Instant::now();
                match tokio::time::timeout(timeout, backend.search(&query)).await {
                    Ok(Ok(results)) => BackendResult { /* success */ },
                    Ok(Err(e)) => BackendResult { /* error */ },
                    Err(_) => BackendResult { /* timeout */ },
                }
            });
        }
        // Collect all results as they complete
        let mut all = Vec::new();
        while let Some(Ok(result)) = join_set.join_next().await {
            all.push(result);
        }
        all
    }
}
```

**Acceptance criteria:**
- [ ] Dispatches to all registered backends in parallel
- [ ] Per-backend timeout prevents slow backends from blocking
- [ ] Failed backends produce `BackendResult` with error, not panic
- [ ] `enabled_backends` filter works
- [ ] Results include timing metadata
- [ ] All tests pass: `cargo test -p hsx-core search::orchestrator`

**Testing instructions:**
```bash
cargo test -p hsx-core search::orchestrator::tests
# Mock backends (fast, slow, failing), verify parallel execution
# Verify timeout behavior and partial results on failure
```

---

### P2-E3-T2: Cross-Source Deduplication

**ID:** `P2-E3-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 2 days
**Dependencies:** P2-E3-T1 (orchestrator)

**Description:**
Two-strategy deduplication: (1) URL dedup with normalization (strip tracking params), (2) SimHash content similarity for near-duplicates. Merge metadata on match.

**PRD References:**
- SS8.1 "HyperFusion" -- `duplicate_penalty * SimHash(result.content, seen_content)`
- Gap 8 -- Cross-source deduplication and merging

**Files to create/modify:**
```
crates/hsx-core/src/search/
  dedup.rs            -- Deduplication engine
```

**Step-by-step implementation:**

```rust
/// SimHash fingerprint for content similarity.
pub struct SimHash(pub u64);

impl SimHash {
    /// Compute from text using word-level shingles + FNV-1a hashing.
    pub fn compute(text: &str) -> Self {
        // For each word: hash with FNV-1a, increment/decrement 64-bit vector
        // Final: positive bits = 1, negative bits = 0
        /* ... */
    }

    pub fn distance(&self, other: &SimHash) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    pub fn is_similar(&self, other: &SimHash, threshold: u32) -> bool {
        self.distance(other) <= threshold
    }
}

/// Normalize URL: remove utm_*, fbclid, gclid, ref; strip fragment and trailing slash.
pub fn normalize_url(url: &str) -> String { /* ... */ }

/// Deduplicate results: URL-exact match first, then SimHash content similarity.
pub fn deduplicate(results: Vec<SearchResult>, simhash_threshold: u32) -> Vec<SearchResult> {
    // 1. For each result: normalize URL, check seen_urls map
    // 2. If URL match: merge (increment seen_in_sources count)
    // 3. If no URL match: compute SimHash, check against seen_hashes
    // 4. If SimHash similar (distance <= threshold): skip
    // 5. Otherwise: add to output
    /* ... */
}
```

**Acceptance criteria:**
- [ ] `normalize_url()` strips tracking params, fragment, trailing slash
- [ ] `SimHash::compute()` produces consistent fingerprints
- [ ] `SimHash::distance()` measures Hamming distance correctly
- [ ] `deduplicate()` merges URL-identical results from different backends
- [ ] `deduplicate()` detects near-duplicate content via SimHash
- [ ] All tests pass: `cargo test -p hsx-core search::dedup`

**Testing instructions:**
```bash
cargo test -p hsx-core search::dedup::tests
# Test normalize_url with utm_source, fbclid, trailing slash, fragment
# Test SimHash: identical text -> 0 distance, different text -> high distance
# Test deduplicate: feed duplicate URLs from different backends, verify merge
```

---

### P2-E3-T3: Fallback Chains and Error Recovery

**ID:** `P2-E3-T3`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 1-2 days
**Dependencies:** P2-E3-T1 (orchestrator)

**Description:**
Fallback chains: when primary backends fail, secondary backends are tried. If a backend returns suspiciously few results, fall back to next in chain.

**PRD References:**
- SS44 "Error Handling" -- Fallback: cache -> alternative source -> Wayback -> partial

**Files to create/modify:**
```
crates/hsx-core/src/search/
  fallback.rs         -- Fallback chain logic
```

**Step-by-step implementation:**

```rust
pub struct FallbackChain {
    /// Ordered: (backend, min_results_threshold).
    chains: Vec<(Arc<dyn SearchBackend + Send + Sync>, usize)>,
}

impl FallbackChain {
    pub fn new() -> Self { /* ... */ }
    pub fn add(&mut self, backend: Arc<dyn SearchBackend + Send + Sync>, min_results: usize) { /* ... */ }

    /// Try each backend until one meets the min_results threshold.
    pub async fn execute(&self, query: &SearchQuery) -> BackendResult {
        for (backend, min_results) in &self.chains {
            match backend.search(query).await {
                Ok(results) if results.len() >= *min_results => return /* success */,
                Ok(_) => continue,  // Too few results
                Err(_) => continue, // Error
            }
        }
        BackendResult { error: Some("All fallback backends exhausted".into()), .. }
    }
}

/// Predefined web search chain: Google -> Bing -> SearXNG -> Brave -> DDG.
pub fn web_search_chain(backends: &HashMap<String, Arc<dyn SearchBackend + Send + Sync>>) -> FallbackChain {
    /* ... */
}
```

**Acceptance criteria:**
- [ ] `FallbackChain::execute()` tries backends in priority order
- [ ] Backends with fewer than `min_results` trigger fallback
- [ ] Backend errors trigger fallback
- [ ] Returns error when all backends exhausted
- [ ] `web_search_chain()` builds correct priority order
- [ ] All tests pass: `cargo test -p hsx-core search::fallback`

**Testing instructions:**
```bash
cargo test -p hsx-core search::fallback::tests
# Mock: first fails, second succeeds -> second's results returned
# Mock: all fail -> exhausted error
```

---

## Epic 2.4: HyperFusion 8-Signal Ranking

> **PRD Sections:** SS8.1 (HyperFusion Ranking Algorithm), SS21 (Ranking)
> **Crate:** `hsx-core` -- `src/rank/`
> **Priority:** P0 | **Tasks:** 3

### P2-E4-T1: Eight Ranking Signals Implementation

**ID:** `P2-E4-T1`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 3-4 days
**Dependencies:** P1-E3-T1 (BM25 ranking via tantivy)

**Description:**
Implement all 8 HyperFusion signals. Phase 1 provides BM25. This adds the remaining 7.

**PRD References:**
- SS8.1 "HyperFusion" -- Full formula with 8 signals + intent categories

**Files to create/modify:**
```
crates/hsx-core/src/rank/
  signals.rs          -- Individual signal implementations
```

**Step-by-step implementation:**

Scoring context tracks state across results:

```rust
pub struct ScoringContext {
    pub query: String,
    pub query_terms: Vec<String>,
    pub seen_domains: HashSet<String>,
    pub all_claims: Vec<String>,
}
```

**Signal implementations** (each returns `f64` in `[0.0, 1.0]`):

| # | Signal | Function | Logic |
|---|--------|----------|-------|
| 1 | BM25 | `bm25_score(result, ctx)` | Term frequency with k1=1.2, b=0.75 over title+snippet |
| 2 | Semantic | `semantic_score(result, ctx)` | Jaccard word overlap (proxy until embeddings) |
| 3 | Temporal | `temporal_score(result, freshness)` | Exponential decay: `exp(-freshness * days_old / 365)` |
| 4 | Authority | `authority_score(result)` | Domain tier list + `ln(citation_count)` boost |
| 5 | Evidence | `evidence_score(result)` | Detect numbers, citations, percentages, "et al" |
| 6 | Diversity | `diversity_score(result, ctx)` | 1.0 for new domain, 0.2 for seen domain |
| 7 | Depth | `depth_score(result)` | `min(word_count / 100, 1.0)` |
| 8 | Consensus | `consensus_score(result, ctx)` | Term overlap with other results' claims |

Authority domain tiers:
- **High (0.9):** wikipedia.org, github.com, arxiv.org, nature.com, nih.gov, .gov
- **Medium (0.7):** stackoverflow.com, medium.com, bbc.com, nytimes.com, reuters.com
- **Default (0.5):** Everything else

**Acceptance criteria:**
- [ ] All 8 signals return values in `[0.0, 1.0]`
- [ ] `temporal_score` decays with age, returns 0.5 for unknown dates
- [ ] `authority_score` gives higher scores to known authoritative domains
- [ ] `evidence_score` detects statistical/citation patterns
- [ ] `diversity_score` penalizes same-domain results
- [ ] `consensus_score` measures agreement across results
- [ ] All tests pass: `cargo test -p hsx-core rank::signals`

**Testing instructions:**
```bash
cargo test -p hsx-core rank::signals::tests
# Test each signal with known inputs and expected ranges
# authority_score: wikipedia.org (0.9) vs random-blog.xyz (0.5)
# temporal_score: 1 day old vs 365 days old with freshness=1.0
```

---

### P2-E4-T2: Score Normalization and Fusion

**ID:** `P2-E4-T2`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 2 days
**Dependencies:** P2-E4-T1 (signals)

**Description:**
Combine 8 signals into HyperFusion score. Min-max normalize across result set, then apply intent-weighted linear combination.

**PRD References:**
- SS8.1 -- `w_intent` weight vector per query intent category

**Files to create/modify:**
```
crates/hsx-core/src/rank/
  fusion.rs           -- HyperFusion score computation
```

**Step-by-step implementation:**

```rust
pub enum QueryIntent {
    Factual, HowTo, Comparison, Verification, CurrentEvents,
    DeepAnalysis, Code, Academic, Opinion, Data,
}

pub struct IntentWeights {
    pub bm25: f64, pub semantic: f64, pub temporal: f64, pub authority: f64,
    pub evidence: f64, pub diversity: f64, pub depth: f64, pub consensus: f64,
}

impl IntentWeights {
    pub fn for_intent(intent: QueryIntent) -> Self {
        match intent {
            QueryIntent::Factual => Self {
                bm25: 0.20, semantic: 0.15, temporal: 0.05, authority: 0.20,
                evidence: 0.15, diversity: 0.05, depth: 0.05, consensus: 0.15,
            },
            QueryIntent::CurrentEvents => Self {
                bm25: 0.15, semantic: 0.10, temporal: 0.30, authority: 0.10,
                evidence: 0.10, diversity: 0.05, depth: 0.05, consensus: 0.15,
            },
            QueryIntent::Academic => Self {
                bm25: 0.15, semantic: 0.20, temporal: 0.05, authority: 0.25,
                evidence: 0.15, diversity: 0.05, depth: 0.10, consensus: 0.05,
            },
            QueryIntent::DeepAnalysis => Self {
                bm25: 0.10, semantic: 0.15, temporal: 0.05, authority: 0.10,
                evidence: 0.10, diversity: 0.10, depth: 0.30, consensus: 0.10,
            },
            _ => Self { /* balanced: 0.15/0.15/0.10/0.15/0.10/0.10/0.10/0.15 */ },
        }
    }
}
```

**Fusion pipeline:**
1. Compute raw scores for all 8 signals per result
2. Min-max normalize each signal across the result set to `[0, 1]`
3. Apply weighted sum: `score = sum(w_i * normalized_i)`
4. Sort results by fusion score descending

```rust
pub fn hyperfusion_rank(
    results: &mut [SearchResult], query: &str,
    intent: QueryIntent, freshness_need: f64,
) {
    // Phase 1: compute raw scores, Phase 2: min-max normalize
    // Phase 3: weighted fusion, Phase 4: sort descending
}

fn min_max_normalize(scores: &[RawScores]) -> Vec<RawScores> {
    // For each of 8 fields: find min/max, scale to [0,1]
    // If range < epsilon: use 0.5
}
```

**Acceptance criteria:**
- [ ] `IntentWeights::for_intent()` returns different weights per intent
- [ ] All weight vectors sum to ~1.0
- [ ] `min_max_normalize()` scales all signals to [0, 1]
- [ ] `hyperfusion_rank()` sorts results by fused score descending
- [ ] Results have `fusion_score` populated after ranking
- [ ] All tests pass: `cargo test -p hsx-core rank::fusion`

**Testing instructions:**
```bash
cargo test -p hsx-core rank::fusion::tests
# 3 results with known signals -> verify correct ranking order
# All-identical scores -> 0.5 after normalization
# CurrentEvents intent -> temporal signal upweighted
```

---

### P2-E4-T3: Configurable Weights

**ID:** `P2-E4-T3`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 1 day
**Dependencies:** P2-E4-T2 (fusion)

**Description:**
Allow weight overrides via config file (`[ranking]` section) and CLI flags (`--boost-signal temporal=0.5`).

**Files to create/modify:**
```
crates/hsx-core/src/config.rs  -- Add RankingConfig with weight_overrides
crates/hsx-core/src/rank/fusion.rs -- IntentWeights::with_overrides()
```

**Step-by-step implementation:**

Add to config:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    #[serde(default)]
    pub weight_overrides: HashMap<String, f64>,
    #[serde(default = "default_freshness")]
    pub freshness_need: f64,          // Default: 0.5
    #[serde(default = "default_simhash")]
    pub simhash_threshold: u32,       // Default: 6
}
```

Add to IntentWeights:

```rust
impl IntentWeights {
    pub fn with_overrides(mut self, overrides: &HashMap<String, f64>) -> Self {
        // Apply each override by signal name; unset signals keep defaults
    }
}
```

**Acceptance criteria:**
- [ ] Weight overrides from `~/.config/hsx/config.toml` under `[ranking]`
- [ ] CLI flag `--boost-signal temporal=0.5` overrides individual weights
- [ ] Overrides merge with intent defaults
- [ ] Invalid values produce config validation error
- [ ] All tests pass: `cargo test -p hsx-core rank`

**Testing instructions:**
```bash
cargo test -p hsx-core rank::fusion::tests
# with_overrides: set temporal=0.9, verify it persists
# Config round-trip: serialize -> deserialize -> verify
```

---

## Epic 2.5: CEP Layers 3-5 + QADD

> **PRD Sections:** SS8.3 (CEP), SS8.10 (QADD)
> **Crate:** `hsx-core` -- `src/extract/`, `src/qadd/`
> **Priority:** P1 | **Tasks:** 3

### P2-E5-T1: CEP Layer 3 -- JavaScript Rendering via Headless

**ID:** `P2-E5-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 2-3 days
**Dependencies:** P2-E1-T1 (browser pool), P1-E1-T2 (CEP layers 1-2)

**Description:**
Extend CEP with Layer 3: headless Chromium rendering for JS-heavy pages. Triggered when layers 1-2 detect insufficient content (SPAs, dynamic content).

**PRD References:**
- SS8.3 "CEP" -- Layer 3: ~400ms, ~100MB, check rendered_content_differs_from_static

**Files to create/modify:**
```
crates/hsx-core/src/extract/
  layer3.rs           -- Headless rendering extraction
  pipeline.rs         -- Update CEP pipeline with Layer 3 escalation
```

**Step-by-step implementation:**

```rust
const CONTENT_DIFF_THRESHOLD: f64 = 0.3;

pub struct Layer3Extractor { pool: Arc<BrowserPool> }

impl Layer3Extractor {
    pub async fn extract(&self, url: &str) -> anyhow::Result<ExtractionResult> {
        let tab = self.pool.acquire_tab().await?;
        tab.navigate(url, 15_000).await?;
        tokio::time::sleep(Duration::from_millis(500)).await; // Wait for network idle
        let rendered_html = tab.content().await?;
        // Use Layer 2 extraction on rendered HTML
        let result = Layer2Extractor::extract_from_html(&rendered_html, url)?;
        Ok(ExtractionResult { layer_used: 3, ..result })
    }

    pub fn was_beneficial(static_len: usize, rendered_len: usize) -> bool {
        if static_len == 0 { return true; }
        (rendered_len as f64 - static_len as f64) / static_len as f64 > CONTENT_DIFF_THRESHOLD
    }
}
```

Update `pipeline.rs` to escalate to Layer 3 when layers 1-2 yield `content.len() < min_threshold`.

**Acceptance criteria:**
- [ ] Renders page via headless Chromium, waits for load + network idle
- [ ] Uses Layer 2 extraction on rendered HTML
- [ ] CEP pipeline auto-escalates to Layer 3 when Layers 1-2 insufficient
- [ ] `was_beneficial()` detects significant content difference
- [ ] Timeout prevents hanging on slow pages
- [ ] All tests pass: `cargo test -p hsx-core extract::layer3`

**Testing instructions:**
```bash
cargo test -p hsx-core extract::layer3::tests
# Integration: extract from known SPA, verify more content than layers 1-2
# Timeout test: page that never finishes loading
```

---

### P2-E5-T2: CEP Layers 4-5 -- PDF/Document Extraction + Screenshot OCR

**ID:** `P2-E5-T2`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 2-3 days
**Dependencies:** P2-E5-T1 (Layer 3)

**Description:**
Layer 4: PDF/document extraction via `pdf-extract`/`lopdf`. Layer 5: screenshot OCR for canvas-rendered/image-based text (requires `tesseract` installed).

**PRD References:**
- SS8.3 "CEP" -- Layer 4: ~2s, Layer 5: ~5s (last resort)

**Files to create/modify:**
```
crates/hsx-core/src/extract/
  layer4.rs           -- PDF/document extraction
  layer5.rs           -- Screenshot OCR extraction
  pipeline.rs         -- Update with Layer 4-5 escalation
```

**Step-by-step implementation:**

**Layer 4 (`extract/layer4.rs`):**
```rust
pub struct Layer4Extractor;

impl Layer4Extractor {
    pub fn extract_pdf(bytes: &[u8], url: &str) -> anyhow::Result<ExtractionResult> {
        let text = pdf_extract::extract_text_from_mem(bytes)?;
        Ok(ExtractionResult { content: text, layer_used: 4, content_type: "application/pdf".into(), .. })
    }
    pub fn is_document(content_type: &str) -> bool {
        content_type.contains("pdf") || content_type.contains("msword")
            || content_type.contains("officedocument")
    }
}
```

**Layer 5 (`extract/layer5.rs`):**
```rust
pub struct Layer5Extractor { pool: Arc<BrowserPool> }

impl Layer5Extractor {
    pub async fn extract(&self, url: &str) -> anyhow::Result<ExtractionResult> {
        let tab = self.pool.acquire_tab().await?;
        tab.navigate(url, 20_000).await?;
        // Scroll to trigger lazy loading
        tab.evaluate("window.scrollTo(0, document.body.scrollHeight)").await?;
        tokio::time::sleep(Duration::from_secs(2)).await;
        // Screenshot -> PNG bytes -> tesseract OCR
        let screenshot = tab.page().screenshot(/* PNG format */).await?;
        let text = run_tesseract_ocr(&screenshot).await?;
        Ok(ExtractionResult { content: text, layer_used: 5, .. })
    }
}

async fn run_tesseract_ocr(png_bytes: &[u8]) -> anyhow::Result<String> {
    // Spawn `tesseract stdin stdout --psm 3`, pipe PNG bytes, read stdout
}
```

**Acceptance criteria:**
- [ ] Layer 4 extracts text from PDF bytes
- [ ] `is_document()` detects PDF/DOCX content types
- [ ] Layer 5 takes screenshot and runs OCR
- [ ] Layer 5 scrolls to trigger lazy loading before screenshot
- [ ] Graceful fallback when tesseract not installed
- [ ] Pipeline escalates to Layer 4 for documents, Layer 5 as last resort
- [ ] All tests pass: `cargo test -p hsx-core extract::layer4 extract::layer5`

**Testing instructions:**
```bash
cargo test -p hsx-core extract::layer4::tests   # Parse test PDF fixture
cargo test -p hsx-core extract::layer5::tests   # Requires tesseract installed
```

---

### P2-E5-T3: QADD -- Query-Aware DOM Distillation

**ID:** `P2-E5-T3`
**Status:** `TODO`
**Priority:** P0
**Estimated effort:** 3-4 days
**Dependencies:** P2-E5-T1 (Layer 3 for rendered DOM), P1-E3-T1 (BM25 scoring)

**Description:**
QADD reduces DOM to query-relevant nodes BEFORE extraction. 5-step pipeline: structural pruning -> BM25 scoring -> semantic check -> context preservation -> token budget packing. Target: 50K tokens -> ~2.5K tokens.

**PRD References:**
- SS8.10 "QADD" -- Full 5-step pipeline
- Combined with D2Snap DOM downsampling for 5-10x additional reduction

**Files to create/modify:**
```
crates/hsx-core/src/qadd/
  mod.rs              -- Module root
  pipeline.rs         -- QADD 5-step pipeline
  pruning.rs          -- Structural and relevance pruning
```

**Step-by-step implementation:**

**Step 1: Structural pruning (`qadd/pruning.rs`)**

```rust
/// Tags removed in Step 1: nav, footer, aside, script, style, noscript, iframe, svg, form, header
const STRUCTURAL_PRUNE_TAGS: &[&str] = &["nav", "footer", "aside", "script", "style", ...];
/// CSS selectors: [role='navigation'], .sidebar, .ad, .social-share, .cookie-notice, #comments
const PRUNE_SELECTORS: &[&str] = &["[role='navigation']", ".sidebar", ".ad", ...];

pub struct TextNode {
    pub text: String,
    pub tag_context: String,
    pub depth: usize,
    pub estimated_tokens: usize,   // ~1.33 tokens/word
    pub relevance_score: f64,
}

/// Walk DOM, skip pruned tags/selectors, collect text nodes > 10 chars.
pub fn structural_prune(html: &str) -> Vec<TextNode> { /* ... */ }
```

**Step 2: QADD pipeline (`qadd/pipeline.rs`)**

```rust
pub struct QaddConfig {
    pub bm25_threshold: f64,       // Default: 0.1
    pub semantic_threshold: f64,   // Default: 0.2
    pub token_budget: usize,       // Default: 2000
}

pub struct QaddResult {
    pub distilled_content: String,
    pub tokens_original: usize,
    pub tokens_distilled: usize,
    pub nodes_kept: usize,
    pub nodes_pruned: usize,
}

pub struct QaddPipeline { config: QaddConfig }

impl QaddPipeline {
    pub fn distill(&self, html: &str, query: &str) -> QaddResult {
        // Step 1: structural_prune(html) -> Vec<TextNode>
        // Step 2: BM25 score each node against query, prune below threshold
        // Step 3: Word overlap check (semantic proxy), prune below threshold
        // Step 4: Preserve heading context near retained nodes
        // Step 5: Sort by relevance desc, greedy knapsack into token_budget
        //         (truncate last node if partial fit, min 50 tokens remaining)
    }
}
```

**Acceptance criteria:**
- [ ] Step 1: removes nav, footer, aside, scripts, ads
- [ ] Step 2: BM25 prunes irrelevant text nodes
- [ ] Step 3: semantic check provides additional filtering
- [ ] Step 4: heading context preserved near relevant nodes
- [ ] Step 5: greedy knapsack packs within token budget
- [ ] `QaddResult` reports original vs distilled token counts
- [ ] Achieves 10-20x token reduction on typical pages
- [ ] All tests pass: `cargo test -p hsx-core qadd`

**Testing instructions:**
```bash
cargo test -p hsx-core qadd::pipeline::tests
# Test with large HTML fixture, query "specific topic"
# Verify output fits within token_budget
# Verify relevant content preserved, boilerplate removed
# Benchmark: time pipeline on 10 real-world pages
```

---

## Phase 2 Dependency Graph

```
P2-E1-T1 (Browser Pool) ──┬── P2-E1-T2 (Google) ───┐
                           ├── P2-E1-T3 (Bing) ─────┤
                           ├── P2-E1-T4 (Scholar) ──┤
                           ├── P2-E5-T1 (CEP L3) ── P2-E5-T2 (CEP L4/5)
                           └── P2-E5-T3 (QADD)      │
                                                     │
P1-E1-T1 (HTTP Client) ──┬── P2-E2-T1 (SearXNG) ──┤
                          ├── P2-E2-T2 (Wikipedia) ─┤
                          └── P2-E2-T3 (Additional) ┤
                                                     │
                          All search backends ───────┤
                                                     ▼
                          P2-E3-T1 (Orchestrator) ── P2-E3-T2 (Dedup)
                                   │                 │
                                   └── P2-E3-T3 (Fallback)
                                                     │
P1-E3-T1 (BM25) ─────── P2-E4-T1 (8 Signals) ── P2-E4-T2 (Fusion)
                                                     │
                                                 P2-E4-T3 (Config Weights)
```

---

## Phase 2 Completion Checklist

- [ ] All 16 tasks completed and passing tests
- [ ] Browser pool manages Chromium lifecycle correctly
- [ ] All 11 search backends implemented and returning results
- [ ] Parallel orchestrator dispatches to all backends with timeout
- [ ] Cross-source deduplication working (URL + SimHash)
- [ ] Fallback chains handle backend failures gracefully
- [ ] HyperFusion 8-signal ranking produces correctly ordered results
- [ ] Configurable weights work via config file and CLI
- [ ] CEP layers 3-5 handle JS rendering, PDFs, and OCR
- [ ] QADD achieves 10x+ token reduction
- [ ] No clippy warnings: `cargo clippy --workspace`
- [ ] All tests pass: `cargo test --workspace`
- [ ] Integration tests verify end-to-end multi-engine search

---

## Estimated Phase 2 Timeline

| Week | Epic | Tasks | Focus |
|------|------|-------|-------|
| 5 | 2.1 | T1-T4 | Browser pool + Google/Bing/Scholar scrapers |
| 6 | 2.2 + 2.3 | T1-T3 + T1-T3 | HTTP backends + orchestrator + dedup |
| 7 | 2.4 | T1-T3 | HyperFusion 8-signal ranking |
| 8 | 2.5 | T1-T3 | CEP layers 3-5 + QADD |

**Total estimated effort:** 28-38 developer-days
