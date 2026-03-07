# Phase 4: AI Engine, Deep Research & MCP

> **Phase:** 4 of 8 | **Priority:** P1-P2 | **Duration:** Weeks 13-18
> **Depends on:** Phase 3 (Validation, Research & Citations) fully complete
> **PRD Reference:** `prd.md` v4.0.0 -- Sections 8.5, 8.8, 9, 10 (Modes C/D), 23, 24, 30
> **Epics:** 5 | **Tasks:** 8

---

## Phase 4 Summary

Phase 4 transforms Fetchium from a search-and-research tool into an AI-powered deep research platform with external interfaces. It adds:

1. **AI Preview Engine** -- Local LLM synthesis via Ollama with intelligent model routing and sandwich layout context assembly (PRD SS23)
2. **AMRS Deep Research** -- Adaptive Multi-Agent Research Swarm with 4 specialized agent types communicating over tokio channels (PRD SS8.8)
3. **Speculative Research Pipelining** -- Stream-first architecture that delivers results before all sources are processed (PRD SS8.5)
4. **MCP Server** -- Model Context Protocol server exposing 5 composite tools for Claude and MCP clients (PRD SS30)
5. **REST API** -- axum-based HTTP API for programmatic access from any language (PRD SS9)

---

## Prerequisites

All of the following must be `DONE` before starting any Phase 4 task:

| Dependency | Phase | What It Provides |
|-----------|-------|-----------------|
| P3-E1 (Validation + RAR) | Phase 3 | 6-layer validation pipeline, RAR self-correction loop |
| P3-E2 (Citations + EGP) | Phase 3 | Citation system (6 styles), Evidence Graph Protocol |
| P3-E3 (Research mode) | Phase 3 | `fetchium research` command, research pipeline |
| P1-E3 (QATBE + SCS + PDS) | Phase 1 | Token budgeting, content segmentation, progressive detail |
| P2-E3 (Parallel orchestrator) | Phase 2 | Full multi-backend search orchestrator |
| P2-E4 (HyperFusion ranking) | Phase 2 | 8-signal ranking engine |

---

## Epic 4.1: AI Preview Engine

> **PRD Sections:** SS23 (AI Preview Engine), SS10 Mode D (AI Preview)
> **Crate:** `fetchium-core` -- `src/ai/`
> **Priority:** P1 | **Tasks:** 2

### P4-E1-T1: Ollama Integration & Model Router

**ID:** `P4-E1-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 3-4 days

**Description:**
Build the Ollama HTTP client that communicates with a local Ollama instance at `localhost:11434`. Implement model listing, chat completion with streaming, model routing based on query complexity, and the sandwich layout context assembly algorithm inspired by Ms-PoE (Multi-scale Positional Encoding) to mitigate the "lost in the middle" problem.

**PRD References:**
- SS23 "AI Preview Engine" -- Architecture diagram: `Search Results -> QATBE -> Sandwich Layout Assembly -> Local LLM -> Citation Injection -> Output`
- SS23 "Sandwich Layout (Ms-PoE inspired)" -- High-confidence at start/end, low in middle
- SS23 "Model Integration" -- Ollama HTTP API at localhost:11434
- SS23 "Model Routing" -- Simple factual -> 1-3B, Standard -> 7-8B, Complex synthesis -> 13B+
- SS44 "Error Handling" -- `AiUnavailable` error type

**Files to create/modify:**
```
crates/fetchium-core/src/ai/
  mod.rs              -- Module root, re-exports
  ollama.rs           -- Ollama HTTP client (list models, chat, streaming)
  router.rs           -- Model routing by query complexity
  sandwich.rs         -- Sandwich layout context assembly
  prompt.rs           -- System prompt templates for synthesis
  types.rs            -- AI-specific types (AiConfig, ModelTier, ChatMessage, etc.)
```

**Dependencies:**
- P1-E3-T2 (QATBE) -- For token-budgeted extraction feeding the AI context
- P3-E2 (Citations) -- Citation injection into AI output
- P0-E1-T2 (Types) -- Core data types

**Step-by-step Rust implementation:**

**Step 1: Define AI types (`ai/types.rs`)**

```rust
use serde::{Deserialize, Serialize};

/// Which model tier to route to based on query complexity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    /// Simple factual queries: 1-3B models (phi-3-mini, qwen2.5:1.5b)
    Small,
    /// Standard queries: 7-8B models (llama3.2:8b, mistral:7b)
    Medium,
    /// Complex synthesis: 13B+ models (llama3.2:70b, mixtral)
    Large,
}

/// A single chat message in the Ollama API format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,     // "system", "user", "assistant"
    pub content: String,
}

/// Configuration for the AI engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub ollama_host: String,       // default: "http://localhost:11434"
    pub ollama_port: u16,          // default: 11434
    pub default_model: Option<String>,  // override auto-routing
    pub timeout_secs: u64,         // default: 120
    pub max_context_tokens: usize, // default: 4096
    pub temperature: f32,          // default: 0.3 for factual synthesis
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            ollama_host: "http://localhost".into(),
            ollama_port: 11434,
            default_model: None,
            timeout_secs: 120,
            max_context_tokens: 4096,
            temperature: 0.3,
        }
    }
}

/// Ollama model info returned by /api/tags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,           // bytes
    pub parameter_size: Option<String>,  // e.g., "7B", "13B"
    pub quantization_level: Option<String>,
}

/// A streamed chunk from Ollama chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatChunk {
    pub model: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

/// A source with its confidence score, used for sandwich layout ordering.
#[derive(Debug, Clone)]
pub struct RankedSource {
    pub index: usize,         // original position
    pub content: String,      // extracted text
    pub confidence: f64,      // 0.0-1.0 from ranking/validation
    pub url: String,
    pub title: String,
}

