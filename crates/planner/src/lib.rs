//! Intent to plan transformation: goal decomposition, task sequencing.
//!
//! This crate re-exports planning types from `voxy-cognition` and provides
//! additional planning utilities.

pub mod error;
pub mod task_graph;

pub use error::{PlannerError, Result};
pub use task_graph::{
    GraphError, GraphState, ResourceRequirements, TaskGraph, TaskGraphBuilder, TaskGraphExecutor,
    TaskGraphId, TaskNode, TaskNodeId, TaskNodeState, TaskPriority, TaskType,
};

// Re-export cognition's planner types for compatibility
pub use voxy_cognition::{
    Goal, GoalDecomposer, GoalDecomposition, GoalId, GoalPriority, GoalState,
    InMemoryGoalDecomposer, InMemoryPlanner, Plan, PlanId, PlanState, PlanStep, Planner, StepId,
    StepState, StepType,
};

use voxy_cognition::CognitionConfig;

/// Convenience function to create a default planner with cognition integration.
pub fn default_planner() -> InMemoryPlanner {
    InMemoryPlanner::new(CognitionConfig::default())
}

/// Convenience function to create a default goal decomposer.
pub fn default_goal_decomposer() -> InMemoryGoalDecomposer {
    InMemoryGoalDecomposer::new(CognitionConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planner_creates() {
        let _p = default_planner();
    }

    #[test]
    fn goal_decomposer_creates() {
        let _d = default_goal_decomposer();
    }
}
