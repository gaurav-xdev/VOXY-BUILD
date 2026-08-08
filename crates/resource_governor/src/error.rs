//! Resource governor error types.

use std::fmt;

/// Governor error type.
#[derive(Debug)]
pub enum GovernorError {
    MemoryExceeded,
    ConcurrencyExceeded,
    TimeoutExceeded,
}

impl fmt::Display for GovernorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MemoryExceeded => write!(f, "Memory budget exceeded"),
            Self::ConcurrencyExceeded => write!(f, "Concurrency budget exceeded"),
            Self::TimeoutExceeded => write!(f, "Timeout exceeded"),
        }
    }
}

impl std::error::Error for GovernorError {}

pub type Result<T> = std::result::Result<T, GovernorError>;
