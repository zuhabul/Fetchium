# Phase 6: Intelligence Algorithms

> **Duration:** Weeks 27-36 | **Priority:** P2-P3
> **Depends On:** Phase 5 complete (Semantic Search & Advanced Features)
> **PRD Sections:** 8.11-8.17, 31, 32, 35, 39.3

---

## Overview

Phase 6 implements the 7 novel intelligence algorithms that make HyperSearchX unique. Each algorithm is a standalone module in `crates/hsx-core/src/intelligence/` with well-defined traits so they can be composed, tested, and evolved independently.

```
crates/hsx-core/src/intelligence/
  mod.rs         -- Module root, shared traits
  pie.rs         -- Persistent Intelligence Engine
  totr.rs        -- Tree-of-Thoughts Research
  crp.rs         -- Contradiction Resolution Protocol
  edf.rs         -- Evidence Decay Function
  sgt.rs         -- Source Genealogy Tracker
  cce.rs         -- Confidence Calibration Engine
  acs.rs         -- Adversarial Content Shield
```

---

## P6-E1-T1: Persistent Intelligence Engine (PIE)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T1` |
| **Status** | `DONE` |
| **Priority** | P2 |
| **Description** | Build the 4-layer Persistent Intelligence Engine that learns across sessions. PIE stores a Personal Knowledge Graph (PKG), Source Trust Memory (STM), Failure Pattern Memory (FPM), and Query Prediction Model (QPM) in local SQLite databases. It uses Bayesian trust updates to continuously improve source ranking and query prediction. |
| **PRD Ref** | 8.11, 31 (Cross-Session Learning & Persistent Intelligence), 31.1 (PIE architecture) |
| **Depends On** | `P1-E6` (SQLite), `P3-E2` (EGP), `P5-E1-T1` (embeddings for PKG) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/mod.rs` | Module root with shared traits |
| `crates/hsx-core/src/intelligence/pie.rs` | PIE engine core |
| `crates/hsx-core/src/intelligence/pie/pkg.rs` | Personal Knowledge Graph layer |
| `crates/hsx-core/src/intelligence/pie/stm.rs` | Source Trust Memory layer |
| `crates/hsx-core/src/intelligence/pie/fpm.rs` | Failure Pattern Memory layer |
| `crates/hsx-core/src/intelligence/pie/qpm.rs` | Query Prediction Model layer |
| `crates/hsx-core/src/intelligence/pie/storage.rs` | SQLite storage for all 4 layers |
| `crates/hsx-cli/src/commands/intelligence.rs` | CLI: `hsx intelligence stats/reset/export/suggest` |

#### Architecture

```
PIE (Persistent Intelligence Engine)
  |
  +-- Layer 1: PKG (Personal Knowledge Graph)
  |     Schema: entities(id, name, type, first_seen, last_seen, frequency)
  |             relationships(entity_a, entity_b, relation, weight, source)
  |     Updates: After every search, extract entities + relationships
  |     Query: "What concepts are related to X across all my research?"
  |
  +-- Layer 2: STM (Source Trust Memory)
  |     Schema: domains(domain, trust_score, fetch_count, success_count,
  |                      avg_relevance, last_updated)
  |     Updates: Bayesian update after each fetch (success/fail, relevance)
  |     Query: "How reliable is domain X?"
  |
  +-- Layer 3: FPM (Failure Pattern Memory)
  |     Schema: failures(domain, error_type, extraction_layer, count,
  |                       last_occurred, cooldown_until)
  |             successes(domain, extraction_layer, count, avg_time_ms)
  |     Updates: After each extraction attempt
  |     Query: "What extraction method works for domain X?"
  |
  +-- Layer 4: QPM (Query Prediction Model)
        Schema: queries(query_hash, query_text, timestamp, follow_ups)
                patterns(topic, frequency, last_queried, related_topics)
        Updates: After each query, record topic + follow-ups
        Query: "What will the user likely search next?"
```

#### Step-by-Step Implementation Guide

**Step 1: Define shared traits and storage**

```rust
// crates/hsx-core/src/intelligence/mod.rs
pub mod pie;

/// Trait for any intelligence layer that persists knowledge.
pub trait IntelligenceLayer: Send + Sync {
    /// Update the layer with new observation data.
    fn observe(&self, observation: &Observation) -> Result<(), crate::Error>;
    /// Query the layer for intelligence.
    fn query(&self, query: &IntelligenceQuery) -> Result<IntelligenceResult, crate::Error>;
    /// Get statistics about this layer.
    fn stats(&self) -> Result<LayerStats, crate::Error>;
    /// Reset all learned data.
    fn reset(&self) -> Result<(), crate::Error>;
}

#[derive(Debug)]
pub enum Observation {
    SearchCompleted {
        query: String,
        results: Vec<String>,  // URLs
        selected: Vec<String>, // URLs user interacted with
    },
    FetchAttempt {
        domain: String,
        url: String,
        extraction_layer: u8,
        success: bool,
        error: Option<String>,
        duration_ms: u64,
    },
    ContentRelevance {
        domain: String,
        query: String,
        relevance_score: f64,
    },
    EntityDiscovered {
        name: String,
        entity_type: String,
        context: String,
        source_url: String,
    },
    RelationshipDiscovered {
        entity_a: String,
        entity_b: String,
        relation: String,
        source_url: String,
    },
}
```

**Step 2: Source Trust Memory (STM) with Bayesian updates**

```rust
// crates/hsx-core/src/intelligence/pie/stm.rs
use rusqlite::Connection;
use std::sync::Mutex;

pub struct SourceTrustMemory {
    conn: Mutex<Connection>,
}

impl SourceTrustMemory {
    pub fn new(db_path: &std::path::Path) -> Result<Self, crate::Error> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS domain_trust (
                domain TEXT PRIMARY KEY,
                trust_score REAL DEFAULT 0.5,
                alpha REAL DEFAULT 1.0,       -- Beta distribution alpha (successes)
                beta REAL DEFAULT 1.0,        -- Beta distribution beta (failures)
                fetch_count INTEGER DEFAULT 0,
                success_count INTEGER DEFAULT 0,
                avg_relevance REAL DEFAULT 0.0,
                last_updated TEXT DEFAULT (datetime('now'))
            );"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Bayesian trust update using Beta distribution.
    /// Prior: Beta(alpha, beta), Posterior after success: Beta(alpha+1, beta),
    /// Posterior after failure: Beta(alpha, beta+1).
    /// Trust score = E[Beta] = alpha / (alpha + beta)
    pub fn update_trust(
        &self,
        domain: &str,
        success: bool,
        relevance: f64,
    ) -> Result<f64, crate::Error> {
        let conn = self.conn.lock().unwrap();

        // Upsert domain
        conn.execute(
            "INSERT INTO domain_trust (domain) VALUES (?1)
             ON CONFLICT(domain) DO NOTHING",
            [domain],
        )?;

        // Bayesian update
        if success {
            conn.execute(
                "UPDATE domain_trust SET
                    alpha = alpha + 1.0,
                    fetch_count = fetch_count + 1,
                    success_count = success_count + 1,
                    avg_relevance = (avg_relevance * fetch_count + ?2) / (fetch_count + 1),
                    trust_score = (alpha + 1.0) / (alpha + 1.0 + beta),
                    last_updated = datetime('now')
                 WHERE domain = ?1",
                rusqlite::params![domain, relevance],
            )?;
        } else {
            conn.execute(
                "UPDATE domain_trust SET
                    beta = beta + 1.0,
                    fetch_count = fetch_count + 1,
                    trust_score = alpha / (alpha + beta + 1.0),
                    last_updated = datetime('now')
                 WHERE domain = ?1",
                [domain],
            )?;
        }

        // Return new trust score
        let score: f64 = conn.query_row(
            "SELECT trust_score FROM domain_trust WHERE domain = ?1",
            [domain],
            |row| row.get(0),
        )?;
        Ok(score)
    }

