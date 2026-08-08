//! Agent lifecycle, registry, task delegation, and supervision.

pub mod error;
pub mod orchestrator;

pub use error::{AgentError, Result};
pub use orchestrator::{
    AgentId, AgentInfo, AgentMessage, AgentRole, AgentStatus, AgentTask, MessageType,
    MultiAgentOrchestrator, OrchestratorError, TaskAssignmentId, TaskAssignmentStatus,
};

/// Agent runtime managing multiple agents.
pub struct AgentRuntime;

impl AgentRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent trait.
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    fn id(&self) -> &str;
    async fn run(&self) -> Result<()>;
    async fn stop(&self) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_runtime_creates() {
        let _r = AgentRuntime::new();
    }
}
