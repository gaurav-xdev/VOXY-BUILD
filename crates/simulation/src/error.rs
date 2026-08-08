//! Simulation error types.

use std::fmt;

/// Simulation error type.
#[derive(Debug)]
pub enum SimulationError {
    MockFailed(String),
    StateError(String),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MockFailed(msg) => write!(f, "Mock failed: {}", msg),
            Self::StateError(msg) => write!(f, "State error: {}", msg),
        }
    }
}

impl std::error::Error for SimulationError {}

pub type Result<T> = std::result::Result<T, SimulationError>;
