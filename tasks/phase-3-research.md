# Phase 3: Validation, Research & Citations

> **Phase:** 3 of 8 | **Priority:** P1 | **Duration:** Weeks 9-12
> **Depends on:** Phase 2 (Multi-Engine Search & Headless) fully complete
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 8.6, 8.7, 10 (Mode B), 19, 24, 43, 44
> **Epics:** 3 | **Tasks:** 12

---

## Phase 3 Summary

Phase 3 transforms Fetchium from a multi-engine search tool into a validated research platform with verifiable evidence chains. It adds:

1. **6-Layer Validation Pipeline** -- Source, content, freshness, cross-source, extraction quality, and output integrity checks with configurable strictness (PRD SS19)
2. **RAR Self-Correction** -- Reflection-Augmented Research loop inspired by Self-RAG and CRAG that detects and auto-corrects bad retrievals at 5 reflection checkpoints (PRD SS8.6)
3. **Citation System (6 styles)** -- APA, MLA, Chicago, IEEE, BibTeX, and inline citation formatters with strict evidence mode (PRD SS24)
4. **Evidence Graph Protocol (EGP)** -- Graph-based evidence linking with claim provenance, SHA-256 content hashes, contradiction edges, and cryptographic verification (PRD SS8.7, SS24)
5. **Research Mode** -- `fetchium research` and `fetchium agent-research` commands for structured multi-source analysis (PRD SS10 Mode B, SS9)

---

## Prerequisites

All of the following must be `DONE` before starting any Phase 3 task:

| Dependency                    | Phase   | What It Provides                                     |
| ----------------------------- | ------- | ---------------------------------------------------- |
| P2-E3 (Parallel orchestrator) | Phase 2 | Full multi-backend search orchestrator               |
| P2-E4 (HyperFusion ranking)   | Phase 2 | 8-signal ranking with per-result confidence scores   |
| P2-E5 (CEP L3-5 + QADD)       | Phase 2 | Headless extraction and query-aware DOM distillation |
| P1-E3-T2 (QATBE)              | Phase 1 | Token-budgeted extraction                            |
| P1-E3-T3 (SCS)                | Phase 1 | Semantic content segmentation                        |
| P1-E3-T4 (PDS)                | Phase 1 | Progressive detail streaming tiers                   |
| P1-E5-T1 (BM25)               | Phase 1 | BM25 scoring for relevance evaluation                |
| P1-E6-T1 (Cache)              | Phase 1 | Memory + disk cache                                  |
| P1-E1-T1 (HTTP client)        | Phase 1 | HTTP fetching with retry                             |

---

## Epic 3.1: 6-Layer Validation + RAR Self-Correction

> **PRD Sections:** SS19 (Validation & Reliability Layer), SS8.6 (Reflection-Augmented Research)
> **Crate:** `fetchium-core` -- `src/validate/`
> **Priority:** P1 | **Tasks:** 6

### P3-E1-T1: Cross-Source Verification (V4)

**ID:** `P3-E1-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Build cross-source verification (V4 in the 6-layer pipeline). Compares claims across multiple sources to detect agreement, contradiction, and partial support. Each claim receives a consensus score and contradictions get a severity rating. Also defines all shared validation types used by every other task in this epic.

**PRD References:**

- SS19 "V4: Cross-Source Validation -- Claim consistency, triangulation, contradiction detection"
- SS43 `Contradiction`, `EvidenceLink` with `evidence_type: Supports | Contradicts | PartiallySupports`
- SS42 Features 9-10: contradiction detection, consensus scoring

**Files to create/modify:**

```
crates/fetchium-core/src/validate/mod.rs          -- Module root, re-exports
crates/fetchium-core/src/validate/types.rs        -- All validation types
crates/fetchium-core/src/validate/cross_source.rs -- V4 engine
```

**Dependencies:** P2-E4 (HyperFusion), P1-E5-T1 (BM25), P0-E1-T2 (Types)

**Step 1: Define validation types (`validate/types.rs`)**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub layers_run: Vec<ValidationLayerId>,
    pub layer_results: Vec<LayerResult>,
    pub passed: bool,
    pub confidence: f64,
    pub contradictions: Vec<Contradiction>,
    pub consensus: Vec<ClaimConsensus>,
    pub mode: ValidationMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationLayerId {
    V1Source, V2Content, V3Freshness, V4CrossSource,
    V5ExtractionQuality, V6OutputIntegrity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    pub layer: ValidationLayerId,
    pub passed: bool,
    pub score: f64,
    pub issues: Vec<ValidationIssue>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub code: String,
    pub message: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity { Error, Warning, Info }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ValidationMode {
    Strict,   // All 6 layers + RAR
    #[default]
    Standard, // V1-V3 + basic V4
    Fast,     // V1 only
    Off,      // Skip
}

impl ValidationMode {
    pub fn active_layers(&self) -> Vec<ValidationLayerId> {
        match self {
            Self::Strict => vec![
                ValidationLayerId::V1Source, ValidationLayerId::V2Content,
                ValidationLayerId::V3Freshness, ValidationLayerId::V4CrossSource,
                ValidationLayerId::V5ExtractionQuality, ValidationLayerId::V6OutputIntegrity,
            ],
            Self::Standard => vec![
                ValidationLayerId::V1Source, ValidationLayerId::V2Content,
                ValidationLayerId::V3Freshness, ValidationLayerId::V4CrossSource,
            ],
            Self::Fast => vec![ValidationLayerId::V1Source],
            Self::Off => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub text: String,
    pub normalized: String,
    pub source_url: String,
    pub source_index: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub claim_a: Claim,
    pub claim_b: Claim,
    pub severity: ContradictionSeverity,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContradictionSeverity { High, Medium, Low }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimConsensus {
    pub claim: Claim,
    pub supporting_sources: usize,
    pub contradicting_sources: usize,
    pub total_sources: usize,
    pub consensus_score: f64,
}
```

**Step 2: Build cross-source verifier (`validate/cross_source.rs`)**

```rust
use crate::validate::types::*;
use sha2::{Sha256, Digest};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct SourceContent {
    pub url: String,
    pub index: usize,
    pub title: String,
    pub claims: Vec<String>,
    pub full_text: String,
    pub confidence: f64,
}

pub struct CrossSourceVerifier {
    similarity_threshold: f64,
    min_sources: usize,
}

impl Default for CrossSourceVerifier {
    fn default() -> Self { Self { similarity_threshold: 0.6, min_sources: 2 } }
}

impl CrossSourceVerifier {
    pub fn new() -> Self { Self::default() }

    pub fn with_threshold(mut self, t: f64) -> Self { self.similarity_threshold = t; self }

    /// Run V4 cross-source verification.
    pub fn verify(&self, sources: &[SourceContent]) -> LayerResult {
        let start = std::time::Instant::now();
        if sources.len() < self.min_sources {
            return LayerResult {
                layer: ValidationLayerId::V4CrossSource, passed: true, score: 0.5,
                issues: vec![ValidationIssue {
                    severity: IssueSeverity::Info, code: "V4_INSUFFICIENT_SOURCES".into(),
                    message: format!("Need {} sources, have {}", self.min_sources, sources.len()),
                    source_url: None,
                }],
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }
        let all_claims = self.extract_claims(sources);
        let clusters = self.cluster_claims(&all_claims);
        let mut issues = Vec::new();
        let mut scores = Vec::new();
        for cluster in &clusters {
            let consensus = self.score_consensus(cluster, sources.len());
            if consensus.consensus_score < 0.5 && consensus.contradicting_sources > 0 {
                for c in self.detect_contradictions(cluster) {
                    issues.push(ValidationIssue {
                        severity: match c.severity {
                            ContradictionSeverity::High => IssueSeverity::Error,
                            ContradictionSeverity::Medium => IssueSeverity::Warning,
                            ContradictionSeverity::Low => IssueSeverity::Info,
                        },
                        code: "V4_CONTRADICTION".into(),
                        message: c.description.clone(),
                        source_url: Some(c.claim_a.source_url.clone()),
                    });
                }
            }
            scores.push(consensus.consensus_score);
        }
        let avg = if scores.is_empty() { 1.0 }
                  else { scores.iter().sum::<f64>() / scores.len() as f64 };
        let high_ct = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
        LayerResult {
            layer: ValidationLayerId::V4CrossSource,
            passed: high_ct == 0 && avg >= 0.4, score: avg, issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn extract_claims(&self, sources: &[SourceContent]) -> Vec<Claim> {
        sources.iter().flat_map(|s| {
            s.claims.iter().enumerate().map(move |(i, text)| Claim {
                id: format!("{}:{}", &Self::hash_short(&s.url), i),
                text: text.clone(), normalized: Self::normalize(text),
                source_url: s.url.clone(), source_index: s.index, confidence: s.confidence,
            })
        }).collect()
    }

    fn normalize(text: &str) -> String {
        text.to_lowercase().chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn cluster_claims(&self, claims: &[Claim]) -> Vec<Vec<Claim>> {
        let mut assigned = vec![false; claims.len()];
        let mut clusters = Vec::new();
        for i in 0..claims.len() {
            if assigned[i] { continue; }
            let mut cluster = vec![claims[i].clone()];
            assigned[i] = true;
            for j in (i + 1)..claims.len() {
                if assigned[j] || claims[i].source_index == claims[j].source_index { continue; }
                if Self::jaccard(&claims[i].normalized, &claims[j].normalized) >= self.similarity_threshold {
                    cluster.push(claims[j].clone());
                    assigned[j] = true;
                }
            }
            if cluster.len() > 1 { clusters.push(cluster); }
        }
        clusters
    }

    fn jaccard(a: &str, b: &str) -> f64 {
        let ba: HashSet<_> = a.split_whitespace().collect::<Vec<_>>()
            .windows(2).map(|w| format!("{} {}", w[0], w[1])).collect();
        let bb: HashSet<_> = b.split_whitespace().collect::<Vec<_>>()
            .windows(2).map(|w| format!("{} {}", w[0], w[1])).collect();
        let inter = ba.intersection(&bb).count();
        let union = ba.union(&bb).count();
        if union == 0 { return if ba.is_empty() && bb.is_empty() { 1.0 } else { 0.0 }; }
        inter as f64 / union as f64
    }

    fn score_consensus(&self, cluster: &[Claim], total: usize) -> ClaimConsensus {
        let anchor = &cluster[0];
        let mut sup = HashSet::new();
        let mut con = HashSet::new();
        sup.insert(anchor.source_index);
        for c in &cluster[1..] {
            if Self::claims_agree(&anchor.normalized, &c.normalized) >= 0.5 {
                sup.insert(c.source_index);
            } else { con.insert(c.source_index); }
        }
        ClaimConsensus {
            claim: anchor.clone(), supporting_sources: sup.len(),
            contradicting_sources: con.len(), total_sources: total,
            consensus_score: sup.len() as f64 / total as f64,
        }
    }

    fn claims_agree(a: &str, b: &str) -> f64 {
        let neg = ["not", "no", "never", "neither", "doesnt", "isnt", "cant", "wont"];
        let a_neg = neg.iter().any(|w| a.contains(w));
        let b_neg = neg.iter().any(|w| b.contains(w));
        if a_neg != b_neg { return 0.2; }
        Self::jaccard(a, b).min(1.0)
    }

    fn detect_contradictions(&self, cluster: &[Claim]) -> Vec<Contradiction> {
        let mut out = Vec::new();
        for i in 0..cluster.len() {
            for j in (i + 1)..cluster.len() {
                if cluster[i].source_index == cluster[j].source_index { continue; }
                let agr = Self::claims_agree(&cluster[i].normalized, &cluster[j].normalized);
                if agr < 0.5 {
                    let sev = if agr < 0.2 { ContradictionSeverity::High }
                              else if agr < 0.35 { ContradictionSeverity::Medium }
                              else { ContradictionSeverity::Low };
                    out.push(Contradiction {
                        claim_a: cluster[i].clone(), claim_b: cluster[j].clone(), severity: sev,
                        description: format!("{:?}: \"{}\" vs \"{}\"", sev, cluster[i].text, cluster[j].text),
                    });
                }
            }
        }
        out
    }

    fn hash_short(input: &str) -> String {
        let mut h = Sha256::new(); h.update(input.as_bytes());
        hex::encode(&h.finalize()[..4])
    }
}
```

