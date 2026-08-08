use crate::error::Result;
use crate::provider::ContextProvider;
use crate::types::{ContextPriority, ContextSnapshot, ContextSource};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub category: String,
    pub created_at: String,
    pub relevance: f64,
}

/// Provides memory context (facts, preferences, past events).
pub struct MemoryContextProvider {
    memories: Vec<MemoryEntry>,
    max_memories: usize,
}

impl MemoryContextProvider {
    pub fn new(max_memories: usize) -> Self {
        Self {
            memories: Vec::new(),
            max_memories,
        }
    }

    pub fn add_memory(&mut self, entry: MemoryEntry) {
        if self.memories.len() >= self.max_memories {
            if let Some(min_idx) = self
                .memories
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.relevance
                        .partial_cmp(&b.1.relevance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                self.memories.remove(min_idx);
            }
        }
        self.memories.push(entry);
    }

    pub fn remove_memory(&mut self, id: &str) -> bool {
        let len_before = self.memories.len();
        self.memories.retain(|m| m.id != id);
        self.memories.len() < len_before
    }

    pub fn memories(&self) -> &[MemoryEntry] {
        &self.memories
    }

    pub fn len(&self) -> usize {
        self.memories.len()
    }

    pub fn is_empty(&self) -> bool {
        self.memories.is_empty()
    }
}

impl Default for MemoryContextProvider {
    fn default() -> Self {
        Self::new(100)
    }
}

#[async_trait]
impl ContextProvider for MemoryContextProvider {
    fn name(&self) -> &str {
        "memory"
    }

    fn source(&self) -> ContextSource {
        ContextSource::Memory
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::Medium
    }

    async fn collect(&self) -> Result<ContextSnapshot> {
        let data = serde_json::json!({
            "memory_count": self.memories.len(),
            "memories": self.memories.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "category": m.category,
                    "relevance": m.relevance,
                })
            }).collect::<Vec<_>>(),
        });

        Ok(ContextSnapshot::new(ContextSource::Memory, data))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_memory(id: &str, relevance: f64) -> MemoryEntry {
        MemoryEntry {
            id: id.to_string(),
            content: format!("content-{id}"),
            category: "fact".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            relevance,
        }
    }

    #[test]
    fn test_add_and_remove_memory() {
        let mut provider = MemoryContextProvider::new(3);
        provider.add_memory(make_memory("1", 0.9));
        provider.add_memory(make_memory("2", 0.5));
        assert_eq!(provider.len(), 2);

        provider.remove_memory("1");
        assert_eq!(provider.len(), 1);
        assert!(!provider.is_empty());
    }

    #[test]
    fn test_lru_eviction() {
        let mut provider = MemoryContextProvider::new(2);
        provider.add_memory(make_memory("low", 0.2));
        provider.add_memory(make_memory("high", 0.8));

        provider.add_memory(make_memory("new", 0.5));
        assert_eq!(provider.len(), 2);
        assert!(provider.memories.iter().all(|m| m.id != "low"));
    }

    #[tokio::test]
    async fn test_collect_memory() {
        let mut provider = MemoryContextProvider::new(5);
        provider.add_memory(make_memory("1", 0.9));

        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.source, ContextSource::Memory);
        assert_eq!(snapshot.data["memory_count"].as_u64(), Some(1));
    }
}
