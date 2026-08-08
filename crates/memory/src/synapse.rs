use crate::api::MemoryApi;
use crate::event::MemoryEvent;
use crate::hermes::HermesEngine;
use crate::types::{MemoryId, MemoryItem, MemoryState, MemoryType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};
use voxy_world_model::event::WorldModelEvent;

static MEMORY_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn next_memory_id(prefix: &str) -> MemoryId {
    let seq = MEMORY_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    MemoryId(format!(
        "{}-{}-{}",
        prefix,
        Utc::now().timestamp_millis(),
        seq
    ))
}

pub struct MemorySynapse {
    memory_api: Arc<dyn MemoryApi>,
    #[allow(dead_code)]
    hermes: Arc<dyn HermesEngine>,
    #[allow(dead_code)]
    event_tx: mpsc::Sender<MemoryEvent>,
    state: Arc<RwLock<SynapseState>>,
}

struct SynapseState {
    pending_events: Vec<WorldModelEvent>,
    processed_count: u64,
    error_count: u64,
    #[allow(dead_code)]
    id_counter: u64,
}

const SYNAPSE_CHANNEL_CAPACITY: usize = 1024;

impl MemorySynapse {
    pub fn new(
        memory_api: Arc<dyn MemoryApi>,
        hermes: Arc<dyn HermesEngine>,
    ) -> (Self, mpsc::Receiver<MemoryEvent>) {
        let (event_tx, event_rx) = mpsc::channel(SYNAPSE_CHANNEL_CAPACITY);

        (
            Self {
                memory_api,
                hermes,
                event_tx,
                state: Arc::new(RwLock::new(SynapseState {
                    pending_events: Vec::new(),
                    processed_count: 0,
                    error_count: 0,
                    id_counter: 0,
                })),
            },
            event_rx,
        )
    }

    pub async fn process_world_event(
        &self,
        event: WorldModelEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let memory_item = match &event {
            WorldModelEvent::DesktopUpdated {
                window_count,
                focused_app,
            } => {
                self.create_desktop_memory(window_count, focused_app)
                    .await?
            }
            WorldModelEvent::ApplicationLaunched { app_id, app_name } => {
                self.create_app_launch_memory(app_id, app_name).await?
            }
            WorldModelEvent::ApplicationClosed { app_id } => {
                self.create_app_close_memory(app_id).await?
            }
            WorldModelEvent::DeviceConnected {
                device_id,
                device_type,
            } => {
                self.create_device_memory(device_id, device_type, "connected")
                    .await?
            }
            WorldModelEvent::DeviceDisconnected { device_id } => {
                self.create_device_memory(device_id, "", "disconnected")
                    .await?
            }
            WorldModelEvent::TaskCreated {
                task_id,
                description,
            } => {
                self.create_task_memory(task_id, description, "created")
                    .await?
            }
            WorldModelEvent::TaskUpdated { task_id, status } => {
                self.create_task_memory(task_id, status, "updated").await?
            }
            WorldModelEvent::TaskCompleted { task_id } => {
                self.create_task_memory(task_id, "completed", "completed")
                    .await?
            }
            WorldModelEvent::EnvironmentChanged { description } => {
                self.create_environment_memory(description).await?
            }
            WorldModelEvent::WindowChanged {
                app_id,
                window_title,
                ..
            } => {
                self.create_window_change_memory(app_id, window_title)
                    .await?
            }
            WorldModelEvent::ActivityChanged {
                app_id,
                activity_type,
                confidence,
                ..
            } => {
                self.create_activity_change_memory(app_id, activity_type, *confidence)
                    .await?
            }
            WorldModelEvent::ProjectDetected {
                project_name,
                language,
                ..
            } => {
                self.create_project_detected_memory(project_name, language.as_deref())
                    .await?
            }
            WorldModelEvent::PreferenceLearned {
                category,
                key,
                value,
                confidence,
                ..
            } => {
                self.create_preference_learned_memory(category, key, value, *confidence)
                    .await?
            }
            WorldModelEvent::ContextUpdated {
                focused_app,
                activity_type,
                ..
            } => {
                self.create_context_update_memory(focused_app, activity_type.as_deref())
                    .await?
            }
            WorldModelEvent::ApplicationFocused { app_id, .. } => {
                self.create_app_focus_memory(app_id).await?
            }
            WorldModelEvent::IdleStarted {
                last_active_app, ..
            } => {
                self.create_idle_start_memory(last_active_app.as_deref())
                    .await?
            }
            WorldModelEvent::IdleEnded { new_app, .. } => {
                self.create_idle_end_memory(new_app).await?
            }
        };

        if let Some(item) = memory_item {
            let id = self.memory_api.store(item).await?;
            debug!(memory_id = %id, "Stored memory from world event");

            let mut state = self.state.write().await;
            state.processed_count += 1;
        }

        Ok(())
    }

