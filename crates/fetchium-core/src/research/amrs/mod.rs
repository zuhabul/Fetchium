//! Adaptive Multi-Agent Research Swarm (AMRS) — PRD §8.8.
//!
//! Implements 4 specialized research agents that communicate via `tokio::sync::mpsc` channels:
//! - **SearchAgent**: Query dispatch + multi-hop follow-up detection
//! - **ExtractAgent**: URL fetching + CEP content extraction
//! - **VerifyAgent**: Cross-source contradiction detection + confidence scoring
//! - **SynthesizeAgent**: Evidence graph + final report assembly
//!
//! The `Coordinator` manages the full pipeline and adapts parallelism to the resource tier.

pub mod agent;
pub mod channel;
pub mod decompose;
pub mod extract_agent;
pub mod search_agent;
pub mod synthesize_agent;
pub mod verify_agent;

use crate::citation::evidence_graph::{EvidenceGraph, EvidenceGraphBuilder};
use crate::config::FetchiumConfig;
use crate::error::FetchiumError;
use crate::http::client::HttpClient;
use crate::types::ResourceTier;
use std::time::Duration;
use tokio::sync::mpsc;

use agent::Agent;
use channel::{AgentMessage, AgentType, AmrsContradiction, AmrsFinding, AmrsSource, AuditEntry};
use decompose::{decompose_query, QueryNode};
use extract_agent::ExtractAgent;
use search_agent::SearchAgent;
use synthesize_agent::SynthesizeAgent;
use verify_agent::VerifyAgent;

/// AMRS configuration derived from the machine's resource tier.
#[derive(Debug, Clone)]
pub struct AmrsConfig {
    /// Maximum multi-hop depth (default: 3 for Standard tier)
    pub max_depth: usize,
    /// Maximum concurrent agents
    pub max_agents: usize,
    /// Channel buffer size
    pub channel_buffer: usize,
    /// Global timeout for the entire run() in seconds
    pub timeout_secs: u64,
}

impl AmrsConfig {
    /// Create config from the detected resource tier.
    pub fn from_resource_tier(tier: &ResourceTier) -> Self {
        match tier {
            ResourceTier::Minimal => Self {
                max_depth: 1,
                max_agents: 1,
                channel_buffer: 32,
                timeout_secs: 60,
            },
            ResourceTier::Standard => Self {
                max_depth: 2,
                max_agents: 4,
                channel_buffer: 64,
                timeout_secs: 60,
            },
            ResourceTier::Performance => Self {
                max_depth: 3,
                max_agents: 8,
                channel_buffer: 128,
                timeout_secs: 120,
            },
            ResourceTier::Server => Self {
                max_depth: 5,
                max_agents: 16,
                channel_buffer: 256,
                timeout_secs: 300,
            },
        }
    }
}

impl Default for AmrsConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_agents: 4,
            channel_buffer: 64,
            timeout_secs: 60,
        }
    }
}

/// Full result of a deep research session.
#[derive(Debug)]
pub struct DeepResearchResult {
    pub report: String,
    pub evidence_graph: EvidenceGraph,
    pub contradictions: Vec<AmrsContradiction>,
    pub decomposition_tree: Vec<QueryNode>,
    pub audit_trail: Vec<AuditEntry>,
    pub sources_analyzed: usize,
    pub claims_verified: usize,
    pub depth_reached: usize,
}

/// Manages the AMRS pipeline lifecycle and agent coordination.
pub struct Coordinator {
    config: AmrsConfig,
    http_client: HttpClient,
    fetchium_config: FetchiumConfig,
}

impl Coordinator {
    pub fn new(
        config: AmrsConfig,
        http_client: HttpClient,
        fetchium_config: FetchiumConfig,
    ) -> Self {
        Self {
            config,
            http_client,
            fetchium_config,
        }
    }

