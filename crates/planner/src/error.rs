//! Planner error types.

use std::fmt;

/// Planner error type.
#[derive(Debug)]
pub enum PlannerError {
    PlanningFailed(String),
    InvalidGoal(String),
    DependencyCycle,
}

impl fmt::Display for PlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PlanningFailed(msg) => write!(f, "Planning failed: {}", msg),
            Self::InvalidGoal(msg) => write!(f, "Invalid goal: {}", msg),
            Self::DependencyCycle => write!(f, "Dependency cycle detected"),
        }
    }
}

impl std::error::Error for PlannerError {}

pub type Result<T> = std::result::Result<T, PlannerError>;
