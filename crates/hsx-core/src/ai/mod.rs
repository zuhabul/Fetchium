//! AI integration — multi-provider synthesis, model routing, sandwich layout (PRD §23).
//!
//! ## Pipeline (Mode D — `hsx ai`)
//! `Search → QATBE Extract → Sandwich Layout → AI Provider → Citation Injection → Output`
//!
//! ## Supported Providers (2026)
//! | Provider      | Auth                        | Models                                          |
//! |---------------|-----------------------------|-------------------------------------------------|
//! | Antigravity   | OAuth (opencode-antgrav.)   | antigravity-gemini-3-*, antigravity-claude-*    |
//! | Ollama        | local (no key)              | gemma3:1b … qwen3:30b-a3b                       |
//! | OpenAI        | API key or Codex CLI OAuth  | gpt-4o-mini, gpt-4o, o3-mini                   |
//! | Anthropic     | API key or Claude Code OAuth| claude-haiku-4-5, claude-sonnet-4-6             |
//! | Gemini        | API key or Gemini CLI OAuth | gemini-2.0-flash (default), gemini-2.5-pro      |
//! | OpenRouter    | API key                     | 100+ models via one API key                     |
//! | GeminiCli     | local binary + subscription | gemini-2.0-flash                                |

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

pub use credentials::{detect_subscription_auth, hsx_auth_all, hsx_auth_get, hsx_auth_path, hsx_auth_remove, hsx_auth_set, invalidate_gemini_creds, HsxAuth, SubscriptionAuth};
pub use ollama::OllamaClient;
pub use pipeline::run_ai_pipeline;
pub use provider_client::{chat_with_fallback, check_provider, ChatResult, ProviderStatus};
pub use providers::{ProviderEntry, ProviderKind, ProvidersConfig};
pub use router::{route_model, select_model};
pub use sandwich::{assemble_context, sandwich_layout};
pub use setup::{best_model_name, format_no_models_hint, format_setup_guide, recommend_models, DeviceSpec, ModelRecommendation, RecommendCategory};
pub use types::{AiConfig, AiPreviewResult, ChatMessage, ModelTier, OllamaModel, RankedSource, SandwichContext};
