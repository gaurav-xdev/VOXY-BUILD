//! Platform error types.

use std::fmt;

/// Platform error type.
#[derive(Debug)]
pub enum PlatformError {
    UnsupportedPlatform(String),
    ApiNotAvailable(String),
    PermissionDenied(String),
    InitializationFailed(String),
    QueryFailed(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform(os) => write!(f, "Unsupported platform: {}", os),
            Self::ApiNotAvailable(api) => write!(f, "API not available: {}", api),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            Self::InitializationFailed(msg) => write!(f, "Initialization failed: {}", msg),
            Self::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
        }
    }
}

impl std::error::Error for PlatformError {}

/// Platform result type.
pub type Result<T> = std::result::Result<T, PlatformError>;
