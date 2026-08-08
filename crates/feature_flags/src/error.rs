//! Feature flag error types.

use std::fmt;

/// Feature flag error type.
#[derive(Debug)]
pub enum FeatureFlagError {
    ParseError(String),
    NotFound(String),
}

impl fmt::Display for FeatureFlagError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NotFound(key) => write!(f, "Flag not found: {}", key),
        }
    }
}

impl std::error::Error for FeatureFlagError {}

pub type Result<T> = std::result::Result<T, FeatureFlagError>;
