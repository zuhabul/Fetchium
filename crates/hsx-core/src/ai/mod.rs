//! AI integration — multi-provider synthesis, model routing, sandwich layout (PRD §23).
//!
//! ## Pipeline (Mode D — `hsx ai`)
//! `Search → QATBE Extract → Sandwich Layout → AI Provider → Citation Injection → Output`
//!
//! ## Supported Providers (2026)
//! | Provider   | Models                                           |
//! |------------|--------------------------------------------------|
//! | Ollama     | gemma3:1b … qwen3:30b-a3b (local, no key)       |
//! | OpenAI     | gpt-4o-mini, gpt-4o, o3-mini                     |
//! | Anthropic  | claude-haiku-4-5, claude-sonnet-4-6              |
//! | Gemini     | gemini-2.0-flash (default), gemini-1.5-pro       |
//! | OpenRouter | 100+ models via one API key                      |
//! | GeminiCli  | gemini-2.0-flash (local CLI, no key needed)      |

pub mod credentials;
pub mod ollama;
pub mod pipeline;
pub mod prompt;
pub mod provider_client;
pub mod providers;
pub mod router;
pub mod sandwich;
pub mod setup;
pub mod types;

pub use credentials::{detect_subscription_auth, SubscriptionAuth};
pub use ollama::OllamaClient;
pub use pipeline::run_ai_pipeline;
pub use provider_client::{chat_with_fallback, check_provider, ChatResult, ProviderStatus};
pub use providers::{ProviderEntry, ProviderKind, ProvidersConfig};
pub use router::{route_model, select_model};
pub use sandwich::{assemble_context, sandwich_layout};
pub use setup::{best_model_name, format_no_models_hint, format_setup_guide, recommend_models, DeviceSpec, ModelRecommendation, RecommendCategory};
pub use types::{AiConfig, AiPreviewResult, ChatMessage, ModelTier, OllamaModel, RankedSource, SandwichContext};
