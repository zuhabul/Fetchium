//! Source Genealogy Tracker (SGT) — PRD §8.15.
//!
//! Traces claim provenance through citation chains to the primary source.
//! Detects "mutations" where claims are altered as they propagate.
//! Computes trust cascade scores (trust degrades with each hop).
//!
//! Uses bigram-Jaccard similarity (from `intelligence::string_similarity`) for
//! mutation detection — no external `strsim` dependency required.

use crate::intelligence::string_similarity;

// ─── Types ───────────────────────────────────────────────────────────────────

/// A single node in a citation chain.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GenealogyNode {
    pub url: String,
    pub title: String,
    pub claim_text: String,
    /// Trust at this depth: 1.0 − (depth × 0.15).
    pub trust_score: f64,
    /// Depth in the chain (0 = original, higher = further from original).
    pub depth: usize,
}

/// Severity of a detected claim mutation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationSeverity {
    /// Similarity ≥ 0.8 — minor phrasing change.
    Low,
    /// 0.5 ≤ similarity < 0.8 — significant rewording.
    Medium,
    /// similarity < 0.5 — claim substantially altered.
    High,
}

/// A mutation detected between two adjacent chain nodes.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Mutation {
    /// Index of the source node (earlier in chain).
    pub from_depth: usize,
    /// Index of the citing node (later in chain).
    pub to_depth: usize,
    pub original_claim: String,
    pub mutated_claim: String,
    pub similarity: f64,
    pub severity: MutationSeverity,
    pub description: String,
}

/// A complete citation chain with mutations and trust cascade.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CitationChain {
    /// Ordered from original claim (index 0) to most recent cite.
    pub nodes: Vec<GenealogyNode>,
    pub mutations: Vec<Mutation>,
    /// Trust cascade: one value per node, matching `nodes` order.
    pub trust_cascade: Vec<f64>,
    /// The last node — best guess for the primary source.
    pub primary_source: Option<GenealogyNode>,
}

// ─── Builder ─────────────────────────────────────────────────────────────────

/// Build a citation chain synchronously from pre-fetched `hops`.
///
/// Each `hop` is `(url, title, claim_text)` ordered from original → most recent.
/// Callers are responsible for fetching and ordering the hops.
pub fn build_chain(hops: Vec<(String, String, String)>) -> CitationChain {
    let nodes: Vec<GenealogyNode> = hops
        .into_iter()
        .enumerate()
        .map(|(depth, (url, title, claim_text))| GenealogyNode {
            url,
            title,
            claim_text,
            trust_score: (1.0 - depth as f64 * 0.15).clamp(0.1, 1.0),
            depth,
        })
        .collect();

    let mutations = detect_mutations(&nodes);
    let trust_cascade: Vec<f64> = nodes.iter().map(|n| n.trust_score).collect();
    let primary_source = nodes.last().cloned();

    CitationChain {
        nodes,
        mutations,
        trust_cascade,
        primary_source,
    }
}

// ─── Mutation detection ───────────────────────────────────────────────────────

/// Detect mutations between adjacent nodes using bigram-Jaccard similarity.
pub fn detect_mutations(nodes: &[GenealogyNode]) -> Vec<Mutation> {
    let mut mutations = Vec::new();

    for i in 0..nodes.len().saturating_sub(1) {
        let from = &nodes[i];
        let to = &nodes[i + 1];

        let similarity =
            string_similarity(&from.claim_text.to_lowercase(), &to.claim_text.to_lowercase());

        // Only flag when similarity drops below 0.95.
        if similarity < 0.95 {
            let severity = if similarity < 0.5 {
                MutationSeverity::High
            } else if similarity < 0.8 {
                MutationSeverity::Medium
            } else {
                MutationSeverity::Low
            };

            let description = describe_mutation(similarity, &severity);

            mutations.push(Mutation {
                from_depth: from.depth,
                to_depth: to.depth,
                // "from" is closer to origin; "to" is the citing version.
                original_claim: from.claim_text.clone(),
                mutated_claim: to.claim_text.clone(),
                similarity,
                severity,
                description,
            });
        }
    }

    mutations
}

