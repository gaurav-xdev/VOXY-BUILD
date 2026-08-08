//! Application lifecycle, runtime manager, and service registry.
//!
//! The kernel manages the startup and shutdown of all services in the correct
//! dependency order.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use voxy_config::AppConfig;
use voxy_shared::HealthStatus;

/// Service registration entry.
struct ServiceEntry {
    /// The service instance.
    service: Arc<dyn ManagedService>,
    /// Services this one depends on.
    dependencies: Vec<String>,
}

/// Trait for managed services in the kernel.
///
/// Services are started in dependency order and stopped in reverse order.
/// Implement this trait to participate in the kernel's lifecycle management.
#[async_trait::async_trait]
pub trait ManagedService: Send + Sync + 'static {
    /// Get the service name.
    fn name(&self) -> &str;

    /// Initialize the service.
    async fn initialize(&self) -> voxy_shared::Result<()> {
        Ok(())
    }

    /// Start the service.
    async fn start(&self) -> voxy_shared::Result<()>;

    /// Pause the service (optional).
    async fn pause(&self) -> voxy_shared::Result<()> {
        Ok(())
    }

    /// Resume the service (optional).
    async fn resume(&self) -> voxy_shared::Result<()> {
        Ok(())
    }

    /// Stop the service.
    async fn stop(&self) -> voxy_shared::Result<()>;

    /// Restart the service (optional).
    async fn restart(&self) -> voxy_shared::Result<()> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Reload configuration (optional).
    async fn reload(&self, _config: &dyn std::any::Any) -> voxy_shared::Result<()> {
        Ok(())
    }

    /// Check health.
    fn health_check(&self) -> HealthStatus;

    /// Get dependencies.
    fn dependencies(&self) -> Vec<String> {
        vec![]
    }
}

/// Service registry for managing application services.
pub struct ServiceRegistry {
    services: RwLock<HashMap<String, ServiceEntry>>,
    startup_order: RwLock<Vec<String>>,
    shutdown_order: RwLock<Vec<String>>,
}

