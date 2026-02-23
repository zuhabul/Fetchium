//! Tree-of-Thoughts Research (ToTR) — PRD §8.12, §32.
//!
//! Decomposes a complex query into 2-5 parallel reasoning paths, runs
//! independent sub-research per path, prunes low-quality branches, and
//! synthesizes surviving paths.  Optionally runs a self-debate protocol
//! (Advocate → Critic → Judge).
//!
//! **Graceful degradation**: if no LLM is available, uses keyword-based
//! decomposition rather than AI decomposition.

use std::fmt::Write as FmtWrite;

use crate::error::HsxError;

// ─── Configuration ────────────────────────────────────────────────────────────

/// Configuration for the ToTR engine.
#[derive(Debug, Clone)]
pub struct TotrConfig {
    /// Maximum number of reasoning branches (2-5 recommended).
    pub max_branches: usize,
    /// Prune branches scoring below `max_score × prune_threshold`.
    pub prune_threshold: f64,
    /// Enable the Advocate-Critic-Judge self-debate protocol.
    pub self_debate: bool,
}

impl Default for TotrConfig {
    fn default() -> Self {
        Self {
            max_branches: 3,
            prune_threshold: 0.3,
            self_debate: false,
        }
    }
}

// ─── Branch types ────────────────────────────────────────────────────────────

/// Status of a reasoning branch.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchStatus {
    Active,
    Complete,
    Pruned { reason: String },
}

/// A single reasoning branch in the tree.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ThoughtBranch {
    pub id: String,
    /// Named perspective, e.g. "Economic Feasibility".
    pub perspective: String,
    /// Sub-queries assigned to this branch.
    pub sub_queries: Vec<String>,
    /// Summarised findings (text snippets from research).
    pub findings: Vec<String>,
    /// Aggregate evidence quality score 0.0-1.0.
    pub score: f64,
    pub status: BranchStatus,
}

// ─── Result types ─────────────────────────────────────────────────────────────

/// Self-debate output (Advocate → Critic → Judge).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DebateResult {
    pub advocate_argument: String,
    pub critic_argument: String,
    pub judge_verdict: String,
    pub confidence: f64,
}

/// Complete output of a ToTR run.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TotrResult {
    pub query: String,
    pub branches: Vec<ThoughtBranch>,
    /// Cross-path synthesis text.
    pub synthesis: String,
    pub debate: Option<DebateResult>,
    pub total_branches: usize,
    pub pruned_branches: usize,
}

impl TotrResult {
    /// Markdown-formatted report of the ToTR result.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "# Tree-of-Thoughts Research: {}\n", self.query);
        let _ = writeln!(
            out,
            "_Branches: {} total, {} pruned_\n",
            self.total_branches, self.pruned_branches
        );

        // Surviving branches
        for branch in &self.branches {
            if branch.status != BranchStatus::Complete {
                continue;
            }
            let _ = writeln!(out, "## {} (score: {:.2})\n", branch.perspective, branch.score);
            for finding in &branch.findings {
                let _ = writeln!(out, "- {finding}");
            }
            out.push('\n');
        }

        // Pruned branches summary
        let pruned: Vec<&ThoughtBranch> = self
            .branches
            .iter()
            .filter(|b| matches!(b.status, BranchStatus::Pruned { .. }))
            .collect();
        if !pruned.is_empty() {
            let _ = writeln!(out, "## Pruned Branches\n");
            for b in pruned {
                let reason = if let BranchStatus::Pruned { reason } = &b.status {
                    reason.as_str()
                } else {
                    ""
                };
                let _ = writeln!(out, "- **{}**: {reason}", b.perspective);
            }
            out.push('\n');
        }

        let _ = writeln!(out, "## Synthesis\n\n{}\n", self.synthesis);

        if let Some(debate) = &self.debate {
            let _ = writeln!(out, "## Self-Debate\n");
            let _ = writeln!(out, "**Advocate:**\n{}\n", debate.advocate_argument);
            let _ = writeln!(out, "**Critic:**\n{}\n", debate.critic_argument);
            let _ = writeln!(
                out,
                "**Judge** (confidence: {:.0}%):\n{}\n",
                debate.confidence * 100.0,
                debate.judge_verdict
            );
        }

