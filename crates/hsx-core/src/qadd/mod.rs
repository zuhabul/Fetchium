//! QADD — Query-Aware DOM Distillation (PRD §8.10).
//!
//! 5-step pipeline: structural pruning → BM25 scoring → semantic check →
//! heading context preservation → greedy token-budget packing.
//!
//! Target: 50K token HTML → ~2.5K tokens (10-20x reduction).

pub mod pipeline;
pub mod pruning;

pub use pipeline::{QaddConfig, QaddPipeline, QaddResult};
pub use pruning::TextNode;
