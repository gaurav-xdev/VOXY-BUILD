//! Runtime manager error types.

use std::fmt;

/// Runtime manager error type.
#[derive(Debug)]
pub enum RuntimeManagerError {
    StartupFailed(String),
    ShutdownFailed(String),
    RestartFailed(String),
    DependencyCycle(String),
    RuntimeNotFound(String),
    HealthCheckFailed(String),
}

impl fmt::Display for RuntimeManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StartupFailed(msg) => write!(f, "Startup failed: {}", msg),
            Self::ShutdownFailed(msg) => write!(f, "Shutdown failed: {}", msg),
            Self::RestartFailed(msg) => write!(f, "Restart failed: {}", msg),
            Self::DependencyCycle(msg) => write!(f, "Dependency cycle: {}", msg),
            Self::RuntimeNotFound(name) => write!(f, "Runtime not found: {}", name),
            Self::HealthCheckFailed(msg) => write!(f, "Health check failed: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeManagerError {}

pub type Result<T> = std::result::Result<T, RuntimeManagerError>;
