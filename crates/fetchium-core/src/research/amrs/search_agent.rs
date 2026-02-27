//! Search Agent — dispatches queries to the search orchestrator (PRD §8.8).

use crate::config::HsxConfig;
use crate::error::HsxError;
use crate::http::client::HttpClient;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::{AgentMessage, AgentReceiver, AgentSender, AgentType};
use crate::search::orchestrator::{OrchestratorConfig, SearchOrchestrator};
use crate::types::ResultItem;
use async_trait::async_trait;
use std::collections::HashSet;
use std::time::Duration;

/// Executes search queries and discovers multi-hop follow-ups.
pub struct SearchAgent {
    http_client: HttpClient,
    hsx_config: HsxConfig,
}

impl SearchAgent {
    pub fn new(http_client: HttpClient, hsx_config: HsxConfig) -> Self {
        Self {
            http_client,
            hsx_config,
        }
    }

    /// Detect follow-up queries from result snippets not covered by the original query.
    fn detect_follow_ups(&self, query: &str, results: &[ResultItem]) -> Vec<String> {
        let query_words: HashSet<&str> = query.split_whitespace().collect();
        let mut follow_ups = Vec::new();

        for result in results.iter().take(5) {
            let new_terms: Vec<&str> = result
                .snippet
                .split_whitespace()
                .filter(|w| !query_words.contains(w) && w.len() > 5)
                .take(2)
                .collect();
            if !new_terms.is_empty() {
                follow_ups.push(format!("{} {}", query, new_terms.join(" ")));
            }
        }

        follow_ups.truncate(3);
        follow_ups
    }
}

#[async_trait]
impl Agent for SearchAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Search
    }

    async fn run(&self, mut rx: AgentReceiver, tx: AgentSender) -> Result<(), HsxError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnSearch { query, depth } => {
                    if tx
                        .send(AgentMessage::ProgressUpdate {
                            agent_type: AgentType::Search,
                            message: format!("Searching: {}", &query),
                            progress: 0.0,
                        })
                        .await
                        .is_err()
                    {
                        break; // coordinator gone
                    }

                    let orch_config = OrchestratorConfig::from_hsx_config(&self.hsx_config, 10);
                    let orchestrator =
                        SearchOrchestrator::new(self.http_client.clone(), orch_config);

                    // Timeout on search to prevent hanging on unresponsive backends
                    let results = match tokio::time::timeout(
                        Duration::from_secs(30),
                        orchestrator.search(&query, Some(10)),
                    )
                    .await
                    {
                        Ok(Ok(r)) => r,
                        Ok(Err(e)) => {
                            tracing::warn!("Search failed for '{}': {}", query, e);
                            Vec::new()
                        }
                        Err(_) => {
                            tracing::warn!("Search timed out for '{}'", query);
                            Vec::new()
                        }
                    };

                    let follow_ups = if depth > 0 {
                        self.detect_follow_ups(&query, &results)
                    } else {
                        Vec::new()
                    };

                    if tx
                        .send(AgentMessage::SearchComplete {
                            sub_query: query,
                            results,
                            follow_up_queries: follow_ups,
                        })
                        .await
                        .is_err()
                    {
                        break; // coordinator gone
                    }
                }
                AgentMessage::Shutdown => break,
                _ => {}
            }
        }
        Ok(())
    }
}
