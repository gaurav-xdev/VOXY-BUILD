//! Tool error types.

use std::fmt;

/// Tool error type.
#[derive(Debug)]
pub enum ToolError {
    ExecutionFailed(String),
    InvalidParams(String),
    ToolNotFound(String),
    PermissionDenied(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::InvalidParams(msg) => write!(f, "Invalid params: {}", msg),
            Self::ToolNotFound(msg) => write!(f, "Tool not found: {}", msg),
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
        }
    }
}

impl std::error::Error for ToolError {}

pub type Result<T> = std::result::Result<T, ToolError>;