**Acceptance criteria:**

- [ ] `verify()` returns `passed: true` when sources agree on claims
- [ ] Contradictions detected when sources contain negated or numerically conflicting claims
- [ ] `ContradictionSeverity` classified correctly (High/Medium/Low)
- [ ] Graceful neutral score (0.5) when fewer than `min_sources` provided
- [ ] `jaccard()` returns 1.0 for identical, 0.0 for disjoint inputs
- [ ] All types implement `Serialize + Deserialize + Debug + Clone`
- [ ] `cargo test` and `cargo clippy` pass cleanly

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    fn src(i: usize, url: &str, claims: &[&str], conf: f64) -> SourceContent {
        SourceContent { url: url.into(), index: i, title: format!("S{i}"),
            claims: claims.iter().map(|c| c.to_string()).collect(),
            full_text: claims.join(". "), confidence: conf }
    }
    #[test]
    fn consistent_sources_pass() {
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[
            src(0, "https://a.com", &["Rust released in 2015"], 0.9),
            src(1, "https://b.com", &["Rust released in 2015"], 0.8),
        ]);
        assert!(r.passed);
    }
    #[test]
    fn contradictions_detected() {
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[
            src(0, "https://a.com", &["The project launched in 2024"], 0.9),
            src(1, "https://b.com", &["The project did not launch in 2024"], 0.8),
        ]);
        assert!(!r.issues.is_empty());
    }
    #[test]
    fn insufficient_sources() {
        let v = CrossSourceVerifier::new();
        let r = v.verify(&[src(0, "https://a.com", &["claim"], 0.9)]);
        assert!(r.passed); assert_eq!(r.score, 0.5);
    }
}
```

---

### P3-E1-T2: Temporal Validation (V3)

**ID:** `P3-E1-T2`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2-3 days

**Description:**
Build the temporal/freshness validation layer (V3). Checks published dates, detects staleness using query-intent-aware thresholds, and penalizes undated content. Uses exponential decay scoring.

**PRD References:**

- SS19 "V3: Freshness Validation -- Published date, staleness, cache freshness"
- SS21 "Temporal: Exponential decay with intent-calibrated half-life"

**Files to create:** `crates/fetchium-core/src/validate/temporal.rs`

**Dependencies:** P3-E1-T1 (types)

**Step 1: Build temporal validator**

```rust
use crate::validate::types::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemporalIntent { Recent, Historical, Default }

#[derive(Debug, Clone)]
pub struct SourceFreshness {
    pub url: String,
    pub published_date: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
}

pub struct TemporalValidator {
    pub default_max_age_days: u64,
    pub recent_max_age_days: u64,
    pub historical_max_age_days: u64,
}

impl Default for TemporalValidator {
    fn default() -> Self {
        Self { default_max_age_days: 365, recent_max_age_days: 30, historical_max_age_days: 3650 }
    }
}

impl TemporalValidator {
    pub fn classify_intent(query: &str) -> TemporalIntent {
        let q = query.to_lowercase();
        if ["latest","newest","recent","2026","2025","this year","just released"]
            .iter().any(|s| q.contains(s)) { TemporalIntent::Recent }
        else if ["history of","origin of","when was","first","originally","founded"]
            .iter().any(|s| q.contains(s)) { TemporalIntent::Historical }
        else { TemporalIntent::Default }
    }

    pub fn validate(&self, sources: &[SourceFreshness], query: &str) -> LayerResult {
        let start = std::time::Instant::now();
        let intent = Self::classify_intent(query);
        let max_age = match intent {
            TemporalIntent::Recent => self.recent_max_age_days,
            TemporalIntent::Historical => self.historical_max_age_days,
            TemporalIntent::Default => self.default_max_age_days,
        };
        let now = Utc::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();
        for s in sources {
            let best = s.published_date.or(s.last_modified);
            match best {
                Some(date) => {
                    let age = (now - date).num_days().unsigned_abs();
                    let score = (-2.0 * age as f64 / max_age as f64).exp();
                    scores.push(score);
                    if age > max_age {
                        issues.push(ValidationIssue {
                            severity: if age > max_age * 2 { IssueSeverity::Error }
                                      else { IssueSeverity::Warning },
                            code: "V3_STALE_CONTENT".into(),
                            message: format!("{} days old (max {} for {:?})", age, max_age, intent),
                            source_url: Some(s.url.clone()),
                        });
                    }
                }
                None => {
                    scores.push(0.3);
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Warning, code: "V3_NO_DATE".into(),
                        message: "No published date found".into(),
                        source_url: Some(s.url.clone()),
                    });
                }
            }
        }
        let avg = if scores.is_empty() { 0.5 }
                  else { scores.iter().sum::<f64>() / scores.len() as f64 };
        LayerResult {
            layer: ValidationLayerId::V3Freshness, passed: avg >= 0.3,
            score: avg, issues, duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}
```

**Acceptance criteria:**

- [ ] `classify_intent` returns `Recent` for "latest Rust 2026", `Historical` for "history of Linux"
- [ ] Fresh content (age < max) yields score > 0.9; stale content yields warning/error
- [ ] Undated content penalized to 0.3
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Duration};

    #[test]
    fn fresh_content_passes() {
        let v = TemporalValidator::default();
        let r = v.validate(&[SourceFreshness {
            url: "https://x.com".into(),
            published_date: Some(Utc::now() - Duration::days(5)),
            last_modified: None,
        }], "what is Rust");
        assert!(r.passed); assert!(r.score > 0.9);
    }
    #[test]
    fn stale_for_recent_query() {
        let v = TemporalValidator::default();
        let r = v.validate(&[SourceFreshness {
            url: "https://x.com".into(),
            published_date: Some(Utc::now() - Duration::days(90)),
            last_modified: None,
        }], "latest Rust news 2026");
        assert!(r.issues.iter().any(|i| i.code == "V3_STALE_CONTENT"));
    }
    #[test]
    fn intent_classification() {
        assert_eq!(TemporalValidator::classify_intent("latest news"), TemporalIntent::Recent);
        assert_eq!(TemporalValidator::classify_intent("history of Rust"), TemporalIntent::Historical);
        assert_eq!(TemporalValidator::classify_intent("what is Rust"), TemporalIntent::Default);
    }
}
```

---

### P3-E1-T3: Authority Scoring (V1 partial)

**ID:** `P3-E1-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2 days

**Description:**
Build authority scoring as part of V1 (Source Validation). Assigns a trust score to each source based on domain reputation tiers, SSL validity, redirect analysis, and known-good/known-bad domain lists. This feeds into both the validation pipeline and HyperFusion ranking.

**PRD References:**

- SS19 "V1: Source Validation -- Reachability, SSL, domain reputation, redirect analysis"
- SS21 "Authority: Domain scoring + citation chain analysis"

**Files to create:** `crates/fetchium-core/src/validate/authority.rs`

**Dependencies:** P3-E1-T1 (types), P1-E1-T1 (HTTP client)

**Step 1: Build authority scorer**

```rust
use crate::validate::types::*;
use std::collections::HashMap;

/// Domain reputation tiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainTier {
    /// Tier 1: Highly authoritative (.gov, .edu, major outlets).
    Authoritative,
    /// Tier 2: Generally reliable (established tech blogs, official docs).
    Reliable,
    /// Tier 3: Mixed quality (forums, personal blogs).
    Mixed,
    /// Tier 4: Low trust (unknown, new domains).
    Unknown,
    /// Tier 5: Blocklisted (known spam, SEO farms).
    Blocked,
}

pub struct AuthorityScorer {
    domain_tiers: HashMap<String, DomainTier>,
}

impl Default for AuthorityScorer {
    fn default() -> Self {
        let mut tiers = HashMap::new();
        // Tier 1: Authoritative
        for d in &["github.com","docs.rs","rust-lang.org","wikipedia.org","arxiv.org",
                   "nature.com","science.org","ieee.org","acm.org","nih.gov","cdc.gov"] {
            tiers.insert(d.to_string(), DomainTier::Authoritative);
        }
        // Tier 2: Reliable
        for d in &["stackoverflow.com","developer.mozilla.org","medium.com",
                   "dev.to","hacker-news.firebaseio.com","reddit.com"] {
            tiers.insert(d.to_string(), DomainTier::Reliable);
        }
        Self { domain_tiers: tiers }
    }
}

impl AuthorityScorer {
    /// Score a single source. Returns 0.0-1.0.
    pub fn score(&self, url: &str, has_ssl: bool, redirect_count: u32) -> (f64, DomainTier) {
        let domain = Self::extract_domain(url);
        let tier = self.domain_tiers.get(&domain).copied().unwrap_or_else(|| {
            // Heuristic: .gov/.edu = Authoritative, .org = Reliable
            if domain.ends_with(".gov") || domain.ends_with(".edu") { DomainTier::Authoritative }
            else if domain.ends_with(".org") { DomainTier::Reliable }
            else { DomainTier::Unknown }
        });
        let mut score = match tier {
            DomainTier::Authoritative => 0.95,
            DomainTier::Reliable => 0.75,
            DomainTier::Mixed => 0.50,
            DomainTier::Unknown => 0.40,
            DomainTier::Blocked => 0.05,
        };
        if !has_ssl { score *= 0.7; }
        if redirect_count > 2 { score *= 0.9; }
        if redirect_count > 5 { score *= 0.8; }
        (score.clamp(0.0, 1.0), tier)
    }

