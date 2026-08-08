//! Manifest error types.

use std::fmt;

/// Manifest error type.
#[derive(Debug)]
pub enum ManifestError {
    InvalidManifest(String),
    DuplicateId(String),
    NotFound(String),
    ValidationError(String),
    SerializationError(String),
}

impl fmt::Display for ManifestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest(msg) => write!(f, "Invalid manifest: {}", msg),
            Self::DuplicateId(id) => write!(f, "Duplicate manifest ID: {}", id),
            Self::NotFound(id) => write!(f, "Manifest not found: {}", id),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ManifestError {}

pub type Result<T> = std::result::Result<T, ManifestError>;
