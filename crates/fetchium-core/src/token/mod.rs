//! Token system — QATBE, SCS, PDS, budget management (PRD SS17-18, SS27).

pub mod budget;
pub mod counter;
pub mod pds;
pub mod qatbe;
pub mod scs;

pub use budget::{compute_budget, suggest_pds_tier, AdaptiveBudget, BudgetConfig, BudgetTier};
pub use counter::{count_tokens, estimate_tokens_fast, TokenBudget};