    /// Get trust score for a domain. Returns 0.5 (uninformed prior) if unknown.
    pub fn get_trust(&self, domain: &str) -> Result<f64, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let score = conn.query_row(
            "SELECT trust_score FROM domain_trust WHERE domain = ?1",
            [domain],
            |row| row.get::<_, f64>(0),
        ).unwrap_or(0.5);
        Ok(score)
    }

    /// Get top trusted domains.
    pub fn top_trusted(&self, limit: usize) -> Result<Vec<(String, f64)>, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT domain, trust_score FROM domain_trust
             WHERE fetch_count > 5
             ORDER BY trust_score DESC LIMIT ?1"
        )?;
        let results = stmt.query_map([limit], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }
}
```

**Step 3: Failure Pattern Memory (FPM)**

```rust
// crates/hsx-core/src/intelligence/pie/fpm.rs

pub struct FailurePatternMemory {
    conn: Mutex<Connection>,
}

impl FailurePatternMemory {
    /// Record an extraction attempt (success or failure).
    pub fn record_attempt(
        &self,
        domain: &str,
        layer: u8,
        success: bool,
        error: Option<&str>,
        duration_ms: u64,
    ) -> Result<(), crate::Error> {
        let conn = self.conn.lock().unwrap();
        if success {
            conn.execute(
                "INSERT INTO extraction_successes (domain, layer, count, total_time_ms)
                 VALUES (?1, ?2, 1, ?3)
                 ON CONFLICT(domain, layer) DO UPDATE SET
                    count = count + 1,
                    total_time_ms = total_time_ms + ?3",
                rusqlite::params![domain, layer, duration_ms],
            )?;
        } else {
            conn.execute(
                "INSERT INTO extraction_failures (domain, layer, error_type, count, last_occurred)
                 VALUES (?1, ?2, ?3, 1, datetime('now'))
                 ON CONFLICT(domain, layer, error_type) DO UPDATE SET
                    count = count + 1,
                    last_occurred = datetime('now')",
                rusqlite::params![domain, layer, error.unwrap_or("unknown")],
            )?;
        }
        Ok(())
    }

    /// Get the best extraction layer for a domain based on historical data.
    /// Returns (recommended_layer, confidence).
    pub fn recommend_layer(&self, domain: &str) -> Result<(u8, f64), crate::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT layer, count,
                    (SELECT COALESCE(SUM(f.count), 0) FROM extraction_failures f
                     WHERE f.domain = s.domain AND f.layer = s.layer) as fail_count
             FROM extraction_successes s
             WHERE s.domain = ?1
             ORDER BY count DESC"
        )?;

        let results: Vec<(u8, i64, i64)> = stmt.query_map([domain], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.filter_map(|r| r.ok()).collect();

        if let Some((layer, successes, failures)) = results.first() {
            let total = (*successes + *failures) as f64;
            let confidence = *successes as f64 / total;
            Ok((*layer, confidence))
        } else {
            Ok((2, 0.5)) // Default: Layer 2 (HTTP + Readability) with neutral confidence
        }
    }
}
```

**Step 4: Query Prediction Model (QPM)**

```rust
// crates/hsx-core/src/intelligence/pie/qpm.rs

pub struct QueryPredictionModel {
    conn: Mutex<Connection>,
}

impl QueryPredictionModel {
    /// Record a query and its context.
    pub fn record_query(
        &self,
        query: &str,
        topic: &str,
        follow_up: Option<&str>,
    ) -> Result<(), crate::Error> {
        let conn = self.conn.lock().unwrap();
        let hash = sha256_hex(query);

        conn.execute(
            "INSERT INTO query_history (query_hash, query_text, topic)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![hash, query, topic],
        )?;

        // Record follow-up pattern if available
        if let Some(follow_up) = follow_up {
            conn.execute(
                "INSERT INTO follow_up_patterns (topic, follow_up_query, count)
                 VALUES (?1, ?2, 1)
                 ON CONFLICT(topic, follow_up_query) DO UPDATE SET count = count + 1",
                rusqlite::params![topic, follow_up],
            )?;
        }

        // Update topic frequency
        conn.execute(
            "INSERT INTO topic_frequency (topic, count, last_queried)
             VALUES (?1, 1, datetime('now'))
             ON CONFLICT(topic) DO UPDATE SET
                count = count + 1,
                last_queried = datetime('now')",
            [topic],
        )?;

        Ok(())
    }

    /// Predict likely follow-up queries based on current query and history.
    pub fn predict_follow_ups(
        &self,
        current_topic: &str,
        limit: usize,
    ) -> Result<Vec<String>, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT follow_up_query FROM follow_up_patterns
             WHERE topic = ?1
             ORDER BY count DESC
             LIMIT ?2"
        )?;
        let results = stmt.query_map(
            rusqlite::params![current_topic, limit],
            |row| row.get::<_, String>(0),
        )?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }
}
```

**Step 5: Personal Knowledge Graph (PKG)**

```rust
// crates/hsx-core/src/intelligence/pie/pkg.rs

pub struct PersonalKnowledgeGraph {
    conn: Mutex<Connection>,
}

impl PersonalKnowledgeGraph {
    /// Add an entity discovered during research.
    pub fn add_entity(
        &self,
        name: &str,
        entity_type: &str,
        source_url: &str,
    ) -> Result<u64, crate::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO entities (name, type, source_url, frequency)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(name) DO UPDATE SET
                frequency = frequency + 1,
                last_seen = datetime('now')",
            rusqlite::params![name, entity_type, source_url],
        )?;
        let id = conn.last_insert_rowid() as u64;
        Ok(id)
    }

    /// Add a relationship between two entities.
    pub fn add_relationship(
        &self,
        entity_a: &str,
        entity_b: &str,
        relation: &str,
        weight: f64,
    ) -> Result<(), crate::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO relationships (entity_a, entity_b, relation, weight)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(entity_a, entity_b, relation) DO UPDATE SET
                weight = (weight + ?4) / 2.0",
            rusqlite::params![entity_a, entity_b, relation, weight],
        )?;
        Ok(())
    }

    /// Find entities related to a given concept.
    pub fn related_entities(
        &self,
        entity: &str,
        limit: usize,
    ) -> Result<Vec<(String, String, f64)>, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT entity_b, relation, weight FROM relationships
             WHERE entity_a = ?1
             UNION
             SELECT entity_a, relation, weight FROM relationships
             WHERE entity_b = ?1
             ORDER BY weight DESC
             LIMIT ?2"
        )?;
        let results = stmt.query_map(
            rusqlite::params![entity, limit],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?.filter_map(|r| r.ok()).collect();
        Ok(results)
    }
}
```

**Step 6: PIE orchestrator and CLI**

```rust
// crates/hsx-core/src/intelligence/pie.rs

