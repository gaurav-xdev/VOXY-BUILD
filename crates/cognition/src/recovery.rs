use async_trait::async_trait;

use crate::error::Result;
use crate::planner::{Plan, PlanStep};
use crate::types::{PlanId, StepId};

#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    Retry { max_attempts: u32, backoff_ms: u64 },
    FallbackPlan(PlanId),
    SimplifyGoal,
    SkipStep,
    AbortWithError(String),
    RequestHumanAssistance(String),
}

#[derive(Debug, Clone)]
pub struct RecoveryPlan {
    pub strategy: RecoveryStrategy,
    pub affected_steps: Vec<StepId>,
    pub rationale: String,
    pub estimated_recovery_time_ms: u64,
    pub requires_user_consent: bool,
}

#[async_trait]
pub trait RecoveryManager: Send + Sync {
    async fn diagnose(
        &self,
        plan_id: &PlanId,
        failed_step: &PlanStep,
        error: &str,
    ) -> Result<Vec<RecoveryStrategy>>;
    async fn create_recovery_plan(
        &self,
        strategies: &[RecoveryStrategy],
        plan: &Plan,
    ) -> Result<RecoveryPlan>;
    async fn execute_recovery(&self, plan: &RecoveryPlan) -> Result<()>;
    async fn report_failure(&self, plan_id: &PlanId, error: &str) -> Result<()>;
}