/// The assembled context after sandwich layout reordering.
#[derive(Debug, Clone)]
pub struct SandwichContext {
    pub system_prompt: String,
    pub user_context: String,   // sources in sandwich order
    pub source_map: Vec<usize>, // maps sandwich position -> original source index
    pub total_tokens: usize,
}
```

**Step 2: Build the Ollama HTTP client (`ai/ollama.rs`)**

```rust
use reqwest::Client;
use tokio::io::AsyncBufReadExt;
use futures::StreamExt;
use crate::ai::types::*;
use crate::error::HsxError;

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(config: &AiConfig) -> Self {
        let base_url = format!("{}:{}", config.ollama_host, config.ollama_port);
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_secs))
                .build()
                .expect("Failed to build HTTP client"),
            base_url,
        }
    }

    /// Check if Ollama is running and reachable.
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .is_ok()
    }

    /// List all locally available models from Ollama.
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>, HsxError> {
        let resp = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama unreachable: {e}")))?;

        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<OllamaModel>,
        }

        let tags: TagsResponse = resp.json().await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid Ollama response: {e}")))?;

        Ok(tags.models)
    }

    /// Non-streaming chat completion. Returns the full response.
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
            "options": {
                "temperature": temperature,
            }
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama chat failed: {e}")))?;

        let chunk: OllamaChatChunk = resp.json().await
            .map_err(|e| HsxError::AiUnavailable(format!("Invalid chat response: {e}")))?;

        Ok(chunk.message.content)
    }

    /// Streaming chat completion. Yields chunks via a callback.
    /// The callback receives each token chunk as it arrives.
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
            "options": {
                "temperature": temperature,
            }
        });

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| HsxError::AiUnavailable(format!("Ollama stream failed: {e}")))?;

        let mut full_response = String::new();
        let mut stream = resp.bytes_stream();

        // Ollama streams newline-delimited JSON
        let mut buffer = Vec::new();
        while let Some(chunk_result) = stream.next().await {
            let bytes = chunk_result
                .map_err(|e| HsxError::AiUnavailable(format!("Stream read error: {e}")))?;
            buffer.extend_from_slice(&bytes);

            // Process complete JSON lines from the buffer
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
```

**Step 3: Build the Model Router (`ai/router.rs`)**

```rust
use crate::ai::types::*;

/// Complexity signals extracted from a query for model routing.
#[derive(Debug)]
struct ComplexitySignals {
    word_count: usize,
    has_comparison: bool,       // "vs", "compare", "difference"
    has_synthesis: bool,        // "explain", "analyze", "summarize"
    has_multi_hop: bool,        // "and then", "because", "implications"
    question_depth: usize,      // number of sub-questions detected
    source_count: usize,        // how many sources will feed context
}

/// Determine the appropriate model tier for a query.
pub fn route_model(query: &str, source_count: usize) -> ModelTier {
    let signals = analyze_complexity(query, source_count);
    let score = compute_complexity_score(&signals);

    match score {
        s if s < 0.3 => ModelTier::Small,   // simple factual
        s if s < 0.7 => ModelTier::Medium,  // standard
        _ => ModelTier::Large,              // complex synthesis
    }
}

fn analyze_complexity(query: &str, source_count: usize) -> ComplexitySignals {
    let lower = query.to_lowercase();
    let words: Vec<&str> = query.split_whitespace().collect();

    ComplexitySignals {
        word_count: words.len(),
        has_comparison: lower.contains(" vs ")
            || lower.contains("compare")
            || lower.contains("difference between")
            || lower.contains("better"),
        has_synthesis: lower.contains("explain")
            || lower.contains("analyze")
            || lower.contains("summarize")
            || lower.contains("synthesize")
            || lower.contains("implications"),
        has_multi_hop: lower.contains("and then")
            || lower.contains("because")
            || lower.contains("implications")
            || lower.contains("how does")
            || lower.contains("why does"),
        question_depth: lower.matches('?').count()
            + lower.matches(" and ").count(),
        source_count,
    }
}

fn compute_complexity_score(signals: &ComplexitySignals) -> f64 {
    let mut score = 0.0;

    // Word count contribution (longer = more complex)
    score += match signals.word_count {
        0..=5 => 0.1,
        6..=12 => 0.2,
        13..=25 => 0.4,
        _ => 0.6,
    };

    if signals.has_comparison { score += 0.2; }
    if signals.has_synthesis { score += 0.2; }
    if signals.has_multi_hop { score += 0.15; }
    score += (signals.question_depth as f64) * 0.1;
    score += (signals.source_count as f64) * 0.02; // more sources = harder synthesis

    score.min(1.0)
}

/// Given available models and the desired tier, select the best model.
/// Falls back to the closest available model if the ideal tier is missing.
pub fn select_model(
    available: &[OllamaModel],
    tier: ModelTier,
    override_model: Option<&str>,
) -> Option<String> {
    // User override takes priority
    if let Some(name) = override_model {
        if available.iter().any(|m| m.name == name) {
            return Some(name.to_string());
        }
    }

    // Classify available models by parameter size
    let mut small: Vec<&OllamaModel> = Vec::new();
    let mut medium: Vec<&OllamaModel> = Vec::new();
    let mut large: Vec<&OllamaModel> = Vec::new();

    for model in available {
        let param_b = estimate_param_billions(model);
        match param_b {
            b if b <= 3.0 => small.push(model),
            b if b <= 10.0 => medium.push(model),
            _ => large.push(model),
        }
    }

    // Pick from target tier, fall back to next available
    let candidates = match tier {
        ModelTier::Small => [&small, &medium, &large],
        ModelTier::Medium => [&medium, &small, &large],
        ModelTier::Large => [&large, &medium, &small],
    };

    for bucket in candidates {
        if let Some(model) = bucket.first() {
            return Some(model.name.clone());
        }
    }

    None
}

/// Estimate parameter count in billions from model metadata.
fn estimate_param_billions(model: &OllamaModel) -> f64 {
    // Try to parse from parameter_size field (e.g., "7B", "13B")
    if let Some(ref ps) = model.parameter_size {
        let cleaned = ps.to_lowercase().replace('b', "");
        if let Ok(val) = cleaned.trim().parse::<f64>() {
            return val;
        }
    }

    // Fallback: estimate from file size (very rough)
    // Q4 quantized: ~0.5 GB per billion parameters
    let gb = model.size as f64 / (1024.0 * 1024.0 * 1024.0);
    gb / 0.5
}
```

**Step 4: Build the Sandwich Layout Algorithm (`ai/sandwich.rs`)**

```rust
use crate::ai::types::RankedSource;

/// Reorder sources using the sandwich layout (Ms-PoE inspired).
///
/// The "lost in the middle" problem: LLMs attend most strongly to the
/// beginning and end of the context window, paying less attention to the
/// middle. The sandwich layout places the HIGHEST confidence sources at
/// the start and end, with LOWER confidence sources in the middle.
///
/// Given sources ranked by confidence [1, 2, 3, 4, 5, 6] (1=best):
/// Sandwich order: [1, 3, 5, 6, 4, 2]
///   - Start: 1 (best)
///   - Middle: 3, 5, 6 (weakest attention zone)
///   - End: 4, 2 (second best)
///
/// This maximizes the LLM's attention on the most important evidence.
pub fn sandwich_layout(mut sources: Vec<RankedSource>) -> Vec<RankedSource> {
    if sources.len() <= 2 {
        return sources;
    }

    // Sort by confidence descending (best first)
    sources.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let n = sources.len();
    let mut result: Vec<Option<RankedSource>> = vec![None; n];

    // Interleave: odd-ranked (0, 2, 4...) go to the front,
    // even-ranked (1, 3, 5...) go to the back (reversed)
    let mut front_idx = 0;
    let mut back_idx = n - 1;

    for (i, source) in sources.into_iter().enumerate() {
        if i % 2 == 0 {
            // High confidence -> front positions (start of context)
            result[front_idx] = Some(source);
            front_idx += 1;
        } else {
            // Slightly lower -> back positions (end of context)
            result[back_idx] = Some(source);
            if back_idx > 0 {
                back_idx -= 1;
            }
        }
    }

    result.into_iter().flatten().collect()
}

/// Assemble the full context string from sandwich-ordered sources.
/// Returns the context string and a source index map for citation injection.
pub fn assemble_context(
    sources: &[RankedSource],
    token_budget: usize,
) -> (String, Vec<usize>) {
    let mut context = String::new();
    let mut source_map = Vec::new();
    let mut tokens_used = 0;

    for (pos, source) in sources.iter().enumerate() {
        // Rough token estimate: 1 token ~= 4 chars
        let source_tokens = source.content.len() / 4;

        if tokens_used + source_tokens > token_budget {
            // Truncate this source to fit remaining budget
            let remaining_tokens = token_budget - tokens_used;
            let remaining_chars = remaining_tokens * 4;
            let truncated = &source.content[..remaining_chars.min(source.content.len())];

            context.push_str(&format!(
                "[Source {}] {}\nURL: {}\n{}\n\n",
                pos + 1,
                source.title,
                source.url,
                truncated,
            ));
            source_map.push(source.index);
            break;
        }

        context.push_str(&format!(
            "[Source {}] {}\nURL: {}\n{}\n\n",
            pos + 1,
            source.title,
            source.url,
            source.content,
        ));
        source_map.push(source.index);
        tokens_used += source_tokens;
    }

    (context, source_map)
}
```

**Step 5: Build system prompts (`ai/prompt.rs`)**

```rust
/// System prompt for AI synthesis with citation requirements.
pub fn synthesis_system_prompt(query: &str, source_count: usize) -> String {
    format!(
        r#"You are a research synthesis assistant for Fetchium. Your task is to provide a clear, accurate, and well-cited answer to the user's query.

RULES:
1. Base your answer ONLY on the provided sources. Never fabricate information.
2. Cite every factual claim using [N] notation where N is the source number.
3. If sources disagree, note the contradiction and cite both sides.
4. If no source adequately answers the query, say so explicitly.
5. Be concise but thorough. Prefer specificity over vagueness.
6. Structure your answer with clear paragraphs. Use bullet points for lists.
7. End with a "Sources" section listing [N] URL pairs used.

You have {source_count} sources available. The user's query is: "{query}"

Respond with your synthesized answer now."#,
        source_count = source_count,
        query = query,
    )
}

/// Lighter prompt for simple factual queries.
pub fn factual_system_prompt(query: &str) -> String {
    format!(
        r#"Answer the following question concisely based on the provided sources. Cite with [N]. If unsure, say so.

Question: "{query}""#,
        query = query,
    )
}
```

**Step 6: Wire up module root (`ai/mod.rs`)**

```rust
pub mod ollama;
pub mod router;
pub mod sandwich;
pub mod prompt;
pub mod types;

pub use ollama::OllamaClient;
pub use router::{route_model, select_model};
pub use sandwich::{sandwich_layout, assemble_context};
pub use types::*;
```

**Acceptance criteria:**
- [ ] `OllamaClient::is_available()` returns `true` when Ollama is running, `false` otherwise
- [ ] `OllamaClient::list_models()` returns all locally installed models with name and size
- [ ] `OllamaClient::chat()` sends messages and returns the complete response string
- [ ] `OllamaClient::chat_stream()` yields token chunks via the callback as they arrive from Ollama
- [ ] `route_model()` classifies "what is Rust" as `Small`, "explain async Rust" as `Medium`, "compare Rust vs Go vs C++ for systems programming with tradeoffs" as `Large`
- [ ] `select_model()` picks from the correct tier and falls back gracefully if the target tier is empty
- [ ] `sandwich_layout()` places highest-confidence sources at positions 0 and N-1, lowest in the middle
- [ ] `assemble_context()` respects the token budget and truncates the last source if needed
- [ ] System prompts include citation instructions ([N] format) and source count
- [ ] All functions have `///` doc comments
- [ ] `cargo test` passes with unit tests for router scoring, sandwich ordering, and context assembly
- [ ] `cargo clippy` produces zero warnings

**Pitfalls:**
- Ollama's `/api/chat` streaming returns newline-delimited JSON, not SSE. Each line is a complete JSON object. Do not try to parse it as SSE.
- The `/api/tags` endpoint returns `{"models": [...]}`, not a flat array. Deserialize accordingly.
- Ollama may be slow to load large models on first request (cold start). Set an adequate timeout (120s+).
- Model `parameter_size` is not always present in older Ollama versions. Always have the file-size fallback heuristic.
- The sandwich layout must handle edge cases: 0 sources (empty), 1 source (passthrough), 2 sources (no reorder needed).
- Token estimation via `len() / 4` is rough. For production accuracy, reuse the token counter from P1-E3-T1. The rough estimate is acceptable for context budgeting.

---

### P4-E1-T2: `fetchium ai` Command

**ID:** `P4-E1-T2`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 2-3 days

**Description:**
Build the `fetchium ai` CLI command that orchestrates the full AI preview pipeline: search -> QATBE extraction -> sandwich layout -> Ollama chat -> citation injection -> output. Support a `--model` flag for manual model override, streaming output to the terminal, and graceful fallback when Ollama is not available.

**PRD References:**
- SS10 Mode D "AI Preview (`fetchium ai`)" -- `fetchium ai "explain WebDriver BiDi protocol"`
- SS23 "Architecture" -- `Search Results -> QATBE -> Sandwich Layout Assembly -> Local LLM -> Citation Injection -> Output`
- SS23 "Model Routing" -- Auto-select model based on query complexity
- SS11 "CLI Interface Design" -- clap derive command definitions

**Files to create/modify:**
```
crates/fetchium-cli/src/commands/ai.rs        -- The `fetchium ai` command implementation
crates/fetchium-cli/src/cli.rs                -- Add AiCommand variant to clap enum
crates/fetchium-core/src/ai/pipeline.rs       -- Full AI preview pipeline (search -> QATBE -> sandwich -> LLM -> citations)
```

**Dependencies:**
- P4-E1-T1 (Ollama integration) -- OllamaClient, router, sandwich layout
- P1-E2-T2 (Search orchestrator) -- For the search step
- P1-E3-T2 (QATBE) -- Token-budgeted extraction
- P3-E2 (Citation system) -- Citation injection into AI output
- P1-E7-T1 (Output formatters) -- Terminal output formatting

**Step-by-step Rust implementation:**

**Step 1: Define the AI pipeline (`ai/pipeline.rs`)**

```rust
use crate::ai::{OllamaClient, AiConfig, ChatMessage, RankedSource};
use crate::ai::router::{route_model, select_model};
use crate::ai::sandwich::{sandwich_layout, assemble_context};
use crate::ai::prompt::{synthesis_system_prompt, factual_system_prompt};
use crate::search::SearchOrchestrator;
use crate::token::qatbe::QatbeExtractor;
use crate::error::HsxError;

/// Result of the AI preview pipeline.
#[derive(Debug)]
pub struct AiPreviewResult {
    pub answer: String,
    pub model_used: String,
    pub sources_used: usize,
    pub streaming: bool,
    pub fallback: bool,    // true if Ollama was unavailable
}

/// Execute the full AI preview pipeline.
pub async fn run_ai_pipeline(
    query: &str,
    model_override: Option<&str>,
    token_budget: usize,
    streaming: bool,
    config: &AiConfig,
    orchestrator: &SearchOrchestrator,
    extractor: &QatbeExtractor,
) -> Result<AiPreviewResult, HsxError> {
    // Step 1: Search
    let search_results = orchestrator.search(query).await?;

    // Step 2: QATBE extraction on top results (limit to top 6-8)
    let top_n = search_results.results.len().min(8);
    let per_source_budget = token_budget / top_n.max(1);

    let mut ranked_sources = Vec::new();
    for (i, result) in search_results.results.iter().take(top_n).enumerate() {
        let extracted = extractor.extract(
            &result.url,
            query,
            per_source_budget,
        ).await.unwrap_or_default();

        ranked_sources.push(RankedSource {
            index: i,
            content: extracted.content,
            confidence: result.score.unwrap_or(0.5),
            url: result.url.clone(),
            title: result.title.clone(),
        });
    }

    // Step 3: Sandwich layout
    let ordered_sources = sandwich_layout(ranked_sources);
    let (context, source_map) = assemble_context(&ordered_sources, token_budget);

    // Step 4: Check Ollama availability
    let ollama = OllamaClient::new(config);

    if !ollama.is_available().await {
        // Fallback: return search results without AI synthesis
        return Ok(AiPreviewResult {
            answer: format!(
                "AI synthesis unavailable (Ollama not running at {}:{}).\n\n\
                 Search results for \"{}\":\n\n{}",
                config.ollama_host, config.ollama_port, query,
                format_fallback_results(&ordered_sources),
            ),
            model_used: "none".into(),
            sources_used: ordered_sources.len(),
            streaming: false,
            fallback: true,
        });
    }

    // Step 5: Route model
    let available_models = ollama.list_models().await?;
    let tier = route_model(query, ordered_sources.len());
    let model_name = select_model(&available_models, tier, model_override)
        .ok_or_else(|| HsxError::AiUnavailable(
            "No models available in Ollama. Run `ollama pull llama3.2` first.".into()
        ))?;

    // Step 6: Build messages
    let system_prompt = synthesis_system_prompt(query, ordered_sources.len());
    let messages = vec![
        ChatMessage { role: "system".into(), content: system_prompt },
        ChatMessage { role: "user".into(), content: format!(
            "Sources:\n\n{}\n\nAnswer the query: \"{}\"",
            context, query
        )},
    ];

    // Step 7: Call Ollama (streaming or not)
    let answer = if streaming {
        ollama.chat_stream(
            &model_name,
            &messages,
            config.temperature,
            |chunk| {
                // Print each chunk to stdout immediately
                use std::io::Write;
                print!("{}", chunk);
                let _ = std::io::stdout().flush();
            },
        ).await?
    } else {
        ollama.chat(&model_name, &messages, config.temperature).await?
    };

    Ok(AiPreviewResult {
        answer,
        model_used: model_name,
        sources_used: ordered_sources.len(),
        streaming,
        fallback: false,
    })
}

fn format_fallback_results(sources: &[RankedSource]) -> String {
    sources.iter().enumerate().map(|(i, s)| {
        format!("[{}] {} (confidence: {:.2})\n    {}\n    {}\n",
            i + 1, s.title, s.confidence, s.url,
            s.content.chars().take(200).collect::<String>(),
        )
    }).collect::<Vec<_>>().join("\n")
}
```

**Step 2: Add clap command definition (modify `cli.rs`)**

```rust
/// AI-powered answer synthesis using local LLM
#[derive(Debug, clap::Args)]
pub struct AiCommand {
    /// The query to answer with AI synthesis
    pub query: String,

    /// Override model selection (e.g., "llama3.2:8b", "mistral:7b")
    #[arg(long)]
    pub model: Option<String>,

    /// Token budget for context assembly (default: 4096)
    #[arg(long, default_value = "4096")]
    pub budget: usize,

    /// Disable streaming output (wait for full response)
    #[arg(long)]
    pub no_stream: bool,

    /// Maximum number of sources to include in context
    #[arg(long, default_value = "8")]
    pub max_sources: usize,

    /// Output format: text, json, markdown
    #[arg(long, default_value = "text")]
    pub format: String,
}
```

**Step 3: Implement the command handler (`commands/ai.rs`)**

```rust
use crate::cli::AiCommand;
use hsx_core::ai::pipeline::run_ai_pipeline;
use hsx_core::ai::AiConfig;
use hsx_core::config::Config;

pub async fn handle_ai_command(args: AiCommand, config: &Config) -> anyhow::Result<()> {
    let ai_config = AiConfig::from_config(config);
    let orchestrator = /* obtain from app state */;
    let extractor = /* obtain from app state */;

    let spinner = indicatif::ProgressBar::new_spinner();
    spinner.set_message(format!("Searching for: {}", &args.query));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let streaming = !args.no_stream;

    if streaming {
        spinner.finish_and_clear();
        // Header before streaming starts
        println!("{}", console::style("AI Answer").bold().cyan());
        println!("{}", console::style("─".repeat(60)).dim());
    }

    let result = run_ai_pipeline(
        &args.query,
        args.model.as_deref(),
        args.budget,
        streaming,
        &ai_config,
        &orchestrator,
        &extractor,
    ).await?;

    if !streaming {
        spinner.finish_and_clear();
    }

    // Print newline after streaming output
    if streaming && !result.fallback {
        println!("\n");
    }

    // Print metadata footer
    println!("{}", console::style("─".repeat(60)).dim());
    println!(
        "{} {} | {} {} | {} {}",
        console::style("Model:").dim(),
        result.model_used,
        console::style("Sources:").dim(),
        result.sources_used,
        console::style("Fallback:").dim(),
        if result.fallback { "yes" } else { "no" },
    );

    if result.fallback {
        eprintln!(
            "\n{} Install Ollama (https://ollama.ai) and run `ollama pull llama3.2` for AI synthesis.",
            console::style("Tip:").yellow().bold(),
        );
    }

    Ok(())
}
```

**Acceptance criteria:**
- [ ] `fetchium ai "what is Rust"` searches, extracts, assembles context, sends to Ollama, and prints a cited answer
- [ ] `fetchium ai "query" --model mistral:7b` uses the specified model regardless of routing
- [ ] Streaming mode (default) prints tokens as they arrive from Ollama with no buffering delay
- [ ] `fetchium ai "query" --no-stream` waits for the full response before printing
- [ ] When Ollama is not running, the command prints a fallback message with raw search results and a helpful tip to install Ollama
- [ ] When Ollama is running but has no models, the command prints an error suggesting `ollama pull`
- [ ] The `--budget` flag controls how many tokens of context are assembled
- [ ] The answer contains `[N]` citation markers that correspond to the source URLs
- [ ] A metadata footer shows model used, source count, and whether fallback was triggered
- [ ] `cargo build` compiles with no errors
- [ ] `cargo clippy` produces zero warnings

**Pitfalls:**
- Streaming and spinner conflict: the spinner must be cleared before streaming output begins, otherwise the spinner overwrites the streamed tokens.
- `std::io::stdout().flush()` is essential after each `print!()` in the streaming callback, otherwise output buffers and appears in chunks.
- The search step is async and may take 2-5 seconds. Show the spinner only during the search/extraction phase, not during streaming.
- Fallback mode must still be useful: format search results with titles, URLs, and snippets so the user gets value even without Ollama.
- The `--model` flag value must match Ollama's exact model name (including tag, e.g., `llama3.2:8b` not just `llama3.2`).

---

## Epic 4.2: Deep Research / AMRS

> **PRD Sections:** SS8.8 (AMRS), SS10 Mode C (Deep Research)
> **Crate:** `fetchium-core` -- `src/research/`
> **Priority:** P1-P2 | **Tasks:** 2

### P4-E2-T1: AMRS Multi-Agent Architecture

**ID:** `P4-E2-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 5-6 days

**Description:**
Implement the Adaptive Multi-Agent Research Swarm (AMRS) as defined in PRD SS8.8. Build 4 specialized agent types (Search, Extract, Verify, Synthesize) that communicate via tokio `mpsc` channels. The coordinator spawns agents dynamically based on the machine's resource tier. Implement query decomposition, multi-hop follow-ups, and cross-source contradiction detection.

**PRD References:**
- SS8.8 "Adaptive Multi-Agent Research Swarm (AMRS)" -- 4 agent types, dynamic spawning, resource-aware
- SS10 Mode C "Deep Research Mode" -- Query decomposition tree, multi-hop follow-ups, contradiction detection, evidence graph, audit trail
- SS8.7 "Evidence Graph Protocol" -- Graph-based evidence linking with cryptographic hashes
- SS13 "Machine Resource Awareness Engine" -- Resource tier determines agent parallelism

**Files to create/modify:**
```
crates/fetchium-core/src/research/
  amrs/
    mod.rs              -- Module root, Coordinator
    agent.rs            -- Agent trait + AgentMessage enum
    search_agent.rs     -- Search Agent: query decomposition, backend orchestration
    extract_agent.rs    -- Extract Agent: QATBE + CEP extraction
    verify_agent.rs     -- Verify Agent: cross-source validation, contradiction detection
    synthesize_agent.rs -- Synthesize Agent: evidence graph construction, report generation
    decompose.rs        -- Query decomposition tree
    channel.rs          -- Channel types and message definitions
```

**Dependencies:**
- P1-E2-T2 (Search orchestrator) -- Search Agent delegates to this
- P1-E3-T2 (QATBE) -- Extract Agent uses this
- P3-E1 (Validation + RAR) -- Verify Agent uses validation pipeline
- P3-E2 (Citations + EGP) -- Synthesize Agent builds evidence graphs
- P1-E1-T2 (Content extraction) -- Extract Agent uses CEP

**Step-by-step Rust implementation:**

**Step 1: Define channel messages and agent trait (`amrs/channel.rs` and `amrs/agent.rs`)**

```rust
// channel.rs
use tokio::sync::mpsc;
use crate::types::{SearchResult, ExtractedContent, EvidenceGraph, Source};

/// Messages flowing between agents through the coordinator.
#[derive(Debug, Clone)]
pub enum AgentMessage {
    // Search Agent -> Coordinator
    SearchResults {
        sub_query: String,
        results: Vec<SearchResult>,
        follow_up_queries: Vec<String>,  // multi-hop queries discovered
    },

    // Extract Agent -> Coordinator
    ExtractedContent {
        url: String,
        content: ExtractedContent,
        source_hash: String,  // SHA-256 of raw content
    },

    // Verify Agent -> Coordinator
    VerificationResult {
        claim: String,
        supported: bool,
        contradictions: Vec<Contradiction>,
        confidence: f64,
    },

    // Synthesize Agent -> Coordinator
    SynthesisReady {
        report: String,
        evidence_graph: EvidenceGraph,
        audit_entries: Vec<AuditEntry>,
    },

    // Coordinator -> Agents
    SpawnSearch { query: String, depth: usize },
    SpawnExtract { urls: Vec<String>, query: String },
    SpawnVerify { claims: Vec<String>, sources: Vec<Source> },
    SpawnSynthesize { findings: Vec<Finding>, sources: Vec<Source> },

    // Control
    Shutdown,
    ProgressUpdate { agent_type: AgentType, message: String, progress: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    Search,
    Extract,
    Verify,
    Synthesize,
}

#[derive(Debug, Clone)]
pub struct Contradiction {
    pub claim: String,
    pub source_a: String,
    pub source_b: String,
    pub source_a_says: String,
    pub source_b_says: String,
    pub severity: f64,  // 0.0-1.0
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub agent: AgentType,
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub claim: String,
    pub confidence: f64,
    pub sources: Vec<usize>,
    pub evidence_type: String,
}

pub type AgentSender = mpsc::Sender<AgentMessage>;
pub type AgentReceiver = mpsc::Receiver<AgentMessage>;
```

```rust
// agent.rs
use async_trait::async_trait;
use crate::research::amrs::channel::*;
use crate::error::HsxError;

/// Trait that all AMRS agents implement.
#[async_trait]
pub trait Agent: Send + Sync {
    /// The type of this agent.
    fn agent_type(&self) -> AgentType;

    /// Run the agent's main loop, processing messages from `rx`
    /// and sending results to `tx` (back to coordinator).
    async fn run(
        &self,
        rx: AgentReceiver,
        tx: AgentSender,
    ) -> Result<(), HsxError>;
}
```

**Step 2: Build the Query Decomposition Tree (`amrs/decompose.rs`)**

```rust
/// A node in the query decomposition tree.
#[derive(Debug, Clone)]
pub struct QueryNode {
    pub query: String,
    pub depth: usize,
    pub parent: Option<usize>,   // index into tree's node list
    pub children: Vec<usize>,
    pub status: QueryStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryStatus {
    Pending,
    InProgress,
    Complete,
    Failed(String),
}

/// Decompose a complex query into sub-queries.
/// Returns a tree of queries to execute in order.
pub fn decompose_query(query: &str, max_depth: usize) -> Vec<QueryNode> {
    let mut nodes = Vec::new();

    // Root query
    nodes.push(QueryNode {
        query: query.to_string(),
        depth: 0,
        parent: None,
        children: Vec::new(),
        status: QueryStatus::Pending,
    });

    if max_depth == 0 {
        return nodes;
    }

    // Heuristic decomposition: split on comparison keywords
    let lower = query.to_lowercase();

    // Pattern: "A vs B" or "compare A and B"
    if lower.contains(" vs ") || lower.contains("compare") {
        let parts = split_comparison(query);
        for part in parts {
            let child_idx = nodes.len();
            nodes.push(QueryNode {
                query: part,
                depth: 1,
                parent: Some(0),
                children: Vec::new(),
                status: QueryStatus::Pending,
            });
            nodes[0].children.push(child_idx);
        }
    }

    // Pattern: multi-faceted ("implications", "pros and cons", "advantages and disadvantages")
    if lower.contains("implications") || lower.contains("pros and cons") {
        // Generate aspect-specific sub-queries
        let aspects = extract_aspects(query);
        for aspect in aspects {
            let child_idx = nodes.len();
            nodes.push(QueryNode {
                query: aspect,
                depth: 1,
                parent: Some(0),
                children: Vec::new(),
                status: QueryStatus::Pending,
            });
            nodes[0].children.push(child_idx);
        }
    }

    nodes
}

fn split_comparison(query: &str) -> Vec<String> {
    // Split on " vs " or " versus " or "compare X and Y"
    let parts: Vec<&str> = if query.to_lowercase().contains(" vs ") {
        query.split(" vs ").collect()
    } else if query.to_lowercase().contains(" versus ") {
        query.split(" versus ").collect()
    } else {
        // "compare A and B" -> ["A", "B"]
        let stripped = query.to_lowercase()
            .replace("compare ", "")
            .replace("comparing ", "");
        return stripped.split(" and ")
            .map(|s| s.trim().to_string())
            .collect();
    };

    parts.iter().map(|p| p.trim().to_string()).collect()
}

fn extract_aspects(query: &str) -> Vec<String> {
    let base = query.replace("implications of ", "")
        .replace("pros and cons of ", "")
        .replace("advantages and disadvantages of ", "");
    vec![
        format!("benefits of {}", base),
        format!("drawbacks of {}", base),
        format!("current state of {}", base),
    ]
}
```

**Step 3: Build the AMRS Coordinator (`amrs/mod.rs`)**

```rust
use tokio::sync::mpsc;
use crate::research::amrs::agent::Agent;
use crate::research::amrs::channel::*;
use crate::research::amrs::decompose::*;
use crate::resource::ResourceTier;
use crate::error::HsxError;

pub mod agent;
pub mod channel;
pub mod decompose;
pub mod search_agent;
pub mod extract_agent;
pub mod verify_agent;
pub mod synthesize_agent;

/// Configuration for an AMRS deep research session.
pub struct AmrsConfig {
    pub max_depth: usize,          // max multi-hop depth (default: 3)
    pub max_agents: usize,         // max concurrent agents
    pub channel_buffer: usize,     // mpsc channel buffer size
}

impl AmrsConfig {
    pub fn from_resource_tier(tier: &ResourceTier) -> Self {
        match tier {
            ResourceTier::Minimal => Self {
                max_depth: 1,
                max_agents: 1,  // sequential execution
                channel_buffer: 32,
            },
            ResourceTier::Standard => Self {
                max_depth: 2,
                max_agents: 4,
                channel_buffer: 64,
            },
            ResourceTier::Performance => Self {
                max_depth: 3,
                max_agents: 8,
                channel_buffer: 128,
            },
            ResourceTier::Workstation => Self {
                max_depth: 5,
                max_agents: 16,
                channel_buffer: 256,
            },
        }
    }
}

/// Result of a full AMRS deep research session.
#[derive(Debug)]
pub struct DeepResearchResult {
    pub report: String,
    pub evidence_graph: EvidenceGraph,
    pub contradictions: Vec<Contradiction>,
    pub decomposition_tree: Vec<QueryNode>,
    pub audit_trail: Vec<AuditEntry>,
    pub sources_analyzed: usize,
    pub claims_verified: usize,
    pub depth_reached: usize,
}

/// The AMRS Coordinator manages the lifecycle of all agents.
pub struct Coordinator {
    config: AmrsConfig,
    // Channel from agents -> coordinator
    agent_tx: AgentSender,
    agent_rx: AgentReceiver,
}

impl Coordinator {
    pub fn new(config: AmrsConfig) -> Self {
        let (agent_tx, agent_rx) = mpsc::channel(config.channel_buffer);
        Self { config, agent_tx, agent_rx }
    }

    /// Execute a full deep research session.
    pub async fn run(&mut self, query: &str) -> Result<DeepResearchResult, HsxError> {
        let mut audit = Vec::new();
        let mut all_contradictions = Vec::new();
        let decomposition = decompose_query(query, self.config.max_depth);

        audit.push(AuditEntry {
            timestamp: chrono::Utc::now(),
            agent: AgentType::Search,
            action: "decompose".into(),
            detail: format!("Decomposed into {} sub-queries", decomposition.len()),
        });

        // Phase 1: Search -- spawn search agents for each sub-query
        let mut all_search_results = Vec::new();
        let mut follow_ups = Vec::new();

        for node in &decomposition {
            if node.depth > self.config.max_depth {
                continue;
            }
            let (search_tx, search_rx) = mpsc::channel(self.config.channel_buffer);
            let coordinator_tx = self.agent_tx.clone();

            let search_agent = search_agent::SearchAgent::new(/* deps */);

            // Send the search task
            search_tx.send(AgentMessage::SpawnSearch {
                query: node.query.clone(),
                depth: node.depth,
            }).await.map_err(|e| HsxError::Internal(e.to_string()))?;

            // Spawn the agent as a tokio task
            tokio::spawn(async move {
                let _ = search_agent.run(search_rx, coordinator_tx).await;
            });
        }

        // Collect search results from agents
        while let Some(msg) = self.agent_rx.recv().await {
            match msg {
                AgentMessage::SearchResults { sub_query, results, follow_up_queries } => {
                    all_search_results.extend(results);
                    follow_ups.extend(follow_up_queries);
                    // Check if all search agents have reported
                    // (track count, break when all done)
                }
                AgentMessage::ProgressUpdate { agent_type, message, progress } => {
                    // Forward to progress bars
                    tracing::info!(?agent_type, %message, %progress, "Agent progress");
                }
                _ => {}
            }
            // Break condition: all search tasks complete
            break; // simplified; real impl uses a counter
        }

        // Phase 2: Extract -- spawn extract agents for discovered URLs
        // Phase 3: Verify -- spawn verify agents for extracted claims
        // Phase 4: Synthesize -- spawn synthesize agent for final report

        // (Follow the same pattern: create channel, spawn agent task, collect results)

        // Handle multi-hop follow-ups (recursive depth)
        // ...

        Ok(DeepResearchResult {
            report: String::new(),         // filled by synthesize agent
            evidence_graph: todo!(),       // filled by synthesize agent
            contradictions: all_contradictions,
            decomposition_tree: decomposition,
            audit_trail: audit,
            sources_analyzed: all_search_results.len(),
            claims_verified: 0,
            depth_reached: self.config.max_depth,
        })
    }
}
```

**Step 4: Implement individual agents**

Each agent follows the same pattern. Here is the Search Agent as the primary example:

```rust
// search_agent.rs
use async_trait::async_trait;
use crate::research::amrs::agent::{Agent, AgentType};
use crate::research::amrs::channel::*;
use crate::search::SearchOrchestrator;
use crate::error::HsxError;

pub struct SearchAgent {
    orchestrator: SearchOrchestrator,
}

impl SearchAgent {
    pub fn new(orchestrator: SearchOrchestrator) -> Self {
        Self { orchestrator }
    }

    /// Detect follow-up queries from search results.
    fn detect_follow_ups(&self, query: &str, results: &[SearchResult]) -> Vec<String> {
        let mut follow_ups = Vec::new();

        // Heuristic: look for "see also", "related", entity mentions
        // that were not in the original query
        for result in results.iter().take(5) {
            // Extract entities from snippets not present in query
            // This is a simplified heuristic; a full implementation would
            // use NER or keyword extraction
            if let Some(snippet) = &result.snippet {
                // If snippet mentions something not in query, create follow-up
                let query_words: std::collections::HashSet<&str> =
                    query.split_whitespace().collect();
                let new_terms: Vec<&str> = snippet.split_whitespace()
                    .filter(|w| !query_words.contains(w) && w.len() > 5)
                    .take(2)
                    .collect();
                if !new_terms.is_empty() {
                    follow_ups.push(format!("{} {}", query, new_terms.join(" ")));
                }
            }
        }

        follow_ups.truncate(3); // limit follow-ups
        follow_ups
    }
}

#[async_trait]
impl Agent for SearchAgent {
    fn agent_type(&self) -> AgentType {
        AgentType::Search
    }

    async fn run(
        &self,
        mut rx: AgentReceiver,
        tx: AgentSender,
    ) -> Result<(), HsxError> {
        while let Some(msg) = rx.recv().await {
            match msg {
                AgentMessage::SpawnSearch { query, depth } => {
                    tx.send(AgentMessage::ProgressUpdate {
                        agent_type: AgentType::Search,
                        message: format!("Searching: {}", &query),
                        progress: 0.0,
                    }).await.ok();

                    // Execute search
                    let results = self.orchestrator.search(&query).await?;

                    // Detect multi-hop follow-ups
                    let follow_ups = if depth > 0 {
                        self.detect_follow_ups(&query, &results.results)
                    } else {
                        Vec::new()
                    };

                    tx.send(AgentMessage::SearchResults {
                        sub_query: query,
                        results: results.results,
                        follow_up_queries: follow_ups,
                    }).await.ok();
                }
                AgentMessage::Shutdown => break,
                _ => {}
            }
        }
        Ok(())
    }
}
```

```rust
// verify_agent.rs (cross-source contradiction detection)
pub struct VerifyAgent {
    // uses validation pipeline from Phase 3
}

impl VerifyAgent {
    /// Detect contradictions between sources for a given claim.
    fn detect_contradictions(
        &self,
        claim: &str,
        sources: &[Source],
    ) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        // Compare each pair of sources
        for i in 0..sources.len() {
            for j in (i + 1)..sources.len() {
                let a = &sources[i];
                let b = &sources[j];

                // Use BM25 or keyword overlap to find relevant passages
                // Then check for negation patterns, conflicting numbers, etc.
                if let Some(contradiction) = self.check_pair(claim, a, b) {
                    contradictions.push(contradiction);
                }
            }
        }

        contradictions
    }

    fn check_pair(
        &self,
        claim: &str,
        source_a: &Source,
        source_b: &Source,
    ) -> Option<Contradiction> {
        // Heuristic contradiction detection:
        // 1. Find claim-relevant passages in both sources
        // 2. Check for negation words ("not", "never", "incorrect")
        // 3. Check for conflicting numbers (dates, versions, counts)
        // 4. Check for opposing sentiment
        // Returns Some(Contradiction) if conflict found
        None // placeholder
    }
}
```

**Acceptance criteria:**
- [ ] `Coordinator::run()` decomposes a complex query into sub-queries and executes them through the agent pipeline
- [ ] Search Agent sends results and detected follow-up queries back via the channel
- [ ] Extract Agent processes URLs through QATBE and returns extracted content with SHA-256 hashes
- [ ] Verify Agent detects contradictions between sources and returns confidence scores
- [ ] Synthesize Agent produces a report with evidence graph and audit trail
- [ ] All agent communication happens via `tokio::sync::mpsc` channels -- no shared mutable state
- [ ] `AmrsConfig::from_resource_tier()` correctly limits parallelism: Minimal=1 sequential, Standard=4, Performance=8, Workstation=16
- [ ] Query decomposition handles "A vs B" patterns and multi-faceted queries
- [ ] Multi-hop follow-ups are generated from search results (limited by `max_depth`)
- [ ] Contradiction detection compares source pairs and flags conflicts with severity scores
- [ ] Audit trail records timestamps, agent type, action, and detail for every step
- [ ] All agents shut down cleanly when receiving `Shutdown` message
- [ ] `cargo test` passes with unit tests for decomposition, channel communication, and contradiction detection
- [ ] `cargo clippy` produces zero warnings

**Pitfalls:**
- `mpsc::channel` will block the sender if the buffer is full. Use a large enough buffer (64-256) to avoid deadlocks between agents.
- Agents must handle channel closure gracefully. When the coordinator drops the sender, `rx.recv()` returns `None` -- agents should exit cleanly.
- The Coordinator must track how many agents are active and wait for all to complete before moving to the next phase. Use a `JoinSet` or counter.
- Query decomposition is heuristic-based (no LLM needed for basic patterns). Do not over-engineer it -- the 80/20 rule applies. "A vs B" splitting and aspect extraction cover most cases.
- Multi-hop follow-ups can explode combinatorially. Always cap with `max_depth` and limit follow-ups per node (3-5 max).
- Contradiction detection without an LLM is approximate. Focus on number conflicts, negation patterns, and date mismatches. This is acceptable for Phase 4; LLM-assisted verification comes in later phases.

---

### P4-E2-T2: `fetchium deep` Command

**ID:** `P4-E2-T2`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 3-4 days

**Description:**
Build the `fetchium deep` CLI command that executes the full AMRS pipeline and presents results as a structured deep research report. Support `--max-depth` for controlling multi-hop depth, evidence graph output, contradiction reports, audit trails, and per-agent progress bars using `indicatif`.

**PRD References:**
- SS10 Mode C "Deep Research Mode" -- `fetchium deep "query" --max-depth 3`
- SS8.8 "AMRS" -- Full agent pipeline
- SS8.7 "EGP" -- Evidence graph output
- SS11 "CLI Interface Design" -- Command structure

**Files to create/modify:**
```
crates/fetchium-cli/src/commands/deep.rs     -- The `fetchium deep` command implementation
crates/fetchium-cli/src/cli.rs              -- Add DeepCommand variant to clap enum
```

**Dependencies:**
- P4-E2-T1 (AMRS architecture) -- Coordinator, all agents
- P3-E2 (Citations + EGP) -- Evidence graph rendering
- P0-E3-T1 (CLI skeleton) -- clap integration

**Step-by-step Rust implementation:**

**Step 1: Define clap command**

```rust
/// Deep agentic research with multi-hop follow-ups and evidence graphs
#[derive(Debug, clap::Args)]
pub struct DeepCommand {
    /// The research query
    pub query: String,

    /// Maximum depth for multi-hop follow-ups (default: 3)
    #[arg(long, default_value = "3")]
    pub max_depth: usize,

    /// Export evidence graph as JSON
    #[arg(long)]
    pub evidence_graph: bool,

    /// Output file for the research report
    #[arg(long, short)]
    pub output: Option<String>,

    /// Output format: text, json, markdown
    #[arg(long, default_value = "markdown")]
    pub format: String,

    /// Show full audit trail
    #[arg(long)]
    pub audit: bool,

    /// Token budget for the entire research session
    #[arg(long, default_value = "20000")]
    pub budget: usize,
}
```

**Step 2: Implement command handler with progress bars**

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use hsx_core::research::amrs::{Coordinator, AmrsConfig, DeepResearchResult};
use hsx_core::research::amrs::channel::AgentType;

pub async fn handle_deep_command(args: DeepCommand, config: &Config) -> anyhow::Result<()> {
    // Set up multi-progress for per-agent progress bars
    let multi = MultiProgress::new();
    let style = ProgressStyle::default_spinner()
        .template("{spinner:.cyan} [{elapsed_precise}] {msg}")
        .unwrap();

    let mut bars: HashMap<AgentType, ProgressBar> = HashMap::new();
    for agent_type in &[AgentType::Search, AgentType::Extract, AgentType::Verify, AgentType::Synthesize] {
        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(style.clone());
        pb.set_message(format!("{:?} agent: waiting...", agent_type));
        bars.insert(*agent_type, pb);
    }

    // Create coordinator
    let resource_tier = hsx_core::resource::detect_tier();
    let amrs_config = AmrsConfig::from_resource_tier(&resource_tier);
    let mut coordinator = Coordinator::new(amrs_config);

    // Run deep research
    let result = coordinator.run(&args.query).await?;

    // Clear progress bars
    for pb in bars.values() {
        pb.finish_and_clear();
    }

    // Render report
    println!("\n{}", console::style("Deep Research Report").bold().cyan());
    println!("{}\n", console::style("=".repeat(60)).dim());
    println!("{}", result.report);

    // Contradictions section
    if !result.contradictions.is_empty() {
        println!("\n{}", console::style("Contradictions Found").bold().yellow());
        println!("{}", console::style("-".repeat(40)).dim());
        for (i, c) in result.contradictions.iter().enumerate() {
            println!(
                "  {}. {} (severity: {:.0}%)\n     Source A: {}\n     Source B: {}\n",
                i + 1, c.claim, c.severity * 100.0, c.source_a_says, c.source_b_says,
            );
        }
    }

    // Audit trail
    if args.audit {
        println!("\n{}", console::style("Audit Trail").bold().dim());
        println!("{}", console::style("-".repeat(40)).dim());
        for entry in &result.audit_trail {
            println!(
                "  [{}] {:?}: {} -- {}",
                entry.timestamp.format("%H:%M:%S"),
                entry.agent,
                entry.action,
                entry.detail,
            );
        }
    }

    // Evidence graph export
    if args.evidence_graph {
        let graph_json = serde_json::to_string_pretty(&result.evidence_graph)?;
        let path = format!("evidence_graph_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        std::fs::write(&path, &graph_json)?;
        println!("\nEvidence graph saved to: {}", console::style(&path).green());
    }

    // Optional file output
    if let Some(output_path) = &args.output {
        std::fs::write(output_path, &result.report)?;
        println!("Report saved to: {}", console::style(output_path).green());
    }

    // Summary footer
    println!("\n{}", console::style("-".repeat(60)).dim());
    println!(
        "Sources: {} | Claims verified: {} | Contradictions: {} | Depth: {}",
        result.sources_analyzed,
        result.claims_verified,
        result.contradictions.len(),
        result.depth_reached,
    );

    Ok(())
}
```

**Acceptance criteria:**
- [ ] `fetchium deep "Compare Rust vs Go for microservices"` runs the full AMRS pipeline and outputs a structured report
- [ ] `--max-depth 1` limits research to a single hop; `--max-depth 5` allows 5 hops
- [ ] Per-agent progress bars show real-time status (Search agent: searching..., Extract agent: extracting 5 URLs...)
- [ ] `--evidence-graph` exports a JSON file with the full EGP graph (nodes, edges, hashes)
- [ ] `--audit` prints the full audit trail with timestamps and agent actions
- [ ] `--output report.md` saves the report to a file
- [ ] Contradiction report section shows conflicting claims with source quotes and severity
- [ ] Summary footer shows sources analyzed, claims verified, contradictions found, and depth reached
- [ ] `cargo build` compiles; `cargo clippy` zero warnings

**Pitfalls:**
- `indicatif::MultiProgress` must be used from the main thread or a dedicated render thread. Do not update progress bars from within tokio tasks without using `ProgressBar::clone()` (which is safe across threads).
- Deep research can take 1-10 minutes. The progress bars must tick continuously to show the user that work is happening. Use `enable_steady_tick()`.
- Evidence graph JSON can be large. Use `serde_json::to_string_pretty` for human readability but warn about file size.
- The `--max-depth` flag must be validated (min 1, max 10) to prevent runaway recursion.

---

## Epic 4.3: Speculative Research Pipelining (SRP)

> **PRD Section:** SS8.5 (Speculative Research Pipelining)
> **Crate:** `fetchium-core` -- `src/research/`
> **Priority:** P2 | **Tasks:** 1

### P4-E3-T1: SRP Streaming Pipeline

**ID:** `P4-E3-T1`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 3-4 days

**Description:**
Implement Speculative Research Pipelining (SRP) per PRD SS8.5. Stream initial findings to the user as soon as the first search results are extracted, while continuing to fetch and process more sources in the background. If new data contradicts earlier findings, stream corrections. Support a `--stream` flag on search/research commands.

**PRD References:**
- SS8.5 "Speculative Research Pipelining (SRP)" -- Full timeline diagram
- SS10 Mode A "Search Mode" -- "SRP: Stream first results while remaining sources load"
- SS10 Mode C "Deep Research" -- "SRP: Streams findings as they're discovered"

**Files to create/modify:**
```
crates/fetchium-core/src/research/
  srp.rs                 -- SRP pipeline: streaming findings with corrections
  srp_types.rs           -- SrpChunk, SrpEvent, SrpStream types
```

**Dependencies:**
- P1-E2-T2 (Search orchestrator) -- Parallel search results arrive asynchronously
- P1-E1-T2 (Content extraction) -- Extract content as results arrive
- P3-E1 (Validation) -- Validate findings and detect contradictions

**Step-by-step Rust implementation:**

**Step 1: Define SRP types (`research/srp_types.rs`)**

```rust
use serde::{Deserialize, Serialize};

/// A chunk emitted by the SRP pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrpChunk {
    pub event: SrpEvent,
    pub content: String,
    pub sources: Vec<usize>,     // source indices supporting this chunk
    pub confidence: f64,
    pub timestamp_ms: u64,       // ms since pipeline start
}

/// The type of SRP event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SrpEvent {
    /// First findings from initial sources
    Initial,
    /// Additional data confirms/extends earlier findings
    Update,
    /// New data contradicts earlier findings -- correction
    Correction,
    /// All sources processed -- final validated output
    Final,
}

/// Configuration for the SRP pipeline.
#[derive(Debug, Clone)]
pub struct SrpConfig {
    /// Minimum number of sources before emitting INITIAL
    pub min_initial_sources: usize,      // default: 2
    /// Confidence threshold below which corrections trigger
    pub correction_threshold: f64,       // default: 0.3
    /// Maximum time to wait for all sources (ms)
    pub max_wait_ms: u64,               // default: 30_000
}

impl Default for SrpConfig {
    fn default() -> Self {
        Self {
            min_initial_sources: 2,
            correction_threshold: 0.3,
            max_wait_ms: 30_000,
        }
    }
}
```

**Step 2: Build the SRP pipeline (`research/srp.rs`)**

```rust
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration, Instant};
use crate::research::srp_types::*;
use crate::search::SearchOrchestrator;
use crate::extract::ContentExtractor;
use crate::error::HsxError;

/// Run the SRP pipeline, emitting chunks through a channel.
///
/// The caller receives SrpChunk values as findings become available.
/// The pipeline:
/// 1. Launches parallel searches across all backends
/// 2. As each backend returns, extracts top results immediately
/// 3. Emits INITIAL chunk once min_initial_sources are processed
/// 4. Continues processing remaining sources
/// 5. Emits UPDATE chunks for confirmations, CORRECTION for contradictions
/// 6. Emits FINAL chunk when all sources are processed and validated
pub async fn run_srp_pipeline(
    query: &str,
    config: SrpConfig,
    orchestrator: &SearchOrchestrator,
    extractor: &ContentExtractor,
) -> mpsc::Receiver<SrpChunk> {
    let (tx, rx) = mpsc::channel::<SrpChunk>(64);
    let query = query.to_string();

    tokio::spawn(async move {
        let start = Instant::now();
        let mut processed_sources = Vec::new();
        let mut emitted_claims: Vec<(String, f64)> = Vec::new(); // (claim, confidence)
        let mut initial_emitted = false;

        // Launch parallel search -- results arrive as a stream
        // In a real implementation, the orchestrator returns a stream of results
        // as each backend responds, rather than waiting for all.
        let all_results = orchestrator.search_streaming(&query).await;

        // Process results as they arrive
        let mut results_stream = all_results; // this would be a real async stream

        // Simplified: process in batches as they arrive
        // Real implementation would use tokio::select! on multiple futures

        // Batch 1: first results (fast backends like DDG)
        // Extract and emit INITIAL
        let first_batch = extract_batch(&extractor, &query, &results_stream[..3.min(results_stream.len())]).await;
        processed_sources.extend(first_batch.clone());

        if processed_sources.len() >= config.min_initial_sources && !initial_emitted {
            let findings = synthesize_findings(&processed_sources);
            emitted_claims = findings.iter().map(|f| (f.clone(), 0.7)).collect();

            let _ = tx.send(SrpChunk {
                event: SrpEvent::Initial,
                content: format!("[INITIAL] Based on {} sources: {}", processed_sources.len(), findings.join("; ")),
                sources: (0..processed_sources.len()).collect(),
                confidence: 0.7,
                timestamp_ms: start.elapsed().as_millis() as u64,
            }).await;
            initial_emitted = true;
        }

        // Remaining batches: check for contradictions and updates
        for batch_start in (3..results_stream.len()).step_by(3) {
            let batch_end = (batch_start + 3).min(results_stream.len());
            let batch = extract_batch(&extractor, &query, &results_stream[batch_start..batch_end]).await;
            processed_sources.extend(batch);

            // Check if new data contradicts emitted claims
            let new_findings = synthesize_findings(&processed_sources);
            for (old_claim, old_conf) in &emitted_claims {
                if let Some(contradiction) = find_contradiction(old_claim, &new_findings) {
                    let _ = tx.send(SrpChunk {
                        event: SrpEvent::Correction,
                        content: format!("[CORRECTION] {}", contradiction),
                        sources: (0..processed_sources.len()).collect(),
                        confidence: 0.8,
                        timestamp_ms: start.elapsed().as_millis() as u64,
                    }).await;
                }
            }

            // Emit update for new confirmed information
            let _ = tx.send(SrpChunk {
                event: SrpEvent::Update,
                content: format!("[UPDATE] {} sources processed. Confidence: {:.0}%",
                    processed_sources.len(), 0.85 * 100.0),
                sources: (0..processed_sources.len()).collect(),
                confidence: 0.85,
                timestamp_ms: start.elapsed().as_millis() as u64,
            }).await;
        }

        // Final emission
        let _ = tx.send(SrpChunk {
            event: SrpEvent::Final,
            content: format!("[FINAL] {} sources analyzed and validated. Full report below.",
                processed_sources.len()),
            sources: (0..processed_sources.len()).collect(),
            confidence: 0.95,
            timestamp_ms: start.elapsed().as_millis() as u64,
        }).await;
    });

    rx
}

async fn extract_batch(
    extractor: &ContentExtractor,
    query: &str,
    results: &[SearchResult],
) -> Vec<ProcessedSource> {
    let mut processed = Vec::new();
    for result in results {
        if let Ok(content) = extractor.extract(&result.url, query).await {
            processed.push(ProcessedSource {
                url: result.url.clone(),
                title: result.title.clone(),
                content,
            });
        }
    }
    processed
}

fn synthesize_findings(sources: &[ProcessedSource]) -> Vec<String> {
    // Extract key claims from all processed sources
    // Simple heuristic: first sentence of each source
    sources.iter()
        .filter_map(|s| s.content.split('.').next().map(|s| s.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn find_contradiction(claim: &str, new_findings: &[String]) -> Option<String> {
    // Check if any new finding contradicts the claim
    // Heuristic: look for negation or conflicting numbers
    // Full implementation would use the Verify Agent from AMRS
    None
}
```

**Acceptance criteria:**
- [ ] `run_srp_pipeline()` returns a channel receiver that emits `SrpChunk` values as they become available
- [ ] `SrpEvent::Initial` is emitted within 2 seconds of pipeline start (after first 2-3 sources are extracted)
- [ ] `SrpEvent::Update` is emitted as additional sources confirm findings
- [ ] `SrpEvent::Correction` is emitted when new data contradicts earlier findings (with the correction content)
- [ ] `SrpEvent::Final` is emitted after all sources are processed and validated
- [ ] Each chunk includes the source indices that support it and a confidence score
- [ ] The pipeline completes within `max_wait_ms` even if some sources are slow
- [ ] `--stream` flag on `fetchium search` and `fetchium research` enables SRP output
- [ ] SRP chunks are printed incrementally to the terminal as `[INITIAL]`, `[UPDATE]`, `[CORRECTION]`, `[FINAL]`
- [ ] `cargo test` passes; `cargo clippy` zero warnings

**Pitfalls:**
- The tokio task spawned by `run_srp_pipeline` must not hold references to non-`Send` types. All data must be moved into the task.
- Channel backpressure: if the consumer is slow, the producer will block on `tx.send()`. Use `try_send()` with a fallback if needed.
- Contradiction detection is approximate without an LLM. Focus on detectable patterns (number differences, date conflicts, negation). Accept false negatives.
- The `max_wait_ms` timeout must use `tokio::time::timeout` to hard-limit the pipeline, otherwise a single slow URL can stall everything.
- When integrating SRP into existing commands (`fetchium search --stream`), the non-streaming code path must remain the default. SRP adds latency complexity that not all users want.

---

## Epic 4.4: MCP Server

> **PRD Section:** SS30 (MCP Server Mode)
> **Crate:** `fetchium-mcp`
> **Priority:** P1 | **Tasks:** 1

### P4-E4-T1: MCP Server with 5 Composite Tools

**ID:** `P4-E4-T1`
**Status:** `TODO`
**Priority:** P1
**Estimated effort:** 4-5 days

**Description:**
Build the MCP (Model Context Protocol) server that exposes Fetchium as a tool provider for Claude, Claude Code, and any MCP-compatible client. Implement 5 composite tools (`hypersearch_search`, `hypersearch_fetch`, `hypersearch_research`, `hypersearch_estimate`, `hypersearch_expand`) that handle the full pipeline in a single call. Support both stdio and SSE transports.

**PRD References:**
- SS30 "MCP Server Mode" -- Full tool schemas, composite tool rationale
- SS9 "AI-Native Agent Architecture" -- MCP as interface mode 4
- SS30 "Why Composite Tools Matter" -- Single call handles search+fetch+extract+rank+validate

**Files to create/modify:**
```
crates/fetchium-mcp/
  Cargo.toml              -- Dependencies: rmcp, serde_json, tokio, fetchium-core
  src/
    lib.rs                -- MCP server setup, transport selection
    tools.rs              -- Tool schema definitions (5 tools)
    handlers.rs           -- Handler implementations for each tool
    transport.rs          -- stdio and SSE transport implementations
```

**Dependencies:**
- P1-E2-T2 (Search orchestrator) -- `hypersearch_search` tool
- P1-E1-T2 (Content extraction) -- `hypersearch_fetch` tool
- P3-E3 (Research mode) -- `hypersearch_research` tool
- P1-E3 (QATBE + SCS + PDS) -- Token budgeting, progressive detail tiers

**Step-by-step Rust implementation:**

**Step 1: Set up Cargo.toml for fetchium-mcp**

```toml
[package]
name = "fetchium-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
fetchium-core = { path = "../fetchium-core" }
rmcp = { version = "0.1", features = ["server", "transport-stdio", "transport-sse"] }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["full"] }
anyhow = { workspace = true }
tracing = { workspace = true }
```

**Step 2: Define tool schemas (`tools.rs`)**

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

/// All 5 composite MCP tool definitions.
pub fn tool_definitions() -> Vec<serde_json::Value> {
    vec![
        json!({
            "name": "hypersearch_search",
            "description": "Search the web and return token-efficient results. Handles the full pipeline: multi-backend search, ranking, validation, and token budgeting in a single call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum tokens in response (default: 2000)"
                    },
                    "tier": {
                        "type": "string",
                        "enum": ["key_facts", "summary", "detailed", "complete"],
                        "description": "Detail level (default: summary)"
                    },
                    "max_sources": {
                        "type": "integer",
                        "description": "Maximum number of sources to include (default: 10)"
                    },
                    "validate": {
                        "type": "boolean",
                        "description": "Run validation pipeline on results (default: true)"
                    }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "hypersearch_fetch",
            "description": "Fetch a URL with query-aware extraction. Extracts only content relevant to the query, within the token budget. Far more efficient than raw scraping.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    },
                    "query": {
                        "type": "string",
                        "description": "Extract only content relevant to this query (optional but recommended)"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum tokens in response (default: 3000)"
                    },
                    "format": {
                        "type": "string",
                        "enum": ["markdown", "segments", "json"],
                        "description": "Output format (default: markdown)"
                    }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": "hypersearch_research",
            "description": "Conduct multi-source research with citations and evidence tracking. Searches, extracts, ranks, validates, and synthesizes findings with full citation chains.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The research query"
                    },
                    "token_budget": {
                        "type": "integer",
                        "description": "Maximum tokens in response (default: 4000)"
                    },
                    "depth": {
                        "type": "string",
                        "enum": ["shallow", "standard", "deep"],
                        "description": "Research depth (default: standard)"
                    },
                    "strict_evidence": {
                        "type": "boolean",
                        "description": "Require citation for every claim (default: false)"
                    },
                    "citation_style": {
                        "type": "string",
                        "description": "Citation style: inline, footnote, apa, ieee (default: inline)"
                    }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "hypersearch_estimate",
            "description": "Estimate token cost of fetching a URL without actually fetching it. Use this to decide whether to fetch before committing tokens.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to estimate"
                    }
                },
                "required": ["url"]
            }
        }),
        json!({
            "name": "hypersearch_expand",
            "description": "Get more detail on previous search results without re-fetching. Uses Progressive Detail Streaming (PDS) to expand from key_facts to summary to detailed to complete.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "result_id": {
                        "type": "string",
                        "description": "The result_id from a previous search or fetch call"
                    },
                    "tier": {
                        "type": "string",
                        "enum": ["key_facts", "summary", "detailed", "complete"],
                        "description": "The detail tier to expand to"
                    }
                },
                "required": ["result_id", "tier"]
            }
        }),
    ]
}