    /// Execute a full deep research session.
    ///
    /// # Pipeline
    /// 1. Decompose query into sub-query tree
    /// 2. SearchAgent executes each sub-query (parallel up to max_agents)
    /// 3. ExtractAgent fetches and extracts all discovered URLs
    /// 4. VerifyAgent cross-validates sources and finds contradictions
    /// 5. SynthesizeAgent builds the final report + evidence graph
    /// 6. Optional: enhance report with AI synthesis
    pub async fn run(&mut self, query: &str) -> Result<DeepResearchResult, FetchiumError> {
        let global_timeout = Duration::from_secs(self.config.timeout_secs);
        match tokio::time::timeout(global_timeout, self.run_inner(query)).await {
            Ok(result) => result,
            Err(_) => Err(FetchiumError::OperationTimeout {
                operation: "deep research".into(),
                timeout_ms: self.config.timeout_secs * 1000,
                suggestion: format!(
                    "Deep research timed out after {}s. Try --timeout with a higher value.",
                    self.config.timeout_secs
                ),
            }),
        }
    }

    /// Inner implementation wrapped by the global timeout.
    async fn run_inner(&mut self, query: &str) -> Result<DeepResearchResult, FetchiumError> {
        let phase_timeout = Duration::from_secs(self.config.timeout_secs / 2);
        let mut audit: Vec<AuditEntry> = Vec::new();
        let decomposition = decompose_query(query, self.config.max_depth);

        audit.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            agent: AgentType::Search,
            action: "decompose".into(),
            detail: format!("Decomposed into {} sub-queries", decomposition.len()),
        });

        // ── Phase 1: Search ──────────────────────────────────────────
        let mut all_results: Vec<crate::types::ResultItem> = Vec::new();

        let search_nodes: Vec<&QueryNode> = decomposition
            .iter()
            .filter(|n| n.depth <= self.config.max_depth)
            .collect();

        let (coord_tx, mut coord_rx) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);
        let pending = search_nodes.len();

        for node in &search_nodes {
            let (agent_tx, agent_rx) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);
            let coordinator_tx = coord_tx.clone();
            let agent = SearchAgent::new(self.http_client.clone(), self.fetchium_config.clone());

            agent_tx
                .send(AgentMessage::SpawnSearch {
                    query: node.query.clone(),
                    depth: node.depth,
                })
                .await
                .map_err(|e| FetchiumError::Internal(e.to_string()))?;
            agent_tx.send(AgentMessage::Shutdown).await.ok();

            tokio::spawn(async move {
                let _ = agent.run(agent_rx, coordinator_tx).await;
            });
        }
        drop(coord_tx);

        let mut received = 0;
        let search_deadline = tokio::time::Instant::now() + phase_timeout;
        loop {
            match tokio::time::timeout_at(search_deadline, coord_rx.recv()).await {
                Ok(Some(msg)) => match msg {
                    AgentMessage::SearchComplete {
                        sub_query, results, ..
                    } => {
                        audit.push(AuditEntry {
                            timestamp: chrono::Utc::now(),
                            agent: AgentType::Search,
                            action: "search_complete".into(),
                            detail: format!("sub_query='{}' results={}", sub_query, results.len()),
                        });
                        all_results.extend(results);
                        received += 1;
                        if received >= pending {
                            break;
                        }
                    }
                    AgentMessage::ProgressUpdate {
                        agent_type,
                        message,
                        ..
                    } => {
                        tracing::debug!(?agent_type, %message, "Agent progress");
                    }
                    _ => {}
                },
                Ok(None) => break, // channel closed
                Err(_) => {
                    tracing::warn!(
                        "Search phase timed out after {:?} ({}/{} agents responded)",
                        phase_timeout,
                        received,
                        pending
                    );
                    break;
                }
            }
        }

        // ── Phase 2: Extract ─────────────────────────────────────────
        let urls: Vec<String> = all_results.iter().map(|r| r.url.clone()).collect();
        let (coord_tx2, mut coord_rx2) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);
        let (agent_tx2, agent_rx2) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);

        agent_tx2
            .send(AgentMessage::SpawnExtract {
                urls,
                query: query.to_string(),
            })
            .await
            .map_err(|e| FetchiumError::Internal(e.to_string()))?;
        agent_tx2.send(AgentMessage::Shutdown).await.ok();

        let extract_agent = ExtractAgent::new(self.http_client.clone());
        let coord_tx2_clone = coord_tx2.clone();
        tokio::spawn(async move {
            let _ = extract_agent.run(agent_rx2, coord_tx2_clone).await;
        });
        drop(coord_tx2);

        let mut extracted_sources: Vec<AmrsSource> = Vec::new();
        match tokio::time::timeout(phase_timeout, async {
            while let Some(msg) = coord_rx2.recv().await {
                if let AgentMessage::ExtractComplete { sources } = msg {
                    return sources;
                }
            }
            Vec::new()
        })
        .await
        {
            Ok(sources) => extracted_sources = sources,
            Err(_) => {
                tracing::warn!("Extract phase timed out after {:?}", phase_timeout);
            }
        }

        audit.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            agent: AgentType::Extract,
            action: "extract_complete".into(),
            detail: format!("Extracted {} sources", extracted_sources.len()),
        });

        // ── Phase 3: Verify ──────────────────────────────────────────
        let (coord_tx3, mut coord_rx3) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);
        let (agent_tx3, agent_rx3) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);

        agent_tx3
            .send(AgentMessage::SpawnVerify {
                sources: extracted_sources.clone(),
                query: query.to_string(),
            })
            .await
            .map_err(|e| FetchiumError::Internal(e.to_string()))?;
        agent_tx3.send(AgentMessage::Shutdown).await.ok();

        let verify_agent = VerifyAgent::new();
        let coord_tx3_clone = coord_tx3.clone();
        tokio::spawn(async move {
            let _ = verify_agent.run(agent_rx3, coord_tx3_clone).await;
        });
        drop(coord_tx3);

        let mut all_findings: Vec<AmrsFinding> = Vec::new();
        let mut all_contradictions: Vec<AmrsContradiction> = Vec::new();
        match tokio::time::timeout(phase_timeout, async {
            while let Some(msg) = coord_rx3.recv().await {
                if let AgentMessage::VerifyComplete {
                    findings,
                    contradictions,
                } = msg
                {
                    return (findings, contradictions);
                }
            }
            (Vec::new(), Vec::new())
        })
        .await
        {
            Ok((findings, contradictions)) => {
                all_findings = findings;
                all_contradictions = contradictions;
            }
            Err(_) => {
                tracing::warn!("Verify phase timed out after {:?}", phase_timeout);
            }
        }

        audit.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            agent: AgentType::Verify,
            action: "verify_complete".into(),
            detail: format!(
                "findings={} contradictions={}",
                all_findings.len(),
                all_contradictions.len()
            ),
        });

        // ── Phase 4: Synthesize ──────────────────────────────────────
        let (coord_tx4, mut coord_rx4) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);
        let (agent_tx4, agent_rx4) = mpsc::channel::<AgentMessage>(self.config.channel_buffer);

        agent_tx4
            .send(AgentMessage::SpawnSynthesize {
                findings: all_findings.clone(),
                sources: extracted_sources.clone(),
                query: query.to_string(),
            })
            .await
            .map_err(|e| FetchiumError::Internal(e.to_string()))?;
        agent_tx4.send(AgentMessage::Shutdown).await.ok();

        let synth_agent = SynthesizeAgent::new();
        let coord_tx4_clone = coord_tx4.clone();
        tokio::spawn(async move {
            let _ = synth_agent.run(agent_rx4, coord_tx4_clone).await;
        });
        drop(coord_tx4);

        let mut report = String::new();
        match tokio::time::timeout(phase_timeout, async {
            while let Some(msg) = coord_rx4.recv().await {
                if let AgentMessage::SynthesisComplete {
                    report: r,
                    audit_entries,
                } = msg
                {
                    return Some((r, audit_entries));
                }
            }
            None
        })
        .await
        {
            Ok(Some((r, entries))) => {
                report = r;
                audit.extend(entries);
            }
            Ok(None) => {
                tracing::warn!("Synthesize phase returned no report");
            }
            Err(_) => {
                tracing::warn!("Synthesize phase timed out after {:?}", phase_timeout);
            }
        }

        // ── Phase 5: AI Enhancement (optional) ──────────────────────
        let enhanced_report = self
            .enhance_report_with_ai(query, &report, &extracted_sources)
            .await;
        if !enhanced_report.is_empty() {
            report = enhanced_report;
        }

        // ── Build Evidence Graph ─────────────────────────────────────
        let mut egp = EvidenceGraphBuilder::new(query);
        for source in &extracted_sources {
            egp.add_source(&source.url, &source.title, &source.content, 0.8);
        }
        let evidence_graph = egp.build();

        let sources_analyzed = extracted_sources.len();
        let claims_verified = all_findings.len();

        Ok(DeepResearchResult {
            report,
            evidence_graph,
            contradictions: all_contradictions,
            decomposition_tree: decomposition,
            audit_trail: audit,
            sources_analyzed,
            claims_verified,
            depth_reached: self.config.max_depth,
        })
    }

    /// Enhance the heuristic report using AI synthesis.
    /// Falls back to the original report if AI is unavailable.
    async fn enhance_report_with_ai(
        &self,
        query: &str,
        heuristic_report: &str,
        sources: &[AmrsSource],
    ) -> String {
        use crate::ai::types::{AiConfig, ChatMessage};

        if sources.is_empty() {
            return String::new();
        }

        let ai_config = AiConfig::from_fetchium_config(&self.fetchium_config);
        if ai_config.providers.fallback_chain.is_empty() && ai_config.default_model.is_none() {
            return String::new();
        }

        // Build numbered source context
        let mut context = String::new();
        for (i, source) in sources.iter().enumerate().take(10) {
            let snippet: String = source.content.chars().take(1500).collect();
            context.push_str(&format!(
                "[{}] {} ({})\n{}\n\n",
                i + 1,
                source.title,
                source.url,
                snippet,
            ));
        }

        let user_prompt = format!(
            "QUERY: \"{query}\"\n\n\
             SOURCES:\n{context}\n\
             HEURISTIC ANALYSIS:\n{heuristic_report}\n\n\
             Write a thorough, well-structured report with:\n\
             1. An executive summary (2-3 sentences)\n\
             2. Key findings with [N] source citations\n\
             3. Analysis of agreements and contradictions between sources\n\
             4. Conclusion with confidence assessment\n\n\
             Use markdown formatting. Cite every claim with [N]."
        );

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: "You are a deep research analyst for Fetchium. Synthesize comprehensive, well-cited reports from provided sources.".into(),
            },
            ChatMessage {
                role: "user".into(),
                content: user_prompt,
            },
        ];

        let providers = ai_config.providers.clone();
        let mut noop = |_: &str| {};

        match tokio::time::timeout(
            Duration::from_secs(30),
            crate::ai::provider_client::chat_with_fallback(
                &messages, None, &ai_config, &providers, false, &mut noop,
            ),
        )
        .await
        {
            Ok(Ok(result)) => result.content,
            _ => String::new(), // fall back to heuristic report
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amrs_config_from_standard_tier() {
        let config = AmrsConfig::from_resource_tier(&ResourceTier::Standard);
        assert_eq!(config.max_depth, 2);
        assert_eq!(config.max_agents, 4);
    }

    #[test]
    fn amrs_config_from_server_tier() {
        let config = AmrsConfig::from_resource_tier(&ResourceTier::Server);
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.max_agents, 16);
    }
}
