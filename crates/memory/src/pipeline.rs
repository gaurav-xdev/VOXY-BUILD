use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::Utc;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::config::MemoryConfig;
use crate::error::Result;
use crate::hermes::{HermesClassification, ReflectionLearning};
use crate::types::HermesDecision;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use crate::types::MemoryType;
use voxy_world_model::context::WorldContext;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MemoryPhase {
    Input,
    ImportanceAnalysis,
    HermesAnalysis,
    Classification,
    Storage,
    Indexing,
    Retrieval,
    Reflection,
    Learning,
}

#[async_trait::async_trait]
pub trait MemoryPipeline: Send + Sync {
    async fn process(
        &self,
        item: MemoryItem,
        context: Option<&WorldContext>,
    ) -> Result<(MemoryId, HermesClassification)>;

    async fn process_batch(
        &self,
        items: Vec<MemoryItem>,
        context: Option<&WorldContext>,
    ) -> Result<Vec<(MemoryId, HermesClassification)>>;

    async fn run_reflection(
        &self,
        goal: &str,
        outcome: &str,
        steps: &[String],
        success: bool,
    ) -> Result<ReflectionLearning>;

    async fn get_pipeline_status(&self) -> Result<Vec<(MemoryPhase, bool)>>;
}

type ItemStore = RwLock<HashMap<MemoryId, MemoryItem>>;

pub struct InMemoryMemoryPipeline {
    config: MemoryConfig,
    working: ItemStore,
    short_term: ItemStore,
    episodic: ItemStore,
    semantic: ItemStore,
    procedural: ItemStore,
    vector: ItemStore,
    status: RwLock<HashMap<MemoryPhase, bool>>,
    item_counter: AtomicUsize,
}

impl InMemoryMemoryPipeline {
    pub fn new(config: MemoryConfig) -> Self {
        let mut status = HashMap::new();
        status.insert(MemoryPhase::Input, false);
        status.insert(MemoryPhase::ImportanceAnalysis, false);
        status.insert(MemoryPhase::HermesAnalysis, false);
        status.insert(MemoryPhase::Classification, false);
        status.insert(MemoryPhase::Storage, false);
        status.insert(MemoryPhase::Indexing, false);
        status.insert(MemoryPhase::Retrieval, false);
        status.insert(MemoryPhase::Reflection, false);
        status.insert(MemoryPhase::Learning, false);

        Self {
            config,
            working: RwLock::new(HashMap::new()),
            short_term: RwLock::new(HashMap::new()),
            episodic: RwLock::new(HashMap::new()),
            semantic: RwLock::new(HashMap::new()),
            procedural: RwLock::new(HashMap::new()),
            vector: RwLock::new(HashMap::new()),
            status: RwLock::new(status),
            item_counter: AtomicUsize::new(0),
        }
    }

    fn generate_id(&self) -> MemoryId {
        let n = self.item_counter.fetch_add(1, Ordering::SeqCst);
        MemoryId(format!("mem-{:06}", n))
    }

    fn calculate_importance(&self, item: &MemoryItem) -> f64 {
        let base = item.importance.clamp(0.0, 1.0);
        let hours_elapsed = (Utc::now() - item.timestamp).num_hours() as f64;
        let recency = (1.0 - (hours_elapsed / 720.0).min(1.0)).max(0.0);
        let frequency = (item.access_count as f64).ln_1p() / 10.0;
        let tag_richness = (item.context_tags.len() as f64).min(10.0) / 10.0;
        let score = base * 0.50 + recency * 0.20 + frequency.min(1.0) * 0.20 + tag_richness * 0.10;
        score.clamp(0.0, 1.0)
    }

    fn classify_memory_type(&self, importance: f64, item: &MemoryItem) -> MemoryType {
        let threshold_working = self.config.importance_threshold_working;
        let threshold_long_term = self.config.importance_threshold_long_term;
        if importance < threshold_working {
            MemoryType::Working
        } else if importance < threshold_long_term {
            MemoryType::ShortTerm
        } else {
            match item.memory_type {
                MemoryType::Episodic
                | MemoryType::Semantic
                | MemoryType::Procedural
                | MemoryType::Vector => item.memory_type.clone(),
                _ => MemoryType::Episodic,
            }
        }
    }