/// Input structs for each tool (deserialized from MCP call arguments).
#[derive(Debug, Deserialize)]
pub struct SearchInput {
    pub query: String,
    pub token_budget: Option<usize>,
    pub tier: Option<String>,
    pub max_sources: Option<usize>,
    pub validate: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct FetchInput {
    pub url: String,
    pub query: Option<String>,
    pub token_budget: Option<usize>,
    pub format: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResearchInput {
    pub query: String,
    pub token_budget: Option<usize>,
    pub depth: Option<String>,
    pub strict_evidence: Option<bool>,
    pub citation_style: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EstimateInput {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ExpandInput {
    pub result_id: String,
    pub tier: String,
}
```

**Step 3: Implement handlers (`handlers.rs`)**

```rust
use crate::tools::*;
use hsx_core::search::SearchOrchestrator;
use hsx_core::extract::ContentExtractor;
use hsx_core::token::qatbe::QatbeExtractor;
use hsx_core::research::ResearchPipeline;
use hsx_core::cache::CacheManager;
use hsx_core::error::HsxError;
use serde_json::json;
use std::sync::Arc;

/// Shared application state for all MCP handlers.
pub struct McpAppState {
    pub orchestrator: Arc<SearchOrchestrator>,
    pub extractor: Arc<ContentExtractor>,
    pub qatbe: Arc<QatbeExtractor>,
    pub research: Arc<ResearchPipeline>,
    pub cache: Arc<CacheManager>,
}

/// Handle the hypersearch_search tool call.
pub async fn handle_search(
    input: SearchInput,
    state: &McpAppState,
) -> Result<serde_json::Value, HsxError> {
    let budget = input.token_budget.unwrap_or(2000);
    let tier = input.tier.as_deref().unwrap_or("summary");
    let max_sources = input.max_sources.unwrap_or(10);
    let validate = input.validate.unwrap_or(true);

    // Full pipeline in one call:
    // 1. Search across all backends
    let mut results = state.orchestrator.search(&input.query).await?;

    // 2. Rank and limit sources
    results.results.truncate(max_sources);

    // 3. Extract with QATBE (query-aware, token-budgeted)
    let per_source_budget = budget / results.results.len().max(1);
    let mut extracted = Vec::new();
    for result in &results.results {
        let content = state.qatbe.extract(
            &result.url, &input.query, per_source_budget,
        ).await.unwrap_or_default();
        extracted.push(content);
    }

    // 4. Validate if requested
    // (validation pipeline from Phase 3)

    // 5. Apply PDS tier
    // (progressive detail tier from Phase 1)

    // 6. Return structured response
    Ok(json!({
        "meta": {
            "query": input.query,
            "tier": tier,
            "tokens_used": budget,  // approximate
            "sources_count": results.results.len(),
            "result_id": uuid::Uuid::new_v4().to_string(),
        },
        "results": results.results.iter().zip(extracted.iter()).map(|(r, e)| {
            json!({
                "title": r.title,
                "url": r.url,
                "snippet": r.snippet,
                "content": e.content,
                "score": r.score,
            })
        }).collect::<Vec<_>>(),
    }))
}

/// Handle the hypersearch_fetch tool call.
pub async fn handle_fetch(
    input: FetchInput,
    state: &McpAppState,
) -> Result<serde_json::Value, HsxError> {
    let budget = input.token_budget.unwrap_or(3000);
    let query = input.query.as_deref().unwrap_or("");

    let content = if query.is_empty() {
        // No query: extract everything within budget
        state.extractor.extract(&input.url).await?
    } else {
        // Query-aware extraction
        state.qatbe.extract(&input.url, query, budget).await?
    };

    Ok(json!({
        "url": input.url,
        "content": content.content,
        "tokens": content.tokens,
        "format": input.format.as_deref().unwrap_or("markdown"),
        "result_id": uuid::Uuid::new_v4().to_string(),
    }))
}

/// Handle the hypersearch_research tool call.
pub async fn handle_research(
    input: ResearchInput,
    state: &McpAppState,
) -> Result<serde_json::Value, HsxError> {
    let budget = input.token_budget.unwrap_or(4000);
    let depth = input.depth.as_deref().unwrap_or("standard");

    let result = state.research.run(
        &input.query,
        depth,
        budget,
        input.strict_evidence.unwrap_or(false),
    ).await?;

    Ok(json!({
        "meta": {
            "query": input.query,
            "depth": depth,
            "tokens_used": result.tokens_used,
            "sources_count": result.sources.len(),
            "result_id": uuid::Uuid::new_v4().to_string(),
        },
        "report": result.report,
        "sources": result.sources,
        "evidence_links": result.evidence_links,
        "contradictions": result.contradictions,
        "confidence": result.overall_confidence,
    }))
}

/// Handle the hypersearch_estimate tool call.
pub async fn handle_estimate(
    input: EstimateInput,
    state: &McpAppState,
) -> Result<serde_json::Value, HsxError> {
    // HEAD request + heuristic estimation without full fetch
    let estimate = state.extractor.estimate_tokens(&input.url).await?;

    Ok(json!({
        "url": input.url,
        "estimated_tokens": estimate.total_tokens,
        "estimated_relevant_tokens": estimate.relevant_tokens,
        "extraction_layer": estimate.recommended_layer,
        "content_type": estimate.content_type,
    }))
}

/// Handle the hypersearch_expand tool call.
pub async fn handle_expand(
    input: ExpandInput,
    state: &McpAppState,
) -> Result<serde_json::Value, HsxError> {
    // Look up previous result by ID in cache
    let cached = state.cache.get_by_result_id(&input.result_id).await
        .ok_or_else(|| HsxError::NotFound(
            format!("Result ID {} not found in cache. Results expire after the session.", input.result_id)
        ))?;

    // Expand to requested tier (PDS)
    let expanded = cached.expand_to_tier(&input.tier)?;

    Ok(json!({
        "result_id": input.result_id,
        "tier": input.tier,
        "content": expanded.content,
        "tokens": expanded.tokens,
    }))
}
```

**Step 4: Wire up the MCP server (`lib.rs`)**

```rust
use rmcp::{McpServer, McpServerConfig, Transport};
use crate::tools::tool_definitions;
use crate::handlers::*;
use std::sync::Arc;

pub async fn start_mcp_server(
    transport: TransportType,
    state: Arc<McpAppState>,
) -> anyhow::Result<()> {
    let tools = tool_definitions();

    let config = McpServerConfig {
        name: "fetchium".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: rmcp::Capabilities {
            tools: Some(rmcp::ToolsCapability {}),
            ..Default::default()
        },
    };

    let server = McpServer::new(config);

    // Register tool handlers
    server.register_tool("hypersearch_search", move |args| {
        let state = state.clone();
        async move {
            let input: SearchInput = serde_json::from_value(args)?;
            let result = handle_search(input, &state).await?;
            Ok(rmcp::ToolResult::text(serde_json::to_string(&result)?))
        }
    });

    server.register_tool("hypersearch_fetch", move |args| {
        let state = state.clone();
        async move {
            let input: FetchInput = serde_json::from_value(args)?;
            let result = handle_fetch(input, &state).await?;
            Ok(rmcp::ToolResult::text(serde_json::to_string(&result)?))
        }
    });

    server.register_tool("hypersearch_research", move |args| {
        let state = state.clone();
        async move {
            let input: ResearchInput = serde_json::from_value(args)?;
            let result = handle_research(input, &state).await?;
            Ok(rmcp::ToolResult::text(serde_json::to_string(&result)?))
        }
    });

    server.register_tool("hypersearch_estimate", move |args| {
        let state = state.clone();
        async move {
            let input: EstimateInput = serde_json::from_value(args)?;
            let result = handle_estimate(input, &state).await?;
            Ok(rmcp::ToolResult::text(serde_json::to_string(&result)?))
        }
    });

    server.register_tool("hypersearch_expand", move |args| {
        let state = state.clone();
        async move {
            let input: ExpandInput = serde_json::from_value(args)?;
            let result = handle_expand(input, &state).await?;
            Ok(rmcp::ToolResult::text(serde_json::to_string(&result)?))
        }
    });

    // Start transport
    match transport {
        TransportType::Stdio => {
            tracing::info!("Starting MCP server on stdio");
            server.run_stdio().await?;
        }
        TransportType::Sse { port } => {
            tracing::info!("Starting MCP server on SSE port {}", port);
            server.run_sse(port).await?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub enum TransportType {
    Stdio,
    Sse { port: u16 },
}
```

**Acceptance criteria:**
- [ ] `fetchium serve --mcp` starts the MCP server on stdio transport
- [ ] `fetchium serve --mcp --transport sse --port 3001` starts on SSE transport
- [ ] `hypersearch_search` tool accepts a query, runs the full search pipeline, and returns token-efficient results with a `result_id`
- [ ] `hypersearch_fetch` tool fetches a URL with optional query-aware extraction and respects `token_budget`
- [ ] `hypersearch_research` tool runs the full research pipeline with citations and returns a report
- [ ] `hypersearch_estimate` tool returns token estimates without fetching the full page
- [ ] `hypersearch_expand` tool expands a previous result to a higher PDS tier using the `result_id`
- [ ] Tool schemas match the PRD SS30 specification exactly (field names, types, descriptions)
- [ ] All 5 tools are discoverable via MCP `tools/list` protocol
- [ ] Error responses follow MCP error format (not raw panics)
- [ ] The server works with Claude Code when configured in `claude_desktop_config.json`
- [ ] `cargo build` compiles; `cargo clippy` zero warnings

**Pitfalls:**
- The `rmcp` crate API may differ from the pseudocode above. Check the actual `rmcp` docs for the correct server setup pattern. The key concepts (register tools, run transport) remain the same.
- stdio transport reads from stdin and writes to stdout. All `println!` / `tracing` output must go to stderr, not stdout. Use `tracing_subscriber` with a stderr writer.
- MCP tool results must be valid JSON strings. Use `serde_json::to_string` (not `to_string_pretty`) to minimize token usage.
- The `result_id` for `hypersearch_expand` must persist in the cache for the session lifetime. Use an in-memory LRU cache keyed by UUID.
- SSE transport requires keeping the connection alive with heartbeats. The `rmcp` crate should handle this, but verify.
- Tool descriptions are part of the LLM's context in MCP. Keep them concise but precise -- every extra word costs tokens for the MCP client.

---

## Epic 4.5: REST API

> **PRD Section:** SS9 (AI-Native Agent Architecture, REST API Mode)
> **Crate:** `fetchium-api`
> **Priority:** P2 | **Tasks:** 1

### P4-E5-T1: REST API with axum

**ID:** `P4-E5-T1`
**Status:** `TODO`
**Priority:** P2
**Estimated effort:** 3-4 days

**Description:**
Build a REST API server using axum that exposes Fetchium functionality over HTTP. Implement POST endpoints for `/api/search`, `/api/fetch`, `/api/research`, and `/api/estimate`. Add rate limiting middleware, CORS support, and a health check endpoint. This enables any programming language to use Fetchium via HTTP requests.

**PRD References:**
- SS9 "AI-Native Agent Architecture" -- REST API Mode (Interface Mode 3)
- SS9 "REST API Mode" -- `fetchium serve --api --port 3000`
- SS9 POST `/api/search` example -- request/response schema

**Files to create/modify:**
```
crates/fetchium-api/
  Cargo.toml              -- Dependencies: axum, tower, tower-http, serde, fetchium-core
  src/
    lib.rs                -- Server setup, start function
    routes.rs             -- Route definitions
    handlers.rs           -- Handler implementations
    middleware.rs          -- Rate limiting, CORS, request logging
    types.rs              -- Request/response types
```

**Dependencies:**
- P1-E2-T2 (Search orchestrator) -- Search endpoint
- P1-E1-T2 (Content extraction) -- Fetch endpoint
- P3-E3 (Research mode) -- Research endpoint
- P1-E3 (QATBE) -- Token budgeting for all endpoints

**Step-by-step Rust implementation:**

**Step 1: Set up Cargo.toml**

```toml
[package]
name = "fetchium-api"
version = "0.1.0"
edition = "2021"

[dependencies]
fetchium-core = { path = "../fetchium-core" }
axum = { version = "0.7", features = ["json", "macros"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["cors", "trace", "limit"] }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true, features = ["full"] }
anyhow = { workspace = true }
tracing = { workspace = true }
uuid = { version = "1", features = ["v4"] }
```

**Step 2: Define request/response types (`types.rs`)**

```rust
use serde::{Deserialize, Serialize};

// --- Search ---

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub token_budget: Option<usize>,
    pub tier: Option<String>,
    pub max_sources: Option<usize>,
    pub validate: Option<bool>,
    pub format: Option<String>,
    pub schema: Option<serde_json::Value>,  // custom output schema
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub meta: ResponseMeta,
    pub results: Vec<SearchResultItem>,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    pub query: String,
    pub tier: String,
    pub tokens_used: usize,
    pub sources_count: usize,
    pub duration_ms: u64,
    pub result_id: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
    pub content: Option<String>,
    pub score: Option<f64>,
}

// --- Fetch ---

#[derive(Debug, Deserialize)]
pub struct FetchRequest {
    pub url: String,
    pub query: Option<String>,
    pub token_budget: Option<usize>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FetchResponse {
    pub url: String,
    pub content: String,
    pub tokens: usize,
    pub format: String,
    pub content_hash: String,
    pub result_id: String,
}

// --- Research ---

#[derive(Debug, Deserialize)]
pub struct ResearchRequest {
    pub query: String,
    pub token_budget: Option<usize>,
    pub depth: Option<String>,
    pub strict_evidence: Option<bool>,
    pub citation_style: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResearchResponse {
    pub meta: ResponseMeta,
    pub report: String,
    pub sources: Vec<SourceInfo>,
    pub evidence_links: Vec<serde_json::Value>,
    pub contradictions: Vec<serde_json::Value>,
    pub confidence: f64,
}

#[derive(Debug, Serialize)]
pub struct SourceInfo {
    pub index: usize,
    pub title: String,
    pub url: String,
    pub trust_score: Option<f64>,
}

// --- Estimate ---

#[derive(Debug, Deserialize)]
pub struct EstimateRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct EstimateResponse {
    pub url: String,
    pub estimated_tokens: usize,
    pub estimated_relevant_tokens: Option<usize>,
    pub extraction_layer: u8,
    pub content_type: String,
}

// --- Error ---

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub error_type: String,
    pub status: u16,
}
```

**Step 3: Define routes (`routes.rs`)**

```rust
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::handlers;
use crate::middleware::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health check
        .route("/health", get(handlers::health_check))
        .route("/api/health", get(handlers::health_check))

        // Core endpoints
        .route("/api/search", post(handlers::search))
        .route("/api/fetch", post(handlers::fetch))
        .route("/api/research", post(handlers::research))
        .route("/api/estimate", post(handlers::estimate))

        // Attach shared state
        .with_state(state)
}
```

**Step 4: Implement handlers (`handlers.rs`)**

```rust
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use std::sync::Arc;
use std::time::Instant;
use crate::types::*;
use crate::middleware::AppState;

/// GET /health -- health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": 0,  // TODO: track actual uptime
    }))
}

/// POST /api/search -- full search pipeline
pub async fn search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> Result<Json<SearchResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();
    let budget = req.token_budget.unwrap_or(2000);
    let tier = req.tier.as_deref().unwrap_or("summary");
    let max_sources = req.max_sources.unwrap_or(10);

    // Execute search pipeline
    let results = state.orchestrator.search(&req.query).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
            error: e.to_string(),
            error_type: "search_failed".into(),
            status: 500,
        })))?;

    let items: Vec<SearchResultItem> = results.results.iter().take(max_sources).map(|r| {
        SearchResultItem {
            title: r.title.clone(),
            url: r.url.clone(),
            snippet: r.snippet.clone(),
            content: None, // populated after extraction
            score: r.score,
        }
    }).collect();

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(Json(SearchResponse {
        meta: ResponseMeta {
            query: req.query,
            tier: tier.to_string(),
            tokens_used: budget,
            sources_count: items.len(),
            duration_ms,
            result_id: uuid::Uuid::new_v4().to_string(),
        },
        results: items,
    }))
}

