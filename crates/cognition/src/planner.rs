use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::context::AssembledContext;
use crate::error::Result;
use crate::goals::GoalDecomposition;
use crate::types::{GoalId, PlanId, StepId};
use voxy_skills::capabilities::CapabilityId;

#[derive(Debug, Clone, PartialEq)]
pub enum StepState {
    Pending,
    Ready,
    InProgress,
    Completed,
    Failed(String),
    Skipped,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum StepType {
    Atomic,
    Composite(Vec<StepId>),
    Conditional {
        condition: String,
        then_branch: Vec<StepId>,
        else_branch: Vec<StepId>,
    },
    Parallel(Vec<StepId>),
    Loop {
        max_iterations: u32,
    },
}

#[derive(Debug, Clone)]
pub struct PlanStep {
    pub id: StepId,
    pub description: String,
    pub step_type: StepType,
    pub dependencies: Vec<StepId>,
    pub required_capabilities: Vec<CapabilityId>,
    pub state: StepState,
    pub estimated_duration_ms: u64,
    pub max_retries: u32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum PlanState {
    Draft,
    Validated,
    InProgress,
    Completed { success: bool },
    Failed(String),
    Cancelled,
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub id: PlanId,
    pub goals: Vec<GoalId>,
    pub steps: Vec<PlanStep>,
    pub state: PlanState,
    pub estimated_total_duration_ms: u64,
    pub parallelism_possible: bool,
    pub fallback_plan_id: Option<PlanId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for context-aware planning decisions.
#[derive(Debug, Clone)]
pub struct PlanningContext {
    /// The assembled context from the fusion engine.
    pub assembled: AssembledContext,

    /// Whether context has changed since last plan.
    pub context_changed: bool,

    /// Stale sources that need refresh.
    pub stale_sources: Vec<String>,

    /// Overall context confidence.
    pub context_confidence: f64,
}

#[async_trait]
pub trait Planner: Send + Sync {
    async fn create_plan(
        &self,
        decomposition: &GoalDecomposition,
        planning_context: &PlanningContext,
    ) -> Result<Plan>;
    async fn validate_plan(&self, plan: &Plan) -> Result<bool>;
    async fn optimize_plan(&self, plan: &Plan, planning_context: &PlanningContext) -> Result<Plan>;
    async fn create_fallback(&self, plan: &Plan, reason: &str) -> Result<Plan>;
    async fn estimate_duration(&self, plan: &Plan) -> Result<u64>;
    async fn can_parallelize(&self, plan: &Plan) -> Result<bool>;

    /// React to context changes — may reschedule or modify the plan.
    async fn on_context_change(
        &self,
        plan: &Plan,
        planning_context: &PlanningContext,
    ) -> Result<PlanAction>;
}

/// Action the planner recommends when context changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanAction {
    /// Continue as planned.
    Continue,

    /// Pause the plan due to context changes.
    Pause(String),

    /// Reschedule affected steps.
    Reschedule(Vec<StepId>),

    /// Abort the plan entirely.
    Abort(String),

    /// Switch to fallback plan.
    SwitchToFallback(PlanId),
}
