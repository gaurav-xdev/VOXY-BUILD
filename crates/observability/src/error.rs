//! Observability error types.

use std::fmt;

/// Observability error type.
#[derive(Debug)]
pub enum ObservabilityError {
    TrackingFailed(String),
    ExportFailed(String),
}

impl fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrackingFailed(msg) => write!(f, "Tracking failed: {}", msg),
            Self::ExportFailed(msg) => write!(f, "Export failed: {}", msg),
        }
    }
}

impl std::error::Error for ObservabilityError {}

pub type Result<T> = std::result::Result<T, ObservabilityError>;
