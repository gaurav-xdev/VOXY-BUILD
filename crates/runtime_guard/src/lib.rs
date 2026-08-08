//! VOXY Runtime Guard
//!
//! Unified runtime health monitor, self-healing engine, watchdog with heartbeats,
//! and dashboard for the VOXY autonomous assistant.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │              RuntimeGuard                       │
//! │  ┌──────────┐  ┌───────────┐  ┌─────────────┐ │
//! │  │ Health   │  │ Heartbeat │  │ SelfHealer  │ │
//! │  │ Monitor  │  │ Tracker   │  │             │ │
//! │  └────┬─────┘  └─────┬─────┘  └──────┬──────┘ │
//! │       │              │               │         │
//! │       └──────┬───────┴───────┬───────┘         │
//! │              │               │                  │
//! │       ┌──────▼──────┐ ┌─────▼──────┐          │
//! │       │  Runtime    │ │ Dashboard  │          │
//! │       │  Snapshot   │ │   Data     │          │
//! │       └─────────────┘ └────────────┘          │
//! └─────────────────────────────────────────────────┘
//!          │              │              │
//!    ┌─────▼────┐  ┌──────▼─────┐ ┌────▼──────┐
//!    │ Audio    │  │ Whisper    │ │ Ollama    │
//!    │ Capture  │  │ STT        │ │ LLM       │
//!    └──────────┘  └────────────┘ └───────────┘
//!    ┌──────────┐  ┌────────────┐ ┌───────────┐
//!    │ Desktop  │  │ Experience │ │ Cognitive │
//!    │ Watcher  │  │ Layer      │ │ Orch      │
//!    └──────────┘  └────────────┘ └───────────┘
//!    ┌──────────┐  ┌────────────┐ ┌───────────┐
//!    │ Memory   │  │ Visual     │ │Automation │
//!    │ System   │  │ Presence   │ │ Backend   │
//!    └──────────┘  └────────────┘ └───────────┘
//! ```

pub mod dashboard;
pub mod error;
pub mod guard;
pub mod heartbeat;
pub mod self_healing;
pub mod snapshot;

pub use dashboard::DashboardData;
pub use error::{GuardError, Result};
pub use guard::{GuardConfig, RuntimeGuard};
pub use heartbeat::{HeartbeatConfig, HeartbeatTracker};
pub use self_healing::{HealingConfig, RecoveryState, SelfHealer};
pub use snapshot::{RuntimeSnapshot, SubsystemStatus};
