use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::context::AssembledContext;
use crate::error::Result;
use crate::intent::IntentAnalysis;
use crate::types::GoalId;
use crate::types::IntentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GoalState {
    Pending,
    Active,
    InProgress,
    Blocked(String),
    Completed,
    Failed(String),
    Cancelled,
}

impl fmt::Display for GoalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Active => write!(f, "Active"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Blocked(reason) => write!(f, "Blocked({})", reason),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(reason) => write!(f, "Failed({})", reason),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: GoalId,
    pub description: String,
    pub priority: GoalPriority,
    pub dependencies: Vec<GoalId>,
    pub state: GoalState,
    pub acceptance_criteria: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Context snapshot at goal creation time.
    pub context_snapshot: Option<AssembledContext>,
    /// Whether this goal was paused due to context change.
    pub paused_by_context: bool,
}

#[derive(Debug, Clone)]
pub struct GoalDecomposition {
    pub intent_id: IntentId,
    pub goals: Vec<Goal>,
    pub dependency_graph: Vec<(GoalId, GoalId)>,
    pub estimated_complexity: f64,
    pub timestamp: DateTime<Utc>,
}

/// Result of goal re-prioritization based on context.
#[derive(Debug, Clone)]
pub struct PrioritizationResult {
    /// Goals in new priority order.
    pub goals: Vec<(Goal, GoalPriority)>,

    /// Goals that were paused due to context changes.
    pub paused: Vec<GoalId>,

    /// Goals that were activated.
    pub activated: Vec<GoalId>,

    /// Goals that were blocked by new constraints.
    pub blocked: Vec<(GoalId, String)>,
}

#[async_trait]
pub trait GoalDecomposer: Send + Sync {
    async fn decompose(&self, intent: &IntentAnalysis) -> Result<GoalDecomposition>;
    async fn refine_goal(&self, goal: &Goal, context: &AssembledContext) -> Result<Goal>;
    async fn prioritize(&self, goals: Vec<Goal>) -> Result<Vec<(Goal, GoalPriority)>>;

    /// Re-prioritize goals based on updated context.
    /// Handles: meeting starts → pause coding goal, activate meeting goal.
    async fn reprioritize(
        &self,
        goals: Vec<Goal>,
        context: &AssembledContext,
    ) -> Result<PrioritizationResult>;

    /// Check if any goal should be paused due to context changes.
    async fn check_context_triggers(
        &self,
        goals: &[Goal],
        context: &AssembledContext,
    ) -> Result<Vec<GoalTrigger>>;

    /// Get all active goals.
    async fn active_goals(&self) -> Result<Vec<Goal>>;
}

/// A trigger that fires when context changes affect goals.
#[derive(Debug, Clone)]
pub enum GoalTrigger {
    /// Goal should be paused with a reason.
    Pause { goal_id: GoalId, reason: String },

    /// Goal should be resumed.
    Resume { goal_id: GoalId },

    /// Goal should be activated.
    Activate { goal_id: GoalId, reason: String },

    /// Goal priority should change.
    Reprioritize {
        goal_id: GoalId,
        new_priority: GoalPriority,
    },
}
