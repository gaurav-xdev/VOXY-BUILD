use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::attention::AttentionScore;
use crate::error::Result;
use crate::self_monitoring::SystemSnapshot;

/// Current context summary for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    /// Number of active context sources.
    pub source_count: usize,

    /// Overall context confidence.
    pub confidence: f64,

    /// List of active source names.
    pub sources: Vec<String>,

    /// Context freshness (seconds since last update).
    pub freshness_secs: u64,
}

/// Diagnostics snapshot — developer-only view of cognitive state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSnapshot {
    /// Current cognitive state.
    pub cognitive_state: String,

    /// Current context summary.
    pub context: ContextSummary,

    /// Fusion confidence score.
    pub fusion_confidence: f64,

    /// Currently active goal descriptions.
    pub active_goals: Vec<String>,

    /// Current attention score.
    pub attention: AttentionScore,

    /// Reasoning stage if active.
    pub reasoning_stage: Option<String>,

    /// Current decision if any.
    pub current_decision: Option<String>,

    /// Latest reflection report summary.
    pub reflection_summary: Option<String>,

    /// System health snapshot.
    pub system_health: SystemSnapshot,

    /// Per-subsystem latencies.
    pub latencies: HashMap<String, LatencyDiagnostics>,
}

/// Latency diagnostics for a single subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDiagnostics {
    pub last_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
    pub sample_count: u64,
}

/// Trait for the diagnostics interface — provides developer-only system visibility.
#[async_trait]
pub trait Diagnostics: Send + Sync {
    /// Get a full diagnostics snapshot.
    async fn snapshot(&self) -> Result<DiagnosticsSnapshot>;

    /// Get current cognitive state.
    async fn cognitive_state(&self) -> Result<String>;

    /// Get current context summary.
    async fn context_summary(&self) -> Result<ContextSummary>;

    /// Get fusion confidence.
    async fn fusion_confidence(&self) -> Result<f64>;

    /// Get active goals.
    async fn active_goals(&self) -> Result<Vec<String>>;

    /// Get attention score.
    async fn attention_score(&self) -> Result<AttentionScore>;

    /// Get reasoning stage.
    async fn reasoning_stage(&self) -> Result<Option<String>>;

    /// Get current decision.
    async fn current_decision(&self) -> Result<Option<String>>;

    /// Get reflection summary.
    async fn reflection_summary(&self) -> Result<Option<String>>;
}