pub struct PersistentIntelligenceEngine {
    pub pkg: pkg::PersonalKnowledgeGraph,
    pub stm: stm::SourceTrustMemory,
    pub fpm: fpm::FailurePatternMemory,
    pub qpm: qpm::QueryPredictionModel,
}

impl PersistentIntelligenceEngine {
    pub fn new() -> Result<Self, crate::Error> {
        let base = crate::config::data_dir().join("intelligence");
        std::fs::create_dir_all(&base)?;
        Ok(Self {
            pkg: pkg::PersonalKnowledgeGraph::new(&base.join("knowledge_graph.db"))?,
            stm: stm::SourceTrustMemory::new(&base.join("source_trust.db"))?,
            fpm: fpm::FailurePatternMemory::new(&base.join("failure_patterns.db"))?,
            qpm: qpm::QueryPredictionModel::new(&base.join("query_patterns.db"))?,
        })
    }

    /// Process observations from a completed search/fetch operation.
    pub fn observe_search(
        &self,
        query: &str,
        results: &[crate::types::SearchResult],
    ) -> Result<(), crate::Error> {
        // Update STM for each source domain
        for result in results {
            let domain = extract_domain(&result.url);
            self.stm.update_trust(&domain, true, result.relevance)?;
        }
        // Record query for QPM
        let topic = extract_topic(query);
        self.qpm.record_query(query, &topic, None)?;
        Ok(())
    }

    pub fn stats(&self) -> Result<PieStats, crate::Error> {
        Ok(PieStats {
            entities: self.pkg.entity_count()?,
            relationships: self.pkg.relationship_count()?,
            tracked_domains: self.stm.domain_count()?,
            failure_patterns: self.fpm.pattern_count()?,
            query_history_size: self.qpm.query_count()?,
        })
    }

    pub fn reset_all(&self) -> Result<(), crate::Error> {
        self.pkg.reset()?;
        self.stm.reset()?;
        self.fpm.reset()?;
        self.qpm.reset()?;
        Ok(())
    }
}
```

#### Acceptance Criteria

- [x] PIE persists data across sessions in `~/.hypersearchx/intelligence/`
- [x] STM: After 10 successful fetches from a domain, trust score > 0.7
- [x] STM: After 10 failed fetches from a domain, trust score < 0.3
- [x] FPM: `recommend_layer("spa-site.com")` returns Layer 3 after repeated Layer 1 failures
- [x] QPM: After researching "Rust" 5 times, `predict_follow_ups("Rust")` returns relevant suggestions
- [x] PKG: Entities and relationships accumulate across sessions
- [x] `hsx intelligence stats` shows counts for all 4 layers
- [x] `hsx intelligence reset` clears all learned data
- [x] `hsx intelligence export` exports data as JSON
- [x] Trust scores use Bayesian Beta distribution (not simple averages)
- [x] All DBs use WAL mode for concurrent read/write

#### Pitfalls

- **SQLite concurrent access**: PIE is accessed from multiple async tasks simultaneously. Use `Mutex<Connection>` and enable WAL mode.
- **Entity extraction quality**: Simple regex-based entity extraction will produce noise. Start with named entity patterns (capitalized phrases, domain-specific terms) and improve incrementally.
- **Trust score drift**: Without decay, trust scores become "stuck" at extreme values. Implement a time-weighted decay that slowly pulls scores toward 0.5 when a domain hasn't been accessed recently.
- **Privacy**: All data is local but may contain sensitive query history. Ensure `hsx intelligence reset` truly purges everything (VACUUM after DELETE).

---

## P6-E1-T2: Tree-of-Thoughts Research (ToTR)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T2` |
| **Status** | `DONE` |
| **Priority** | P2 |
| **Description** | Implement Tree-of-Thoughts Research for complex multi-faceted queries. Decomposes a query into 2-5 parallel reasoning paths, runs independent search-extract-rank pipelines per path, scores and prunes low-quality branches, synthesizes surviving paths, and optionally runs a self-debate protocol (Advocate vs Critic vs Judge). |
| **PRD Ref** | 8.12, 32 (Tree-of-Thoughts & Advanced Reasoning), 32.1-32.3 |
| **Depends On** | `P4-E1` (AI engine for decomposition), `P4-E2` (AMRS agents), `P3-E3` (research mode) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/totr.rs` | ToTR engine |
| `crates/hsx-core/src/intelligence/totr/branch.rs` | Branch generation and management |
| `crates/hsx-core/src/intelligence/totr/scorer.rs` | Branch scoring and pruning |
| `crates/hsx-core/src/intelligence/totr/synthesis.rs` | Cross-path synthesis |
| `crates/hsx-core/src/intelligence/totr/debate.rs` | Self-debate protocol |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/totr.rs

use tokio::sync::mpsc;

/// A reasoning path in the Tree-of-Thoughts.
#[derive(Debug, Clone)]
pub struct ThoughtBranch {
    pub id: String,
    pub perspective: String,          // e.g., "Technical feasibility"
    pub sub_queries: Vec<String>,     // research queries for this path
    pub findings: Vec<Finding>,
    pub score: f64,                   // evidence quality score
    pub status: BranchStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BranchStatus {
    Active,
    Pruned { reason: String },
    Complete,
}

/// Run Tree-of-Thoughts research on a complex query.
pub async fn run_totr(
    query: &str,
    config: &TotrConfig,
    ai_client: &crate::ai::AiClient,
) -> Result<TotrResult, crate::Error> {
    // Step 1: Decompose query into reasoning paths
    let branches = generate_branches(query, ai_client, config.max_branches).await?;
    tracing::info!(
        query = query,
        branches = branches.len(),
        "ToTR: Generated reasoning paths"
    );

    // Step 2: Run each branch in parallel
    let (tx, mut rx) = mpsc::channel(32);
    let mut handles = Vec::new();

    for branch in branches {
        let tx = tx.clone();
        let handle = tokio::spawn(async move {
            let result = explore_branch(branch).await;
            let _ = tx.send(result).await;
        });
        handles.push(handle);
    }
    drop(tx);

    // Collect results
    let mut completed_branches = Vec::new();
    while let Some(result) = rx.recv().await {
        match result {
            Ok(branch) => completed_branches.push(branch),
            Err(e) => tracing::warn!("ToTR branch failed: {e}"),
        }
    }

    // Step 3: Score and prune
    score_and_prune(&mut completed_branches, config.prune_threshold);

    // Step 4: Cross-path synthesis
    let synthesis = synthesize_paths(&completed_branches, query, ai_client).await?;

    // Step 5: Self-debate (optional)
    let debate_result = if config.self_debate {
        Some(self_debate(&completed_branches, query, ai_client).await?)
    } else {
        None
    };

    Ok(TotrResult {
        query: query.to_string(),
        branches: completed_branches,
        synthesis,
        debate: debate_result,
    })
}

/// Step 1: Generate reasoning branches from query decomposition.
async fn generate_branches(
    query: &str,
    ai_client: &crate::ai::AiClient,
    max_branches: usize,
) -> Result<Vec<ThoughtBranch>, crate::Error> {
    let prompt = format!(
        "Decompose this research question into {max_branches} distinct \
         perspectives or angles to investigate. For each, provide:\n\
         1. A perspective name (2-4 words)\n\
         2. 2-3 specific search queries to investigate that angle\n\n\
         Question: {query}\n\n\
         Respond in JSON: [{{\"perspective\": \"...\", \"queries\": [\"...\"]}}]"
    );

    let response = ai_client.complete(&prompt, Default::default()).await?;
    let decomposition: Vec<BranchSpec> = serde_json::from_str(&response.text)?;

    Ok(decomposition.into_iter().enumerate().map(|(i, spec)| {
        ThoughtBranch {
            id: format!("branch-{}", i),
            perspective: spec.perspective,
            sub_queries: spec.queries,
            findings: Vec::new(),
            score: 0.0,
            status: BranchStatus::Active,
        }
    }).collect())
}

/// Step 2: Explore a single branch by running its sub-queries.
async fn explore_branch(
    mut branch: ThoughtBranch,
) -> Result<ThoughtBranch, crate::Error> {
    for query in &branch.sub_queries {
        let research = crate::research::run_research(query, &Default::default()).await?;
        branch.findings.extend(research.findings);
    }

    // Score branch based on evidence quality
    branch.score = branch.findings.iter()
        .map(|f| f.confidence)
        .sum::<f64>()
        / branch.findings.len().max(1) as f64;

    branch.status = BranchStatus::Complete;
    Ok(branch)
}

/// Step 3: Prune low-scoring branches.
fn score_and_prune(branches: &mut Vec<ThoughtBranch>, threshold: f64) {
    let max_score = branches.iter()
        .map(|b| b.score)
        .fold(f64::NEG_INFINITY, f64::max);

    for branch in branches.iter_mut() {
        if branch.score < max_score * threshold {
            branch.status = BranchStatus::Pruned {
                reason: format!(
                    "Score {:.2} below threshold ({:.2} * {:.2} = {:.2})",
                    branch.score, max_score, threshold, max_score * threshold
                ),
            };
        }
    }
}
```

