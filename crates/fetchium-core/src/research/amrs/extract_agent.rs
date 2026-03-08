//! Extract Agent — fetches URLs and runs CEP content extraction (PRD §8.8).

use crate::error::FetchiumError;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::{
    AgentMessage, AgentReceiver, AgentSender, AgentType, AmrsSource,
};
use async_trait::async_trait;
use sha2::{Digest, Sha256};
use std::time::Duration;

/// Fetches URLs and extracts content using the CEP pipeline.
pub struct ExtractAgent {
    http_client: HttpClient,
}

impl ExtractAgent {
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    /// Compute SHA-256 hex digest of raw content.
    fn sha256_hex(data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

#[async_trait]
impl Agent for ExtractAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Extract
    }

    async fn run(&self, mut rx: AgentReceiver, tx: AgentSender) -> Result<(), FetchiumError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnExtract { urls, query: _ } => {
                    if tx
                        .send(AgentMessage::ProgressUpdate {
                            agent_type: AgentType::Extract,
                            message: format!("Extracting {} URLs...", urls.len()),
                            progress: 0.0,
                        })
                        .await
                        .is_err()
                    {
                        break; // coordinator gone
                    }

                    let mut sources = Vec::new();
                    let mut handles = Vec::new();
                    let total = urls.len();

                    for url in urls {
                        let client = self.http_client.clone();
                        handles.push(tokio::spawn(async move {
                            // Per-URL timeout to prevent hanging on unresponsive servers
                            match tokio::time::timeout(
                                Duration::from_secs(8),
                                client.fetch_text(&url),
                            )
                            .await
                            {
                                Ok(Ok(html)) => (url, html),
                                Ok(Err(_)) => (url, String::new()),
                                Err(_) => (url, String::new()),
                            }
                        }));
                    }

                    for (i, handle) in handles.into_iter().enumerate() {
                        if let Ok((url, html)) = handle.await {
                            if !html.is_empty() {
                                let ext = extract(&html, &url);
                                let hash = Self::sha256_hex(&html);
                                sources.push(AmrsSource {
                                    url,
                                    title: ext.title,
                                    content: ext.text,
                                    content_hash: hash,
                                });
                            }
                        }

                        if tx
                            .send(AgentMessage::ProgressUpdate {
                                agent_type: AgentType::Extract,
                                message: format!("Extracted {}/{} sources", i + 1, total),
                                progress: (i + 1) as f64 / total.max(1) as f64,
                            })
                            .await
                            .is_err()
                        {
                            break; // coordinator gone
                        }
                    }

                    if tx
                        .send(AgentMessage::ExtractComplete { sources })
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
