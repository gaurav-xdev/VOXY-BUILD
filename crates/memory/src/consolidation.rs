use crate::error::Result;
use crate::types::ConsolidationPolicy;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use crate::types::MemoryType;

#[async_trait::async_trait]
pub trait ConsolidationEngine: Send + Sync {
    async fn consolidate(&self, policy: &ConsolidationPolicy) -> Result<usize>;
    async fn consolidate_item(
        &self,
        item: &MemoryItem,
        target_type: MemoryType,
    ) -> Result<MemoryId>;
    async fn should_consolidate(&self, item: &MemoryItem, policy: &ConsolidationPolicy) -> bool;
    async fn get_consolidation_candidates(
        &self,
        policy: &ConsolidationPolicy,
    ) -> Result<Vec<MemoryItem>>;
    async fn run_scheduled(&self) -> Result<usize>;
}
