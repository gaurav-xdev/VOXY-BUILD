use crate::error::Result;
use crate::types::{FeedbackEntry, PreferenceEntry};
use async_trait::async_trait;

#[async_trait]
pub trait FeedbackProcessing: Send + Sync {
    async fn process_feedback(&self, feedback: FeedbackEntry) -> Result<()>;
    async fn extract_sentiment(&self, text: &str) -> Result<f64>;
    async fn get_user_feedback(&self, user_id: &str, limit: usize) -> Result<Vec<FeedbackEntry>>;
    async fn get_feedback_stats(&self) -> Result<(usize, f64, f64)>;
    async fn get_target_feedback(&self, target_id: &str) -> Result<Vec<FeedbackEntry>>;
    async fn apply_feedback_to_preferences(
        &self,
        feedback: &FeedbackEntry,
    ) -> Result<Vec<PreferenceEntry>>;
}