**Step 5 (Self-Debate Protocol):**

```rust
// crates/hsx-core/src/intelligence/totr/debate.rs

#[derive(Debug, Clone)]
pub struct DebateResult {
    pub advocate_argument: String,
    pub critic_argument: String,
    pub judge_verdict: String,
    pub confidence: f64,
}

/// Run the Advocate-Critic-Judge self-debate protocol.
pub async fn self_debate(
    branches: &[ThoughtBranch],
    query: &str,
    ai_client: &crate::ai::AiClient,
) -> Result<DebateResult, crate::Error> {
    let evidence_summary = summarize_evidence(branches);

    // Advocate: argue FOR the main conclusion
    let advocate_prompt = format!(
        "You are the Advocate. Based on this evidence, argue strongly FOR \
         the main conclusion about: {query}\n\nEvidence:\n{evidence_summary}\n\n\
         Present your strongest argument with citations."
    );
    let advocate = ai_client.complete(&advocate_prompt, Default::default()).await?;

    // Critic: argue AGAINST or find weaknesses
    let critic_prompt = format!(
        "You are the Critic. Find weaknesses, gaps, and counterarguments to \
         this position about: {query}\n\nAdvocate's argument:\n{}\n\n\
         Evidence:\n{evidence_summary}\n\nPresent your strongest counterargument.",
        advocate.text
    );
    let critic = ai_client.complete(&critic_prompt, Default::default()).await?;

    // Judge: weigh both sides and produce balanced synthesis
    let judge_prompt = format!(
        "You are the Judge. Weigh both arguments and produce a balanced, \
         nuanced conclusion about: {query}\n\n\
         Advocate:\n{}\n\nCritic:\n{}\n\n\
         Provide your verdict with a confidence score (0.0-1.0).",
        advocate.text, critic.text
    );
    let judge = ai_client.complete(&judge_prompt, Default::default()).await?;

    Ok(DebateResult {
        advocate_argument: advocate.text,
        critic_argument: critic.text,
        judge_verdict: judge.text,
        confidence: extract_confidence(&judge.text),
    })
}
```

#### Acceptance Criteria

- [x] Complex queries (e.g., "Is nuclear fusion economically viable by 2035?") produce 3-5 parallel reasoning paths
- [x] Each path generates independent research findings with citations
- [x] Low-scoring branches are pruned (visible in output with pruning reason)
- [x] Cross-path synthesis produces a unified conclusion referencing multiple perspectives
- [x] Self-debate (`--self-debate`) produces Advocate, Critic, and Judge outputs
- [x] All branches execute concurrently (visible in `--profile` timing)
- [x] `hsx deep "query" --tree-of-thoughts` activates ToTR mode
- [x] ToTR results include per-branch scores, source counts, and synthesis
- [x] Graceful degradation: if LLM is unavailable, fall back to keyword-based decomposition

#### Pitfalls

- **LLM dependency**: ToTR relies heavily on the LLM for decomposition and synthesis. Ensure fallback to simpler decomposition (keyword splitting, conjunction splitting) when no LLM is available.
- **Branch explosion**: Limit to 5 branches maximum to avoid excessive resource consumption.
- **Redundant research**: Different branches may hit the same URLs. Use cross-branch dedup.
- **Self-debate quality**: Low-quality LLMs may produce superficial debates. Consider skipping debate for models smaller than 7B parameters.

---