    fn store_item(&self, memory_id: MemoryId, item: MemoryItem, memory_type: &MemoryType) {
        match memory_type {
            MemoryType::Working => {
                let mut store = self.working.write();
                Self::evict_to_capacity(&mut store, self.config.working_memory_capacity);
                store.insert(memory_id, item);
            }
            MemoryType::ShortTerm => {
                let mut store = self.short_term.write();
                Self::evict_to_capacity(&mut store, self.config.short_term_capacity);
                store.insert(memory_id, item);
            }
            MemoryType::Episodic => {
                let mut store = self.episodic.write();
                Self::evict_to_capacity(&mut store, self.config.graph_max_nodes / 3);
                store.insert(memory_id, item);
            }
            MemoryType::Semantic => {
                let mut store = self.semantic.write();
                Self::evict_to_capacity(&mut store, self.config.graph_max_nodes / 3);
                store.insert(memory_id, item);
            }
            MemoryType::Procedural => {
                let mut store = self.procedural.write();
                Self::evict_to_capacity(&mut store, self.config.graph_max_nodes / 3);
                store.insert(memory_id, item);
            }
            MemoryType::Vector => {
                let mut store = self.vector.write();
                Self::evict_to_capacity(&mut store, self.config.max_vector_items);
                store.insert(memory_id, item);
            }
        }
    }

    fn evict_to_capacity(store: &mut HashMap<MemoryId, MemoryItem>, capacity: usize) {
        while store.len() >= capacity && capacity > 0 {
            let oldest_id = store
                .iter()
                .min_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp))
                .map(|(id, _)| id.clone());
            if let Some(id) = oldest_id {
                store.remove(&id);
            } else {
                break;
            }
        }
    }

    fn mark_phase(&self, phase: MemoryPhase) {
        self.status.write().insert(phase, true);
    }
}

#[async_trait::async_trait]
impl MemoryPipeline for InMemoryMemoryPipeline {
    async fn process(
        &self,
        item: MemoryItem,
        _context: Option<&WorldContext>,
    ) -> Result<(MemoryId, HermesClassification)> {
        self.mark_phase(MemoryPhase::Input);
        let importance = self.calculate_importance(&item);
        self.mark_phase(MemoryPhase::ImportanceAnalysis);
        let memory_type = self.classify_memory_type(importance, &item);
        self.mark_phase(MemoryPhase::HermesAnalysis);

        let decision = match memory_type {
            MemoryType::Working => {
                if importance < self.config.importance_threshold_working * 0.5 {
                    HermesDecision::Ignore
                } else {
                    HermesDecision::TemporaryMemory
                }
            }
            MemoryType::ShortTerm => HermesDecision::TemporaryMemory,
            MemoryType::Episodic => HermesDecision::LongTermMemory,
            MemoryType::Semantic => HermesDecision::Knowledge,
            MemoryType::Procedural => HermesDecision::ProceduralSkill,
            MemoryType::Vector => HermesDecision::LongTermMemory,
        };

        let classification = HermesClassification {
            item_id: item.id.clone(),
            decision,
            confidence: importance,
            reasons: vec![
                format!("importance: {:.4}", importance),
                format!("memory_type: {}", memory_type),
                format!("source: {}", item.source),
                format!("tags: [{}]", item.context_tags.join(", ")),
            ],
            timestamp: Utc::now(),
        };
        self.mark_phase(MemoryPhase::Classification);

        let memory_id = self.generate_id();
        let stored_item = MemoryItem {
            id: memory_id.clone(),
            memory_type: memory_type.clone(),
            importance,
            ..item
        };
        self.store_item(memory_id.clone(), stored_item, &memory_type);
        self.mark_phase(MemoryPhase::Storage);
        self.mark_phase(MemoryPhase::Indexing);

        Ok((memory_id, classification))
    }

    async fn process_batch(
        &self,
        items: Vec<MemoryItem>,
        context: Option<&WorldContext>,
    ) -> Result<Vec<(MemoryId, HermesClassification)>> {
        let mut results = Vec::with_capacity(items.len());
        for item in items {
            let result = self.process(item, context).await?;
            results.push(result);
        }
        Ok(results)
    }

    async fn run_reflection(
        &self,
        goal: &str,
        outcome: &str,
        steps: &[String],
        success: bool,
    ) -> Result<ReflectionLearning> {
        self.mark_phase(MemoryPhase::Reflection);

        let (success_factors, failure_factors) = if success {
            (
                vec![
                    "Goal completed successfully.".to_string(),
                    format!("Outcome: {}", outcome),
                ],
                vec![],
            )
        } else {
            (
                vec![],
                vec![
                    "Goal was not completed.".to_string(),
                    format!("Actual outcome: {}", outcome),
                ],
            )
        };

        let reflection = ReflectionLearning {
            reflection_id: format!("ref-{}", Uuid::new_v4()),
            task_id: format!("task-{}", Uuid::new_v4()),
            goal: goal.to_string(),
            outcome: outcome.to_string(),
            success_factors,
            failure_factors,
            lessons_learned: vec![format!(
                "Goal '{}' ended with '{}'. Steps taken: {}",
                goal,
                outcome,
                steps.join(" -> ")
            )],
            pattern_updates: vec![],
            timestamp: Utc::now(),
        };

        self.mark_phase(MemoryPhase::Learning);
        Ok(reflection)
    }

