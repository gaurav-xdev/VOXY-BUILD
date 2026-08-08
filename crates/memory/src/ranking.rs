use crate::error::Result;
use crate::retrieval::SearchResult;
use crate::types::ImportanceFactors;
use crate::types::ImportanceScore;
use crate::types::MemoryItem;
use crate::types::MemoryQuery;
use voxy_world_model::context::WorldContext;

#[async_trait::async_trait]
pub trait ImportanceScorer: Send + Sync {
    async fn score(
        &self,
        item: &MemoryItem,
        context: Option<&WorldContext>,
    ) -> Result<ImportanceScore>;
    async fn calculate_factors(&self, item: &MemoryItem) -> Result<ImportanceFactors>;
    async fn normalize(&self, score: f64) -> f64;
}

#[async_trait::async_trait]
pub trait MemoryRanker: Send + Sync {
    async fn rank(&self, items: Vec<MemoryItem>, query: &MemoryQuery) -> Result<Vec<SearchResult>>;
    async fn rank_by_importance(&self, items: Vec<MemoryItem>) -> Result<Vec<(MemoryItem, f64)>>;
    async fn rank_by_recency(&self, items: Vec<MemoryItem>) -> Result<Vec<(MemoryItem, f64)>>;
    async fn rank_by_relevance(
        &self,
        items: Vec<MemoryItem>,
        context: &str,
    ) -> Result<Vec<(MemoryItem, f64)>>;
}