## P6-E1-T3: Contradiction Resolution Protocol (CRP)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T3` |
| **Status** | `DONE` |
| **Priority** | P2 |
| **Description** | Implement the 5-step Contradiction Resolution Protocol. When sources disagree, automatically investigate via date checking, authority analysis, context analysis, investigation agent spawning, and weighted synthesis. |
| **PRD Ref** | 8.13 (CRP), 19 (V4: Cross-Source Validation) |
| **Depends On** | `P3-E1` (validation layer), `P4-E2` (agent spawning), `P6-E1-T1` (PIE for trust scores) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/crp.rs` | CRP engine |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/crp.rs

#[derive(Debug, Clone)]
pub struct Contradiction {
    pub claim_a: String,
    pub source_a: Source,
    pub claim_b: String,
    pub source_b: Source,
    pub severity: Severity,        // Low, Medium, High, Critical
}

#[derive(Debug, Clone)]
pub struct Resolution {
    pub original: Contradiction,
    pub steps_taken: Vec<ResolutionStep>,
    pub synthesis: String,
    pub confidence: f64,
    pub resolution_type: ResolutionType,
}

#[derive(Debug, Clone)]
pub enum ResolutionType {
    TemporalPrecedence,     // Newer source wins
    AuthorityPrecedence,    // More authoritative source wins
    ScopeDependent,         // Not really contradictory — different contexts
    Unresolved,             // Genuine disagreement, report both sides
    InvestigationRequired,  // Spawned agent found additional evidence
}

/// Resolve a contradiction through the 5-step protocol.
pub async fn resolve(
    contradiction: &Contradiction,
    pie: &crate::intelligence::pie::PersistentIntelligenceEngine,
) -> Result<Resolution, crate::Error> {
    let mut steps = Vec::new();

    // Step 1: Date Check
    let date_step = check_dates(&contradiction);
    steps.push(date_step.clone());
    if date_step.conclusive {
        return Ok(Resolution {
            original: contradiction.clone(),
            steps_taken: steps,
            synthesis: date_step.conclusion,
            confidence: date_step.confidence,
            resolution_type: ResolutionType::TemporalPrecedence,
        });
    }

    // Step 2: Authority Check
    let auth_step = check_authority(&contradiction, pie).await?;
    steps.push(auth_step.clone());
    if auth_step.conclusive {
        return Ok(Resolution {
            original: contradiction.clone(),
            steps_taken: steps,
            synthesis: auth_step.conclusion,
            confidence: auth_step.confidence,
            resolution_type: ResolutionType::AuthorityPrecedence,
        });
    }

    // Step 3: Context Check (are they really contradicting?)
    let context_step = check_context(&contradiction);
    steps.push(context_step.clone());
    if context_step.conclusive {
        return Ok(Resolution {
            original: contradiction.clone(),
            steps_taken: steps,
            synthesis: context_step.conclusion,
            confidence: context_step.confidence,
            resolution_type: ResolutionType::ScopeDependent,
        });
    }

    // Step 4: Investigation Agent (spawn additional search)
    let investigation_step = spawn_investigation(&contradiction).await?;
    steps.push(investigation_step.clone());

    // Step 5: Weighted Synthesis
    let synthesis = weighted_synthesis(&contradiction, &steps)?;
    let confidence = steps.iter().map(|s| s.confidence).sum::<f64>() / steps.len() as f64;

    Ok(Resolution {
        original: contradiction.clone(),
        steps_taken: steps,
        synthesis,
        confidence,
        resolution_type: if investigation_step.conclusive {
            ResolutionType::InvestigationRequired
        } else {
            ResolutionType::Unresolved
        },
    })
}

fn check_dates(contradiction: &Contradiction) -> ResolutionStep {
    let date_a = contradiction.source_a.published_date;
    let date_b = contradiction.source_b.published_date;

    match (date_a, date_b) {
        (Some(a), Some(b)) if (b - a).num_days().abs() > 180 => {
            let newer = if b > a { &contradiction.source_b } else { &contradiction.source_a };
            let newer_claim = if b > a { &contradiction.claim_b } else { &contradiction.claim_a };
            ResolutionStep {
                name: "Date Check".into(),
                conclusion: format!(
                    "Newer source ({}, {}) supersedes older source. Claim: {}",
                    newer.domain, newer.published_date.unwrap().format("%Y-%m-%d"),
                    newer_claim
                ),
                confidence: 0.7,
                conclusive: true,
            }
        }
        _ => ResolutionStep {
            name: "Date Check".into(),
            conclusion: "Sources are from similar timeframes; date alone is inconclusive.".into(),
            confidence: 0.3,
            conclusive: false,
        },
    }
}
```

#### Acceptance Criteria

- [x] Contradictions between sources are automatically detected during cross-source validation
- [x] 5-step resolution pipeline executes in order, short-circuiting when conclusive
- [x] Date check correctly identifies newer superseding sources
- [x] Authority check integrates PIE trust scores
- [x] Context check identifies "false contradictions" (different scopes/populations)
- [x] Investigation agent spawns additional search queries for Step 4
- [x] Weighted synthesis produces a nuanced paragraph citing both sources
- [x] Resolution includes confidence score and resolution type
- [x] Output in research reports shows contradiction resolution under each disputed claim

#### Pitfalls

- **False contradiction detection**: Two sources may use different terminology for the same thing. Normalize claims before comparison.
- **Investigation agent resource consumption**: Spawning additional searches can be expensive. Limit to 2-3 additional queries.
- **Circular resolution**: Ensure the investigation agent doesn't find the same contradicting sources again. Track already-seen URLs.

---

## P6-E1-T4: Evidence Decay Function (EDF)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T4` |
| **Status** | `DONE` |
| **Priority** | P3 |
| **Description** | Implement domain-calibrated evidence half-lives. Claims decay in reliability based on their domain: AI benchmarks decay in months, math proofs are eternal. EDF auto-flags stale evidence and is self-calibrating based on whether flagged content turns out to still be valid. |
| **PRD Ref** | 8.14 (Evidence Decay Function) |
| **Depends On** | `P3-E2` (EGP evidence system), `P6-E1-T1` (PIE for calibration storage) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/edf.rs` | EDF implementation |
| `data/domain_half_lives.toml` | Default half-life configuration |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/edf.rs

use std::collections::HashMap;

/// Domain-calibrated half-lives in days.
static DEFAULT_HALF_LIVES: &[(&str, f64)] = &[
    ("ai_ml_benchmarks", 90.0),      // 3 months
    ("tech_news", 14.0),              // 2 weeks
    ("medical_trials", 730.0),        // 2 years
    ("legal_precedent", 3650.0),      // 10 years
    ("mathematics", 36500.0),         // 100 years
    ("stock_prices", 1.0),            // 1 day
    ("software_docs", 180.0),         // 6 months
    ("historical_facts", 18250.0),    // 50 years
    ("security_advisories", 30.0),    // 1 month
    ("social_media", 7.0),            // 1 week
];

pub struct EvidenceDecayFunction {
    half_lives: HashMap<String, f64>,
    calibration_db: rusqlite::Connection,
}

impl EvidenceDecayFunction {
    /// Calculate the decayed confidence of a claim.
    ///
    /// Formula: decayed = base_confidence * e^(-lambda * age_days)
    /// where lambda = ln(2) / half_life_days
    pub fn decay(
        &self,
        base_confidence: f64,
        domain: &str,
        age_days: f64,
    ) -> DecayResult {
        let half_life = self.get_half_life(domain);
        let lambda = (2.0_f64).ln() / half_life;
        let decayed = base_confidence * (-lambda * age_days).exp();

        let staleness = if decayed < 0.3 {
            Staleness::Stale
        } else if decayed < 0.5 {
            Staleness::PotentiallyStale
        } else {
            Staleness::Fresh
        };

        DecayResult {
            original_confidence: base_confidence,
            decayed_confidence: decayed,
            domain: domain.to_string(),
            age_days,
            half_life,
            staleness,
        }
    }

    /// Self-calibration: if a flagged-stale claim turns out to still be valid,
    /// increase the half-life for that domain.
    pub fn calibrate(
        &mut self,
        domain: &str,
        was_flagged_stale: bool,
        actually_still_valid: bool,
    ) -> Result<(), crate::Error> {
        let conn = &self.calibration_db;
        conn.execute(
            "INSERT INTO calibration_events (domain, flagged_stale, actually_valid)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![domain, was_flagged_stale, actually_still_valid],
        )?;

        // Check if we need to adjust half-life
        if was_flagged_stale && actually_still_valid {
            // False positive: increase half-life by 10%
            let current = self.get_half_life(domain);
            let new_half_life = current * 1.1;
            self.half_lives.insert(domain.to_string(), new_half_life);
            tracing::info!(
                domain = domain,
                old = current,
                new = new_half_life,
                "EDF: Increased half-life (false stale flag)"
            );
        } else if !was_flagged_stale && !actually_still_valid {
            // False negative: decrease half-life by 10%
            let current = self.get_half_life(domain);
            let new_half_life = (current * 0.9).max(1.0);
            self.half_lives.insert(domain.to_string(), new_half_life);
        }

        Ok(())
    }

    fn get_half_life(&self, domain: &str) -> f64 {
        self.half_lives.get(domain)
            .copied()
            .unwrap_or(180.0) // Default: 6 months
    }
}

#[derive(Debug, Clone)]
pub struct DecayResult {
    pub original_confidence: f64,
    pub decayed_confidence: f64,
    pub domain: &str,
    pub age_days: f64,
    pub half_life: f64,
    pub staleness: Staleness,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Staleness {
    Fresh,
    PotentiallyStale,
    Stale,
}
```

