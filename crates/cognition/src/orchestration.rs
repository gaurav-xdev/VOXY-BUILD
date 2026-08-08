use async_trait::async_trait;

use crate::attention::AttentionScore;
use crate::config::CognitionConfig;
use crate::context::AssembledContext;
use crate::error::Result;
use crate::intent::{IntentAnalysis, IntentInput};
use crate::planner::Plan;
use crate::reflection::ReflectionReport;
use crate::types::{CognitiveState, ConfidenceScore, IntentId, PlanId};

/// Decision output with explanation for debugging.
#[derive(Debug, Clone)]
pub struct DecisionOutput {
    /// The decision made.
    pub decision: String,

    /// Confidence in the decision.
    pub confidence: ConfidenceScore,

    /// Reason for the decision.
    pub reason: String,

    /// Context that influenced the decision.
    pub context_summary: String,

    /// Priority of the decision.
    pub priority: String,

    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Full cognitive result with all subsystem outputs.
#[derive(Debug, Clone)]
pub struct CognitiveResult {
    pub intent: IntentAnalysis,
    pub plan: Option<Plan>,
    pub context: Option<AssembledContext>,
    pub result: serde_json::Value,
    pub confidence: ConfidenceScore,
    pub reflection: Option<ReflectionReport>,
    pub duration_ms: u64,
    pub success: bool,
    pub errors: Vec<String>,
    /// Decision output with explanation.
    pub decision: Option<DecisionOutput>,
    /// Attention score at time of processing.
    pub attention_score: Option<AttentionScore>,
}

#[async_trait]
pub trait CognitiveEngine: Send + Sync {
    async fn init(&self, config: &CognitionConfig) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn process(&self, input: &IntentInput) -> Result<CognitiveResult>;
    async fn process_streaming(&self, input: &IntentInput) -> Result<CognitiveResult>;
    async fn state(&self) -> CognitiveState;
    async fn current_intent(&self) -> Option<IntentId>;
    async fn current_plan(&self) -> Option<PlanId>;
    async fn cancel(&self, intent_id: &IntentId) -> Result<()>;
    async fn pause(&self) -> Result<()>;
    async fn resume(&self) -> Result<()>;

    /// Process with explicit context from the fusion engine.
    async fn process_with_context(
        &self,
        input: &IntentInput,
        assembled: &AssembledContext,
    ) -> Result<CognitiveResult>;

    /// Get the latest diagnostics snapshot.
    async fn diagnostics(&self) -> Result<serde_json::Value>;
}
