use crate::error::Result;
use crate::types::MemoryItem;
use crate::types::MemoryQuery;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub item: MemoryItem,
    pub score: f64,
    pub match_reasons: Vec<String>,
}

#[async_trait::async_trait]
pub trait RetrievalEngine: Send + Sync {
    async fn search(&self, query: &MemoryQuery) -> Result<Vec<SearchResult>>;
    async fn search_by_similarity(
        &self,
        query_text: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    async fn search_by_importance(
        &self,
        min_importance: f64,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    async fn search_by_context(
        &self,
        context_tags: &[String],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    async fn search_by_time_range(
        &self,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
    async fn hybrid_search(
        &self,
        query_text: &str,
        min_importance: f64,
        context_tags: &[String],
        limit: usize,
    ) -> Result<Vec<SearchResult>>;
}
