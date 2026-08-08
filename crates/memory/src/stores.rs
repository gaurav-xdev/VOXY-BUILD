use crate::error::Result;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use chrono::{DateTime, Utc};

#[async_trait::async_trait]
pub trait WorkingMemory: Send + Sync {
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn remove(&self, id: &MemoryId) -> Result<()>;
    async fn list(&self) -> Result<Vec<MemoryItem>>;
    async fn clear(&self) -> Result<()>;
    async fn capacity(&self) -> usize;
    async fn used(&self) -> usize;
}

#[async_trait::async_trait]
pub trait ShortTermMemory: Send + Sync {
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn remove(&self, id: &MemoryId) -> Result<()>;
    async fn list(&self) -> Result<Vec<MemoryItem>>;
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<MemoryItem>>;
    async fn find_by_time_range(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<MemoryItem>>;
    async fn count(&self) -> usize;
    async fn clear_expired(&self) -> Result<usize>;
    async fn clear(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait EpisodicMemory: Send + Sync {
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn remove(&self, id: &MemoryId) -> Result<()>;
    async fn list(&self) -> Result<Vec<MemoryItem>>;
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<MemoryItem>>;
    async fn find_by_time_range(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<MemoryItem>>;
    async fn count(&self) -> usize;
    async fn clear_expired(&self) -> Result<usize>;
    async fn clear(&self) -> Result<()>;
    async fn find_by_context(&self, context_tags: &[String]) -> Result<Vec<MemoryItem>>;
    async fn replay_episode(&self, id: &MemoryId) -> Result<Vec<MemoryItem>>;
}

#[async_trait::async_trait]
pub trait SemanticMemory: Send + Sync {
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn remove(&self, id: &MemoryId) -> Result<()>;
    async fn list(&self) -> Result<Vec<MemoryItem>>;
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<MemoryItem>>;
    async fn find_by_time_range(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<MemoryItem>>;
    async fn count(&self) -> usize;
    async fn clear_expired(&self) -> Result<usize>;
    async fn clear(&self) -> Result<()>;
    async fn find_by_source(&self, source: &str) -> Result<Vec<MemoryItem>>;
    async fn get_facts(&self) -> Result<Vec<MemoryItem>>;
}

#[async_trait::async_trait]
pub trait ProceduralMemory: Send + Sync {
    async fn store_procedure(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn retrieve_procedure(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn find_by_step(&self, step_description: &str) -> Result<Vec<MemoryItem>>;
    async fn list_procedures(&self) -> Result<Vec<MemoryItem>>;
    async fn remove_procedure(&self, id: &MemoryId) -> Result<()>;
    async fn clear(&self) -> Result<()>;
}

#[async_trait::async_trait]
pub trait VectorMemory: Send + Sync {
    async fn store_vector(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn remove_vector(&self, id: &MemoryId) -> Result<()>;
    async fn similarity_search(
        &self,
        query: &[f32],
        limit: usize,
    ) -> Result<Vec<(MemoryItem, f64)>>;
    async fn similarity_search_by_id(
        &self,
        id: &MemoryId,
        limit: usize,
    ) -> Result<Vec<(MemoryItem, f64)>>;
    async fn count(&self) -> usize;
}
