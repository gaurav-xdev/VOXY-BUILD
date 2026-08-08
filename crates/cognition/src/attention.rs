use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::ConfidenceScore;

/// Factors contributing to attention scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionFactors {
    /// Whether there is an active conversation (0.0 - 1.0).
    pub conversation_active: f64,

    /// Foreground application importance (0.0 - 1.0).
    pub foreground_app_importance: f64,

    /// Voice activity level (0.0 - 1.0).
    pub voice_activity: f64,

    /// Number of pending notifications (normalized 0.0 - 1.0).
    pub notification_pressure: f64,

    /// Urgency of current intent (0.0 - 1.0).
    pub urgency: f64,

    /// Relevance to current goal (0.0 - 1.0).
    pub goal_relevance: f64,

    /// Freshness of context data (0.0 - 1.0).
    pub context_freshness: f64,
}

impl Default for AttentionFactors {
    fn default() -> Self {
        Self {
            conversation_active: 0.0,
            foreground_app_importance: 0.5,
            voice_activity: 0.0,
            notification_pressure: 0.0,
            urgency: 0.0,
            goal_relevance: 0.5,
            context_freshness: 1.0,
        }
    }
}

/// Result of attention scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionScore {
    /// Overall attention score (0.0 - 1.0).
    pub score: f64,

    /// Weighted breakdown by factor.
    pub breakdown: HashMap<String, f64>,

    /// Confidence in the score.
    pub confidence: ConfidenceScore,

    /// Recommended action based on attention.
    pub recommendation: AttentionRecommendation,
}

/// What the attention system recommends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionRecommendation {
    /// Focus on the primary task.
    FocusPrimary,

    /// Switch to a higher-priority task.
    SwitchTask,

    /// Pause current activity.
    Pause,

    /// Resume previous activity.
    Resume,

    /// Alert the user.
    Alert,

    /// No action needed.
    Idle,
}

/// Trait for the attention system — computes attention scores from context.
#[async_trait]
pub trait AttentionSystem: Send + Sync {
    /// Compute attention score from the given factors.
    async fn score(&self, factors: &AttentionFactors) -> Result<AttentionScore>;

    /// Update weights for attention factors.
    async fn update_weights(&self, weights: HashMap<String, f64>) -> Result<()>;

    /// Get current attention state.
    async fn current_attention(&self) -> Result<AttentionScore>;

    /// Check if attention should shift based on new factors.
    async fn should_shift(&self, new_factors: &AttentionFactors) -> Result<bool>;
}
