//! Service trait definitions.

use async_trait::async_trait;

/// Injectable trait for services.
#[async_trait]
pub trait Injectable: Send + Sync {
    /// Initialize the service.
    async fn initialize(&mut self) -> crate::Result<()>;

    /// Shutdown the service.
    async fn shutdown(&mut self) -> crate::Result<()>;
}

/// Service trait for managed services.
#[async_trait]
pub trait Service: Injectable {
    /// Get the service name.
    fn name(&self) -> &str;

    /// Check if the service is healthy.
    fn is_healthy(&self) -> bool;
}
