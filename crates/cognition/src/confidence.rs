use async_trait::async_trait;

use crate::context::AssembledContext;
use crate::error::Result;
use crate::intent::IntentAnalysis;
use crate::planner::{Plan, PlanStep};
use crate::reasoning::ReasoningOutput;
use crate::tools::ToolDescription;
use crate::types::ConfidenceScore;

#[async_trait]
pub trait ConfidenceEstimator: Send + Sync {
    async fn estimate_intent_confidence(&self, intent: &IntentAnalysis) -> Result<ConfidenceScore>;
    async fn estimate_plan_confidence(&self, plan: &Plan) -> Result<ConfidenceScore>;
    async fn estimate_step_confidence(
        &self,
        step: &PlanStep,
        context: &AssembledContext,
    ) -> Result<ConfidenceScore>;
    async fn estimate_reasoning_confidence(
        &self,
        output: &ReasoningOutput,
    ) -> Result<ConfidenceScore>;
    async fn estimate_tool_confidence(
        &self,
        tool: &ToolDescription,
        context: &AssembledContext,
    ) -> Result<ConfidenceScore>;
    async fn aggregate_confidence(&self, scores: &[ConfidenceScore]) -> Result<ConfidenceScore>;
}
