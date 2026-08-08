use crate::config::MemoryConfig;
use crate::error::Result;
use crate::graph::KnowledgeGraph;
use crate::hermes::HermesClassification;
use crate::hermes::HermesEngine;
use crate::retrieval::SearchResult;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use crate::types::MemoryQuery;
use voxy_world_model::context::WorldContext;

pub struct MemoryStats {
    pub total_items: usize,
    pub working_count: usize,
    pub short_term_count: usize,
    pub episodic_count: usize,
    pub semantic_count: usize,
    pub procedural_count: usize,
    pub vector_count: usize,
    pub graph_nodes: usize,
    pub graph_edges: usize,
    pub active_count: usize,
    pub dormant_count: usize,
    pub compressed_count: usize,
    pub archived_count: usize,
}

#[async_trait::async_trait]
pub trait MemoryApi: Send + Sync {
    async fn init(&self, config: &MemoryConfig) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn store(&self, item: MemoryItem) -> Result<MemoryId>;
    async fn store_with_analysis(
        &self,
        item: MemoryItem,
        context: Option<&WorldContext>,
    ) -> Result<(MemoryId, HermesClassification)>;
    async fn retrieve(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn search(&self, query: &MemoryQuery) -> Result<Vec<SearchResult>>;
    async fn update(&self, item: MemoryItem) -> Result<()>;
    async fn delete(&self, id: &MemoryId) -> Result<()>;
    async fn forget(&self, id: &MemoryId) -> Result<()>;
    async fn recall(&self, id: &MemoryId) -> Result<MemoryItem>;
    async fn consolidate(&self) -> Result<usize>;
    async fn run_forgetting(&self) -> Result<usize>;
    async fn graph(&self) -> &dyn KnowledgeGraph;
    async fn hermes(&self) -> &dyn HermesEngine;
    async fn stats(&self) -> Result<MemoryStats>;
    async fn clear(&self) -> Result<()>;
}