#### Acceptance Criteria

- [x] `edf.decay(0.9, "ai_ml_benchmarks", 180.0)` returns ~0.45 (halved after 3 months)
- [x] `edf.decay(0.9, "mathematics", 365.0)` returns ~0.89 (barely decayed after 1 year)
- [x] Stale evidence is flagged in research output with `[potentially stale: 2 years old, domain half-life: 6 months]`
- [x] Self-calibration adjusts half-lives based on feedback
- [x] Default half-lives configurable via `data/domain_half_lives.toml`
- [x] Domain classification works on URL patterns (arxiv.org -> academic, github.com -> software_docs)
- [x] Evidence decay integrates into HyperFusion temporal signal

#### Pitfalls

- **Domain misclassification**: A medical article on a news site should use medical half-life, not news half-life. Use content analysis, not just domain.
- **Exponential underflow**: Very old content with short half-lives may produce f64 values near zero. Clamp to a minimum of 0.01.
- **Calibration data scarcity**: Self-calibration needs sufficient feedback events. Don't adjust until at least 10 events per domain.

---

## P6-E1-T5: Source Genealogy Tracker (SGT)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T5` |
| **Status** | `DONE` |
| **Priority** | P3 |
| **Description** | Trace claim provenance through citation chains to the primary source. Detect "mutations" where claims are altered as they propagate through the citation chain. Compute trust cascade scores. |
| **PRD Ref** | 8.15 (Source Genealogy Tracker) |
| **Depends On** | `P3-E2` (EGP for evidence links), `P1-E1` (HTTP fetch for citation following) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/sgt.rs` | SGT engine |
| `crates/hsx-core/src/intelligence/sgt/chain.rs` | Citation chain builder |
| `crates/hsx-core/src/intelligence/sgt/mutation.rs` | Mutation detection |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/sgt.rs

#[derive(Debug, Clone)]
pub struct GenealogyNode {
    pub url: String,
    pub title: String,
    pub published_date: Option<chrono::NaiveDate>,
    pub claim_text: String,
    pub trust_score: f64,
}

#[derive(Debug, Clone)]
pub struct CitationChain {
    pub nodes: Vec<GenealogyNode>,
    pub mutations: Vec<Mutation>,
    pub primary_source: Option<GenealogyNode>,
    pub trust_cascade: Vec<f64>,  // trust degrades with each hop
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub from_node: usize,  // index in chain
    pub to_node: usize,
    pub original_claim: String,
    pub mutated_claim: String,
    pub severity: MutationSeverity,
    pub description: String,
}

/// Trace a claim back to its primary source.
pub async fn trace_genealogy(
    claim: &str,
    source_url: &str,
    max_depth: usize,
) -> Result<CitationChain, crate::Error> {
    let mut chain = Vec::new();
    let mut current_url = source_url.to_string();
    let mut current_claim = claim.to_string();

    for depth in 0..max_depth {
        // Fetch and extract the current source
        let content = crate::extraction::fetch_and_extract(&current_url).await?;

        chain.push(GenealogyNode {
            url: current_url.clone(),
            title: content.title.clone(),
            published_date: content.published_date,
            claim_text: current_claim.clone(),
            trust_score: 1.0 - (depth as f64 * 0.15), // trust degrades per hop
        });

        // Look for citations/references in the content
        let references = extract_references(&content);
        let matching_ref = find_reference_for_claim(&current_claim, &references);

        match matching_ref {
            Some(ref_url) => {
                current_url = ref_url;
                // Fetch the referenced source and find the corresponding claim
                let ref_content = crate::extraction::fetch_and_extract(&current_url).await?;
                let ref_claim = find_claim_in_source(&current_claim, &ref_content);
                if let Some(ref_claim) = ref_claim {
                    current_claim = ref_claim;
                } else {
                    break; // Can't find the claim in the referenced source
                }
            }
            None => break, // No further references found (this is the primary source)
        }
    }

    // Detect mutations between chain hops
    let mutations = detect_mutations(&chain);

    // Compute trust cascade
    let trust_cascade: Vec<f64> = chain.iter().map(|n| n.trust_score).collect();

    let primary_source = chain.last().cloned();

    Ok(CitationChain {
        nodes: chain,
        mutations,
        primary_source,
        trust_cascade,
    })
}

/// Detect mutations in claims as they propagate through the citation chain.
fn detect_mutations(chain: &[GenealogyNode]) -> Vec<Mutation> {
    let mut mutations = Vec::new();

    for i in 0..chain.len().saturating_sub(1) {
        let from = &chain[i];
        let to = &chain[i + 1];

        // Use fuzzy matching to detect claim changes
        let similarity = strsim::normalized_damerau_levenshtein(
            &from.claim_text.to_lowercase(),
            &to.claim_text.to_lowercase(),
        );

        if similarity < 0.95 {
            let severity = if similarity < 0.5 {
                MutationSeverity::High
            } else if similarity < 0.8 {
                MutationSeverity::Medium
            } else {
                MutationSeverity::Low
            };

            mutations.push(Mutation {
                from_node: i,
                to_node: i + 1,
                original_claim: to.claim_text.clone(),
                mutated_claim: from.claim_text.clone(),
                severity,
                description: describe_mutation(&to.claim_text, &from.claim_text),
            });
        }
    }

    mutations
}
```

#### Acceptance Criteria

- [x] `trace_genealogy("claim", "blog-url", 5)` follows citation chains up to 5 hops
- [x] Primary source identified when the chain reaches a source with no further citations
- [x] Mutations detected when claim text changes significantly between hops
- [x] Trust cascade shows degrading trust: [0.97, 0.85, 0.62, 0.41]
- [x] Output includes genealogy tree visualization in both human and JSON formats
- [x] `hsx research "topic" --trace-sources` enables SGT for all claims
- [x] Mutation severity classified as Low/Medium/High

#### Pitfalls

- **Citation extraction**: Not all pages have machine-readable citations. Use heuristics: look for `<a>` tags near quote blocks, footnote patterns, "Source:" prefixes.
- **Rate limiting**: Following citation chains means many sequential HTTP requests. Respect per-domain rate limits.
- **Dead links**: Citation chains often contain dead links. Log them but continue tracing via alternative paths.
- **Fuzzy claim matching**: The same claim may be rephrased significantly. Use semantic similarity (embeddings) in addition to string matching when available.

---

