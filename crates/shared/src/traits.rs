//! Common traits for the VOXY platform.
//!
//! All subsystems depend on these traits, never on concrete implementations.
//! This ensures extensibility and testability.

use std::fmt;

/// Lifecycle management trait.
///
/// Every subsystem must implement this trait to participate in the kernel's
/// lifecycle management. The lifecycle states are:
///
/// ```text
/// Uninitialized → Initialized → Running → Paused → Running → Stopped
///                                     ↘ Stopped
/// ```
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to allow safe concurrent access.
#[async_trait::async_trait]
pub trait Lifecycle: Send + Sync {
    /// Initialize the subsystem (allocate resources, load config).
    ///
    /// This is called once before `start()`. The subsystem should validate
    /// its configuration and prepare any internal state.
    async fn initialize(&mut self) -> crate::Result<()>;

    /// Start the subsystem (begin processing).
    ///
    /// Called after `initialize()`. The subsystem should begin its main work.
    async fn start(&mut self) -> crate::Result<()>;

    /// Pause the subsystem (temporarily stop processing).
    ///
    /// The subsystem should stop processing new work but keep its resources
    /// allocated. Can be resumed later with `resume()`.
    async fn pause(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Resume the subsystem after a pause.
    ///
    /// The subsystem should resume processing from where it left off.
    async fn resume(&mut self) -> crate::Result<()> {
        Ok(())
    }

    /// Stop the subsystem (release resources, finish pending work).
    ///
    /// Called during shutdown. The subsystem should finish any pending work
    /// and release all resources.
    async fn stop(&mut self) -> crate::Result<()>;

    /// Restart the subsystem (stop then start).
    ///
    /// Default implementation calls `stop()` then `start()`.
    /// Override for custom restart behavior.
    async fn restart(&mut self) -> crate::Result<()> {
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Reload configuration without full restart.
    ///
    /// The subsystem should apply any configuration changes that can be
    /// applied at runtime. Returns an error if the new config is invalid.
    async fn reload(&mut self, _config: &dyn std::any::Any) -> crate::Result<()> {
        Ok(())
    }

    /// Check the health of the subsystem.
    fn health_check(&self) -> HealthStatus;
}

/// Configuration trait.
///
/// Types implementing this trait can provide default configurations
/// and validate their configuration values.
pub trait Configurable: Send + Sync {
    /// The configuration type.
    type Config;

    /// Validate the configuration.
    fn validate_config(config: &Self::Config) -> crate::Result<()>;

    /// Get the default configuration.
    fn default_config() -> Self::Config;
}

/// Health status of a subsystem.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    #[default]
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

impl HealthStatus {
    /// Returns true if healthy.
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    /// Returns true if degraded.
    pub fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded(_))
    }

    /// Returns true if unhealthy.
    pub fn is_unhealthy(&self) -> bool {
        matches!(self, Self::Unhealthy(_))
    }

    /// Get the status message if any.
    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Healthy => None,
            Self::Degraded(msg) | Self::Unhealthy(msg) => Some(msg),
        }
    }

    /// Convert to a serializable format for health endpoints.
    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            Self::Healthy => serde_json::json!({"status": "healthy"}),
            Self::Degraded(msg) => serde_json::json!({"status": "degraded", "message": msg}),
            Self::Unhealthy(msg) => serde_json::json!({"status": "unhealthy", "message": msg}),
        }
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded(msg) => write!(f, "degraded: {}", msg),
            Self::Unhealthy(msg) => write!(f, "unhealthy: {}", msg),
        }
    }
}

/// Severity is re-exported from error module to avoid duplication.
pub use crate::error::Severity;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_checks() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Healthy.is_degraded());
        assert!(!HealthStatus::Healthy.is_unhealthy());

        assert!(HealthStatus::Degraded("slow".to_string()).is_degraded());
        assert!(HealthStatus::Unhealthy("down".to_string()).is_unhealthy());
    }

    #[test]
    fn health_status_message() {
        assert!(HealthStatus::Healthy.message().is_none());
        assert_eq!(
            HealthStatus::Degraded("slow".to_string()).message(),
            Some("slow")
        );
    }

    #[test]
    fn health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Healthy);
    }

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(
            HealthStatus::Degraded("slow".to_string()).to_string(),
            "degraded: slow"
        );
        assert_eq!(
            HealthStatus::Unhealthy("down".to_string()).to_string(),
            "unhealthy: down"
        );
    }

    #[test]
    fn health_status_json() {
        let healthy = HealthStatus::Healthy.to_json_value();
        assert_eq!(healthy["status"], "healthy");

        let degraded = HealthStatus::Degraded("slow".to_string()).to_json_value();
        assert_eq!(degraded["status"], "degraded");
        assert_eq!(degraded["message"], "slow");
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Debug < Severity::Info);
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }
}