/// POST /api/fetch -- fetch and extract a URL
pub async fn fetch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FetchRequest>,
) -> Result<Json<FetchResponse>, (StatusCode, Json<ApiError>)> {
    let budget = req.token_budget.unwrap_or(3000);

    let content = if let Some(query) = &req.query {
        state.qatbe.extract(&req.url, query, budget).await
    } else {
        state.extractor.extract(&req.url).await
    }.map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiError {
        error: e.to_string(),
        error_type: "fetch_failed".into(),
        status: 400,
    })))?;

    Ok(Json(FetchResponse {
        url: req.url,
        content: content.content,
        tokens: content.tokens,
        format: req.format.unwrap_or_else(|| "markdown".to_string()),
        content_hash: content.hash,
        result_id: uuid::Uuid::new_v4().to_string(),
    }))
}

/// POST /api/research -- conduct multi-source research
pub async fn research(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResearchRequest>,
) -> Result<Json<ResearchResponse>, (StatusCode, Json<ApiError>)> {
    let start = Instant::now();
    let budget = req.token_budget.unwrap_or(4000);
    let depth = req.depth.as_deref().unwrap_or("standard");

    let result = state.research.run(
        &req.query, depth, budget,
        req.strict_evidence.unwrap_or(false),
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError {
        error: e.to_string(),
        error_type: "research_failed".into(),
        status: 500,
    })))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    Ok(Json(ResearchResponse {
        meta: ResponseMeta {
            query: req.query,
            tier: "detailed".to_string(),
            tokens_used: result.tokens_used,
            sources_count: result.sources.len(),
            duration_ms,
            result_id: uuid::Uuid::new_v4().to_string(),
        },
        report: result.report,
        sources: result.sources.iter().enumerate().map(|(i, s)| SourceInfo {
            index: i + 1,
            title: s.title.clone(),
            url: s.url.clone(),
            trust_score: s.trust_score,
        }).collect(),
        evidence_links: serde_json::to_value(&result.evidence_links)
            .unwrap_or_default().as_array().cloned().unwrap_or_default(),
        contradictions: serde_json::to_value(&result.contradictions)
            .unwrap_or_default().as_array().cloned().unwrap_or_default(),
        confidence: result.overall_confidence,
    }))
}

