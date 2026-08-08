//! SDK error types.

use std::fmt;

/// SDK error type.
#[derive(Debug)]
pub enum SdkError {
    InvalidApi(String),
    VersionMismatch(String),
}

impl fmt::Display for SdkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidApi(msg) => write!(f, "Invalid API: {}", msg),
            Self::VersionMismatch(msg) => write!(f, "Version mismatch: {}", msg),
        }
    }
}

impl std::error::Error for SdkError {}

pub type Result<T> = std::result::Result<T, SdkError>;
