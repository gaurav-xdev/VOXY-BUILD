use crate::error::Result;
use crate::types::{AdaptiveThreshold, FeedbackEntry};
use async_trait::async_trait;
use serde_json;
use std::collections::HashMap;

#[async_trait]
pub trait LearningPolicy: Send + Sync {
    async fn get_policy(&self, policy_id: &str) -> Result<serde_json::Value>;
    async fn set_policy(&self, policy_id: &str, value: serde_json::Value) -> Result<()>;
    async fn list_policies(&self) -> Result<Vec<(String, serde_json::Value)>>;
    async fn evaluate_policy(
        &self,
        policy_id: &str,
        context: &HashMap<String, String>,
    ) -> Result<bool>;
    async fn reset_policy(&self, policy_id: &str) -> Result<()>;
    async fn update_from_feedback(&self, feedback: &FeedbackEntry) -> Result<Vec<String>>;
}

#[async_trait]
pub trait AdaptiveThresholds: Send + Sync {
    async fn get_threshold(&self, threshold_id: &str) -> Result<AdaptiveThreshold>;
    async fn set_threshold(&self, threshold: AdaptiveThreshold) -> Result<()>;
    async fn adjust_threshold(
        &self,
        threshold_id: &str,
        delta: f64,
        reason: &str,
    ) -> Result<AdaptiveThreshold>;
    async fn reset_threshold(&self, threshold_id: &str) -> Result<()>;
    async fn list_thresholds(&self) -> Result<Vec<AdaptiveThreshold>>;
    async fn auto_adjust(&self, threshold_id: &str, performance: f64) -> Result<AdaptiveThreshold>;
}
