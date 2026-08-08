use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::context::AssembledContext;
use crate::error::Result;
use crate::planner::Plan;
use crate::types::{ReflectionId, StepId};

/// Input for reflection — includes context for comparing expected vs actual.
pub struct ReflectionInput<'a> {
    pub plan: &'a Plan,
    pub execution_results: Vec<(StepId, bool, String)>,
    pub context: &'a AssembledContext,
    /// Expected outcome from planning phase.
    pub expected_outcome: Option<&'a serde_json::Value>,
    /// Actual outcome from execution.
    pub actual_outcome: Option<&'a serde_json::Value>,
    /// Context changes that occurred during execution.
    pub context_changes: Vec<ContextChange>,
}

/// A change in context that occurred during execution.
#[derive(Debug, Clone)]
pub struct ContextChange {
    /// What changed (e.g., "battery_low", "meeting_started").
    pub trigger: String,

    /// When it changed.
    pub timestamp: DateTime<Utc>,

    /// Impact assessment.
    pub impact: String,
}

#[derive(Debug, Clone)]
pub struct ReflectionInsight {
    pub category: String,
    pub description: String,
    pub impact: String,
    pub suggestion: Option<String>,
    pub confidence: f64,
    /// Whether this insight is about context vs execution.
    pub source_type: InsightSource,
}

/// Where a reflection insight originated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsightSource {
    /// Insight about execution performance.
    Execution,

    /// Insight about context accuracy.
    Context,

    /// Insight about goal alignment.
    GoalAlignment,

    /// Insight about planning effectiveness.
    Planning,
}

#[derive(Debug, Clone)]
pub struct ReflectionReport {
    pub reflection_id: ReflectionId,
    pub insights: Vec<ReflectionInsight>,
    pub overall_assessment: String,
    pub improvement_suggestions: Vec<String>,
    pub lessons_learned: Vec<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    /// Comparison of expected vs actual outcome.
    pub outcome_comparison: Option<OutcomeComparison>,
    /// Context changes that affected execution.
    pub context_impact: Vec<ContextChange>,
}

/// Comparison of expected vs actual outcome.
#[derive(Debug, Clone)]
pub struct OutcomeComparison {
    /// How aligned expected and actual outcomes are (0.0 - 1.0).
    pub alignment_score: f64,

    /// Differences found.
    pub differences: Vec<String>,

    /// Whether the difference was due to context changes.
    pub context_caused: bool,

    /// Root cause analysis.
    pub root_cause: Option<String>,
}

#[async_trait]
pub trait ReflectionEngine: Send + Sync {
    async fn reflect(&self, input: &ReflectionInput<'_>) -> Result<ReflectionReport>;
    async fn analyze_performance(
        &self,
        plan: &Plan,
        report: &ReflectionReport,
    ) -> Result<Vec<String>>;
    async fn suggest_improvements(&self, report: &ReflectionReport) -> Result<Vec<String>>;

    /// Compare expected vs actual outcomes.
    async fn compare_outcomes(
        &self,
        expected: &serde_json::Value,
        actual: &serde_json::Value,
        context_changes: &[ContextChange],
    ) -> Result<OutcomeComparison>;

    /// Analyze failure causes.
    async fn analyze_failure_causes(
        &self,
        plan: &Plan,
        execution_results: &[(StepId, bool, String)],
        context: &AssembledContext,
    ) -> Result<Vec<String>>;
}
