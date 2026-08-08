use async_trait::async_trait;

use crate::context::AssembledContext;
use crate::error::Result;
use crate::types::ConfidenceScore;

#[derive(Debug, Clone)]
pub struct ReasoningInput {
    pub query: String,
    pub context: AssembledContext,
    pub constraints: Vec<String>,
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct ReasoningStep {
    pub index: usize,
    pub premise: String,
    pub inference: String,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct ReasoningOutput {
    pub conclusion: String,
    pub confidence: ConfidenceScore,
    pub steps: Vec<ReasoningStep>,
    pub duration_ms: u64,
    /// Context freshness at time of reasoning.
    pub context_freshness: f64,
    /// Which sources contributed to reasoning.
    pub contributing_sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ReasoningStrategy {
    Deductive,
    Inductive,
    Abductive,
    Analogical,
    Causal,
    Critical,
    MultiStep { max_steps: usize },
}

#[async_trait]
pub trait Reasoner: Send + Sync {
    async fn reason(&self, input: &ReasoningInput) -> Result<ReasoningOutput>;
    async fn evaluate(&self, claim: &str, context: &AssembledContext) -> Result<ConfidenceScore>;
    async fn compare(&self, options: &[String], criteria: &[String]) -> Result<Vec<(String, f64)>>;

    /// Reason with context awareness — uses assembled context data.
    async fn reason_with_context(
        &self,
        query: &str,
        context: &AssembledContext,
    ) -> Result<ReasoningOutput>;
}
