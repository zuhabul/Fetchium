//! Ollama HTTP client — list models, chat completion, streaming (PRD §23).

use crate::ai::types::{AiConfig, ChatMessage, OllamaChatChunk, OllamaModel};
use crate::error::HsxError;
use futures::StreamExt;
use serde::Deserialize;

/// HTTP client for the local Ollama inference server.
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    /// Create a new client from the given AI config.
    pub fn new(config: &AiConfig) -> Self {
        let base_url = format!("{}:{}", config.ollama_host, config.ollama_port);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();
        Self { client, base_url }
    }

    /// Check whether Ollama is running and reachable.
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    /// List all locally available models from Ollama's `/api/tags`.
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, HsxError> {
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<OllamaModel>,
        }

        let resp = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama unreachable: {e}")))?;

        let tags: TagsResponse = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid Ollama /api/tags response: {e}")))?;

        Ok(tags.models)
    }

    /// Non-streaming chat completion. Returns the full response text.
    pub async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
    ) -> Result<String, HsxError> {
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": false,
            "options": { "temperature": temperature }
        });

        let resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama chat failed: {e}")))?;

        let chunk: OllamaChatChunk = resp
            .json()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid chat response: {e}")))?;

        Ok(chunk.message.content)
    }

    /// Streaming chat completion. Calls `on_chunk` for each token as it arrives.
    ///
    /// Returns the complete accumulated response when done.
    pub async fn chat_stream<F>(
        &self,
        model: &str,
        messages: &[ChatMessage],
        temperature: f32,
        mut on_chunk: F,
    ) -> Result<String, HsxError>
    where
        F: FnMut(&str),
    {
        let body = serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": { "temperature": temperature }
        });

        let resp = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama stream failed: {e}")))?;

        let mut full_response = String::new();
        let mut stream = resp.bytes_stream();

        // Ollama streams newline-delimited JSON — one complete JSON object per line.
        let mut buffer = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let bytes = chunk_result
                .map_err(|e| HsxError::AiUnavailable(format!("Stream read error: {e}")))?;
            buffer.extend_from_slice(&bytes);

            // Process all complete lines in the buffer.
            while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                let line: Vec<u8> = buffer.drain(..=newline_pos).collect();
                let line_str = String::from_utf8_lossy(&line);
                let trimmed = line_str.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if let Ok(chunk) = serde_json::from_str::<OllamaChatChunk>(trimmed) {
                    let token = &chunk.message.content;
                    on_chunk(token);
                    full_response.push_str(token);
                    if chunk.done {
                        return Ok(full_response);
                    }
                }
            }
        }

        Ok(full_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_builds_correct_base_url() {
        let config = AiConfig {
            ollama_host: "http://localhost".into(),
            ollama_port: 11434,
            ..Default::default()
        };
        let client = OllamaClient::new(&config);
        assert_eq!(client.base_url, "http://localhost:11434");
    }
}
