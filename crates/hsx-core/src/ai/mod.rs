//! AI integration — Ollama local LLM synthesis, model routing, sandwich layout (PRD §23).
//!
//! ## Pipeline (Mode D — `hsx ai`)
//! `Search → QATBE Extract → Sandwich Layout → Ollama Chat → Citation Injection → Output`
//!
//! ## Model Routing (2026 defaults)
//! | Tier   | Models                                            |
//! |--------|---------------------------------------------------|
//! | Small  | gemma3:1b, qwen3:1.7b                            |
//! | Medium | deepseek-r1:7b, qwen3:8b, gemma3:9b              |
//! | Large  | deepseek-r1:14b, qwen3:14b, llama4:scout         |

pub mod ollama;
pub mod pipeline;
pub mod prompt;
pub mod router;
pub mod sandwich;
pub mod types;

pub use ollama::OllamaClient;
pub use pipeline::run_ai_pipeline;
pub use types::AiPreviewResult;
pub use router::{route_model, select_model};
pub use sandwich::{assemble_context, sandwich_layout};
pub use types::{AiConfig, ChatMessage, ModelTier, OllamaModel, RankedSource, SandwichContext};