/// POST /api/estimate -- estimate token cost
pub async fn estimate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EstimateRequest>,
) -> Result<Json<EstimateResponse>, (StatusCode, Json<ApiError>)> {
    let estimate = state.extractor.estimate_tokens(&req.url).await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiError {
            error: e.to_string(),
            error_type: "estimate_failed".into(),
            status: 400,
        })))?;

    Ok(Json(EstimateResponse {
        url: req.url,
        estimated_tokens: estimate.total_tokens,
        estimated_relevant_tokens: Some(estimate.relevant_tokens),
        extraction_layer: estimate.recommended_layer,
        content_type: estimate.content_type,
    }))
}
```

**Step 5: Implement middleware (`middleware.rs`)**

```rust
use axum::extract::FromRef;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower::ServiceBuilder;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;
use tokio::sync::Mutex;

/// Shared application state for all API handlers.
#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<hsx_core::search::SearchOrchestrator>,
    pub extractor: Arc<hsx_core::extract::ContentExtractor>,
    pub qatbe: Arc<hsx_core::token::qatbe::QatbeExtractor>,
    pub research: Arc<hsx_core::research::ResearchPipeline>,
    pub rate_limiter: Arc<RateLimiter>,
}

/// Simple in-memory rate limiter (token bucket per IP).
pub struct RateLimiter {
    /// Map of IP -> (request count, window start)
    buckets: Mutex<HashMap<String, (u32, Instant)>>,
    max_requests: u32,       // per window
    window_secs: u64,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
            max_requests,
            window_secs,
        }
    }

    pub async fn check(&self, ip: &str) -> bool {
        let mut buckets = self.buckets.lock().await;
        let now = Instant::now();

        let entry = buckets.entry(ip.to_string()).or_insert((0, now));

        // Reset window if expired
        if now.duration_since(entry.1).as_secs() > self.window_secs {
            *entry = (0, now);
        }

        if entry.0 >= self.max_requests {
            return false; // rate limited
        }

        entry.0 += 1;
        true
    }
}

