//! State machine error types.

use crate::LifecycleState;
use std::fmt;

/// State machine error type.
#[derive(Debug)]
pub enum StateMachineError {
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
    StateNotFound(String),
}

impl fmt::Display for StateMachineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Invalid transition: {:?} -> {:?}", from, to)
            }
            Self::StateNotFound(name) => write!(f, "State not found: {}", name),
        }
    }
}

impl std::error::Error for StateMachineError {}

pub type Result<T> = std::result::Result<T, StateMachineError>;