    /// Run as V1 validation layer.
    pub fn validate_sources(&self, sources: &[(String, bool, u32)]) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();
        for (url, has_ssl, redirects) in sources {
            let (score, tier) = self.score(url, *has_ssl, *redirects);
            scores.push(score);
            if tier == DomainTier::Blocked {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error, code: "V1_BLOCKED_DOMAIN".into(),
                    message: format!("Domain is blocklisted: {url}"),
                    source_url: Some(url.clone()),
                });
            }
            if !has_ssl {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V1_NO_SSL".into(),
                    message: format!("No SSL/TLS: {url}"), source_url: Some(url.clone()),
                });
            }
        }
        let avg = if scores.is_empty() { 0.5 }
                  else { scores.iter().sum::<f64>() / scores.len() as f64 };
        LayerResult {
            layer: ValidationLayerId::V1Source, passed: avg >= 0.3, score: avg,
            issues, duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn extract_domain(url: &str) -> String {
        url::Url::parse(url).ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default()
    }
}
```

**Acceptance criteria:**

- [ ] `.gov`/`.edu` domains score >= 0.9; blocklisted domains score <= 0.1
- [ ] No-SSL penalty reduces score by 30%
- [ ] Excessive redirects (>5) reduce score
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gov_domain_high_score() {
        let s = AuthorityScorer::default();
        let (score, tier) = s.score("https://nih.gov/article", true, 0);
        assert!(score >= 0.9); assert_eq!(tier, DomainTier::Authoritative);
    }
    #[test]
    fn no_ssl_penalty() {
        let s = AuthorityScorer::default();
        let (with, _) = s.score("https://example.com/page", true, 0);
        let (without, _) = s.score("http://example.com/page", false, 0);
        assert!(without < with);
    }
}
```

---

### P3-E1-T4: Consistency Checking (V2 + V5 + V6)

**ID:** `P3-E1-T4`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2-3 days

**Description:**
Build the remaining validation layers: V2 (content validation -- relevance, language detection, paywall/error page detection), V5 (extraction quality -- completeness, structure, truncation), and V6 (output integrity -- citation verification, link validity). These are grouped because they are lighter layers that share similar patterns.

**PRD References:**

- SS19 V2: "Relevance, language, dedup, paywall, error page detection"
- SS19 V5: "Completeness, structure, encoding, truncation"
- SS19 V6: "Citation verification, link validity, format compliance"

**Files to create:**

```
crates/fetchium-core/src/validate/content.rs     -- V2
crates/fetchium-core/src/validate/extraction.rs  -- V5
crates/fetchium-core/src/validate/output.rs      -- V6
```

**Dependencies:** P3-E1-T1 (types), P1-E5-T1 (BM25 for relevance)

**Step 1: Content validator V2 (`validate/content.rs`)**

```rust
use crate::validate::types::*;

pub struct ContentValidator {
    pub min_relevance_score: f64,
    pub min_content_length: usize,
}

impl Default for ContentValidator {
    fn default() -> Self { Self { min_relevance_score: 0.2, min_content_length: 100 } }
}

impl ContentValidator {
    /// V2: Validate content quality of extracted text.
    pub fn validate(
        &self,
        sources: &[ContentInput],
        query: &str,
    ) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();

        for src in sources {
            let mut score = 1.0;

            // Check minimum content length
            if src.text.len() < self.min_content_length {
                score *= 0.3;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V2_TOO_SHORT".into(),
                    message: format!("Content too short: {} chars", src.text.len()),
                    source_url: Some(src.url.clone()),
                });
            }

            // Check for error/paywall page patterns
            if Self::is_error_page(&src.text) {
                score *= 0.1;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error, code: "V2_ERROR_PAGE".into(),
                    message: "Detected error or paywall page".into(),
                    source_url: Some(src.url.clone()),
                });
            }

            // BM25-lite relevance: check query terms present in content
            let query_terms: Vec<&str> = query.split_whitespace()
                .filter(|w| w.len() > 2).collect();
            let lower = src.text.to_lowercase();
            let term_hits = query_terms.iter()
                .filter(|t| lower.contains(&t.to_lowercase())).count();
            let relevance = if query_terms.is_empty() { 0.5 }
                           else { term_hits as f64 / query_terms.len() as f64 };
            if relevance < self.min_relevance_score {
                score *= 0.4;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V2_LOW_RELEVANCE".into(),
                    message: format!("Low relevance score: {:.2}", relevance),
                    source_url: Some(src.url.clone()),
                });
            }
            scores.push(score * relevance.max(0.1));
        }

        let avg = if scores.is_empty() { 0.5 }
                  else { scores.iter().sum::<f64>() / scores.len() as f64 };
        LayerResult {
            layer: ValidationLayerId::V2Content, passed: avg >= 0.2,
            score: avg, issues, duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    fn is_error_page(text: &str) -> bool {
        let lower = text.to_lowercase();
        let patterns = ["404 not found", "403 forbidden", "access denied",
            "subscribe to continue", "sign in to view", "paywall",
            "enable javascript", "captcha", "too many requests"];
        patterns.iter().any(|p| lower.contains(p))
    }
}

#[derive(Debug, Clone)]
pub struct ContentInput {
    pub url: String,
    pub text: String,
}
```

**Step 2: Extraction quality V5 (`validate/extraction.rs`)**

```rust
use crate::validate::types::*;

pub struct ExtractionValidator;

impl ExtractionValidator {
    pub fn validate(sources: &[ExtractionInput]) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut scores = Vec::new();
        for src in sources {
            let mut score = 1.0;
            if src.truncated { score *= 0.7;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V5_TRUNCATED".into(),
                    message: "Content was truncated during extraction".into(),
                    source_url: Some(src.url.clone()),
                });
            }
            if src.segment_count == 0 { score *= 0.2;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Error, code: "V5_NO_SEGMENTS".into(),
                    message: "Extraction produced zero segments".into(),
                    source_url: Some(src.url.clone()),
                });
            }
            if src.encoding_errors > 0 { score *= 0.8; }
            scores.push(score);
        }
        let avg = if scores.is_empty() { 0.5 }
                  else { scores.iter().sum::<f64>() / scores.len() as f64 };
        LayerResult {
            layer: ValidationLayerId::V5ExtractionQuality, passed: avg >= 0.3,
            score: avg, issues, duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExtractionInput {
    pub url: String,
    pub truncated: bool,
    pub segment_count: usize,
    pub encoding_errors: usize,
}
```

**Step 3: Output integrity V6 (`validate/output.rs`)**

```rust
use crate::validate::types::*;

pub struct OutputValidator;

impl OutputValidator {
    /// V6: Verify citation links are valid and format is well-formed.
    pub fn validate(citations: &[CitationCheck], format_valid: bool) -> LayerResult {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let mut score = 1.0;
        let mut broken = 0usize;
        for c in citations {
            if !c.url_reachable {
                broken += 1;
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V6_BROKEN_CITATION".into(),
                    message: format!("Citation [{}] URL unreachable: {}", c.citation_id, c.url),
                    source_url: Some(c.url.clone()),
                });
            }
            if !c.content_hash_matches {
                issues.push(ValidationIssue {
                    severity: IssueSeverity::Warning, code: "V6_CONTENT_CHANGED".into(),
                    message: format!("Citation [{}] content changed since fetch", c.citation_id),
                    source_url: Some(c.url.clone()),
                });
            }
        }
        if !citations.is_empty() {
            let broken_ratio = broken as f64 / citations.len() as f64;
            score *= 1.0 - broken_ratio;
        }
        if !format_valid { score *= 0.5; }
        LayerResult {
            layer: ValidationLayerId::V6OutputIntegrity,
            passed: score >= 0.5, score, issues,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CitationCheck {
    pub citation_id: String,
    pub url: String,
    pub url_reachable: bool,
    pub content_hash_matches: bool,
}
```

**Acceptance criteria:**

- [ ] V2 detects paywall/error pages and flags low-relevance content
- [ ] V5 flags truncated extractions and zero-segment results
- [ ] V6 detects broken citation links and content-hash mismatches
- [ ] `cargo test` and `cargo clippy` pass

---

### P3-E1-T5: Confidence Calibration

**ID:** `P3-E1-T5`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 1-2 days

**Description:**
Build the confidence calibration module that aggregates scores from all 6 validation layers into a single calibrated confidence score per source, and an overall confidence for the result set. Uses weighted combination based on layer reliability.

**PRD References:**

- SS19 "6-Layer Validation" aggregate confidence
- SS42 Feature 10: "Consensus scoring"

**Files to create:** `crates/fetchium-core/src/validate/calibration.rs`

**Dependencies:** P3-E1-T1 through T4

**Step 1: Build calibration engine**

```rust
use crate::validate::types::*;

/// Weights for each validation layer when computing aggregate confidence.
pub struct CalibrationWeights {
    pub v1_source: f64,
    pub v2_content: f64,
    pub v3_freshness: f64,
    pub v4_cross_source: f64,
    pub v5_extraction: f64,
    pub v6_output: f64,
}

impl Default for CalibrationWeights {
    fn default() -> Self {
        Self {
            v1_source: 0.15, v2_content: 0.25, v3_freshness: 0.15,
            v4_cross_source: 0.25, v5_extraction: 0.10, v6_output: 0.10,
        }
    }
}

pub struct ConfidenceCalibrator {
    weights: CalibrationWeights,
}

impl Default for ConfidenceCalibrator {
    fn default() -> Self { Self { weights: CalibrationWeights::default() } }
}

impl ConfidenceCalibrator {
    /// Aggregate layer results into a single calibrated confidence score.
    pub fn calibrate(&self, layer_results: &[LayerResult]) -> f64 {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        for lr in layer_results {
            let w = self.weight_for(lr.layer);
            weighted_sum += lr.score * w;
            weight_total += w;
        }
        if weight_total == 0.0 { return 0.5; }
        (weighted_sum / weight_total).clamp(0.0, 1.0)
    }

    /// Build a complete ValidationResult from all layer results.
    pub fn build_result(
        &self,
        mode: ValidationMode,
        layer_results: Vec<LayerResult>,
        contradictions: Vec<Contradiction>,
        consensus: Vec<ClaimConsensus>,
    ) -> ValidationResult {
        let confidence = self.calibrate(&layer_results);
        let passed = layer_results.iter().all(|lr| lr.passed);
        ValidationResult {
            layers_run: layer_results.iter().map(|lr| lr.layer).collect(),
            layer_results, passed, confidence, contradictions, consensus, mode,
        }
    }

    fn weight_for(&self, layer: ValidationLayerId) -> f64 {
        match layer {
            ValidationLayerId::V1Source => self.weights.v1_source,
            ValidationLayerId::V2Content => self.weights.v2_content,
            ValidationLayerId::V3Freshness => self.weights.v3_freshness,
            ValidationLayerId::V4CrossSource => self.weights.v4_cross_source,
            ValidationLayerId::V5ExtractionQuality => self.weights.v5_extraction,
            ValidationLayerId::V6OutputIntegrity => self.weights.v6_output,
        }
    }
}
```

**Acceptance criteria:**

