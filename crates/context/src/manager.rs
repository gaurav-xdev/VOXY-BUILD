use crate::cache::{CacheConfig, CacheStats, ContextCache};
use crate::error::Result;
use crate::provider::ContextProvider;
use crate::registry::ContextRegistry;
use crate::types::{ContextSnapshot, ContextSource, ContextUpdate};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

/// Configuration for the context manager.
#[derive(Debug, Clone)]
pub struct ManagerConfig {
    /// Cache configuration.
    pub cache: CacheConfig,

    /// Interval for background collection (None = no background collection).
    pub background_interval: Option<Duration>,

    /// Timeout for individual provider collection.
    pub collection_timeout: Duration,

    /// Maximum number of broadcast receivers.
    pub broadcast_capacity: usize,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            cache: CacheConfig::default(),
            background_interval: Some(Duration::from_secs(30)),
            collection_timeout: Duration::from_secs(5),
            broadcast_capacity: 64,
        }
    }
}

/// Snapshot of all context from a single collection cycle.
#[derive(Debug, Clone)]
pub struct ContextSnapshotSet {
    /// All snapshots collected in this cycle, keyed by source.
    pub snapshots: HashMap<ContextSource, ContextSnapshot>,

    /// When this set was collected.
    pub collected_at: chrono::DateTime<chrono::Utc>,

    /// Total collection time in milliseconds.
    pub collection_time_ms: u64,
}

impl ContextSnapshotSet {
    /// Get a snapshot by source.
    pub fn get(&self, source: &ContextSource) -> Option<&ContextSnapshot> {
        self.snapshots.get(source)
    }

    /// Get the number of snapshots in this set.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if this set is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Get all sources that contributed snapshots.
    pub fn sources(&self) -> Vec<&ContextSource> {
        self.snapshots.keys().collect()
    }
}

/// The context manager orchestrates context collection from all registered providers,
/// manages the cache, and broadcasts context updates.
pub struct ContextManager {
    registry: ContextRegistry,
    cache: ContextCache,
    config: ManagerConfig,
    update_sender: broadcast::Sender<ContextUpdate>,
    last_snapshot: RwLock<Option<ContextSnapshotSet>>,
}

impl ContextManager {
    /// Create a new context manager with the given configuration.
    pub fn new(config: ManagerConfig) -> Self {
        let (update_sender, _) = broadcast::channel(config.broadcast_capacity);
        let cache = ContextCache::new(config.cache.clone());

        Self {
            registry: ContextRegistry::new(),
            cache,
            config,
            update_sender,
            last_snapshot: RwLock::new(None),
        }
    }

