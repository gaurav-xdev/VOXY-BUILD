//! Grounding error types.

use std::fmt;

/// Grounding error type.
#[derive(Debug)]
pub enum GroundingError {
    TargetNotFound(String),
    AmbiguousTarget(String),
    VerificationFailed(String),
}

impl fmt::Display for GroundingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetNotFound(msg) => write!(f, "Target not found: {}", msg),
            Self::AmbiguousTarget(msg) => write!(f, "Ambiguous target: {}", msg),
            Self::VerificationFailed(msg) => write!(f, "Verification failed: {}", msg),
        }
    }
}

impl std::error::Error for GroundingError {}

pub type Result<T> = std::result::Result<T, GroundingError>;
