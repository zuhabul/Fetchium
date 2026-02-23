//! Extract Agent — fetches URLs and runs CEP content extraction (PRD §8.8).

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use crate::error::HsxError;
use crate::extract::pipeline::extract;
use crate::http::client::HttpClient;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::{AgentMessage, AgentReceiver, AgentSender, AgentType, AmrsSource};

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
        hasher.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[async_trait]
impl Agent for ExtractAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Extract
    }

    async fn run(&self, mut rx: AgentReceiver, tx: AgentSender) -> Result<(), HsxError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnExtract { urls, query: _ } => {
                    let _ = tx
                        .send(AgentMessage::ProgressUpdate {
                            agent_type: AgentType::Extract,
                            message: format!("Extracting {} URLs...", urls.len()),
                            progress: 0.0,
                        })
                        .await;

                    let mut sources = Vec::new();
                    let mut handles = Vec::new();

                    for url in urls {
                        let client = self.http_client.clone();
                        handles.push(tokio::spawn(async move {
                            let html = client.fetch_text(&url).await.unwrap_or_default();
                            (url, html)
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

                        let _ = tx
                            .send(AgentMessage::ProgressUpdate {
                                agent_type: AgentType::Extract,
                                message: format!("Extracted {} sources", i + 1),
                                progress: (i + 1) as f64 / 10.0,
                            })
                            .await;
                    }

                    let _ = tx
                        .send(AgentMessage::ExtractComplete { sources })
                        .await;
                }
                AgentMessage::Shutdown => break,
                _ => {}
            }
        }
        Ok(())
    }
}
