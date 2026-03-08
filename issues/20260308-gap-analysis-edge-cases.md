# Fetchium Gap Analysis: Edge Cases To Close For World-Class Reliability

## Scope

This audit focuses on failures that silently degrade answer quality rather than crash loudly:

1. Query understanding and routing
2. Freshness and temporal intent
3. Source diversity and result collapse
4. Extraction robustness on malformed or hostile pages
5. Validation behavior under sparse or conflicting evidence

## What Was Fixed In This Pass

- Query locale routing now uses boundary-aware phrase matching instead of raw substring checks.
- Explicit locale hints such as `site:bbc.co.uk` and `lang:fr` are now detected.
- Known false positives such as food-related `turkey recipe` queries no longer route to Turkey.
- Regression coverage for locale inference now lives in a dedicated test file.

## Highest-Value Remaining Gaps

### 1. Temporal intent is too keyword-fragile

Current behavior in [`temporal.rs`](/home/echo/projects/Fetchium/crates/fetchium-core/src/validate/temporal.rs) relies on a short static keyword list. It will miss common real-world requests like:

- `today`
- `current`
- `as of now`
- `this week`
- `Q1 2026`
- `2024` and earlier year-specific queries

Impact:
- stale sources can score too highly for recency-sensitive queries
- year-scoped questions can be misclassified as generic factual requests

Recommended follow-up:
- classify explicit dates, relative dates, and rolling windows
- treat future dates and impossible timestamps as validation errors
- add tests for `today`, `current`, `as of`, quarter-based, and year-only queries

### 2. Locale inference still needs ambiguity modeling

The current locale matcher is materially better after this pass, but it still lacks confidence scoring and entity disambiguation for ambiguous terms such as:

- `georgia` country vs U.S. state
- `jordan` country vs person name
- `chad` country vs person slang/name
- `paris` city vs person/media title

Impact:
- incorrect proxy routing for ambiguous queries

Recommended follow-up:
- return `(country_code, confidence, reason)` instead of only `Option<&str>`
- prefer explicit patterns like `in georgia` or `site:.ge` over bare tokens
- add an abstain path when ambiguity is high

### 3. Sparse-evidence validation is too forgiving

Several validators default to middling scores when metadata is absent. That is safe for continuity but weak for trust calibration.

Impact:
- undated or weakly sourced results can pass with insufficient evidence
- low-signal result sets may look more certain than they are

Recommended follow-up:
- add a minimum-evidence floor for high-stakes intents
- downgrade confidence when all results come from one domain family or one timestamp band
- surface `insufficient_evidence` as a first-class outcome rather than only warnings

### 4. Result diversity guardrails should be stricter

Search orchestration and fusion are already sophisticated, but the main failure mode for production systems is still “many near-duplicate pages from one source cluster.”

Impact:
- apparent corroboration without real independence
- answer synthesis overfits one publisher or one SEO pattern

Recommended follow-up:
- cap top-N by domain family
- detect mirrored content and syndicated copies earlier
- add tests where five results differ only by URL parameters or mirrored republishing

### 5. Extraction needs hostile-input edge coverage

The extraction stack is broad, but the repository should explicitly regression-test:

- malformed HTML with unclosed tags
- huge boilerplate wrappers with tiny content islands
- pages containing repeated navigation blocks
- pages whose visible body is mostly inside nested shadow-like div structures
- pages with future-dated metadata or invalid RFC3339 timestamps

Impact:
- empty or noisy extractions that still flow into ranking and synthesis

Recommended follow-up:
- build fixture-driven tests for malformed and adversarial HTML
- score extraction quality before downstream ranking consumes it

## Proposed Execution Order

1. Harden temporal intent classification and add date-edge tests.
2. Upgrade locale inference to confidence-based output with abstention.
3. Add sparse-evidence fail-closed behavior for high-stakes queries.
4. Add diversity regression tests for mirrored and same-domain result floods.
5. Expand extraction fixtures for malformed/adversarial pages.

## Success Criteria

- Fewer silent misroutes for locale-sensitive searches
- Better abstention on weak or ambiguous evidence
- Stronger freshness behavior for recency-sensitive prompts
- Measurably higher diversity in top-ranked evidence sets
- New regression fixtures covering malformed, duplicated, and low-signal inputs
