//! Post-execution evaluation and learning.

pub mod error;

pub use error::{ReflectionError, Result};

/// Reflection engine for evaluating outcomes.
pub struct ReflectionEngine;

impl ReflectionEngine {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReflectionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Outcome of an execution.
#[derive(Debug, Clone)]
pub enum Outcome {
    Success,
    Partial { reason: String },
    Failure { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflection_creates() {
        let _r = ReflectionEngine::new();
    }
}
