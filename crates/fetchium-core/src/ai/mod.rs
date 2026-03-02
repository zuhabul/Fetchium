//! AI integration — multi-provider synthesis, model routing, sandwich layout (PRD §23).
//!
//! ## Pipeline (Mode D — `fetchium ai`)
//! `Search → QATBE Extract → Sandwich Layout → AI Provider → Citation Injection → Output`
//!
//! ## Supported Providers (2026)
//! | Provider      | Auth                        | Default Model                    |
//! |---------------|-----------------------------|----------------------------------|
//! | GeminiCli     | local binary + subscription | gemini-3-flash-preview           |
//! | Antigravity   | OAuth (opencode-antgrav.)   | antigravity-gemini-3-flash       |
//! | Anthropic     | API key or Claude Code OAuth| claude-haiku-4-5-20251001        |
//! | OpenAI        | API key or Codex CLI OAuth  | gpt-4o-mini                      |
//! | Gemini        | API key or Gemini CLI OAuth | gemini-3-flash-preview           |
//! | OpenRouter    | API key                     | google/gemini-2.5-flash          |
//! | Ollama        | local (no key)              | qwen3:8b                         |
//!
//! All model defaults are defined in [`ModelRegistry`] — no hardcoded strings elsewhere.

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

pub use credentials::{
    detect_subscription_auth, hsx_auth_all, hsx_auth_get, hsx_auth_path, hsx_auth_remove,
    hsx_auth_set, invalidate_gemini_creds, HsxAuth, SubscriptionAuth,
};
pub use ollama::OllamaClient;
pub use pipeline::run_ai_pipeline;
pub use provider_client::{chat_with_fallback, check_provider, ChatResult, ProviderStatus};
pub use providers::{
    ModelCapability, ModelInfo, ModelRegistry, ProviderEntry, ProviderKind, ProvidersConfig,
};
pub use router::{route_model, select_model};
pub use sandwich::{assemble_context, sandwich_layout};
pub use setup::{
    best_model_name, format_no_models_hint, format_setup_guide, recommend_models, DeviceSpec,
    ModelRecommendation, RecommendCategory,
};
pub use types::{
    AiConfig, AiPreviewResult, ChatMessage, ModelTier, OllamaModel, RankedSource, SandwichContext,
};
