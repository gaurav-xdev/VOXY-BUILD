pub mod autonomous_workflows;
pub mod bridge;
pub mod config;
pub mod curiosity;
pub mod decision_engine;
pub mod error;
pub mod experience_replay;
pub mod goal_engine_v2;
pub mod goal_manager;
pub mod knowledge_validation;
pub mod orchestrator;
pub mod owner_command_center;
pub mod planning;
pub mod project_manager;
pub mod reflection;
pub mod sdk;
pub mod self_improvement;
pub mod skill_discovery;
pub mod workflow_learning;

pub use bridge::{CognitiveBridge, CognitiveEvent};
pub use config::OrchestratorConfig;
pub use error::{CognitiveError, Result};

pub mod prelude {
    pub use crate::autonomous_workflows::WorkflowEngine;
    pub use crate::bridge::{CognitiveBridge, CognitiveEvent};
    pub use crate::config::OrchestratorConfig;
    pub use crate::decision_engine::DecisionEngine;
    pub use crate::error::{CognitiveError, Result};
    pub use crate::goal_engine_v2::GoalEngineV2;
    pub use crate::owner_command_center::OwnerCommandCenter;
    pub use crate::project_manager::ProjectManager;
    pub use crate::sdk::{Platform, PlatformQuery, PluginRegistry};
    pub use crate::self_improvement::SelfImprovementEngine;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        let _config = OrchestratorConfig::default();
    }
}
