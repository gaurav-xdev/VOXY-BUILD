use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuardError {
    #[error("Subsystem not found: {0}")]
    SubsystemNotFound(String),

    #[error("Heartbeat missed for subsystem: {0}")]
    HeartbeatMissed(String),

    #[error("Self-healing failed for {subsystem}: {reason}")]
    SelfHealingFailed { subsystem: String, reason: String },

    #[error("Max restart attempts exceeded for subsystem: {0}")]
    MaxRestartsExceeded(String),

    #[error("Guard is not running")]
    NotRunning,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, GuardError>;