## P6-E1-T6: Confidence Calibration Engine (CCE)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T6` |
| **Status** | `DONE` |
| **Priority** | P3 |
| **Description** | Track historical accuracy of confidence scores and calibrate them using isotonic regression. When the system says "85% confident," CCE ensures that prediction is historically accurate 85% (or close to it) of the time. Maintains per-domain calibration tables. |
| **PRD Ref** | 8.16, 39.3 (Confidence Calibration Engine) |
| **Depends On** | `P6-E1-T1` (PIE for storage), `P3-E1` (validation layer) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/cce.rs` | CCE implementation |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/cce.rs

pub struct ConfidenceCalibrationEngine {
    conn: Mutex<rusqlite::Connection>,
}

impl ConfidenceCalibrationEngine {
    pub fn new(db_path: &std::path::Path) -> Result<Self, crate::Error> {
        let conn = rusqlite::Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS predictions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                domain TEXT NOT NULL,
                stated_confidence REAL NOT NULL,
                actually_correct INTEGER,        -- NULL until verified, 0 or 1
                claim TEXT NOT NULL,
                source_url TEXT,
                created_at TEXT DEFAULT (datetime('now')),
                verified_at TEXT
            );
            CREATE TABLE IF NOT EXISTS calibration_bins (
                domain TEXT NOT NULL,
                bin_lower REAL NOT NULL,          -- e.g., 0.80
                bin_upper REAL NOT NULL,          -- e.g., 0.90
                total_count INTEGER DEFAULT 0,
                correct_count INTEGER DEFAULT 0,
                actual_accuracy REAL,
                PRIMARY KEY (domain, bin_lower)
            );"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Record a prediction for future calibration.
    pub fn record_prediction(
        &self,
        domain: &str,
        stated_confidence: f64,
        claim: &str,
        source_url: Option<&str>,
    ) -> Result<u64, crate::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO predictions (domain, stated_confidence, claim, source_url)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![domain, stated_confidence, claim, source_url],
        )?;
        Ok(conn.last_insert_rowid() as u64)
    }

    /// Verify a prediction outcome (correct or incorrect).
    pub fn verify_prediction(
        &self,
        prediction_id: u64,
        actually_correct: bool,
    ) -> Result<(), crate::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE predictions SET
                actually_correct = ?2,
                verified_at = datetime('now')
             WHERE id = ?1",
            rusqlite::params![prediction_id, actually_correct as i32],
        )?;

        // Update calibration bins
        let (domain, confidence): (String, f64) = conn.query_row(
            "SELECT domain, stated_confidence FROM predictions WHERE id = ?1",
            [prediction_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let bin_lower = (confidence * 10.0).floor() / 10.0; // Round down to nearest 0.1
        let bin_upper = bin_lower + 0.1;

        conn.execute(
            "INSERT INTO calibration_bins (domain, bin_lower, bin_upper, total_count, correct_count)
             VALUES (?1, ?2, ?3, 1, ?4)
             ON CONFLICT(domain, bin_lower) DO UPDATE SET
                total_count = total_count + 1,
                correct_count = correct_count + ?4,
                actual_accuracy = CAST(correct_count + ?4 AS REAL) / (total_count + 1)",
            rusqlite::params![domain, bin_lower, bin_upper, actually_correct as i32],
        )?;

        Ok(())
    }

    /// Calibrate a stated confidence score based on historical accuracy.
    /// Uses isotonic regression (simplified: linear interpolation between bins).
    pub fn calibrate(
        &self,
        domain: &str,
        stated_confidence: f64,
    ) -> Result<CalibratedConfidence, crate::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT bin_lower, actual_accuracy, total_count
             FROM calibration_bins
             WHERE domain = ?1 AND total_count >= 10
             ORDER BY bin_lower"
        )?;

        let bins: Vec<(f64, f64, i64)> = stmt.query_map([domain], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?.filter_map(|r| r.ok()).collect();

        if bins.is_empty() {
            // No calibration data yet; return stated confidence as-is
            return Ok(CalibratedConfidence {
                stated: stated_confidence,
                calibrated: stated_confidence,
                sample_size: 0,
                domain: domain.to_string(),
                has_calibration_data: false,
            });
        }

        // Isotonic regression: find the bin for this confidence and return
        // the actual accuracy for that bin, with linear interpolation.
        let calibrated = isotonic_interpolate(&bins, stated_confidence);
        let total_samples: i64 = bins.iter().map(|(_, _, n)| n).sum();

        Ok(CalibratedConfidence {
            stated: stated_confidence,
            calibrated,
            sample_size: total_samples as u64,
            domain: domain.to_string(),
            has_calibration_data: true,
        })
    }
}

/// Simple isotonic regression via linear interpolation between bins.
fn isotonic_interpolate(bins: &[(f64, f64, i64)], target: f64) -> f64 {
    if bins.is_empty() {
        return target;
    }

    // Find the two bins surrounding the target
    let mut below = None;
    let mut above = None;

    for &(lower, accuracy, _) in bins {
        if lower <= target {
            below = Some((lower, accuracy));
        }
        if lower > target && above.is_none() {
            above = Some((lower, accuracy));
        }
    }

    match (below, above) {
        (Some((l, la)), Some((u, ua))) => {
            // Linear interpolation
            let t = (target - l) / (u - l);
            la + t * (ua - la)
        }
        (Some((_, la)), None) => la,
        (None, Some((_, ua))) => ua,
        (None, None) => target,
    }
}

#[derive(Debug, Clone)]
pub struct CalibratedConfidence {
    pub stated: f64,
    pub calibrated: f64,
    pub sample_size: u64,
    pub domain: String,
    pub has_calibration_data: bool,
}
```

#### Acceptance Criteria

- [x] `cce.calibrate("tech", 0.85)` returns calibrated confidence based on historical bins
- [x] Output format: `"Confidence: 85% (calibrated: 82%, n=1,247)"`
- [x] Calibration bins update incrementally with each verified prediction
- [x] Minimum 10 samples per bin before calibration is applied
- [x] Per-domain calibration: medical and tech domains have separate tables
- [x] `hsx intelligence stats` shows calibration table summary
- [x] Isotonic regression produces monotonically increasing calibration curve

#### Pitfalls

- **Cold start**: Calibration is meaningless with < 50 total predictions. Show "uncalibrated" until sufficient data.
- **Verification challenge**: How do we know if a prediction was correct? Automated verification via follow-up searches, or user feedback. Start with user feedback (`hsx verify <claim-id> --correct/--incorrect`).
- **Domain boundary**: What is a "domain" for calibration? Start with broad categories (tech, medical, financial) rather than per-website.

---

## P6-E1-T7: Adversarial Content Shield (ACS)

