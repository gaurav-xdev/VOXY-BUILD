//! Runtime traits and types.

use async_trait::async_trait;
use voxy_shared::HealthStatus;

/// Runtime lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
    Recovering,
}

/// Shutdown policy for a runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownPolicy {
    Graceful,
    Immediate,
    Timeout { seconds: u64 },
}

/// Startup order strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupOrder {
    DependencyDriven,
    Sequential,
    Parallel,
}

/// Runtime specification.
#[derive(Debug, Clone)]
pub struct RuntimeSpec {
    pub name: String,
    pub dependencies: Vec<String>,
    pub startup_timeout_seconds: u64,
    pub shutdown_policy: ShutdownPolicy,
    pub health_check_interval_ms: u64,
    pub auto_restart: bool,
}

/// Managed runtime trait.
#[async_trait]
pub trait ManagedRuntime: Send + Sync {
    /// Get the runtime name.
    fn name(&self) -> &str;

    /// Get the current state.
    fn state(&self) -> RuntimeState;

    /// Check if the runtime is healthy.
    fn is_healthy(&self) -> bool;

    /// Start the runtime.
    async fn start(&mut self) -> crate::Result<()>;

    /// Stop the runtime.
    async fn stop(&mut self) -> crate::Result<()>;

    /// Pause the runtime.
    async fn pause(&mut self) -> crate::Result<()>;

    /// Resume the runtime.
    async fn resume(&mut self) -> crate::Result<()>;

    /// Restart the runtime.
    async fn restart(&mut self) -> crate::Result<()> {
        self.stop().await?;
        self.start().await
    }

    /// Perform a health check.
    async fn health_check(&self) -> HealthStatus;
}