- [ ] Weighted average correctly combines layer scores
- [ ] All-pass layers yield confidence > 0.8; any critical failure drops confidence below 0.5
- [ ] `build_result` produces complete `ValidationResult` with all fields populated
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_perfect_layers() {
        let cal = ConfidenceCalibrator::default();
        let layers = vec![
            LayerResult { layer: ValidationLayerId::V1Source, passed: true,
                score: 1.0, issues: vec![], duration_ms: 1 },
            LayerResult { layer: ValidationLayerId::V2Content, passed: true,
                score: 1.0, issues: vec![], duration_ms: 1 },
        ];
        let conf = cal.calibrate(&layers);
        assert!(conf > 0.9);
    }
    #[test]
    fn failed_layer_drops_confidence() {
        let cal = ConfidenceCalibrator::default();
        let layers = vec![
            LayerResult { layer: ValidationLayerId::V4CrossSource, passed: false,
                score: 0.1, issues: vec![], duration_ms: 1 },
        ];
        let conf = cal.calibrate(&layers);
        assert!(conf < 0.2);
    }
}
```

---

### P3-E1-T6: RAR Retry-and-Refine Loop

**ID:** `P3-E1-T6`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Build the Reflection-Augmented Research (RAR) self-correction loop. After V4 validation, RAR evaluates retrieval quality at 5 reflection checkpoints (R1-R5) and auto-corrects by reformulating queries, fetching more sources, or flagging contradictions. The loop runs up to `max_reflection_loops` (configurable, default 3) before returning results.

**PRD References:**

- SS8.6 "RAR Research Loop" -- Full flow diagram with R1-R5 checkpoints
- SS8.6 "Reflection tokens: R1 [NEED_MORE], R2 [RELEVANT], R3 [SUFFICIENT], R4 [SUPPORTED], R5 [CONSISTENT]"
- SS19 "RAR Integration -- After V4, RAR reflection kicks in"

**Files to create:** `crates/fetchium-core/src/validate/rar.rs`

**Dependencies:** P3-E1-T1 (types), P3-E1-T1 (cross-source), P3-E1-T2 (temporal), P1-E5-T1 (BM25)

**Step 1: Define RAR types and reflection checkpoints**

```rust
use crate::validate::types::*;
use serde::{Deserialize, Serialize};

/// The 5 RAR reflection checkpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReflectionCheckpoint {
    /// R1: Are there enough relevant results?
    NeedMore,
    /// R2: Are the results relevant to the query?
    Relevant,
    /// R3: Does extracted content actually answer the query?
    Sufficient,
    /// R4: Does synthesis contain only source-supported claims?
    Supported,
    /// R5: Do sources agree with each other?
    Consistent,
}

/// Action the RAR loop decides to take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RarAction {
    /// Retrieval is good; proceed to next checkpoint.
    Proceed,
    /// Expand query and re-search (R1 action).
    ExpandQuery { new_query: String },
    /// Discard irrelevant results and reformulate (R2 action).
    ReformulateQuery { reason: String, new_query: String },
    /// Fetch additional pages for more evidence (R3 action).
    FetchMore { urls: Vec<String> },
    /// Remove unsupported claims from synthesis (R4 action).
    RemoveUnsupported { claim_ids: Vec<String> },
    /// Flag contradictions for the user (R5 action).
    FlagContradictions { contradictions: Vec<Contradiction> },
}

/// Result of one RAR loop iteration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RarIterationResult {
    pub iteration: usize,
    pub checkpoint: ReflectionCheckpoint,
    pub action: RarAction,
    pub quality_before: f64,
    pub quality_after: f64,
}

/// Configuration for the RAR loop.
#[derive(Debug, Clone)]
pub struct RarConfig {
    pub max_loops: usize,
    pub min_relevant_ratio: f64,
    pub min_sufficiency_score: f64,
    pub min_consistency_score: f64,
    pub min_results: usize,
}

impl Default for RarConfig {
    fn default() -> Self {
        Self {
            max_loops: 3,
            min_relevant_ratio: 0.5,
            min_sufficiency_score: 0.4,
            min_consistency_score: 0.5,
            min_results: 3,
        }
    }
}
```

**Step 2: Build the RAR engine**

```rust
/// The RAR engine evaluates retrieval quality and triggers corrective actions.
pub struct RarEngine {
    config: RarConfig,
}

impl Default for RarEngine {
    fn default() -> Self { Self { config: RarConfig::default() } }
}

impl RarEngine {
    pub fn new(config: RarConfig) -> Self { Self { config } }

    /// Run the full RAR reflection loop. Returns the actions taken per iteration.
    ///
    /// The caller is responsible for executing actions (re-search, re-fetch)
    /// and calling this again with updated state. This function evaluates ONE
    /// iteration and returns the recommended action.
    pub fn evaluate(
        &self,
        state: &RarState,
        iteration: usize,
    ) -> RarIterationResult {
        if iteration >= self.config.max_loops {
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::Consistent,
                action: RarAction::Proceed,
                quality_before: state.overall_quality(),
                quality_after: state.overall_quality(),
            };
        }

        // R1: Need more results?
        if state.total_results < self.config.min_results {
            let new_query = Self::expand_query(&state.query);
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::NeedMore,
                action: RarAction::ExpandQuery { new_query },
                quality_before: state.overall_quality(),
                quality_after: state.overall_quality(),
            };
        }

        // R2: Are results relevant?
        let relevant_ratio = state.relevant_count as f64 / state.total_results.max(1) as f64;
        if relevant_ratio < self.config.min_relevant_ratio {
            let new_query = Self::reformulate(&state.query, &state.low_relevance_terms);
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::Relevant,
                action: RarAction::ReformulateQuery {
                    reason: format!("Only {:.0}% relevant (need {:.0}%)",
                        relevant_ratio * 100.0, self.config.min_relevant_ratio * 100.0),
                    new_query,
                },
                quality_before: relevant_ratio,
                quality_after: relevant_ratio,
            };
        }

        // R3: Is content sufficient to answer the query?
        if state.sufficiency_score < self.config.min_sufficiency_score {
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::Sufficient,
                action: RarAction::FetchMore { urls: state.candidate_urls.clone() },
                quality_before: state.sufficiency_score,
                quality_after: state.sufficiency_score,
            };
        }

        // R4: Are synthesis claims supported?
        if !state.unsupported_claims.is_empty() {
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::Supported,
                action: RarAction::RemoveUnsupported {
                    claim_ids: state.unsupported_claims.clone(),
                },
                quality_before: state.support_ratio,
                quality_after: 1.0, // after removal, all remaining claims are supported
            };
        }

        // R5: Are sources consistent?
        if state.consistency_score < self.config.min_consistency_score {
            return RarIterationResult {
                iteration, checkpoint: ReflectionCheckpoint::Consistent,
                action: RarAction::FlagContradictions {
                    contradictions: state.contradictions.clone(),
                },
                quality_before: state.consistency_score,
                quality_after: state.consistency_score,
            };
        }

        // All checks pass
        RarIterationResult {
            iteration, checkpoint: ReflectionCheckpoint::Consistent,
            action: RarAction::Proceed,
            quality_before: state.overall_quality(),
            quality_after: state.overall_quality(),
        }
    }

    /// Simple query expansion: add synonyms or broaden terms.
    fn expand_query(query: &str) -> String {
        // In production, this would use a synonym dictionary or LLM.
        // For now, append "overview" to broaden results.
        format!("{query} overview")
    }

    /// Reformulate query by removing low-relevance terms and refocusing.
    fn reformulate(query: &str, _low_terms: &[String]) -> String {
        // In production, use BM25 IDF analysis to identify non-discriminative terms.
        // For now, simple heuristic: take first N significant words.
        let words: Vec<&str> = query.split_whitespace()
            .filter(|w| w.len() > 3)
            .take(5)
            .collect();
        if words.is_empty() { query.to_string() }
        else { words.join(" ") }
    }
}

/// State snapshot passed to the RAR engine for evaluation.
#[derive(Debug, Clone)]
pub struct RarState {
    pub query: String,
    pub total_results: usize,
    pub relevant_count: usize,
    pub sufficiency_score: f64,
    pub support_ratio: f64,
    pub consistency_score: f64,
    pub unsupported_claims: Vec<String>,
    pub contradictions: Vec<Contradiction>,
    pub candidate_urls: Vec<String>,
    pub low_relevance_terms: Vec<String>,
}

impl RarState {
    pub fn overall_quality(&self) -> f64 {
        let relevance = self.relevant_count as f64 / self.total_results.max(1) as f64;
        (relevance + self.sufficiency_score + self.consistency_score) / 3.0
    }
}
```

**Acceptance criteria:**

- [ ] R1 triggers `ExpandQuery` when `total_results < min_results`
- [ ] R2 triggers `ReformulateQuery` when `relevant_ratio < 0.5`
- [ ] R3 triggers `FetchMore` when `sufficiency_score < 0.4`
- [ ] R4 triggers `RemoveUnsupported` when unsupported claims exist
- [ ] R5 triggers `FlagContradictions` when `consistency_score < 0.5`
- [ ] All checks pass returns `Proceed`
- [ ] Loop respects `max_loops` limit
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn good_state() -> RarState {
        RarState {
            query: "what is Rust".into(), total_results: 10, relevant_count: 8,
            sufficiency_score: 0.8, support_ratio: 1.0, consistency_score: 0.9,
            unsupported_claims: vec![], contradictions: vec![],
            candidate_urls: vec![], low_relevance_terms: vec![],
        }
    }

    #[test]
    fn good_state_proceeds() {
        let engine = RarEngine::default();
        let r = engine.evaluate(&good_state(), 0);
        assert!(matches!(r.action, RarAction::Proceed));
    }

    #[test]
    fn insufficient_results_expands() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.total_results = 1; state.relevant_count = 1;
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::ExpandQuery { .. }));
        assert_eq!(r.checkpoint, ReflectionCheckpoint::NeedMore);
    }

    #[test]
    fn low_relevance_reformulates() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.total_results = 10; state.relevant_count = 2;
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::ReformulateQuery { .. }));
    }

    #[test]
    fn max_loops_respected() {
        let engine = RarEngine::new(RarConfig { max_loops: 2, ..Default::default() });
        let mut state = good_state();
        state.total_results = 1;
        let r = engine.evaluate(&state, 2);
        assert!(matches!(r.action, RarAction::Proceed));
    }

    #[test]
    fn unsupported_claims_removed() {
        let engine = RarEngine::default();
        let mut state = good_state();
        state.unsupported_claims = vec!["claim_1".into()];
        let r = engine.evaluate(&state, 0);
        assert!(matches!(r.action, RarAction::RemoveUnsupported { .. }));
    }
}
```

---

## Epic 3.2: Citation System + Evidence Graph Protocol

> **PRD Sections:** SS24 (Citation & Evidence System), SS8.7 (Evidence Graph Protocol)
> **Crate:** `fetchium-core` -- `src/citation/`
> **Priority:** P1 | **Tasks:** 3

### P3-E2-T1: Citation Formatters (6 styles)

**ID:** `P3-E2-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 3 days

**Description:**
Implement 6 citation formatters: APA, MLA, Chicago, IEEE, BibTeX, and inline `[N]`. Each formatter takes source metadata (author, title, URL, date, publisher) and produces a correctly formatted citation string. The system supports both inline markers (inserted into text) and a reference list appended to output.

**PRD References:**

- SS24 "6 styles: inline [1] | footnote ^1 | apa (Author, Year) | ieee [1] | chicago | bibtex"
- SS42 Feature 134: "6 citation styles (inline, footnote, APA, IEEE, Chicago, BibTeX)"

**Files to create/modify:**

```
crates/fetchium-core/src/citation/mod.rs       -- Module root
crates/fetchium-core/src/citation/types.rs     -- Citation types
crates/fetchium-core/src/citation/formatter.rs -- 6 formatters
```

**Dependencies:** P0-E1-T2 (Types), P1-E7-T1 (Output formatters)

**Step 1: Define citation types (`citation/types.rs`)**

```rust
use serde::{Deserialize, Serialize};

