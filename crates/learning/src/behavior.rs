use crate::error::Result;
use crate::types::{BehaviorPattern, ReinforcementSignal};
use async_trait::async_trait;
use voxy_memory::types::MemoryItem;

#[async_trait]
pub trait BehaviorAdaptation: Send + Sync {
    async fn analyze_behavior(
        &self,
        items: &[MemoryItem],
        window_days: u64,
    ) -> Result<Vec<BehaviorPattern>>;
    async fn adapt_behavior(
        &self,
        pattern: &BehaviorPattern,
        signal: &ReinforcementSignal,
    ) -> Result<BehaviorPattern>;
    async fn suggest_adaptation(&self, pattern: &BehaviorPattern) -> Result<String>;
    async fn apply_adaptation(
        &self,
        pattern: &BehaviorPattern,
        adaptation: &str,
    ) -> Result<BehaviorPattern>;
    async fn track_effectiveness(&self, pattern: &BehaviorPattern) -> Result<f64>;
}