    pub async fn process_batch(
        &self,
        events: Vec<WorldModelEvent>,
    ) -> Result<(usize, usize), Box<dyn std::error::Error + Send + Sync>> {
        let mut processed = 0;
        let mut errors = 0;

        for event in events {
            match self.process_world_event(event).await {
                Ok(_) => processed += 1,
                Err(e) => {
                    warn!(error = %e, "Failed to process world event");
                    errors += 1;
                }
            }
        }

        Ok((processed, errors))
    }

    pub async fn get_stats(&self) -> SynapseStats {
        let state = self.state.read().await;
        SynapseStats {
            processed_count: state.processed_count,
            error_count: state.error_count,
            pending_count: state.pending_events.len(),
        }
    }

    async fn create_desktop_memory(
        &self,
        window_count: &usize,
        focused_app: &Option<String>,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "desktop_update",
            "window_count": window_count,
            "focused_app": focused_app,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("desktop"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.3,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["desktop".to_string(), "environment".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_app_launch_memory(
        &self,
        app_id: &str,
        app_name: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "app_launched",
            "app_id": app_id,
            "app_name": app_name,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("app-launch"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.4,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["application".to_string(), "launch".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_app_close_memory(
        &self,
        app_id: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "app_closed",
            "app_id": app_id,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("app-close"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.2,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["application".to_string(), "close".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_device_memory(
        &self,
        device_id: &str,
        device_type: &str,
        action: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": format!("device_{}", action),
            "device_id": device_id,
            "device_type": device_type,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id(&format!("device-{}", action)),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.5,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["device".to_string(), action.to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_task_memory(
        &self,
        task_id: &str,
        description: &str,
        action: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": format!("task_{}", action),
            "task_id": task_id,
            "description": description,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id(&format!("task-{}", action)),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.6,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["task".to_string(), action.to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_environment_memory(
        &self,
        description: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "environment_changed",
            "description": description,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("env"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.4,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["environment".to_string(), "change".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_window_change_memory(
        &self,
        app_id: &str,
        window_title: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "window_changed",
            "app_id": app_id,
            "window_title": window_title,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("window"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.3,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["window".to_string(), "change".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_activity_change_memory(
        &self,
        app_id: &str,
        activity_type: &str,
        confidence: f64,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "activity_changed",
            "app_id": app_id,
            "activity_type": activity_type,
            "confidence": confidence,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("activity"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.4,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["activity".to_string(), "change".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_project_detected_memory(
        &self,
        project_name: &str,
        language: Option<&str>,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "project_detected",
            "project_name": project_name,
            "language": language,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("project"),
            memory_type: MemoryType::Semantic,
            state: MemoryState::Active,
            content,
            importance: 0.6,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["project".to_string(), "detection".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_preference_learned_memory(
        &self,
        category: &str,
        key: &str,
        value: &str,
        confidence: f64,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "preference_learned",
            "category": category,
            "key": key,
            "value": value,
            "confidence": confidence,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("pref"),
            memory_type: MemoryType::Semantic,
            state: MemoryState::Active,
            content,
            importance: 0.7,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["preference".to_string(), "learning".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_context_update_memory(
        &self,
        focused_app: &Option<String>,
        activity_type: Option<&str>,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "context_updated",
            "focused_app": focused_app,
            "activity_type": activity_type,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("ctx"),
            memory_type: MemoryType::Working,
            state: MemoryState::Active,
            content,
            importance: 0.2,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["context".to_string(), "update".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_app_focus_memory(
        &self,
        app_id: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "app_focused",
            "app_id": app_id,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("focus"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.3,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["application".to_string(), "focus".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_idle_start_memory(
        &self,
        last_active_app: Option<&str>,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "idle_started",
            "last_active_app": last_active_app,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("idle-start"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.2,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["idle".to_string(), "start".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }

    async fn create_idle_end_memory(
        &self,
        new_app: &str,
    ) -> Result<Option<MemoryItem>, Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::json!({
            "type": "idle_ended",
            "new_app": new_app,
            "timestamp": Utc::now().to_rfc3339(),
        });

        Ok(Some(MemoryItem {
            id: next_memory_id("idle-end"),
            memory_type: MemoryType::Episodic,
            state: MemoryState::Active,
            content,
            importance: 0.3,
            timestamp: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
            context_tags: vec!["idle".to_string(), "end".to_string()],
            source: "world_model".to_string(),
            version: 1,
            ttl: None,
            metadata: HashMap::new(),
            embedding: None,
            parent_id: None,
            related_ids: vec![],
        }))
    }
}

#[derive(Debug, Clone)]
pub struct SynapseStats {
    pub processed_count: u64,
    pub error_count: u64,
    pub pending_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::MemoryStats;

    struct MockMemoryApi {
        store: std::sync::Mutex<HashMap<String, MemoryItem>>,
    }

    impl MockMemoryApi {
        fn new() -> Self {
            Self {
                store: std::sync::Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl MemoryApi for MockMemoryApi {
        async fn init(&self, _config: &crate::config::MemoryConfig) -> crate::Result<()> {
            Ok(())
        }
        async fn shutdown(&self) -> crate::Result<()> {
            Ok(())
        }
        async fn store(&self, item: MemoryItem) -> crate::Result<MemoryId> {
            let id = item.id.clone();
            self.store.lock().unwrap().insert(id.0.clone(), item);
            Ok(id)
        }
        async fn store_with_analysis(
            &self,
            item: MemoryItem,
            _context: Option<&voxy_world_model::context::WorldContext>,
        ) -> crate::Result<(MemoryId, crate::HermesClassification)> {
            let id = item.id.clone();
            self.store.lock().unwrap().insert(id.0.clone(), item);
            Ok((
                id.clone(),
                crate::HermesClassification {
                    item_id: id,
                    decision: crate::HermesDecision::LongTermMemory,
                    confidence: 0.9,
                    reasons: vec!["test".to_string()],
                    timestamp: Utc::now(),
                },
            ))
        }
        async fn retrieve(&self, id: &MemoryId) -> crate::Result<MemoryItem> {
            self.store
                .lock()
                .unwrap()
                .get(&id.0)
                .cloned()
                .ok_or_else(|| crate::MemoryError::ItemNotFound(id.0.clone()))
        }
        async fn search(
            &self,
            query: &crate::types::MemoryQuery,
        ) -> crate::Result<Vec<crate::SearchResult>> {
            let store = self.store.lock().unwrap();
            let results: Vec<crate::SearchResult> = store
                .values()
                .take(query.max_results)
                .map(|item| crate::SearchResult {
                    item: item.clone(),
                    score: item.importance,
                    match_reasons: vec!["mock search".to_string()],
                })
                .collect();
            Ok(results)
        }
        async fn update(&self, item: MemoryItem) -> crate::Result<()> {
            self.store.lock().unwrap().insert(item.id.0.clone(), item);
            Ok(())
        }
        async fn delete(&self, id: &MemoryId) -> crate::Result<()> {
            self.store.lock().unwrap().remove(&id.0);
            Ok(())
        }
        async fn forget(&self, id: &MemoryId) -> crate::Result<()> {
            self.store.lock().unwrap().remove(&id.0);
            Ok(())
        }
        async fn recall(&self, id: &MemoryId) -> crate::Result<MemoryItem> {
            self.store
                .lock()
                .unwrap()
                .get(&id.0)
                .cloned()
                .ok_or_else(|| crate::MemoryError::ItemNotFound(id.0.clone()))
        }
        async fn consolidate(&self) -> crate::Result<usize> {
            Ok(0)
        }
        async fn run_forgetting(&self) -> crate::Result<usize> {
            Ok(0)
        }
        async fn graph(&self) -> &dyn crate::KnowledgeGraph {
            panic!("MockMemoryApi does not implement graph")
        }
        async fn hermes(&self) -> &dyn HermesEngine {
            panic!("MockMemoryApi does not implement hermes")
        }
        async fn stats(&self) -> crate::Result<MemoryStats> {
            let count = self.store.lock().unwrap().len();
            Ok(MemoryStats {
                total_items: count,
                working_count: count,
                short_term_count: 0,
                episodic_count: 0,
                semantic_count: 0,
                procedural_count: 0,
                vector_count: 0,
                graph_nodes: 0,
                graph_edges: 0,
                active_count: count,
                dormant_count: 0,
                compressed_count: 0,
                archived_count: 0,
            })
        }
        async fn clear(&self) -> crate::Result<()> {
            self.store.lock().unwrap().clear();
            Ok(())
        }
    }

    struct MockHermesEngine;

    #[async_trait::async_trait]
    impl HermesEngine for MockHermesEngine {
        async fn analyze_experience(
            &self,
            item: &MemoryItem,
            _context: Option<&voxy_world_model::context::WorldContext>,
        ) -> crate::Result<crate::ExperienceAnalysis> {
            Ok(crate::ExperienceAnalysis {
                experience_id: format!("exp-{}", item.id.0),
                summary: item.content.to_string().chars().take(200).collect(),
                emotional_tone: "neutral".to_string(),
                key_elements: item.context_tags.clone(),
                importance: item.importance,
                novelty: 0.5,
                timestamp: item.timestamp,
            })
        }
        async fn extract_preferences(
            &self,
            items: &[MemoryItem],
            limit: usize,
        ) -> crate::Result<Vec<crate::PreferenceExtraction>> {
            Ok(items
                .iter()
                .take(limit)
                .enumerate()
                .map(|(i, item)| crate::PreferenceExtraction {
                    preference_id: format!("pref-{i}"),
                    category: item.source.clone(),
                    value: item.content.to_string().chars().take(100).collect(),
                    confidence: item.importance,
                    evidence: item.context_tags.clone(),
                    is_stable: item.access_count > 3,
                })
                .collect())
        }
        async fn detect_patterns(
            &self,
            _items: &[MemoryItem],
            _window_days: u64,
        ) -> crate::Result<Vec<crate::PatternDetection>> {
            Ok(vec![])
        }
        async fn detect_habits(
            &self,
            _items: &[MemoryItem],
            _window_days: u64,
        ) -> crate::Result<Vec<crate::HabitDetection>> {
            Ok(vec![])
        }
        async fn model_relationship(
            &self,
            entity_a: &str,
            entity_b: &str,
            _interactions: &[MemoryItem],
        ) -> crate::Result<crate::RelationshipModel> {
            Ok(crate::RelationshipModel {
                relationship_id: format!("rel-{entity_a}-{entity_b}"),
                entity_a: entity_a.to_string(),
                entity_b: entity_b.to_string(),
                relationship_type: "interaction".to_string(),
                strength: 0.5,
                interaction_count: 0,
                last_interaction: Utc::now(),
                sentiment: 0.5,
            })
        }
        async fn analyze_behavior(
            &self,
            _items: &[MemoryItem],
            _window_days: u64,
        ) -> crate::Result<Vec<crate::BehaviorAnalysis>> {
            Ok(vec![])
        }
        async fn extract_skills(
            &self,
            _items: &[MemoryItem],
        ) -> crate::Result<Vec<crate::SkillExtraction>> {
            Ok(vec![])
        }
        async fn build_knowledge(
            &self,
            _items: &[MemoryItem],
            _graph: &dyn crate::KnowledgeGraph,
        ) -> crate::Result<usize> {
            Ok(0)
        }
        async fn reflect_on_task(
            &self,
            goal: &str,
            outcome: &str,
            _steps: &[String],
            _success: bool,
        ) -> crate::Result<crate::ReflectionLearning> {
            Ok(crate::ReflectionLearning {
                reflection_id: format!("ref-{}", uuid::Uuid::new_v4()),
                task_id: format!("task-{}", uuid::Uuid::new_v4()),
                goal: goal.to_string(),
                outcome: outcome.to_string(),
                success_factors: vec![],
                failure_factors: vec![],
                lessons_learned: vec!["learned from mock".to_string()],
                pattern_updates: vec![],
                timestamp: Utc::now(),
            })
        }
        async fn classify(
            &self,
            item: &MemoryItem,
            _analysis: &crate::ExperienceAnalysis,
        ) -> crate::Result<crate::HermesClassification> {
            Ok(crate::HermesClassification {
                item_id: item.id.clone(),
                decision: crate::HermesDecision::LongTermMemory,
                confidence: item.importance,
                reasons: vec!["mock classification".to_string()],
                timestamp: Utc::now(),
            })
        }
        async fn consolidate_long_term(
            &self,
            items: &[MemoryItem],
            _policy: &crate::ConsolidationPolicy,
        ) -> crate::Result<Vec<crate::HermesClassification>> {
            Ok(items
                .iter()
                .map(|item| crate::HermesClassification {
                    item_id: item.id.clone(),
                    decision: crate::HermesDecision::LongTermMemory,
                    confidence: item.importance,
                    reasons: vec!["consolidated".to_string()],
                    timestamp: Utc::now(),
                })
                .collect())
        }
        async fn evolve_memory(
            &self,
            item: &MemoryItem,
            _feedback: &str,
        ) -> crate::Result<MemoryItem> {
            let mut evolved = item.clone();
            evolved.version += 1;
            Ok(evolved)
        }
    }

    #[tokio::test]
    async fn test_synapse_creation() {
        let memory_api = Arc::new(MockMemoryApi::new());
        let hermes = Arc::new(MockHermesEngine);
        let (synapse, _event_rx) = MemorySynapse::new(memory_api, hermes);
        let stats = synapse.get_stats().await;
        assert_eq!(stats.processed_count, 0);
    }
}
