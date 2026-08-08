//! Tool framework: registration, validation, invocation.

pub mod error;

pub use error::{Result, ToolError};

/// Tool registry for managing available tools.
pub struct ToolRegistry;

impl ToolRegistry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool trait for implementable tools.
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_registry_creates() {
        let _r = ToolRegistry::new();
    }
}
