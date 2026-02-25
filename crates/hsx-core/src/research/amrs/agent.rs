//! Agent trait for AMRS agents (PRD §8.8).

use crate::error::HsxError;
use crate::research::amrs::channel::{AgentReceiver, AgentSender, AgentType};
use async_trait::async_trait;

/// Trait that all AMRS agents implement.
#[async_trait]
pub trait Agent: Send + Sync {
    /// The type identifier of this agent.
    fn agent_type(&self) -> AgentType;

    /// Run the agent's main loop.
    ///
    /// Receives instructions via `rx`, sends results back via `tx`.
    /// Should exit cleanly when `AgentMessage::Shutdown` is received or `rx` closes.
    async fn run(&self, rx: AgentReceiver, tx: AgentSender) -> Result<(), HsxError>;
}
