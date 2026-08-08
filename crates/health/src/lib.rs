//! Health monitoring, diagnostics, self-test, watchdog, and auto-recovery.

pub mod config;
pub mod diagnostics;
pub mod error;
pub mod monitor;
pub mod report;
pub mod selftest;
pub mod state;
pub mod watchdog;

pub use config::HealthConfig;
pub use diagnostics::{DiagnosticsReport, SystemDiagnostics};
pub use error::{HealthError, Result};
pub use monitor::HealthMonitor;
pub use report::{ComponentType, HealthReport};
pub use selftest::{SelfTestResult, SelfTestRunner, SelfTestSummary};
pub use state::{ComponentState, StateTracker};
pub use watchdog::{RecoveryAction, RecoveryManager, StatusChange, Watchdog, WatchdogConfig};

// Re-export for convenience when used in monitor event bus checks
pub use voxy_event_bus::EventBus;
