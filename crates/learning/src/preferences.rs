use crate::error::Result;
use crate::types::PreferenceEntry;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use voxy_memory::types::MemoryItem;
use voxy_world_model::context::WorldContext;

#[async_trait]
pub trait PreferenceEvolution: Send + Sync {
    async fn extract_preference(
        &self,
        item: &MemoryItem,
        context: Option<&WorldContext>,
    ) -> Result<PreferenceEntry>;
    async fn update_preference(
        &self,
        existing: &PreferenceEntry,
        new_evidence: &PreferenceEntry,
    ) -> Result<PreferenceEntry>;
    async fn merge_preferences(
        &self,
        a: &PreferenceEntry,
        b: &PreferenceEntry,
    ) -> Result<PreferenceEntry>;
    async fn decay_preference(&self, preference: &PreferenceEntry) -> Result<PreferenceEntry>;
    async fn get_stable_preferences(&self, min_confidence: f64) -> Result<Vec<PreferenceEntry>>;
    async fn get_evolving_preferences(&self) -> Result<Vec<PreferenceEntry>>;
    async fn preference_trend(
        &self,
        category: &str,
        days: u64,
    ) -> Result<Vec<(DateTime<Utc>, serde_json::Value)>>;
}
