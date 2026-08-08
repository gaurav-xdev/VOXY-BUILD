use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("Health check not found: {0}")]
    NotFound(String),

    #[error("Health check failed: {0}")]
    CheckFailed(String),

    #[error("Health check timeout")]
    Timeout,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Watchdog is not running")]
    WatchdogNotRunning,

    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    #[error("Self-test failed: {0}")]
    SelfTestFailed(String),
}

pub type Result<T> = std::result::Result<T, HealthError>;