/// Which citation style to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CitationStyle {
    #[default]
    Inline,   // [1], [2], ...
    Footnote, // ^1, ^2, ...
    Apa,      // (Author, Year)
    Mla,      // Author. "Title." Site, Date, URL.
    Chicago,  // Author. "Title." Site. Date. URL.
    Ieee,     // [1] Author, "Title," Site, Date.
    Bibtex,   // @article{key, ...}
}

/// Metadata about a source needed for citation formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMeta {
    pub url: String,
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,  // "YYYY-MM-DD" or "YYYY"
    pub accessed_date: String,           // "YYYY-MM-DD"
}

/// A formatted citation with both inline marker and full reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedCitation {
    /// The inline marker to insert in text, e.g., "[1]" or "(Smith, 2024)".
    pub inline_marker: String,
    /// The full reference entry for the reference list.
    pub reference_entry: String,
    /// The source URL.
    pub url: String,
    /// 1-based index in the reference list.
    pub index: usize,
}
```

**Step 2: Build formatters (`citation/formatter.rs`)**

```rust
use crate::citation::types::*;

pub struct CitationFormatter {
    style: CitationStyle,
}

impl CitationFormatter {
    pub fn new(style: CitationStyle) -> Self { Self { style } }

    /// Format a single source as a citation.
    pub fn format(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        match self.style {
            CitationStyle::Inline => self.format_inline(source, index),
            CitationStyle::Footnote => self.format_footnote(source, index),
            CitationStyle::Apa => self.format_apa(source, index),
            CitationStyle::Mla => self.format_mla(source, index),
            CitationStyle::Chicago => self.format_chicago(source, index),
            CitationStyle::Ieee => self.format_ieee(source, index),
            CitationStyle::Bibtex => self.format_bibtex(source, index),
        }
    }

    /// Format a list of sources into a complete reference section.
    pub fn format_references(&self, sources: &[SourceMeta]) -> String {
        sources.iter().enumerate().map(|(i, s)| {
            let fc = self.format(s, i + 1);
            fc.reference_entry
        }).collect::<Vec<_>>().join("\n")
    }

    fn format_inline(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        FormattedCitation {
            inline_marker: format!("[{index}]"),
            reference_entry: format!("[{index}] {} - {}", source.title, source.url),
            url: source.url.clone(), index,
        }
    }

    fn format_footnote(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        FormattedCitation {
            inline_marker: format!("^{index}"),
            reference_entry: format!("^{index}. {} ({})", source.title, source.url),
            url: source.url.clone(), index,
        }
    }

    fn format_apa(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let year = Self::extract_year(&source.published_date).unwrap_or("n.d.".into());
        let publisher = source.publisher.as_deref().unwrap_or("");
        FormattedCitation {
            inline_marker: format!("({author}, {year})"),
            reference_entry: format!(
                "{author} ({year}). {title}. {pub_}Retrieved {accessed} from {url}",
                author = author, year = year, title = source.title,
                pub_ = if publisher.is_empty() { String::new() }
                       else { format!("{publisher}. ") },
                accessed = source.accessed_date, url = source.url,
            ),
            url: source.url.clone(), index,
        }
    }

    fn format_mla(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("Web");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        FormattedCitation {
            inline_marker: format!("({author})"),
            reference_entry: format!(
                "{author}. \"{title}.\" *{publisher}*, {date}, {url}.",
                author = author, title = source.title, publisher = publisher,
                date = date, url = source.url,
            ),
            url: source.url.clone(), index,
        }
    }

    fn format_chicago(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        FormattedCitation {
            inline_marker: format!("({author} {date})"),
            reference_entry: format!(
                "{author}. \"{title}.\" {pub_}{date}. {url}.",
                author = author, title = source.title,
                pub_ = if publisher.is_empty() { String::new() }
                       else { format!("{publisher}. ") },
                date = date, url = source.url,
            ),
            url: source.url.clone(), index,
        }
    }

    fn format_ieee(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let author = source.author.as_deref().unwrap_or("Unknown");
        let publisher = source.publisher.as_deref().unwrap_or("Online");
        let date = source.published_date.as_deref().unwrap_or("n.d.");
        FormattedCitation {
            inline_marker: format!("[{index}]"),
            reference_entry: format!(
                "[{index}] {author}, \"{title},\" {publisher}, {date}. [Online]. Available: {url}",
                index = index, author = author, title = source.title,
                publisher = publisher, date = date, url = source.url,
            ),
            url: source.url.clone(), index,
        }
    }

    fn format_bibtex(&self, source: &SourceMeta, index: usize) -> FormattedCitation {
        let key = Self::bibtex_key(source);
        let author = source.author.as_deref().unwrap_or("Unknown");
        let year = Self::extract_year(&source.published_date).unwrap_or("0000".into());
        FormattedCitation {
            inline_marker: format!("\\cite{{{key}}}"),
            reference_entry: format!(
                "@misc{{{key},\n  author = {{{author}}},\n  title = {{{title}}},\n  year = {{{year}}},\n  url = {{{url}}},\n  note = {{Accessed: {accessed}}}\n}}",
                key = key, author = author, title = source.title,
                year = year, url = source.url, accessed = source.accessed_date,
            ),
            url: source.url.clone(), index,
        }
    }

    fn extract_year(date: &Option<String>) -> Option<String> {
        date.as_ref().and_then(|d| d.get(..4).map(|s| s.to_string()))
    }

    fn bibtex_key(source: &SourceMeta) -> String {
        let author_part = source.author.as_deref().unwrap_or("unknown")
            .split_whitespace().next().unwrap_or("unknown").to_lowercase();
        let year = Self::extract_year(&source.published_date).unwrap_or("0000".into());
        let title_word = source.title.split_whitespace().next()
            .unwrap_or("untitled").to_lowercase();
        format!("{author_part}{year}{title_word}")
    }
}
```

**Acceptance criteria:**

- [ ] All 6 styles produce correctly formatted inline markers and reference entries
- [ ] APA format: `(Author, Year)` inline, full reference with "Retrieved from"
- [ ] MLA format: `(Author)` inline, italicized publisher
- [ ] IEEE format: `[N]` inline, numbered reference with "Available:"
- [ ] BibTeX format: `\cite{key}` inline, valid `@misc{}` entry
- [ ] Handles missing author/date/publisher gracefully with defaults
- [ ] `format_references()` produces complete reference section
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> SourceMeta {
        SourceMeta {
            url: "https://example.com/article".into(),
            title: "Understanding Rust".into(),
            author: Some("Smith, J.".into()),
            publisher: Some("Tech Blog".into()),
            published_date: Some("2025-03-15".into()),
            accessed_date: "2026-02-23".into(),
        }
    }
    #[test]
    fn inline_format() {
        let f = CitationFormatter::new(CitationStyle::Inline);
        let c = f.format(&sample(), 1);
        assert_eq!(c.inline_marker, "[1]");
        assert!(c.reference_entry.contains("Understanding Rust"));
    }
    #[test]
    fn apa_format() {
        let f = CitationFormatter::new(CitationStyle::Apa);
        let c = f.format(&sample(), 1);
        assert_eq!(c.inline_marker, "(Smith, J., 2025)");
        assert!(c.reference_entry.contains("Retrieved"));
    }
    #[test]
    fn bibtex_format() {
        let f = CitationFormatter::new(CitationStyle::Bibtex);
        let c = f.format(&sample(), 1);
        assert!(c.inline_marker.starts_with("\\cite{"));
        assert!(c.reference_entry.contains("@misc{"));
        assert!(c.reference_entry.contains("year = {2025}"));
    }
    #[test]
    fn missing_author_fallback() {
        let f = CitationFormatter::new(CitationStyle::Apa);
        let mut s = sample(); s.author = None;
        let c = f.format(&s, 1);
        assert!(c.inline_marker.contains("Unknown"));
    }
}
```

---

### P3-E2-T2: Evidence Graph Protocol (EGP)

**ID:** `P3-E2-T2`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Implement the Evidence Graph Protocol -- a graph-based evidence linking system with claim provenance, SHA-256 content hashes, confidence scoring per claim, and contradiction edges. The graph is serializable to JSON and can be exported alongside research reports.

**PRD References:**

- SS8.7 Full EGP specification: `EvidenceGraph`, `EvidenceNode`, `EvidenceEdge` types
- SS24 "EGP: Graph-based evidence linking with claim->source edges, SHA-256 hashes, confidence scoring, contradiction edges"
- SS43 `EvidenceLink` with `quote_hash: String // SHA-256`

**Files to create:**

```
crates/fetchium-core/src/citation/evidence_graph.rs -- EGP implementation
```

**Dependencies:** P3-E2-T1 (citation types), P3-E1-T1 (validation types for Contradiction)

**Step 1: Define EGP types and builder**

```rust
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// The complete evidence graph for a research query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGraph {
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
    pub root_claim: String,
    pub overall_confidence: f64,
    pub content_hashes: HashMap<String, String>, // url -> SHA-256
}

/// A node in the evidence graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: NodeType,
    pub content: String,
    pub confidence: f64,
    pub timestamp: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType { Claim, Source, Fact, Inference }

/// An edge linking a source/fact to a claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    pub quote: String,
    pub quote_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType { Supports, Contradicts, PartiallySupports, InferredFrom }

/// Builder for constructing evidence graphs incrementally.
pub struct EvidenceGraphBuilder {
    nodes: Vec<EvidenceNode>,
    edges: Vec<EvidenceEdge>,
    content_hashes: HashMap<String, String>,
    root_claim: String,
    node_counter: usize,
}

impl EvidenceGraphBuilder {
    pub fn new(root_claim: &str) -> Self {
        let mut builder = Self {
            nodes: Vec::new(), edges: Vec::new(),
            content_hashes: HashMap::new(),
            root_claim: root_claim.to_string(), node_counter: 0,
        };
        builder.add_node(NodeType::Claim, root_claim, 0.0, None);
        builder
    }

    /// Add a node and return its ID.
    pub fn add_node(
        &mut self, node_type: NodeType, content: &str,
        confidence: f64, source_url: Option<&str>,
    ) -> String {
        self.node_counter += 1;
        let id = format!("n{}", self.node_counter);
        self.nodes.push(EvidenceNode {
            id: id.clone(), node_type, content: content.to_string(),
            confidence, timestamp: chrono::Utc::now().to_rfc3339(),
            source_url: source_url.map(|s| s.to_string()),
        });
        id
    }

    /// Add a source node and register its content hash.
    pub fn add_source(&mut self, url: &str, title: &str, content: &str, confidence: f64) -> String {
        let hash = Self::sha256(content);
        self.content_hashes.insert(url.to_string(), hash);
        self.add_node(NodeType::Source, title, confidence, Some(url))
    }

    /// Add an evidence edge with a supporting quote.
    pub fn add_edge(
        &mut self, from: &str, to: &str,
        edge_type: EdgeType, quote: &str,
    ) {
        let quote_hash = Self::sha256(quote);
        self.edges.push(EvidenceEdge {
            from: from.to_string(), to: to.to_string(),
            edge_type, quote: quote.to_string(), quote_hash,
        });
    }

    /// Add a fact extracted from a source and link it.
    pub fn add_fact_from_source(
        &mut self, source_id: &str, fact_text: &str,
        supporting_quote: &str, confidence: f64,
    ) -> String {
        let fact_id = self.add_node(NodeType::Fact, fact_text, confidence, None);
        self.add_edge(source_id, &fact_id, EdgeType::Supports, supporting_quote);
        fact_id
    }

    /// Link a fact to the root claim.
    pub fn link_to_root(&mut self, fact_id: &str, edge_type: EdgeType, quote: &str) {
        self.add_edge(fact_id, "n1", edge_type, quote); // n1 is always the root claim
    }

    /// Build the final evidence graph with computed overall confidence.
    pub fn build(self) -> EvidenceGraph {
        let confidence = self.compute_overall_confidence();
        EvidenceGraph {
            nodes: self.nodes, edges: self.edges,
            root_claim: self.root_claim, overall_confidence: confidence,
            content_hashes: self.content_hashes,
        }
    }

    fn compute_overall_confidence(&self) -> f64 {
        let support_count = self.edges.iter()
            .filter(|e| e.edge_type == EdgeType::Supports).count();
        let contradict_count = self.edges.iter()
            .filter(|e| e.edge_type == EdgeType::Contradicts).count();
        let total = (support_count + contradict_count).max(1);
        support_count as f64 / total as f64
    }

    fn sha256(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(hasher.finalize())
    }
}
```

