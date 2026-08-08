//! OPERATION TITAN FUSION — Unified Service Hub & Integration Layer
//!
//! This crate connects all existing subsystems into one cohesive operating system:
//!
//! - **ServiceHub**: Central service registry + DI container + event bus bridge
//! - **EventBridge**: Connects all subsystems through the existing EventBus
//! - **IntegrationPipeline**: Unified execution flow
//! - **CentralTelemetry**: Aggregated metrics from all subsystems
//! - **SubsystemRecovery**: Crash isolation and auto-restart
//! - **BootSequence**: Deterministic startup with status reporting

pub mod boot;
pub mod diagrams;
pub mod error;
pub mod event_bridge;
pub mod hub;
pub mod pipeline;
pub mod recovery;
pub mod telemetry;

pub use boot::{BootPhase, BootSequence, BootStatus};
pub use error::{IntegrationError, Result};
pub use event_bridge::EventBridge;
pub use hub::ServiceHub;
pub use pipeline::{PipelineStage, UnifiedPipeline};
pub use recovery::{RecoveryConfig, SubsystemRecovery};
pub use telemetry::{CentralTelemetry, SubsystemMetrics};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_compiles() {
        let _ = ServiceHub::new();
    }
}