| Field | Value |
|-------|-------|
| **ID** | `P6-E1-T7` |
| **Status** | `DONE` |
| **Priority** | P3 |
| **Description** | Implement the 4-layer adversarial content detection system: AI content detection, bot farm signal detection, source manipulation detection, and trust aggregation. Starts in "shadow mode" (flag but don't filter) for the first 30 days to collect accuracy data. |
| **PRD Ref** | 8.17, 35 (Adversarial Robustness & Trust Verification) |
| **Depends On** | `P6-E1-T1` (PIE for trust history), `P3-E1` (validation layer) |

#### Files to Create/Modify

| File | Action |
|------|--------|
| `crates/hsx-core/src/intelligence/acs.rs` | ACS engine |
| `crates/hsx-core/src/intelligence/acs/ai_detector.rs` | AI content detection |
| `crates/hsx-core/src/intelligence/acs/bot_detector.rs` | Bot farm signals |
| `crates/hsx-core/src/intelligence/acs/manipulation_detector.rs` | Source manipulation |
| `crates/hsx-core/src/intelligence/acs/trust_aggregator.rs` | Trust score aggregation |

#### Step-by-Step Implementation Guide

```rust
// crates/hsx-core/src/intelligence/acs.rs

pub struct AdversarialContentShield {
    ai_detector: ai_detector::AiContentDetector,
    bot_detector: bot_detector::BotFarmDetector,
    manipulation_detector: manipulation_detector::ManipulationDetector,
    mode: AcsMode,
    first_enabled_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AcsMode {
    Shadow,   // Flag but don't filter (first 30 days)
    Active,   // Flag and filter
    Disabled, // Off entirely
}

#[derive(Debug, Clone)]
pub struct AcsResult {
    pub trust_score: f64,               // 0.0-1.0 aggregate
    pub ai_generated_probability: f64,
    pub bot_farm_probability: f64,
    pub manipulation_probability: f64,
    pub flags: Vec<AcsFlag>,
    pub action: AcsAction,
}

#[derive(Debug, Clone)]
pub enum AcsAction {
    Include,                    // trust > 0.8
    IncludeWithWarning,        // 0.5 < trust < 0.8
    Exclude { reason: String }, // trust < 0.5
    FlagAdversarial,           // trust < 0.2
}

impl AdversarialContentShield {
    pub fn analyze(&self, content: &str, source: &Source) -> AcsResult {
        // Layer 1: AI Content Detection
        let ai_prob = self.ai_detector.detect(content);

        // Layer 2: Bot Farm Signals
        let bot_prob = self.bot_detector.analyze(source);

        // Layer 3: Manipulation Detection
        let manip_prob = self.manipulation_detector.check(content, source);

        // Layer 4: Trust Aggregation
        let trust_score = 1.0 - ai_prob.max(bot_prob).max(manip_prob);

        let action = match self.mode {
            AcsMode::Shadow => AcsAction::Include, // shadow mode: always include
            AcsMode::Disabled => AcsAction::Include,
            AcsMode::Active => {
                if trust_score > 0.8 {
                    AcsAction::Include
                } else if trust_score > 0.5 {
                    AcsAction::IncludeWithWarning
                } else if trust_score > 0.2 {
                    AcsAction::Exclude {
                        reason: format!("Low trust score: {:.2}", trust_score),
                    }
                } else {
                    AcsAction::FlagAdversarial
                }
            }
        };

        let mut flags = Vec::new();
        if ai_prob > 0.7 { flags.push(AcsFlag::LikelyAiGenerated); }
        if bot_prob > 0.7 { flags.push(AcsFlag::LikelyBotFarm); }
        if manip_prob > 0.7 { flags.push(AcsFlag::LikelyManipulated); }

        AcsResult { trust_score, ai_generated_probability: ai_prob, bot_farm_probability: bot_prob, manipulation_probability: manip_prob, flags, action }
    }
}

// crates/hsx-core/src/intelligence/acs/ai_detector.rs

pub struct AiContentDetector;

impl AiContentDetector {
    /// Detect AI-generated content using statistical analysis.
    /// Returns probability 0.0-1.0.
    pub fn detect(&self, content: &str) -> f64 {
        let perplexity_score = self.burstiness_analysis(content);
        let vocabulary_score = self.vocabulary_diversity(content);
        let sentence_variance = self.sentence_length_variance(content);

        // AI text tends to have: low burstiness, uniform vocabulary, regular sentence length
        let ai_signals = [
            (1.0 - perplexity_score) * 0.4,   // Low burstiness = AI
            (1.0 - vocabulary_score) * 0.3,    // Low vocabulary diversity = AI
            (1.0 - sentence_variance) * 0.3,   // Low sentence variance = AI
        ];

        ai_signals.iter().sum::<f64>().clamp(0.0, 1.0)
    }

    /// Burstiness: human text has variable complexity; AI text is uniform.
    fn burstiness_analysis(&self, content: &str) -> f64 {
        let sentences: Vec<&str> = content.split(|c: char| c == '.' || c == '!' || c == '?')
            .filter(|s| s.len() > 10)
            .collect();

        if sentences.len() < 3 { return 0.5; }

        let lengths: Vec<f64> = sentences.iter().map(|s| s.split_whitespace().count() as f64).collect();
        let mean = lengths.iter().sum::<f64>() / lengths.len() as f64;
        let variance = lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / lengths.len() as f64;
        let cv = variance.sqrt() / mean; // Coefficient of variation

        // High CV = bursty = human; Low CV = uniform = AI
        (cv / 0.5).clamp(0.0, 1.0) // Normalize around expected human CV
    }

    /// Vocabulary diversity: unique words / total words (Type-Token Ratio).
    fn vocabulary_diversity(&self, content: &str) -> f64 {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() { return 0.5; }

        let unique: std::collections::HashSet<&str> = words.iter().cloned().collect();
        let ttr = unique.len() as f64 / words.len() as f64;

        // Human text typically has TTR of 0.4-0.7 for 500+ words
        ttr.clamp(0.0, 1.0)
    }

    /// Sentence length variance: humans write varied length; AI is more uniform.
    fn sentence_length_variance(&self, content: &str) -> f64 {
        let sentences: Vec<usize> = content
            .split(|c: char| c == '.' || c == '!' || c == '?')
            .filter(|s| s.len() > 5)
            .map(|s| s.split_whitespace().count())
            .collect();

        if sentences.len() < 3 { return 0.5; }

        let mean = sentences.iter().sum::<usize>() as f64 / sentences.len() as f64;
        let std_dev = (sentences.iter()
            .map(|&l| (l as f64 - mean).powi(2))
            .sum::<f64>() / sentences.len() as f64)
            .sqrt();

        // Normalize: std_dev / mean. Higher = more human-like
        (std_dev / mean.max(1.0)).clamp(0.0, 1.0)
    }
}
```

#### Acceptance Criteria

- [x] ACS starts in shadow mode for the first 30 days (flag but don't filter)
- [x] AI content detection produces probability 0.0-1.0 using burstiness, vocabulary diversity, and sentence variance
- [x] Bot farm detection checks domain age, publishing velocity, and cross-site duplication
- [x] Trust score aggregation: `trust = 1 - max(ai_prob, bot_prob, manip_prob)`
- [x] `trust > 0.8`: include normally; `0.5-0.8`: include with warning; `< 0.5`: exclude
- [x] `hsx search "query" --trust-verify` enables ACS
- [x] `hsx fetch <url> --check-ai` reports AI generation probability
- [x] Shadow mode logs all flags without affecting results
- [x] After 30 days, ACS auto-transitions to active mode (configurable)

#### Pitfalls

- **False positives**: Statistical AI detection is imperfect. Shadow mode is essential for tuning thresholds before enforcement.
- **Legitimate AI content**: Some valuable content (documentation, API references) is AI-assisted. Don't penalize well-structured, accurate content.
- **Performance overhead**: Running all 4 detection layers on every result adds latency. Make ACS opt-in (not default) or run it asynchronously after initial display.
- **Adversarial evasion**: Sophisticated AI text generators can mimic human burstiness. These statistical methods are a starting point, not a complete solution.