fn describe_mutation(similarity: f64, severity: &MutationSeverity) -> String {
    match severity {
        MutationSeverity::Low => format!(
            "Minor phrasing change ({:.0}% similar) — meaning likely preserved.",
            similarity * 100.0
        ),
        MutationSeverity::Medium => format!(
            "Significant rewording ({:.0}% similar) — verify meaning is unchanged.",
            similarity * 100.0
        ),
        MutationSeverity::High => format!(
            "Claim substantially altered ({:.0}% similar) — original meaning may be lost.",
            similarity * 100.0
        ),
    }
}

// ─── Display helpers ──────────────────────────────────────────────────────────

impl CitationChain {
    /// Markdown-formatted genealogy summary.
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        out.push_str("## Citation Chain\n\n");

        for node in &self.nodes {
            out.push_str(&format!(
                "{}. **{}** (trust: {:.0}%)\n   - URL: {}\n   - Claim: \"{}\"\n\n",
                node.depth + 1,
                node.title,
                node.trust_score * 100.0,
                node.url,
                node.claim_text,
            ));
        }

        if !self.mutations.is_empty() {
            out.push_str("### Mutations Detected\n\n");
            for m in &self.mutations {
                out.push_str(&format!(
                    "- **{:?}** mutation (depth {} → {}): {}\n",
                    m.severity, m.from_depth, m.to_depth, m.description
                ));
            }
        }

        if let Some(primary) = &self.primary_source {
            out.push_str(&format!(
                "\n**Primary Source**: {} ({})\n",
                primary.title, primary.url
            ));
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chain(hops: &[(&str, &str, &str)]) -> CitationChain {
        build_chain(
            hops.iter()
                .map(|(u, t, c)| (u.to_string(), t.to_string(), c.to_string()))
                .collect(),
        )
    }

    #[test]
    fn trust_degrades_per_hop() {
        let chain = make_chain(&[
            ("https://original.com", "Original", "Rust is memory safe"),
            ("https://blog.com", "Blog", "Rust is memory safe"),
            ("https://tweet.com", "Tweet", "Rust is memory safe"),
        ]);
        assert!(chain.trust_cascade[0] > chain.trust_cascade[1]);
        assert!(chain.trust_cascade[1] > chain.trust_cascade[2]);
    }

    #[test]
    fn identical_claims_have_no_mutations() {
        let chain = make_chain(&[
            ("https://a.com", "A", "Rust is fast"),
            ("https://b.com", "B", "Rust is fast"),
        ]);
        assert!(chain.mutations.is_empty());
    }

    #[test]
    fn substantially_different_claims_are_high_severity() {
        let chain = make_chain(&[
            ("https://a.com", "A", "Rust provides memory safety without garbage collection"),
            ("https://b.com", "B", "Python is an interpreted scripting language for data science"),
        ]);
        let high = chain
            .mutations
            .iter()
            .any(|m| m.severity == MutationSeverity::High);
        assert!(high, "expected high-severity mutation");
    }

    #[test]
    fn primary_source_is_last_node() {
        let chain = make_chain(&[
            ("https://origin.com", "Origin", "claim"),
            ("https://cite.com", "Cite", "claim slightly modified"),
        ]);
        let ps = chain.primary_source.as_ref().unwrap();
        assert_eq!(ps.url, "https://cite.com");
    }

    #[test]
    fn markdown_output_not_empty() {
        let chain = make_chain(&[
            ("https://a.com", "A", "test claim"),
            ("https://b.com", "B", "slightly changed claim"),
        ]);
        let md = chain.to_markdown();
        assert!(md.contains("Citation Chain"), "md={md}");
    }
}
