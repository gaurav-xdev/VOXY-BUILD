//! Deterministic mock implementations for integration testing.

pub mod error;

pub use error::{Result, SimulationError};

/// Simulation runtime providing mock implementations.
pub struct SimulationRuntime;

impl SimulationRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimulationRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock microphone.
pub struct MockMicrophone;

/// Mock speaker.
pub struct MockSpeaker;

/// Mock automation backend.
pub struct MockAutomationBackend;

/// Mock LLM provider.
pub struct MockLlmProvider;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulation_creates() {
        let _s = SimulationRuntime::new();
    }
}