**Step 2: Add verification method**

```rust
impl EvidenceGraph {
    /// Verify that a quote's hash matches the stored hash, proving
    /// the source contained that exact text at fetch time.
    pub fn verify_quote(&self, edge_index: usize) -> bool {
        if let Some(edge) = self.edges.get(edge_index) {
            let computed = {
                let mut h = Sha256::new();
                h.update(edge.quote.as_bytes());
                hex::encode(h.finalize())
            };
            computed == edge.quote_hash
        } else {
            false
        }
    }

    /// Verify a source's content hash against newly fetched content.
    pub fn verify_source(&self, url: &str, current_content: &str) -> bool {
        if let Some(stored_hash) = self.content_hashes.get(url) {
            let mut h = Sha256::new();
            h.update(current_content.as_bytes());
            let current_hash = hex::encode(h.finalize());
            &current_hash == stored_hash
        } else {
            false
        }
    }

    /// Trace the evidence chain for the root claim: all paths from sources to root.
    pub fn trace_root_evidence(&self) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        // Find all edges pointing to n1 (root)
        for edge in &self.edges {
            if edge.to == "n1" {
                let mut path = vec![edge.from.clone(), "n1".to_string()];
                // Check if there's a source behind this fact
                for inner_edge in &self.edges {
                    if inner_edge.to == edge.from {
                        path.insert(0, inner_edge.from.clone());
                    }
                }
                paths.push(path);
            }
        }
        paths
    }
}
```

**Acceptance criteria:**

- [ ] `EvidenceGraphBuilder::new()` creates root claim node as `n1`
- [ ] `add_source()` registers SHA-256 content hash in `content_hashes` map
- [ ] `add_edge()` computes SHA-256 of the supporting quote
- [ ] `verify_quote()` returns `true` for unmodified quotes, `false` for tampered
- [ ] `verify_source()` returns `true` when content unchanged, `false` when modified
- [ ] `trace_root_evidence()` returns all source->fact->claim paths
- [ ] `overall_confidence` computed as `supports / (supports + contradicts)`
- [ ] Full graph serializes to valid JSON via serde
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn build_and_verify_graph() {
        let mut b = EvidenceGraphBuilder::new("Rust is memory-safe");
        let s1 = b.add_source("https://rust-lang.org", "Rust Lang", "Rust guarantees memory safety without GC", 0.95);
        let f1 = b.add_fact_from_source(&s1, "Rust uses ownership for memory safety",
            "Rust guarantees memory safety without GC", 0.9);
        b.link_to_root(&f1, EdgeType::Supports, "memory safety without GC");
        let graph = b.build();
        assert_eq!(graph.nodes.len(), 3); // root + source + fact
        assert_eq!(graph.edges.len(), 2); // source->fact, fact->root
        assert!(graph.overall_confidence > 0.9);
        assert!(graph.verify_quote(0));
        assert!(graph.verify_source("https://rust-lang.org", "Rust guarantees memory safety without GC"));
        assert!(!graph.verify_source("https://rust-lang.org", "tampered content"));
    }
    #[test]
    fn trace_evidence_chain() {
        let mut b = EvidenceGraphBuilder::new("claim");
        let s = b.add_source("https://a.com", "A", "content", 0.8);
        let f = b.add_fact_from_source(&s, "fact", "content", 0.8);
        b.link_to_root(&f, EdgeType::Supports, "evidence");
        let graph = b.build();
        let paths = graph.trace_root_evidence();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].len(), 3); // source -> fact -> root
    }
    #[test]
    fn serialization_roundtrip() {
        let mut b = EvidenceGraphBuilder::new("test claim");
        b.add_source("https://x.com", "X", "data", 0.7);
        let graph = b.build();
        let json = serde_json::to_string(&graph).unwrap();
        let parsed: EvidenceGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.root_claim, "test claim");
    }
}
```

---

### P3-E2-T3: Evidence Chain Tracking

**ID:** `P3-E2-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2 days

**Description:**
Build the evidence chain tracker that connects citations to EGP nodes, enabling strict evidence mode where every factual claim must cite a source. Uncitable claims are marked `[unverified]`. Integrates citations from P3-E2-T1 with the evidence graph from P3-E2-T2.

**PRD References:**

- SS24 "Strict Evidence Mode -- Every factual statement must cite a source. Uncitable claims marked [unverified]."

**Files to create:** `crates/fetchium-core/src/citation/evidence_tracker.rs`

**Dependencies:** P3-E2-T1 (citations), P3-E2-T2 (EGP)

**Step 1: Build the evidence tracker**

```rust
use crate::citation::types::*;
use crate::citation::evidence_graph::*;
use crate::citation::formatter::CitationFormatter;

/// Result of evidence chain analysis on a piece of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAnalysis {
    /// Claims that have supporting evidence with citations.
    pub cited_claims: Vec<CitedClaim>,
    /// Claims that could not be linked to any source.
    pub unverified_claims: Vec<String>,
    /// The text with citation markers injected.
    pub annotated_text: String,
    /// Whether strict evidence mode passes (all claims cited).
    pub strict_mode_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitedClaim {
    pub claim_text: String,
    pub citation: FormattedCitation,
    pub evidence_node_id: Option<String>,
    pub confidence: f64,
}

pub struct EvidenceTracker {
    formatter: CitationFormatter,
}

impl EvidenceTracker {
    pub fn new(style: CitationStyle) -> Self {
        Self { formatter: CitationFormatter::new(style) }
    }

    /// Analyze text against an evidence graph and inject citations.
    /// In strict mode, marks uncitable claims as [unverified].
    pub fn analyze(
        &self,
        text: &str,
        graph: &EvidenceGraph,
        sources: &[SourceMeta],
        strict: bool,
    ) -> EvidenceAnalysis {
        let sentences: Vec<&str> = text.split(". ")
            .flat_map(|s| s.split(".\n"))
            .collect();

        let mut cited = Vec::new();
        let mut unverified = Vec::new();
        let mut annotated_parts = Vec::new();

        for sentence in &sentences {
            let trimmed = sentence.trim();
            if trimmed.is_empty() { continue; }

            // Try to match this sentence to an evidence node
            if let Some((node, source_idx)) = self.find_supporting_evidence(trimmed, graph, sources) {
                let citation = self.formatter.format(&sources[source_idx], source_idx + 1);
                annotated_parts.push(format!("{} {}", trimmed, citation.inline_marker));
                cited.push(CitedClaim {
                    claim_text: trimmed.to_string(), citation,
                    evidence_node_id: Some(node.id.clone()), confidence: node.confidence,
                });
            } else if strict && Self::is_factual_claim(trimmed) {
                annotated_parts.push(format!("{} [unverified]", trimmed));
                unverified.push(trimmed.to_string());
            } else {
                annotated_parts.push(trimmed.to_string());
            }
        }

        let strict_passed = unverified.is_empty();
        EvidenceAnalysis {
            cited_claims: cited, unverified_claims: unverified,
            annotated_text: annotated_parts.join(". "),
            strict_mode_passed: strict_passed,
        }
    }

    fn find_supporting_evidence<'a>(
        &self, sentence: &str, graph: &'a EvidenceGraph, sources: &[SourceMeta],
    ) -> Option<(&'a EvidenceNode, usize)> {
        let lower = sentence.to_lowercase();
        for node in &graph.nodes {
            if node.node_type == NodeType::Fact || node.node_type == NodeType::Source {
                let node_lower = node.content.to_lowercase();
                // Simple word-overlap check for matching
                let overlap = Self::word_overlap(&lower, &node_lower);
                if overlap > 0.3 {
                    // Find which source this node belongs to
                    let source_idx = node.source_url.as_ref().and_then(|url| {
                        sources.iter().position(|s| &s.url == url)
                    }).unwrap_or(0);
                    return Some((node, source_idx));
                }
            }
        }
        None
    }

    fn word_overlap(a: &str, b: &str) -> f64 {
        let wa: std::collections::HashSet<&str> = a.split_whitespace().collect();
        let wb: std::collections::HashSet<&str> = b.split_whitespace().collect();
        let inter = wa.intersection(&wb).count();
        let union = wa.union(&wb).count();
        if union == 0 { 0.0 } else { inter as f64 / union as f64 }
    }

    /// Heuristic: is this sentence a factual claim (vs. opinion/transition)?
    fn is_factual_claim(text: &str) -> bool {
        let lower = text.to_lowercase();
        // Skip short sentences, questions, and obvious transitions
        if text.len() < 20 { return false; }
        if text.ends_with('?') { return false; }
        let transition_words = ["however", "therefore", "in conclusion", "overall",
            "additionally", "furthermore", "in summary"];
        if transition_words.iter().any(|t| lower.starts_with(t)) { return false; }
        true
    }
}
```

**Acceptance criteria:**