impl ServiceRegistry {
    /// Create a new service registry.
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            startup_order: RwLock::new(Vec::new()),
            shutdown_order: RwLock::new(Vec::new()),
        }
    }

    /// Register a service.
    pub async fn register(&self, service: Arc<dyn ManagedService>) {
        let name = service.name().to_string();
        let deps = service.dependencies();
        let entry = ServiceEntry {
            service,
            dependencies: deps,
        };
        self.services.write().await.insert(name, entry);
    }

    /// Compute startup order based on dependencies (topological sort).
    pub async fn compute_startup_order(&self) -> voxy_shared::Result<()> {
        let services = self.services.read().await;
        let mut visited = HashSet::new();
        let mut order = Vec::new();

        for name in services.keys() {
            if !visited.contains(name.as_str()) {
                Self::visit(name.as_str(), &services, &mut visited, &mut order)?;
            }
        }

        *self.startup_order.write().await = order.clone();
        let mut shutdown = order;
        shutdown.reverse();
        *self.shutdown_order.write().await = shutdown;

        Ok(())
    }

    fn visit(
        name: &str,
        services: &HashMap<String, ServiceEntry>,
        visited: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> voxy_shared::Result<()> {
        if visited.contains(name) {
            return Ok(());
        }

        // Check for missing dependencies
        if let Some(entry) = services.get(name) {
            for dep in &entry.dependencies {
                if !services.contains_key(dep.as_str()) {
                    return Err(voxy_shared::VoxyError::new(
                        voxy_shared::ErrorKind::Dependency,
                        format!(
                            "Service '{}' depends on '{}' which is not registered",
                            name, dep
                        ),
                    ));
                }
                Self::visit(dep, services, visited, order)?;
            }
        }

        visited.insert(name.to_string());
        order.push(name.to_string());
        Ok(())
    }

    /// Start all services in dependency order.
    pub async fn start_all(&self) -> voxy_shared::Result<()> {
        self.compute_startup_order().await?;
        let order = self.startup_order.read().await.clone();

        for name in &order {
            let services = self.services.read().await;
            if let Some(entry) = services.get(name) {
                tracing::info!(service = %name, "Initializing service");
                entry.service.initialize().await?;
                tracing::info!(service = %name, "Starting service");
                entry.service.start().await?;
                tracing::info!(service = %name, "Service started");
            }
        }

        Ok(())
    }

    /// Stop all services in reverse dependency order.
    pub async fn stop_all(&self) -> voxy_shared::Result<()> {
        let order = self.shutdown_order.read().await.clone();

        for name in &order {
            let services = self.services.read().await;
            if let Some(entry) = services.get(name) {
                tracing::info!(service = %name, "Stopping service");
                entry.service.stop().await?;
                tracing::info!(service = %name, "Service stopped");
            }
        }

        Ok(())
    }

    /// Check health of all services.
    pub async fn health_check_all(&self) -> HashMap<String, HealthStatus> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, entry)| (name.clone(), entry.service.health_check()))
            .collect()
    }

    /// Get the number of registered services.
    pub async fn service_count(&self) -> usize {
        self.services.read().await.len()
    }

    /// Check if a service is registered.
    pub async fn has_service(&self, name: &str) -> bool {
        self.services.read().await.contains_key(name)
    }

    /// Get a service by name (for direct invocation).
    pub async fn get_service(&self, name: &str) -> Option<Arc<dyn ManagedService>> {
        self.services
            .read()
            .await
            .get(name)
            .map(|entry| entry.service.clone())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// The core kernel managing application lifecycle.
pub struct Kernel {
    config: AppConfig,
    shutdown_tx: broadcast::Sender<()>,
    runtime: Option<tokio::runtime::Runtime>,
    services: ServiceRegistry,
    initialized: bool,
}

impl Kernel {
    /// Create a new kernel with the given configuration.
    pub fn new(config: AppConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            config,
            shutdown_tx,
            runtime: None,
            services: ServiceRegistry::new(),
            initialized: false,
        }
    }

    /// Initialize the kernel runtime.
    pub fn initialize(&mut self) -> voxy_shared::Result<()> {
        if tokio::runtime::Handle::try_current().is_ok() {
            self.initialized = true;
            tracing::info!(
                thread_pool_size = self.config.kernel().thread_pool_size(),
                "Kernel initialized (using existing runtime)"
            );
            return Ok(());
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(self.config.kernel().thread_pool_size())
            .enable_all()
            .build()
            .map_err(|e| {
                voxy_shared::VoxyError::new(voxy_shared::ErrorKind::Internal, e.to_string())
            })?;

        self.runtime = Some(runtime);
        self.initialized = true;
        tracing::info!(
            thread_pool_size = self.config.kernel().thread_pool_size(),
            "Kernel initialized"
        );
        Ok(())
    }

    /// Start the kernel and all registered services.
    pub async fn start(&mut self) -> voxy_shared::Result<()> {
        if self.runtime.is_none() {
            self.initialize()?;
        }

        self.services.start_all().await?;
        tracing::info!("Kernel started");
        Ok(())
    }

    /// Shutdown the kernel and all services.
    pub async fn shutdown(&self) -> voxy_shared::Result<()> {
        self.services.stop_all().await?;
        let _ = self.shutdown_tx.send(());
        tracing::info!("Kernel shutdown complete");
        Ok(())
    }

    /// Get a handle to the Tokio runtime.
    pub fn get_handle(&self) -> Option<tokio::runtime::Handle> {
        self.runtime.as_ref().map(|r| r.handle().clone())
    }

    /// Check the health of the kernel.
    pub fn health_check(&self) -> HealthStatus {
        if self.initialized {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy("Runtime not initialized".to_string())
        }
    }

    /// Get the kernel configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get the service registry.
    pub fn services(&self) -> &ServiceRegistry {
        &self.services
    }

    /// Subscribe to shutdown signals.
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Wait for a shutdown signal.
    pub async fn wait_for_shutdown(&self) {
        let mut rx = self.shutdown_tx.subscribe();
        let _ = rx.recv().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kernel_creates() {
        let config = AppConfig::default();
        let kernel = Kernel::new(config);
        assert!(kernel.get_handle().is_none());
        assert_eq!(
            kernel.health_check(),
            HealthStatus::Unhealthy("Runtime not initialized".to_string())
        );
    }

    #[test]
    fn kernel_initializes() {
        let config = AppConfig::default();
        let mut kernel = Kernel::new(config);
        kernel.initialize().unwrap();
        assert!(kernel.get_handle().is_some());
        assert_eq!(kernel.health_check(), HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn kernel_start_and_shutdown() {
        let config = AppConfig::default();
        let mut kernel = Kernel::new(config);
        kernel.start().await.unwrap();
        assert_eq!(kernel.health_check(), HealthStatus::Healthy);
        kernel.shutdown().await.unwrap();
    }

    #[test]
    fn service_registry_creation() {
        let registry = ServiceRegistry::new();
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert_eq!(registry.service_count().await, 0);
        });
    }

    #[tokio::test]
    async fn service_registry_register() {
        struct TestService;

        #[async_trait::async_trait]
        impl ManagedService for TestService {
            fn name(&self) -> &str {
                "test"
            }
            async fn start(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            async fn stop(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            fn health_check(&self) -> HealthStatus {
                HealthStatus::Healthy
            }
        }

        let registry = ServiceRegistry::new();
        registry.register(Arc::new(TestService)).await;
        assert_eq!(registry.service_count().await, 1);
        assert!(registry.has_service("test").await);
    }

    #[tokio::test]
    async fn service_registry_health_check() {
        struct TestService;

        #[async_trait::async_trait]
        impl ManagedService for TestService {
            fn name(&self) -> &str {
                "test"
            }
            async fn start(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            async fn stop(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            fn health_check(&self) -> HealthStatus {
                HealthStatus::Healthy
            }
        }

        let registry = ServiceRegistry::new();
        registry.register(Arc::new(TestService)).await;
        let health = registry.health_check_all().await;
        assert_eq!(health.get("test"), Some(&HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn service_registry_missing_dependency() {
        struct DepService;

        #[async_trait::async_trait]
        impl ManagedService for DepService {
            fn name(&self) -> &str {
                "dep"
            }
            async fn start(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            async fn stop(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            fn health_check(&self) -> HealthStatus {
                HealthStatus::Healthy
            }
            fn dependencies(&self) -> Vec<String> {
                vec!["nonexistent".to_string()]
            }
        }

        let registry = ServiceRegistry::new();
        registry.register(Arc::new(DepService)).await;
        let result = registry.compute_startup_order().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn service_registry_get_service() {
        struct TestService;

        #[async_trait::async_trait]
        impl ManagedService for TestService {
            fn name(&self) -> &str {
                "test"
            }
            async fn start(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            async fn stop(&self) -> voxy_shared::Result<()> {
                Ok(())
            }
            fn health_check(&self) -> HealthStatus {
                HealthStatus::Healthy
            }
        }

        let registry = ServiceRegistry::new();
        registry.register(Arc::new(TestService)).await;
        assert!(registry.get_service("test").await.is_some());
        assert!(registry.get_service("nonexistent").await.is_none());
    }
}