/// Build the middleware stack for the API server.
pub fn build_middleware() -> ServiceBuilder<tower::layer::util::Stack<TraceLayer, tower::layer::util::Stack<CorsLayer, tower::layer::util::Identity>>> {
    ServiceBuilder::new()
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .layer(TraceLayer::new_for_http())
}
```

**Step 6: Server startup (`lib.rs`)**

```rust
use std::sync::Arc;
use crate::routes::build_router;
use crate::middleware::{AppState, RateLimiter, build_middleware};

pub struct ApiServerConfig {
    pub port: u16,
    pub host: String,
    pub rate_limit_requests: u32,
    pub rate_limit_window_secs: u64,
}

impl Default for ApiServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            host: "0.0.0.0".into(),
            rate_limit_requests: 100,
            rate_limit_window_secs: 60,
        }
    }
}

pub async fn start_api_server(
    config: ApiServerConfig,
    state: Arc<AppState>,
) -> anyhow::Result<()> {
    let app = build_router(state)
        .layer(build_middleware());

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Fetchium REST API listening on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
```

**Acceptance criteria:**
- [ ] `fetchium serve --api` starts the REST API on port 3000 (default)
- [ ] `fetchium serve --api --port 8080` starts on the specified port
- [ ] `GET /health` returns `{"status": "ok", "version": "..."}` with 200
- [ ] `POST /api/search` with `{"query": "rust web framework"}` returns ranked search results with `result_id`
- [ ] `POST /api/fetch` with `{"url": "...", "query": "...", "token_budget": 1000}` returns extracted content within budget
- [ ] `POST /api/research` with `{"query": "...", "depth": "standard"}` returns a research report with citations
- [ ] `POST /api/estimate` with `{"url": "..."}` returns token estimates without fetching the page
- [ ] CORS headers are present on all responses (tested with `curl -H "Origin: http://example.com"`)
- [ ] Rate limiting returns HTTP 429 after exceeding the limit (100 requests/minute default)
- [ ] Invalid JSON body returns HTTP 400 with a structured `ApiError` response
- [ ] Request tracing logs method, path, status, and latency for every request
- [ ] `cargo build` compiles; `cargo clippy` zero warnings

**Pitfalls:**
- axum 0.7 uses `axum::serve` instead of `axum::Server::bind`. Make sure to use the correct axum 0.7 API.
- `tower_http::cors::CorsLayer` with `Any` origin is fine for development but should be configurable for production. Add a `--cors-origin` flag.
- The rate limiter uses an in-memory `HashMap` that grows unbounded. Add periodic cleanup of expired entries to prevent memory leaks. Use a background tokio task that runs every `window_secs`.
- axum's `Json` extractor returns a 422 Unprocessable Entity on invalid JSON, not 400. To customize this, add a custom rejection handler.
- Long-running endpoints (research, deep fetch) should respect a request timeout. Use `tokio::time::timeout` wrapping the handler logic, returning 504 if exceeded.
- The `AppState` must implement `Clone` (or use `Arc<AppState>` in the router). axum requires this for state extraction.

---

## Task Dependency Graph

```
Phase 3 (all complete)
│
├─── P4-E1-T1 (Ollama integration)
│    └─── P4-E1-T2 (fetchium ai command)
│
├─── P4-E2-T1 (AMRS agents)
│    └─── P4-E2-T2 (fetchium deep command)
│
├─── P4-E3-T1 (SRP streaming) ← independent of E1/E2
│
├─── P4-E4-T1 (MCP server) ← depends on E1 for AI, but can start without it
│
└─── P4-E5-T1 (REST API) ← depends on E1 for AI, but can start without it
```

### Parallelization

These can run simultaneously:
- **Agent A**: P4-E1-T1 then P4-E1-T2 (AI Engine)
- **Agent B**: P4-E2-T1 then P4-E2-T2 (AMRS + Deep)
- **Agent C**: P4-E3-T1 (SRP) + P4-E4-T1 (MCP) + P4-E5-T1 (REST API)

---

## Technology Reference

| Component | Crate | Version | Purpose |
|-----------|-------|---------|---------|
| Ollama client | `reqwest` | latest | HTTP to localhost:11434 |
| Streaming | `futures` + `tokio` | latest | Async streams, channels |
| MCP server | `rmcp` | 0.1+ | Model Context Protocol |
| REST API | `axum` | 0.7+ | HTTP server |
| CORS | `tower-http` | 0.5+ | CORS middleware |
| Rate limiting | custom | - | In-memory token bucket |
| Progress bars | `indicatif` | latest | Multi-progress for agents |
| Agent channels | `tokio::sync::mpsc` | latest | Inter-agent communication |
| Serialization | `serde` + `serde_json` | latest | JSON I/O |
| UUID | `uuid` | 1.x | result_id generation |
| Timestamps | `chrono` | latest | Audit trail timestamps |

---

## Phase Completion Checklist

- [ ] All 8 tasks marked `DONE`
- [ ] `fetchium ai "query"` produces cited AI answers with streaming
- [ ] `fetchium deep "query"` runs multi-agent research with evidence graphs
- [ ] `fetchium search --stream` delivers SRP incremental results
- [ ] `fetchium serve --mcp` works with Claude Code
- [ ] `fetchium serve --api` serves all 4 REST endpoints
- [ ] `cargo build --workspace` compiles with zero errors
- [ ] `cargo test --workspace` passes all tests
- [ ] `cargo clippy --workspace` produces zero warnings
- [ ] All public APIs have `///` doc comments
- [ ] No file exceeds 500 lines (split into submodules as needed)
