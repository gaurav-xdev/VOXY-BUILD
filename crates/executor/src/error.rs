//! Executor error types.

use std::fmt;

/// Executor error type.
#[derive(Debug)]
pub enum ExecutorError {
    ExecutionFailed(String),
    StepFailed(String),
    RollbackFailed(String),
    Timeout,
}

impl fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            Self::StepFailed(msg) => write!(f, "Step failed: {}", msg),
            Self::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            Self::Timeout => write!(f, "Execution timeout"),
        }
    }
}

impl std::error::Error for ExecutorError {}

pub type Result<T> = std::result::Result<T, ExecutorError>;
