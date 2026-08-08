//! Agent error types.

use std::fmt;

/// Agent error type.
#[derive(Debug)]
pub enum AgentError {
    LifecycleFailed(String),
    TaskFailed(String),
    SupervisionFailed(String),
    RegistryError(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LifecycleFailed(msg) => write!(f, "Lifecycle failed: {}", msg),
            Self::TaskFailed(msg) => write!(f, "Task failed: {}", msg),
            Self::SupervisionFailed(msg) => write!(f, "Supervision failed: {}", msg),
            Self::RegistryError(msg) => write!(f, "Registry error: {}", msg),
        }
    }
}

impl std::error::Error for AgentError {}

pub type Result<T> = std::result::Result<T, AgentError>;