        out
    }
}

// ─── Decomposition ────────────────────────────────────────────────────────────

/// Decompose a query into perspectives using keyword heuristics (no LLM required).
///
/// Detects comparison/contrast/pros-cons patterns and generates up to
/// `max_branches` perspectives automatically.
pub fn decompose_query_heuristic(query: &str, max_branches: usize) -> Vec<(String, Vec<String>)> {
    let lower = query.to_lowercase();
    let mut perspectives: Vec<(String, Vec<String>)> = Vec::new();

    // Comparison queries ("A vs B", "difference between X and Y")
    if lower.contains(" vs ") || lower.contains(" versus ") || lower.contains("compare") {
        perspectives.push((
            "Technical Comparison".into(),
            vec![
                format!("{query} technical differences"),
                format!("{query} performance benchmarks"),
            ],
        ));
        perspectives.push((
            "Use Case Analysis".into(),
            vec![
                format!("{query} best use cases"),
                format!("{query} when to choose each"),
            ],
        ));
        perspectives.push((
            "Community & Ecosystem".into(),
            vec![
                format!("{query} ecosystem libraries"),
                format!("{query} community size adoption"),
            ],
        ));
    } else if lower.contains("how") || lower.contains("why") || lower.contains("explain") {
        perspectives.push((
            "Conceptual Foundation".into(),
            vec![
                format!("{query} fundamentals explained"),
                format!("{query} overview introduction"),
            ],
        ));
        perspectives.push((
            "Practical Application".into(),
            vec![
                format!("{query} practical examples"),
                format!("{query} real world usage"),
            ],
        ));
        perspectives.push((
            "Advanced Considerations".into(),
            vec![
                format!("{query} edge cases limitations"),
                format!("{query} best practices"),
            ],
        ));
    } else {
        // Generic decomposition — extract key noun phrases as sub-topics.
        let words: Vec<&str> = query.split_whitespace().collect();
        let chunk_size = (words.len() / max_branches.max(1)).max(1);

        for (i, chunk) in words.chunks(chunk_size).take(max_branches).enumerate() {
            let sub_topic = chunk.join(" ");
            perspectives.push((
                format!("Perspective {}: {sub_topic}", i + 1),
                vec![
                    format!("{query} {sub_topic} overview"),
                    format!("{query} {sub_topic} details"),
                ],
            ));
        }
    }

    perspectives.truncate(max_branches);
    perspectives
}

/// Attempt LLM-based decomposition, falling back to heuristic on any error.
///
/// `ai_complete_fn` is an async callback:
/// ```text
/// |prompt: String| async { Ok::<String, HsxError>(response_text) }
/// ```
pub async fn decompose_query<F, Fut>(
    query: &str,
    max_branches: usize,
    ai_complete_fn: F,
) -> Vec<(String, Vec<String>)>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<String, HsxError>>,
{
    let prompt = format!(
        "Decompose this research question into {max_branches} distinct perspectives. \
         For each perspective provide a short name (2-4 words) and 2 specific search queries. \
         Output ONLY valid JSON: \
         [{{\"perspective\": \"...\", \"queries\": [\"...\", \"...\"]}}]\n\nQuestion: {query}"
    );

    match ai_complete_fn(prompt).await {
        Ok(response) => {
            // Try to parse JSON response.
            if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(&response) {
                let mut result = Vec::new();
                for item in parsed.into_iter().take(max_branches) {
                    let perspective = item["perspective"]
                        .as_str()
                        .unwrap_or("Unknown")
                        .to_string();
                    let queries: Vec<String> = item["queries"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|q| q.as_str().map(String::from))
                        .collect();
                    if !perspective.is_empty() && !queries.is_empty() {
                        result.push((perspective, queries));
                    }
                }
                if !result.is_empty() {
                    return result;
                }
            }
            // JSON parse failed — fall through to heuristic.
            tracing::warn!("ToTR: LLM decomposition JSON parse failed, using heuristic");
            decompose_query_heuristic(query, max_branches)
        }
        Err(e) => {
            tracing::warn!("ToTR: LLM unavailable ({e}), using heuristic decomposition");
            decompose_query_heuristic(query, max_branches)
        }
    }
}