- [ ] Claims matching evidence graph nodes get citation markers injected
- [ ] Strict mode marks unmatched factual claims as `[unverified]`
- [ ] Non-factual sentences (questions, transitions) are not flagged in strict mode
- [ ] `strict_mode_passed` is `true` only when all factual claims are cited
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_mode_flags_unverified() {
        let mut b = EvidenceGraphBuilder::new("root");
        b.add_source("https://a.com", "A", "Rust is memory safe", 0.9);
        let graph = b.build();
        let sources = vec![SourceMeta {
            url: "https://a.com".into(), title: "A".into(),
            author: None, publisher: None,
            published_date: None, accessed_date: "2026-02-23".into(),
        }];
        let tracker = EvidenceTracker::new(CitationStyle::Inline);
        let analysis = tracker.analyze(
            "Rust is memory safe. Python is dynamically typed.",
            &graph, &sources, true,
        );
        assert!(!analysis.strict_mode_passed);
        assert!(analysis.annotated_text.contains("[unverified]"));
    }
}
```

---

## Epic 3.3: Research Mode + Agent Research

> **PRD Sections:** SS10 Mode B (Research Mode), SS9 (Agent Architecture), SS11 (CLI)
> **Crate:** `fetchium-core` -- `src/research/`, `fetchium-cli` -- `src/commands/`
> **Priority:** P1 | **Tasks:** 3

### P3-E3-T1: Multi-Source Research Pipeline

**ID:** `P3-E3-T1`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 4-5 days

**Description:**
Build the research pipeline orchestrator that chains together: query decomposition, parallel multi-backend search, CEP extraction, QATBE budgeting, RAR reflection loop, HyperFusion ranking, cross-source validation, EGP evidence mapping, and citation injection. This is the core engine behind both `fetchium research` and `fetchium agent-research`.

**PRD References:**

- SS10 Mode B behavior steps 1-9
- SS8.6 RAR integration in research pipeline

**Files to create:**

```
crates/fetchium-core/src/research/mod.rs       -- Module root
crates/fetchium-core/src/research/pipeline.rs  -- Research pipeline orchestrator
crates/fetchium-core/src/research/decompose.rs -- Query decomposition
```

**Dependencies:** All P3-E1 (validation + RAR), all P3-E2 (citations + EGP), P2-E3 (search orchestrator), P2-E4 (HyperFusion)

**Step 1: Define research types**

```rust
use serde::{Deserialize, Serialize};
use crate::validate::types::*;
use crate::citation::types::*;
use crate::citation::evidence_graph::EvidenceGraph;

/// Configuration for a research pipeline run.
#[derive(Debug, Clone)]
pub struct ResearchConfig {
    pub query: String,
    pub max_sources: usize,
    pub token_budget: Option<usize>,
    pub citation_style: CitationStyle,
    pub validation_mode: ValidationMode,
    pub strict_evidence: bool,
    pub evidence_graph: bool,
    pub max_rar_loops: usize,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(), max_sources: 10, token_budget: None,
            citation_style: CitationStyle::Inline, validation_mode: ValidationMode::Standard,
            strict_evidence: false, evidence_graph: false, max_rar_loops: 3,
        }
    }
}

/// The output of a research pipeline run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    pub query: String,
    pub sub_queries: Vec<String>,
    pub synthesis: String,
    pub sources: Vec<SourceMeta>,
    pub citations: Vec<FormattedCitation>,
    pub reference_section: String,
    pub validation: ValidationResult,
    pub evidence_graph: Option<EvidenceGraph>,
    pub rar_iterations: Vec<crate::validate::rar::RarIterationResult>,
    pub meta: ResearchMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMeta {
    pub duration_ms: u64,
    pub sources_fetched: usize,
    pub sources_validated: usize,
    pub validation_pass_rate: f64,
    pub overall_confidence: f64,
    pub rar_loops_executed: usize,
}
```

**Step 2: Query decomposition (`research/decompose.rs`)**

```rust
/// Decompose a complex query into parallel sub-questions.
pub fn decompose_query(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let mut sub_queries = Vec::new();

    // Pattern: "compare X vs Y vs Z" -> individual queries per item
    if lower.contains(" vs ") || lower.contains("compare") {
        let items = extract_comparison_items(query);
        if items.len() >= 2 {
            for item in &items {
                sub_queries.push(format!("{item} features overview"));
            }
            sub_queries.push(query.to_string()); // also search the original
            return sub_queries;
        }
    }

    // Pattern: "X implications for Y" -> query about X, query about Y, original
    if lower.contains("implications") || lower.contains("impact") {
        sub_queries.push(query.to_string());
        // Extract topic and context
        if let Some(pos) = lower.find(" for ") {
            let topic = &query[..pos];
            let context = &query[pos + 5..];
            sub_queries.push(topic.trim().to_string());
            sub_queries.push(context.trim().to_string());
        }
        return sub_queries;
    }

    // Default: return the original query as-is
    vec![query.to_string()]
}

fn extract_comparison_items(query: &str) -> Vec<String> {
    let cleaned = query.to_lowercase()
        .replace("compare ", "").replace("comparison of ", "")
        .replace(" and ", " vs ");
    cleaned.split(" vs ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

**Step 3: Research pipeline orchestrator (`research/pipeline.rs`)**

```rust
use crate::research::{ResearchConfig, ResearchReport, ResearchMeta};
use crate::research::decompose::decompose_query;
use crate::validate::rar::{RarEngine, RarState};
use crate::validate::calibration::ConfidenceCalibrator;
use crate::validate::cross_source::CrossSourceVerifier;
use crate::citation::formatter::CitationFormatter;
use crate::citation::evidence_graph::EvidenceGraphBuilder;
use crate::citation::evidence_tracker::EvidenceTracker;
use crate::error::HsxError;

pub struct ResearchPipeline;

impl ResearchPipeline {
    /// Execute the full research pipeline.
    ///
    /// Steps (matching PRD SS10 Mode B):
    /// 1. Query decomposition
    /// 2. Parallel multi-backend search (via SearchOrchestrator)
    /// 3. Top sources fetched via CEP
    /// 4. Content extracted via QATBE
    /// 5. RAR reflection loop validates retrieval quality
    /// 6. HyperFusion ranking
    /// 7. Evidence mapping via EGP
    /// 8. Synthesis with strict citation
    /// 9. Validation layer (6-layer)
    pub async fn execute(
        config: &ResearchConfig,
        // In production these would be injected dependencies:
        // orchestrator: &SearchOrchestrator,
        // extractor: &QatbeExtractor,
        // ranker: &HyperFusionRanker,
    ) -> Result<ResearchReport, HsxError> {
        let start = std::time::Instant::now();

        // Step 1: Decompose query
        let sub_queries = decompose_query(&config.query);

        // Step 2-4: Search + fetch + extract (delegated to orchestrator)
        // In production: parallel search across sub_queries, CEP extraction, QATBE budgeting
        // Here we define the pipeline structure; actual I/O is injected.

        // Step 5: RAR reflection loop
        let rar_engine = RarEngine::default();
        let mut rar_iterations = Vec::new();
        // RAR would be called in a loop here, up to max_rar_loops

        // Step 6: HyperFusion ranking (delegated to ranker)

        // Step 7: Evidence mapping
        let egp = if config.evidence_graph {
            let builder = EvidenceGraphBuilder::new(&config.query);
            // In production: populate from extracted sources
            Some(builder.build())
        } else {
            None
        };

        // Step 8: Citation injection
        let formatter = CitationFormatter::new(config.citation_style);

        // Step 9: Validation
        let calibrator = ConfidenceCalibrator::default();

        let duration_ms = start.elapsed().as_millis() as u64;

        // Assemble report (structure shown; real data comes from pipeline execution)
        Ok(ResearchReport {
            query: config.query.clone(),
            sub_queries,
            synthesis: String::new(), // populated by actual pipeline
            sources: Vec::new(),
            citations: Vec::new(),
            reference_section: String::new(),
            validation: calibrator.build_result(
                config.validation_mode, vec![], vec![], vec![],
            ),
            evidence_graph: egp,
            rar_iterations,
            meta: ResearchMeta {
                duration_ms, sources_fetched: 0, sources_validated: 0,
                validation_pass_rate: 0.0, overall_confidence: 0.0,
                rar_loops_executed: 0,
            },
        })
    }
}
```

**Acceptance criteria:**

- [ ] `decompose_query` splits "compare X vs Y" into per-item sub-queries
- [ ] Pipeline struct defines all 9 steps from PRD SS10 Mode B
- [ ] `ResearchReport` contains all required fields: synthesis, sources, citations, validation, EGP, RAR iterations
- [ ] Evidence graph is only built when `config.evidence_graph` is true
- [ ] RAR loop integrated and respects `max_rar_loops`
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::decompose::*;
    #[test]
    fn comparison_decomposition() {
        let subs = decompose_query("compare Rust vs Go vs C++");
        assert!(subs.len() >= 3);
        assert!(subs.iter().any(|q| q.to_lowercase().contains("rust")));
        assert!(subs.iter().any(|q| q.to_lowercase().contains("go")));
    }
    #[test]
    fn simple_query_no_decomposition() {
        let subs = decompose_query("what is Rust");
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0], "what is Rust");
    }
    #[test]
    fn implications_decomposition() {
        let subs = decompose_query("GDPR implications for AI training");
        assert!(subs.len() >= 2);
    }
}
```

---

### P3-E3-T2: Research Command CLI

**ID:** `P3-E3-T2`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2-3 days

**Description:**
Build the `fetchium research` CLI command that wraps the research pipeline with human-friendly markdown output. Supports `--citations`, `--validate`, `--strict-evidence`, `--evidence-graph`, `--output`, and `--format` flags.

**PRD References:**

- SS10 Mode B: `fetchium research "GDPR implications for AI training data" --citations apa`
- SS11 CLI flags: `--citations`, `--validate`, `--evidence-graph`, `--output`, `--format`

**Files to create/modify:**

```
crates/fetchium-cli/src/commands/research.rs  -- research command
crates/fetchium-cli/src/cli.rs               -- Add ResearchCommand to clap enum
```

**Dependencies:** P3-E3-T1 (pipeline)

**Step 1: Define clap command**

```rust
use clap::Args;
use crate::citation::types::CitationStyle;
use crate::validate::types::ValidationMode;

/// Multi-source research with evidence mapping and citations
#[derive(Debug, Args)]
pub struct ResearchCommand {
    /// The research query
    pub query: String,

    /// Citation style
    #[arg(long, default_value = "inline")]
    pub citations: CitationStyle,

    /// Validation mode
    #[arg(long, default_value = "standard")]
    pub validate: ValidationMode,

    /// Require every claim to cite a source
    #[arg(long)]
    pub strict_evidence: bool,

    /// Generate evidence graph JSON
    #[arg(long)]
    pub evidence_graph: bool,

    /// Maximum number of sources to fetch
    #[arg(long, default_value = "10")]
    pub max_sources: usize,

    /// Output file path
    #[arg(long, short)]
    pub output: Option<String>,

    /// Output format
    #[arg(long, default_value = "md")]
    pub format: String,
}
```

**Step 2: Implement command handler**

```rust
use crate::research::pipeline::ResearchPipeline;
use crate::research::ResearchConfig;

impl ResearchCommand {
    pub async fn run(&self) -> Result<(), HsxError> {
        let config = ResearchConfig {
            query: self.query.clone(),
            max_sources: self.max_sources,
            token_budget: None,
            citation_style: self.citations,
            validation_mode: self.validate,
            strict_evidence: self.strict_evidence,
            evidence_graph: self.evidence_graph,
            max_rar_loops: 3,
        };

        // Show progress spinner
        let spinner = indicatif::ProgressBar::new_spinner();
        spinner.set_message(format!("Researching: {}", self.query));
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let report = ResearchPipeline::execute(&config).await?;
        spinner.finish_and_clear();

        // Format output
        let output = self.format_report(&report);

        // Write to file or stdout
        if let Some(ref path) = self.output {
            std::fs::write(path, &output)?;
            eprintln!("Report written to {path}");
        } else {
            println!("{output}");
        }

        // Write evidence graph if requested
        if self.evidence_graph {
            if let Some(ref graph) = report.evidence_graph {
                let json = serde_json::to_string_pretty(graph)?;
                let graph_path = self.output.as_deref()
                    .map(|p| p.replace(".md", "_evidence.json"))
                    .unwrap_or_else(|| "evidence_graph.json".into());
                std::fs::write(&graph_path, &json)?;
                eprintln!("Evidence graph written to {graph_path}");
            }
        }
        Ok(())
    }

