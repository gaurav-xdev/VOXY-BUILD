use crate::error::Result;
use crate::types::ReinforcementSignal;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait ReinforcementHook: Send + Sync {
    async fn register_hook(&self, hook_id: &str, trigger: &str) -> Result<()>;
    async fn unregister_hook(&self, hook_id: &str) -> Result<()>;
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
    async fn get_recent_signals(&self, limit: usize) -> Result<Vec<ReinforcementSignal>>;
    async fn get_signal_statistics(&self) -> Result<(f64, f64, f64)>;
    async fn clear_signals(&self) -> Result<()>;
}
