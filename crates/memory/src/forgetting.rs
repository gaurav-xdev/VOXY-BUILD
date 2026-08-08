use crate::error::Result;
use crate::types::ForgettingPolicy;
use crate::types::MemoryItem;
use crate::types::MemoryState;

#[async_trait::async_trait]
pub trait ForgettingEngine: Send + Sync {
    async fn decay(&self, item: &MemoryItem, policy: &ForgettingPolicy) -> Result<MemoryState>;
    async fn should_transition(
        &self,
        item: &MemoryItem,
        policy: &ForgettingPolicy,
    ) -> Option<MemoryState>;
    async fn transition(&self, item: &MemoryItem, new_state: MemoryState) -> Result<MemoryItem>;
    async fn get_forgetting_candidates(
        &self,
        policy: &ForgettingPolicy,
    ) -> Result<Vec<(MemoryItem, MemoryState)>>;
    async fn run_scheduled(&self) -> Result<usize>;
    async fn recall(&self, item: &MemoryItem) -> Result<MemoryItem>;
    async fn permanent_delete(&self, item: &MemoryItem) -> Result<()>;
}
