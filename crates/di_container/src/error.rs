//! Container error types.

use std::fmt;

/// Container error type.
#[derive(Debug)]
pub enum ContainerError {
    NotRegistered(String),
    ResolutionFailed(String),
    DuplicateRegistration(String),
    LifecycleError(String),
}

impl fmt::Display for ContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRegistered(name) => write!(f, "Service not registered: {}", name),
            Self::ResolutionFailed(msg) => write!(f, "Resolution failed: {}", msg),
            Self::DuplicateRegistration(name) => write!(f, "Duplicate registration: {}", name),
            Self::LifecycleError(msg) => write!(f, "Lifecycle error: {}", msg),
        }
    }
}

impl std::error::Error for ContainerError {}

pub type Result<T> = std::result::Result<T, ContainerError>;