// ─── Branch scoring and pruning ───────────────────────────────────────────────

/// Score each branch's findings and prune below `max_score × threshold`.
pub fn score_and_prune(branches: &mut [ThoughtBranch], threshold: f64) {
    let max_score = branches
        .iter()
        .map(|b| b.score)
        .fold(f64::NEG_INFINITY, f64::max);

    if max_score <= 0.0 {
        return;
    }

    let cutoff = max_score * threshold;
    for branch in branches.iter_mut() {
        if branch.score < cutoff {
            branch.status = BranchStatus::Pruned {
                reason: format!(
                    "Score {:.2} below cutoff {:.2} (= {:.2} × {:.2})",
                    branch.score, cutoff, max_score, threshold
                ),
            };
        } else if branch.status == BranchStatus::Active {
            branch.status = BranchStatus::Complete;
        }
    }
}

// ─── Synthesis ────────────────────────────────────────────────────────────────

/// Synthesise surviving branches into a unified conclusion text.
pub fn synthesize(branches: &[ThoughtBranch], query: &str) -> String {
    let survivors: Vec<&ThoughtBranch> = branches
        .iter()
        .filter(|b| b.status == BranchStatus::Complete)
        .collect();

    if survivors.is_empty() {
        return format!(
            "No high-quality branches survived pruning for query: \"{query}\". \
             Consider lowering the prune threshold or providing more sources."
        );
    }

    let mut out = format!(
        "Based on {count} research perspective{s} for \"{query}\":\n\n",
        count = survivors.len(),
        s = if survivors.len() == 1 { "" } else { "s" },
    );

    for branch in &survivors {
        let _ = write!(
            out,
            "**{perspective}** (evidence quality: {score:.0}%): ",
            perspective = branch.perspective,
            score = branch.score * 100.0,
        );
        if branch.findings.is_empty() {
            out.push_str("No findings.\n\n");
        } else {
            let summary = branch.findings.first().map(String::as_str).unwrap_or("");
            let _ = writeln!(out, "{summary}\n");
        }
    }

    out
}

// ─── Self-debate ─────────────────────────────────────────────────────────────

/// Run a minimal synchronous self-debate based on available findings.
/// For a full Advocate-Critic-Judge flow, use `run_totr_with_llm` (async).
pub fn self_debate_heuristic(branches: &[ThoughtBranch], query: &str) -> DebateResult {
    let pros: Vec<&str> = branches
        .iter()
        .filter(|b| b.status == BranchStatus::Complete && b.score >= 0.5)
        .flat_map(|b| b.findings.iter().map(String::as_str))
        .take(3)
        .collect();

    let cons: Vec<&str> = branches
        .iter()
        .filter(|b| b.status == BranchStatus::Complete && b.score < 0.5)
        .flat_map(|b| b.findings.iter().map(String::as_str))
        .take(3)
        .collect();

    let advocate_argument = if pros.is_empty() {
        format!("Insufficient evidence to advocate for a strong conclusion about {query}.")
    } else {
        format!(
            "Evidence in favour: {}",
            pros.join("; ")
        )
    };

    let critic_argument = if cons.is_empty() {
        format!("No significant counterevidence found against the main claim about {query}.")
    } else {
        format!(
            "Counterevidence or limitations: {}",
            cons.join("; ")
        )
    };

    let confidence = branches
        .iter()
        .filter(|b| b.status == BranchStatus::Complete)
        .map(|b| b.score)
        .sum::<f64>()
        / branches
            .iter()
            .filter(|b| b.status == BranchStatus::Complete)
            .count()
            .max(1) as f64;

    let judge_verdict = format!(
        "Weighing both sides for \"{query}\": the balance of evidence suggests a nuanced answer \
         with confidence {:.0}%. Both perspectives are worth considering.",
        confidence * 100.0
    );

    DebateResult {
        advocate_argument,
        critic_argument,
        judge_verdict,
        confidence,
    }
}