    async fn get_pipeline_status(&self) -> Result<Vec<(MemoryPhase, bool)>> {
        let status = self.status.read();
        let phase_order = [
            MemoryPhase::Input,
            MemoryPhase::ImportanceAnalysis,
            MemoryPhase::HermesAnalysis,
            MemoryPhase::Classification,
            MemoryPhase::Storage,
            MemoryPhase::Indexing,
            MemoryPhase::Retrieval,
            MemoryPhase::Reflection,
            MemoryPhase::Learning,
        ];
        let result: Vec<_> = phase_order
            .iter()
            .map(|phase| {
                let executed = status.get(phase).copied().unwrap_or(false);
                (phase.clone(), executed)
            })
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MemoryConfig;
    use crate::types::MemoryState;
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_item(text: &str, importance: f64) -> MemoryItem {
        MemoryItem {
            id: MemoryId("test".to_string()),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content: serde_json::json!({"text": text}),
            importance,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            context_tags: vec![],
            source: "test".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }
    }

    fn create_rich_test_item(text: &str, importance: f64, tags: Vec<&str>) -> MemoryItem {
        MemoryItem {
            id: MemoryId("test".to_string()),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content: serde_json::json!({"text": text}),
            importance,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            context_tags: tags.into_iter().map(String::from).collect(),
            source: "test".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }
    }

    #[tokio::test]
    async fn test_process_high_importance_item() {
        // Use an item with high base importance AND context tags to push it over
        // the long-term threshold (0.7). Base=0.95*0.5 + 5 tags*0.02 >= 0.7.
        let pipeline = InMemoryMemoryPipeline::new(MemoryConfig::default());
        let item = create_rich_test_item(
            "Critical system event",
            0.95,
            vec![
                "vital",
                "user_priority",
                "critical",
                "milestone",
                "frequent",
            ],
        );
        let (_id, classification) = pipeline.process(item, None).await.unwrap();
        assert_eq!(classification.decision, HermesDecision::LongTermMemory);
        assert!(classification.confidence >= 0.7);
    }

    #[tokio::test]
    async fn test_process_low_importance_item() {
        let pipeline = InMemoryMemoryPipeline::new(MemoryConfig::default());
        let item = create_test_item("Trivial note", 0.1);
        let (_id, classification) = pipeline.process(item, None).await.unwrap();
        assert!(matches!(
            classification.decision,
            HermesDecision::Ignore | HermesDecision::TemporaryMemory
        ));
    }

    #[tokio::test]
    async fn test_process_batch() {
        let pipeline = InMemoryMemoryPipeline::new(MemoryConfig::default());
        let items = vec![
            create_test_item("Item 1", 0.8),
            create_test_item("Item 2", 0.3),
            create_test_item("Item 3", 0.6),
        ];
        let results = pipeline.process_batch(items, None).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_run_reflection() {
        let pipeline = InMemoryMemoryPipeline::new(MemoryConfig::default());
        let reflection = pipeline
            .run_reflection(
                "Complete project",
                "All tasks done",
                &["plan", "execute", "review"].map(String::from),
                true,
            )
            .await
            .unwrap();
        assert_eq!(reflection.goal, "Complete project");
        assert!(!reflection.success_factors.is_empty());
        assert!(reflection.failure_factors.is_empty());
    }

    #[tokio::test]
    async fn test_get_pipeline_status() {
        let pipeline = InMemoryMemoryPipeline::new(MemoryConfig::default());
        let status = pipeline.get_pipeline_status().await.unwrap();
        for (_phase, executed) in &status {
            assert!(!executed);
        }
        let item = create_test_item("Status test", 0.5);
        pipeline.process(item, None).await.unwrap();
        let status = pipeline.get_pipeline_status().await.unwrap();
        for (phase, executed) in &status {
            match phase {
                MemoryPhase::Input
                | MemoryPhase::ImportanceAnalysis
                | MemoryPhase::HermesAnalysis
                | MemoryPhase::Classification
                | MemoryPhase::Storage
                | MemoryPhase::Indexing => assert!(*executed, "{:?} should be true", phase),
                MemoryPhase::Retrieval | MemoryPhase::Reflection | MemoryPhase::Learning => {
                    assert!(!executed, "{:?} should be false", phase)
                }
            }
        }
    }

    #[tokio::test]
    async fn test_pipeline_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<InMemoryMemoryPipeline>();
    }
}
