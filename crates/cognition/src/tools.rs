use async_trait::async_trait;

use crate::context::AssembledContext;
use crate::error::Result;
use crate::planner::Plan;
use crate::planner::PlanStep;
use crate::types::ConfidenceScore;
use voxy_skills::capabilities::CapabilityId;

#[derive(Debug, Clone)]
pub struct ToolDescription {
    pub tool_id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<CapabilityId>,
    pub confidence_score: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ToolSelection {
    pub selected_tool: ToolDescription,
    pub confidence: ConfidenceScore,
    pub alternatives: Vec<(ToolDescription, f64)>,
    pub selection_rationale: String,
}

pub struct ToolSelectionInput<'a> {
    pub plan: &'a Plan,
    pub step: &'a PlanStep,
    pub context: &'a AssembledContext,
    pub available_tools: Vec<ToolDescription>,
}

#[async_trait]
pub trait ToolSelector: Send + Sync {
    async fn select_tool(&self, input: &ToolSelectionInput<'_>) -> Result<ToolSelection>;
    async fn rank_tools(
        &self,
        tools: &[ToolDescription],
        context: &AssembledContext,
    ) -> Result<Vec<(ToolDescription, f64)>>;
    async fn validate_tool(&self, tool: &ToolDescription, step: &PlanStep) -> Result<bool>;
}
