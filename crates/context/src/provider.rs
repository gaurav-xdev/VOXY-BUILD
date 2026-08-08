use crate::error::Result;
use crate::types::{ContextSnapshot, ContextSource};
use async_trait::async_trait;

/// Trait for providing context from a specific source.
///
/// Implementors of this trait supply context snapshots to the context runtime.
/// Each provider is responsible for collecting, processing, and returning
/// a point-in-time snapshot of its context domain.
#[async_trait]
pub trait ContextProvider: Send + Sync {
    /// Returns the source type this provider handles.
    fn source(&self) -> ContextSource;

    /// Returns a human-readable name for this provider.
    fn name(&self) -> &str;

    /// Collect a fresh context snapshot from this source.
    async fn collect(&self) -> Result<ContextSnapshot>;

    /// Check if this provider is healthy and capable of collecting context.
    async fn health_check(&self) -> bool {
        true
    }

    /// Returns the default priority for this provider's context.
    fn default_priority(&self) -> crate::types::ContextPriority {
        crate::types::ContextPriority::Medium
    }
}
