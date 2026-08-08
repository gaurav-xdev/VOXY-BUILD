//! Plan step execution runtime with retry and rollback.

pub mod error;

pub use error::{ExecutorError, Result};

/// Executor runtime.
pub struct ExecutorRuntime;

impl ExecutorRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExecutorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Retry policy.
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    None,
    Immediate { max_retries: u32 },
    Exponential { max_retries: u32, base_ms: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executor_creates() {
        let _e = ExecutorRuntime::new();
    }
}