    fn format_report(&self, report: &ResearchReport) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Research: {}\n\n", report.query));
        if report.sub_queries.len() > 1 {
            out.push_str("## Sub-queries\n\n");
            for sq in &report.sub_queries {
                out.push_str(&format!("- {sq}\n"));
            }
            out.push_str("\n");
        }
        out.push_str("## Findings\n\n");
        out.push_str(&report.synthesis);
        out.push_str("\n\n## Sources\n\n");
        out.push_str(&report.reference_section);
        out.push_str(&format!(
            "\n\n---\n*Confidence: {:.0}% | Sources: {} | Validated: {} | Duration: {}ms*\n",
            report.meta.overall_confidence * 100.0,
            report.meta.sources_fetched,
            report.meta.sources_validated,
            report.meta.duration_ms,
        ));
        out
    }
}
```

**Acceptance criteria:**

- [ ] `fetchium research "query"` executes the pipeline and prints markdown report to stdout
- [ ] `--citations apa` uses APA citation style throughout
- [ ] `--output report.md` writes to file instead of stdout
- [ ] `--evidence-graph` writes `evidence_graph.json` alongside the report
- [ ] `--strict-evidence` marks uncitable claims as `[unverified]`
- [ ] `--validate strict` runs all 6 validation layers
- [ ] Progress spinner displays during execution
- [ ] `cargo test` and `cargo clippy` pass

---

### P3-E3-T3: Agent Research JSON Output

**ID:** `P3-E3-T3`
**Status:** `DONE`
**Priority:** P1
**Estimated effort:** 2 days

**Description:**
Build the `fetchium agent-research` command that wraps the same research pipeline but outputs structured JSON for consumption by AI agents. Supports `--budget`, `--tier`, `--schema`, and `--framework` flags.

**PRD References:**

- SS9: `fetchium agent-research "query" --budget 4000 --schema output.json --strict-evidence`
- SS43: `AgentSearchResult` data model

**Files to create/modify:**

```
crates/fetchium-cli/src/commands/agent_research.rs  -- agent-research command
crates/fetchium-cli/src/cli.rs                     -- Add AgentResearchCommand
```

**Dependencies:** P3-E3-T1 (pipeline), P1-E3-T4 (PDS tiers)

**Step 1: Define clap command**

```rust
use clap::Args;

/// Structured research output for AI agents
#[derive(Debug, Args)]
pub struct AgentResearchCommand {
    /// The research query
    pub query: String,

    /// Token budget for the entire output
    #[arg(long, default_value = "4000")]
    pub budget: usize,

    /// Detail tier: key_facts, summary, detailed, complete
    #[arg(long, default_value = "detailed")]
    pub tier: String,

    /// JSON schema file for structured output validation
    #[arg(long)]
    pub schema: Option<String>,

    /// Require every claim to cite a source
    #[arg(long)]
    pub strict_evidence: bool,

    /// Framework adapter: langchain, crewai, mcp
    #[arg(long)]
    pub framework: Option<String>,

    /// Maximum sources
    #[arg(long, default_value = "10")]
    pub max_sources: usize,
}
```

**Step 2: Implement agent output formatting**

```rust
use serde::{Serialize, Deserialize};

/// Agent-optimized research output matching PRD SS43 AgentSearchResult.
#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResearchOutput {
    pub meta: AgentMeta,
    pub findings: Vec<Finding>,
    pub sources: Vec<AgentSource>,
    pub evidence_graph: Option<EvidenceGraph>,
    pub contradictions: Vec<Contradiction>,
    pub confidence: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentMeta {
    pub query: String,
    pub mode: String,
    pub tier: String,
    pub tokens_used: usize,
    pub tokens_budget: usize,
    pub sources_fetched: usize,
    pub duration_ms: u64,
    pub result_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Finding {
    pub claim: String,
    pub confidence: f64,
    pub source_indices: Vec<usize>,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentSource {
    pub index: usize,
    pub url: String,
    pub title: String,
    pub relevance: f64,
    pub content_hash: String,
}

impl AgentResearchCommand {
    pub async fn run(&self) -> Result<(), HsxError> {
        let config = ResearchConfig {
            query: self.query.clone(),
            max_sources: self.max_sources,
            token_budget: Some(self.budget),
            citation_style: CitationStyle::Inline,
            validation_mode: ValidationMode::Standard,
            strict_evidence: self.strict_evidence,
            evidence_graph: true,
            max_rar_loops: 3,
        };

        let report = ResearchPipeline::execute(&config).await?;

        let output = AgentResearchOutput {
            meta: AgentMeta {
                query: report.query, mode: "research".into(),
                tier: self.tier.clone(),
                tokens_used: 0, // computed from actual output
                tokens_budget: self.budget,
                sources_fetched: report.meta.sources_fetched,
                duration_ms: report.meta.duration_ms,
                result_id: uuid::Uuid::new_v4().to_string(),
            },
            findings: Vec::new(), // populated from synthesis
            sources: Vec::new(),  // populated from report.sources
            evidence_graph: report.evidence_graph,
            contradictions: report.validation.contradictions,
            confidence: report.meta.overall_confidence,
        };

        // Output as JSON (machine-readable, no pretty-print for agents)
        let json = serde_json::to_string(&output)?;
        println!("{json}");
        Ok(())
    }
}
```

**Acceptance criteria:**

- [ ] `fetchium agent-research "query"` outputs valid JSON to stdout
- [ ] JSON matches `AgentSearchResult` structure from PRD SS43
- [ ] `--budget 4000` limits total output tokens
- [ ] `--tier summary` adjusts detail level
- [ ] `--strict-evidence` includes verification status per finding
- [ ] Output is single-line JSON (no pretty-print) for pipe compatibility
- [ ] `cargo test` and `cargo clippy` pass

**Tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn output_is_valid_json() {
        let output = AgentResearchOutput {
            meta: AgentMeta {
                query: "test".into(), mode: "research".into(), tier: "summary".into(),
                tokens_used: 500, tokens_budget: 4000, sources_fetched: 5,
                duration_ms: 1200, result_id: "test-id".into(),
            },
            findings: vec![Finding {
                claim: "Rust is memory safe".into(), confidence: 0.95,
                source_indices: vec![0, 1], verified: true,
            }],
            sources: vec![], evidence_graph: None,
            contradictions: vec![], confidence: 0.9,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.starts_with('{'));
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["query"], "test");
    }
}
```

---

## Module Wiring

After all tasks are complete, update the module roots:

**`crates/fetchium-core/src/validate/mod.rs`:**

```rust
pub mod types;
pub mod cross_source;
pub mod temporal;
pub mod authority;
pub mod content;
pub mod extraction;
pub mod output;
pub mod calibration;
pub mod rar;

pub use types::*;
pub use cross_source::{CrossSourceVerifier, SourceContent};
pub use temporal::TemporalValidator;
pub use authority::AuthorityScorer;
pub use calibration::ConfidenceCalibrator;
pub use rar::{RarEngine, RarConfig, RarState};
```

**`crates/fetchium-core/src/citation/mod.rs`:**

```rust
pub mod types;
pub mod formatter;
pub mod evidence_graph;
pub mod evidence_tracker;

pub use types::*;
pub use formatter::CitationFormatter;
pub use evidence_graph::{EvidenceGraph, EvidenceGraphBuilder};
pub use evidence_tracker::EvidenceTracker;
```

**`crates/fetchium-core/src/research/mod.rs`:**

```rust
pub mod pipeline;
pub mod decompose;

pub use pipeline::ResearchPipeline;
```

---

## Dependency Matrix

| Task                              | Depends On         | Provides                 |
| --------------------------------- | ------------------ | ------------------------ |
| P3-E1-T1 (Cross-source)           | P2-E4, P1-E5-T1    | V4 layer, shared types   |
| P3-E1-T2 (Temporal)               | P3-E1-T1 (types)   | V3 layer                 |
| P3-E1-T3 (Authority)              | P3-E1-T1 (types)   | V1 layer                 |
| P3-E1-T4 (Content/Extract/Output) | P3-E1-T1 (types)   | V2, V5, V6 layers        |
| P3-E1-T5 (Calibration)            | P3-E1-T1..T4       | Aggregate confidence     |
| P3-E1-T6 (RAR)                    | P3-E1-T1, P3-E1-T5 | Self-correction loop     |
| P3-E2-T1 (Citations)              | P1-E7-T1           | 6 citation formatters    |
| P3-E2-T2 (EGP)                    | P3-E2-T1           | Evidence graph builder   |
| P3-E2-T3 (Evidence tracker)       | P3-E2-T1, P3-E2-T2 | Strict evidence mode     |
| P3-E3-T1 (Pipeline)               | All E1, all E2     | Research orchestrator    |
| P3-E3-T2 (research cmd)           | P3-E3-T1           | `fetchium research` CLI       |
| P3-E3-T3 (agent-research)         | P3-E3-T1           | `fetchium agent-research` CLI |

## Parallelization Guide

```
Agent A                         Agent B
--------                        --------
P3-E1-T1 (Types + Cross-source) P3-E2-T1 (Citation formatters)
P3-E1-T2 (Temporal)             P3-E2-T2 (Evidence Graph)
P3-E1-T3 (Authority)            P3-E2-T3 (Evidence tracker)
P3-E1-T4 (Content/V5/V6)        |
P3-E1-T5 (Calibration)          |
P3-E1-T6 (RAR)                  |
         |                       |
         +---------- merge -----+
                     |
             P3-E3-T1 (Pipeline)
                     |
             +-------+-------+
             |               |
     P3-E3-T2 (research)  P3-E3-T3 (agent-research)
```

## Testing Strategy

| Type        | Scope                 | Approach                                                       |
| ----------- | --------------------- | -------------------------------------------------------------- |
| Unit        | Each validation layer | Fixture data with known properties                             |
| Unit        | Citation formatters   | Known metadata -> expected formatted strings per style         |
| Unit        | EGP builder           | Build graph -> verify structure, hashes, confidence            |
| Unit        | RAR checkpoints       | Mock state with controlled scores -> verify actions            |
| Unit        | Query decomposition   | Known complex queries -> expected sub-questions                |
| Integration | Full pipeline         | Mock search backend (wiremock) -> verify report structure      |
| E2E         | `fetchium research`        | `assert_cmd` -> verify stdout is valid markdown with citations |
| E2E         | `fetchium agent-research`  | `assert_cmd` -> verify stdout is valid JSON matching schema    |
| Snapshot    | Report output         | `insta` snapshots for known queries with mock data             |

---

_For the master task index and dependency graph, see [`../TASKS.md`](../TASKS.md)._
