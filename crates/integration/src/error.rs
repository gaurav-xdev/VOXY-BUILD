//! Error types for the integration layer.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IntegrationError {
    #[error("Service not registered: {0}")]
    ServiceNotFound(String),

    #[error("Service already registered: {0}")]
    ServiceAlreadyRegistered(String),

    #[error("Boot failed at phase {phase}: {reason}")]
    BootFailed { phase: String, reason: String },

    #[error("Subsystem recovery failed for {subsystem}: {reason}")]
    RecoveryFailed { subsystem: String, reason: String },

    #[error("Pipeline stage failed: {stage}")]
    PipelineStageFailed { stage: String },

    #[error("Event bridge error: {0}")]
    EventBridge(String),

    #[error("Telemetry error: {0}")]
    Telemetry(String),

    #[error("Container error: {0}")]
    Container(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, IntegrationError>;