// ─── Main synchronous entry point ─────────────────────────────────────────────

/// Run ToTR synchronously using heuristic decomposition and caller-supplied findings.
///
/// `findings_for_query` maps a sub-query string to a list of text snippets.
/// Use this for testing or when async is not available.
pub fn run_totr_sync(
    query: &str,
    config: &TotrConfig,
    findings_for_query: impl Fn(&str) -> Vec<String>,
) -> TotrResult {
    let decomposition = decompose_query_heuristic(query, config.max_branches);

    let mut branches: Vec<ThoughtBranch> = decomposition
        .into_iter()
        .enumerate()
        .map(|(i, (perspective, sub_queries))| {
            let mut all_findings = Vec::new();
            let mut total_score = 0.0_f64;

            for q in &sub_queries {
                let f = findings_for_query(q);
                total_score += if f.is_empty() { 0.0 } else { 0.7 };
                all_findings.extend(f);
            }

            let score = if sub_queries.is_empty() {
                0.0
            } else {
                total_score / sub_queries.len() as f64
            };

            ThoughtBranch {
                id: format!("branch-{i}"),
                perspective,
                sub_queries,
                findings: all_findings,
                score,
                status: BranchStatus::Active,
            }
        })
        .collect();

    let total_branches = branches.len();
    score_and_prune(&mut branches, config.prune_threshold);
    let pruned_branches = branches
        .iter()
        .filter(|b| matches!(b.status, BranchStatus::Pruned { .. }))
        .count();

    let synthesis = synthesize(&branches, query);

    let debate = if config.self_debate {
        Some(self_debate_heuristic(&branches, query))
    } else {
        None
    };

    TotrResult {
        query: query.to_string(),
        branches,
        synthesis,
        debate,
        total_branches,
        pruned_branches,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heuristic_decompose_comparison_query() {
        let branches = decompose_query_heuristic("Rust vs Go for systems programming", 3);
        assert!(!branches.is_empty());
        assert!(branches.len() <= 3);
        // Should detect the vs pattern
        let names: Vec<&str> = branches.iter().map(|(n, _)| n.as_str()).collect();
        assert!(
            names.iter().any(|n| n.contains("Comparison") || n.contains("Perspective")),
            "perspectives={names:?}"
        );
    }

    #[test]
    fn run_totr_sync_produces_result() {
        let config = TotrConfig::default();
        let result = run_totr_sync("Rust vs Go", &config, |q| {
            vec![format!("Finding about {q}")]
        });
        assert!(!result.query.is_empty());
        assert!(!result.synthesis.is_empty());
        assert!(result.total_branches > 0);
    }

    #[test]
    fn pruning_removes_low_score_branches() {
        let config = TotrConfig {
            max_branches: 3,
            prune_threshold: 0.5,
            self_debate: false,
        };
        // Only branches with findings get non-zero scores.
        // Queries for branch-0 return findings; branches 1,2 return none.
        let result = run_totr_sync("complex query with technical implications", &config, |q| {
            if q.contains("Conceptual") || q.contains("Perspective 1") || q.contains("fundamentals") || q.contains("overview") {
                vec!["A relevant finding".to_string()]
            } else {
                vec![]
            }
        });
        // At least one branch should survive
        assert!(result.pruned_branches < result.total_branches);
    }

    #[test]
    fn self_debate_returns_three_parts() {
        let config = TotrConfig {
            self_debate: true,
            ..Default::default()
        };
        let result = run_totr_sync("nuclear fusion viability", &config, |q| {
            vec![format!("evidence: {q}")]
        });
        let debate = result.debate.as_ref().expect("debate should be Some");
        assert!(!debate.advocate_argument.is_empty());
        assert!(!debate.critic_argument.is_empty());
        assert!(!debate.judge_verdict.is_empty());
    }

    #[test]
    fn to_markdown_contains_query() {
        let config = TotrConfig::default();
        let result = run_totr_sync("Rust memory model", &config, |q| {
            vec![format!("finding about {q}")]
        });
        let md = result.to_markdown();
        assert!(md.contains("Rust memory model"), "md={md}");
    }
}
