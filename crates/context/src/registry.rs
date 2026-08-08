use crate::error::{ContextError, Result};
use crate::provider::ContextProvider;
use crate::types::{ContextSnapshot, ContextSource};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages registered context providers and coordinates context collection.
pub struct ContextRegistry {
    providers: RwLock<HashMap<ContextSource, Arc<dyn ContextProvider>>>,
}

impl ContextRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: RwLock::new(HashMap::new()),
        }
    }

    /// Register a context provider.
    ///
    /// If a provider for the same source already exists, it is replaced.
    pub async fn register(&self, provider: Arc<dyn ContextProvider>) {
        let source = provider.source();
        tracing::info!(
            source = %source,
            name = provider.name(),
            "Registering context provider"
        );
        self.providers.write().await.insert(source, provider);
    }

    /// Unregister a context provider by source.
    pub async fn unregister(&self, source: &ContextSource) -> bool {
        let removed = self.providers.write().await.remove(source).is_some();
        if removed {
            tracing::info!(source = %source, "Unregistered context provider");
        }
        removed
    }

    /// Get a reference to a registered provider by source.
    pub async fn get(&self, source: &ContextSource) -> Option<Arc<dyn ContextProvider>> {
        self.providers.read().await.get(source).cloned()
    }

    /// Check if a provider is registered for the given source.
    pub async fn has(&self, source: &ContextSource) -> bool {
        self.providers.read().await.contains_key(source)
    }

    /// Return the number of registered providers.
    pub async fn count(&self) -> usize {
        self.providers.read().await.len()
    }

    /// Return all registered source types.
    pub async fn sources(&self) -> Vec<ContextSource> {
        self.providers.read().await.keys().cloned().collect()
    }

    /// Collect context from a specific provider.
    pub async fn collect_one(&self, source: &ContextSource) -> Result<ContextSnapshot> {
        let provider = self
            .providers
            .read()
            .await
            .get(source)
            .cloned()
            .ok_or_else(|| ContextError::ProviderNotFound(source.to_string()))?;

        provider.collect().await
    }

    /// Collect context from all registered providers.
    ///
    /// Returns a vector of snapshots. Providers that fail are logged and skipped.
    pub async fn collect_all(&self) -> Vec<ContextSnapshot> {
        let providers: Vec<Arc<dyn ContextProvider>> =
            { self.providers.read().await.values().cloned().collect() };

        let mut snapshots = Vec::with_capacity(providers.len());

        for provider in providers {
            match provider.collect().await {
                Ok(snapshot) => snapshots.push(snapshot),
                Err(e) => {
                    tracing::warn!(
                        provider = provider.name(),
                        error = %e,
                        "Failed to collect context from provider"
                    );
                }
            }
        }

        snapshots
    }

    /// Collect context from all registered providers concurrently.
    ///
    /// Returns a vector of snapshots. Providers that fail are logged and skipped.
    pub async fn collect_all_concurrent(&self) -> Vec<ContextSnapshot> {
        let providers: Vec<Arc<dyn ContextProvider>> =
            { self.providers.read().await.values().cloned().collect() };

        let handles: Vec<_> = providers
            .into_iter()
            .map(|provider| {
                tokio::spawn(async move {
                    match provider.collect().await {
                        Ok(snapshot) => Some(snapshot),
                        Err(e) => {
                            tracing::warn!(
                                provider = provider.name(),
                                error = %e,
                                "Failed to collect context from provider"
                            );
                            None
                        }
                    }
                })
            })
            .collect();

        let mut snapshots = Vec::new();
        for handle in handles {
            if let Ok(Some(snapshot)) = handle.await {
                snapshots.push(snapshot);
            }
        }

        snapshots
    }

    /// Run health checks on all registered providers.
    ///
    /// Returns a map of source to health status.
    pub async fn health_check_all(&self) -> HashMap<ContextSource, bool> {
        let providers: Vec<Arc<dyn ContextProvider>> =
            { self.providers.read().await.values().cloned().collect() };

        let handles: Vec<_> = providers
            .into_iter()
            .map(|provider| {
                let source = provider.source();
                tokio::spawn(async move {
                    let healthy = provider.health_check().await;
                    (source, healthy)
                })
            })
            .collect();

        let mut results = HashMap::new();
        for handle in handles {
            if let Ok((source, healthy)) = handle.await {
                results.insert(source, healthy);
            }
        }

        results
    }
}

impl Default for ContextRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn test_register_and_collect() {
        let registry = ContextRegistry::new();
        let provider: Arc<dyn ContextProvider> = Arc::new(MockProvider {
            source: ContextSource::Environment,
        });

        registry.register(provider).await;
        assert_eq!(registry.count().await, 1);

        let snapshot = registry.collect_one(&ContextSource::Environment).await;
        assert!(snapshot.is_ok());
    }

    #[tokio::test]
    async fn test_collect_all() {
        let registry = ContextRegistry::new();

        registry
            .register(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;
        registry
            .register(Arc::new(MockProvider {
                source: ContextSource::Conversation,
            }))
            .await;

        let snapshots = registry.collect_all().await;
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn test_unregister() {
        let registry = ContextRegistry::new();
        registry
            .register(Arc::new(MockProvider {
                source: ContextSource::Environment,
            }))
            .await;

        assert!(registry.unregister(&ContextSource::Environment).await);
        assert_eq!(registry.count().await, 0);
    }
}
