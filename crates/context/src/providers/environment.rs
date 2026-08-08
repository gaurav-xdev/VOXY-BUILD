use crate::error::Result;
use crate::provider::ContextProvider;
use crate::types::{ContextPriority, ContextSnapshot, ContextSource};
use async_trait::async_trait;

/// Provides environment context (time, system info, network status).
pub struct EnvironmentContextProvider {
    hostname: String,
    os: String,
    network_available: bool,
}

impl EnvironmentContextProvider {
    /// Create a new environment context provider.
    pub fn new() -> Self {
        Self {
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            os: std::env::consts::OS.to_string(),
            network_available: true,
        }
    }

    /// Create with custom values (for testing).
    pub fn with_values(hostname: String, os: String, network_available: bool) -> Self {
        Self {
            hostname,
            os,
            network_available,
        }
    }

    /// Update network availability status.
    pub fn set_network_available(&mut self, available: bool) {
        self.network_available = available;
    }
}

impl Default for EnvironmentContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContextProvider for EnvironmentContextProvider {
    fn name(&self) -> &str {
        "environment"
    }

    fn source(&self) -> ContextSource {
        ContextSource::Environment
    }

    fn default_priority(&self) -> ContextPriority {
        ContextPriority::Medium
    }

    async fn collect(&self) -> Result<ContextSnapshot> {
        let now = chrono::Utc::now();

        let data = serde_json::json!({
            "timestamp": now.to_rfc3339(),
            "hour": now.format("%H").to_string(),
            "minute": now.format("%M").to_string(),
            "day_of_week": now.format("%A").to_string(),
            "date": now.format("%Y-%m-%d").to_string(),
            "timezone": "UTC",
            "hostname": self.hostname,
            "os": self.os,
            "arch": std::env::consts::ARCH,
            "network_available": self.network_available,
        });

        Ok(ContextSnapshot::new(ContextSource::Environment, data))
    }

    async fn health_check(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_collect_environment() {
        let provider = EnvironmentContextProvider::with_values(
            "test-host".to_string(),
            "linux".to_string(),
            true,
        );

        let snapshot = provider.collect().await.unwrap();
        assert_eq!(snapshot.source, ContextSource::Environment);
        assert!(snapshot.data.get("hostname").is_some());
        assert!(snapshot.data.get("os").is_some());
        assert!(snapshot.data.get("network_available").is_some());
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = EnvironmentContextProvider::new();
        assert!(provider.health_check().await);
    }

    #[test]
    fn test_name_and_source() {
        let provider = EnvironmentContextProvider::new();
        assert_eq!(provider.name(), "environment");
        assert_eq!(provider.source(), ContextSource::Environment);
        assert_eq!(provider.default_priority(), ContextPriority::Medium);
    }
}
