use crate::config::LearningConfig;
use crate::error::Result;
use crate::types::{
    AdaptiveThreshold, CalibrationSample, FeedbackEntry, PreferenceEntry, ReinforcementSignal,
    UserPreferenceProfile,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use voxy_memory::types::MemoryItem;

#[derive(Debug, Clone)]
pub struct LearningStats {
    pub preferences_tracked: usize,
    pub behavior_patterns: usize,
    pub reinforcement_signals: usize,
    pub calibration_samples: usize,
    pub feedback_processed: usize,
    pub strategies_active: usize,
    pub thresholds_managed: usize,
    pub is_learning: bool,
    pub last_learning_event: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait LearningEngine: Send + Sync {
    async fn init(&self, config: &LearningConfig) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn process_feedback(&self, feedback: FeedbackEntry) -> Result<()>;
    async fn apply_reinforcement(
        &self,
        action: &str,
        reward: f64,
        context: &HashMap<String, String>,
    ) -> Result<ReinforcementSignal>;
    async fn apply_penalty(
        &self,
        action: &str,
        penalty: f64,
        context: &HashMap<String, String>,
    ) -> Result<ReinforcementSignal>;
    async fn learn_from_memory(&self, memory_items: &[MemoryItem]) -> Result<Vec<PreferenceEntry>>;
    async fn calibrate_confidence(&self, estimator_id: &str) -> Result<AdaptiveThreshold>;
    async fn record_calibration_sample(&self, sample: CalibrationSample) -> Result<()>;
    async fn get_user_profile(&self, user_id: &str) -> Result<UserPreferenceProfile>;
    async fn get_learning_stats(&self) -> Result<LearningStats>;
    async fn pause_learning(&self) -> Result<()>;
    async fn resume_learning(&self) -> Result<()>;
}
