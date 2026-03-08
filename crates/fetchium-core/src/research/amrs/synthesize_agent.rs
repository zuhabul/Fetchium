//! Synthesize Agent — assembles final research report from verified findings (PRD §8.8).

use crate::error::FetchiumError;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::{
    AgentMessage, AgentReceiver, AgentSender, AgentType, AmrsFinding, AmrsSource, AuditEntry,
};
use async_trait::async_trait;

/// Synthesizes verified findings into a structured research report.
pub struct SynthesizeAgent;

impl SynthesizeAgent {
    pub fn new() -> Self {
        Self
    }

    /// Build the research report from findings and sources.
    fn build_report(query: &str, findings: &[AmrsFinding], sources: &[AmrsSource]) -> String {
        let mut report = format!("# Research Report: {query}\n\n");

        report.push_str("## Key Findings\n\n");
        if findings.is_empty() {
            report.push_str("No findings were retrieved for this query.\n");
        } else {
            for (i, finding) in findings.iter().enumerate().take(10) {
                let source_refs: Vec<String> = finding
                    .source_indices
                    .iter()
                    .map(|&i| format!("[{}]", i + 1))
                    .collect();
                report.push_str(&format!(
                    "{}. {} {} *(confidence: {:.0}%)*\n",
                    i + 1,
                    finding.claim,
                    source_refs.join(" "),
                    finding.confidence * 100.0,
                ));
            }
        }

        if !sources.is_empty() {
            report.push_str("\n## Sources\n\n");
            for (i, source) in sources.iter().enumerate() {
                report.push_str(&format!(
                    "[{}] **{}**  \n{}\n\n",
                    i + 1,
                    source.title,
                    source.url,
                ));
            }
        }

        report
    }
}

impl Default for SynthesizeAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Agent for SynthesizeAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Synthesize
    }

    async fn run(&self, mut rx: AgentReceiver, tx: AgentSender) -> Result<(), FetchiumError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnSynthesize {
                    findings,
                    sources,
                    query,
                } => {
                    let _ = tx
                        .send(AgentMessage::ProgressUpdate {
                            agent_type: AgentType::Synthesize,
                            message: "Synthesizing report...".into(),
                            progress: 0.9,
                        })
                        .await;

                    let report = Self::build_report(&query, &findings, &sources);

                    let audit_entry = AuditEntry {
                        timestamp: chrono::Utc::now(),
                        agent: AgentType::Synthesize,
                        action: "synthesize".into(),
                        detail: format!(
                            "Built report from {} findings and {} sources",
                            findings.len(),
                            sources.len()
                        ),
                    };

                    let _ = tx
                        .send(AgentMessage::SynthesisComplete {
                            report,
                            audit_entries: vec![audit_entry],
                        })
                        .await;
                }
                AgentMessage::Shutdown => break,
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_report_contains_query() {
        let report = SynthesizeAgent::build_report("Rust programming", &[], &[]);
        assert!(report.contains("Rust programming"));
        assert!(report.contains("## Key Findings"));
    }

    #[test]
    fn build_report_lists_sources() {
        let sources = vec![AmrsSource {
            url: "https://rust-lang.org".into(),
            title: "Rust Language".into(),
            content: "...".into(),
            content_hash: "abc".into(),
        }];
        let report = SynthesizeAgent::build_report("Rust", &[], &sources);
        assert!(report.contains("https://rust-lang.org"));
        assert!(report.contains("## Sources"));
    }
}