    /// Create a context manager with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ManagerConfig::default())
    }

    /// Register a context provider.
    pub async fn register_provider(&self, provider: Arc<dyn ContextProvider>) {
        self.registry.register(provider).await;
    }

    /// Unregister a context provider by source.
    pub async fn unregister_provider(&self, source: &ContextSource) -> bool {
        self.registry.unregister(source).await
    }

    /// Collect context from all providers and return the assembled snapshot set.
    pub async fn collect(&self) -> Result<ContextSnapshotSet> {
        let start = std::time::Instant::now();

        // Collect from all providers concurrently
        let snapshots_raw = self.registry.collect_all_concurrent().await;

        // Insert into cache and build the snapshot set
        let mut snapshots = HashMap::new();
        for snapshot in snapshots_raw {
            let source = snapshot.source.clone();
            self.cache.insert(snapshot.clone());
            snapshots.insert(source, snapshot);
        }

        let collection_time_ms = start.elapsed().as_millis() as u64;

        let snapshot_set = ContextSnapshotSet {
            snapshots,
            collected_at: chrono::Utc::now(),
            collection_time_ms,
        };

        // Store as last snapshot
        *self.last_snapshot.write() = Some(snapshot_set.clone());

        // Broadcast updates for each source
        for snapshot in snapshot_set.snapshots.values() {
            let update = ContextUpdate::new(snapshot.source.clone(), snapshot.clone());
            let _ = self.update_sender.send(update);
        }

        Ok(snapshot_set)
    }

    /// Collect context from a specific source.
    pub async fn collect_one(&self, source: &ContextSource) -> Result<ContextSnapshot> {
        let snapshot = self.registry.collect_one(source).await?;
        self.cache.insert(snapshot.clone());

        let update = ContextUpdate::new(source.clone(), snapshot.clone());
        let _ = self.update_sender.send(update);

        Ok(snapshot)
    }

    /// Get the latest cached snapshot for a source.
    pub fn get_latest(&self, source: &ContextSource) -> Option<ContextSnapshot> {
        self.cache.get_latest(source)
    }

    /// Get the last assembled snapshot set.
    pub fn last_snapshot(&self) -> Option<ContextSnapshotSet> {
        self.last_snapshot.read().clone()
    }

    /// Subscribe to context updates.
    pub fn subscribe(&self) -> broadcast::Receiver<ContextUpdate> {
        self.update_sender.subscribe()
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Invalidate cached context for a source.
    pub fn invalidate(&self, source: &ContextSource) -> usize {
        self.cache.invalidate_source(source)
    }

    /// Clear all cached context.
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Start background collection at the configured interval.
    pub fn start_background_collection(&self) {
        let interval = match self.config.background_interval {
            Some(i) => i,
            None => return,
        };

        // Stop existing background task if any
        self.stop_background_collection();

        // Background collection requires an Arc<ContextManager> and tokio::spawn.
        // Defer to the integration layer (DI container) which owns the manager as Arc.
        tracing::info!(
            interval_ms = interval.as_millis(),
            "Background collection configured — startup delegated to integration layer"
        );
    }

    /// Stop background collection.
    pub fn stop_background_collection(&self) {
        // Background task handle is owned by the integration layer.
        tracing::info!("Background collection stop requested");
    }

    /// Get the number of registered providers.
    pub async fn provider_count(&self) -> usize {
        self.registry.count().await
    }

    /// Get all registered source types.
    pub async fn sources(&self) -> Vec<ContextSource> {
        self.registry.sources().await
    }

    /// Run health checks on all providers.
    pub async fn health_check(&self) -> HashMap<ContextSource, bool> {
        self.registry.health_check_all().await
    }
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ContextSource;

    struct MockProvider {
        source: ContextSource,
    }

    #[async_trait::async_trait]
    impl ContextProvider for MockProvider {
        fn source(&self) -> ContextSource {
            self.source.clone()
        }

        fn name(&self) -> &str {
            "mock"
        }

        async fn collect(&self) -> Result<ContextSnapshot> {
            Ok(ContextSnapshot::new(
                self.source.clone(),
                serde_json::json!({"test": true}),
            ))
        }
    }

    #[tokio::test]
    async fn test_manager_collect() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;
        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Conversation,
            }))
            .await;

        let snapshot_set = manager.collect().await.unwrap();
        assert_eq!(snapshot_set.len(), 2);
        assert!(snapshot_set.get(&ContextSource::Environment).is_some());
        assert!(snapshot_set.get(&ContextSource::Conversation).is_some());
    }

    #[tokio::test]
    async fn test_manager_collect_one() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        let snapshot = manager
            .collect_one(&ContextSource::Environment)
            .await
            .unwrap();
        assert_eq!(snapshot.source, ContextSource::Environment);
    }

    #[tokio::test]
    async fn test_manager_cache() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        manager.collect().await.unwrap();

        let cached = manager.get_latest(&ContextSource::Environment);
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn test_manager_subscribe() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        let mut rx = manager.subscribe();

        manager.collect().await.unwrap();

        let update = rx.recv().await.unwrap();
        assert_eq!(update.source, ContextSource::Environment);
    }

    #[tokio::test]
    async fn test_manager_invalidate() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        manager.collect().await.unwrap();
        assert!(manager.get_latest(&ContextSource::Environment).is_some());

        manager.invalidate(&ContextSource::Environment);
        assert!(manager.get_latest(&ContextSource::Environment).is_none());
    }

    #[tokio::test]
    async fn test_manager_health_check() {
        let manager = ContextManager::with_defaults();

        manager
            .register_provider(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        let health = manager.health_check().await;
        assert_eq!(health.len(), 1);
        assert!(health[&ContextSource::Environment]);
    }
}
