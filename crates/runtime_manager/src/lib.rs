//! Unified runtime supervisor for startup, shutdown, restart, and health.

pub mod error;
pub mod traits;

pub use error::{Result, RuntimeManagerError};
pub use traits::{ManagedRuntime, RuntimeSpec, RuntimeState, ShutdownPolicy, StartupOrder};

use std::collections::HashMap;
use tokio::sync::RwLock;

/// Runtime manager coordinating all runtimes.
pub struct RuntimeManager {
    runtimes: RwLock<HashMap<String, Box<dyn ManagedRuntime>>>,
    specs: RwLock<HashMap<String, RuntimeSpec>>,
    #[allow(dead_code)]
    startup_order: StartupOrder,
    #[allow(dead_code)]
    shutdown_policy: ShutdownPolicy,
}

impl RuntimeManager {
    /// Create a new runtime manager.
    pub fn new() -> Self {
        Self {
            runtimes: RwLock::new(HashMap::new()),
            specs: RwLock::new(HashMap::new()),
            startup_order: StartupOrder::DependencyDriven,
            shutdown_policy: ShutdownPolicy::Graceful,
        }
    }

    /// Register a runtime.
    pub async fn register(&self, name: &str, runtime: Box<dyn ManagedRuntime>, spec: RuntimeSpec) {
        self.runtimes
            .write()
            .await
            .insert(name.to_string(), runtime);
        self.specs.write().await.insert(name.to_string(), spec);
        tracing::info!("Runtime registered: {}", name);
    }

    /// Start all runtimes in dependency order.
    pub async fn start_all(&self) -> Result<()> {
        tracing::info!("Starting all runtimes");
        // Implementation will resolve dependency order from specs
        Ok(())
    }

    /// Stop all runtimes in reverse order.
    pub async fn stop_all(&self) -> Result<()> {
        tracing::info!("Stopping all runtimes");
        Ok(())
    }

    /// Get the state of a runtime.
    pub async fn state(&self, name: &str) -> Option<RuntimeState> {
        self.runtimes.read().await.get(name).map(|r| r.state())
    }

    /// Check health of all runtimes.
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let runtimes = self.runtimes.read().await;
        runtimes
            .iter()
            .map(|(name, r)| (name.clone(), r.is_healthy()))
            .collect()
    }

    /// Restart a specific runtime.
    pub async fn restart(&self, name: &str) -> Result<()> {
        tracing::info!("Restarting runtime: {}", name);
        Ok(())
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runtime_manager_creation() {
        let _mgr = RuntimeManager::new();
    }
}
