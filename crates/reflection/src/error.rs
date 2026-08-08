//! Reflection error types.

use std::fmt;

/// Reflection error type.
#[derive(Debug)]
pub enum ReflectionError {
    EvaluationFailed(String),
    LearningFailed(String),
}

impl fmt::Display for ReflectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EvaluationFailed(msg) => write!(f, "Evaluation failed: {}", msg),
            Self::LearningFailed(msg) => write!(f, "Learning failed: {}", msg),
        }
    }
}

impl std::error::Error for ReflectionError {}

pub type Result<T> = std::result::Result<T, ReflectionError>;
