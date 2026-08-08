use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Latency record for a subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyRecord {
    /// Subsystem name.
    pub subsystem: String,

    /// Last measured latency.
    pub last_latency: Duration,

    /// Average latency over the measurement window.
    pub avg_latency: Duration,

    /// Maximum latency observed.
    pub max_latency: Duration,

    /// Minimum latency observed.
    pub min_latency: Duration,

    /// Number of measurements taken.
    pub sample_count: u64,
}

impl LatencyRecord {
    pub fn new(subsystem: &str) -> Self {
        Self {
            subsystem: subsystem.to_string(),
            last_latency: Duration::ZERO,
            avg_latency: Duration::ZERO,
            max_latency: Duration::ZERO,
            min_latency: Duration::from_secs(u64::MAX),
            sample_count: 0,
        }
    }

    pub fn record(&mut self, latency: Duration) {
        self.last_latency = latency;
        self.sample_count += 1;
        if latency > self.max_latency {
            self.max_latency = latency;
        }
        if latency < self.min_latency {
            self.min_latency = latency;
        }
        // Running average
        let total_us = self.avg_latency.as_micros() as f64 * (self.sample_count - 1) as f64
            + latency.as_micros() as f64;
        self.avg_latency = Duration::from_micros((total_us / self.sample_count as f64) as u64);
    }
}

/// Overall health status of the cognitive system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Critical(String),
}

/// Snapshot of the cognitive system's internal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// Latency records for all subsystems.
    pub latencies: HashMap<String, LatencyRecord>,

    /// Overall health status.
    pub health: HealthStatus,

    /// Total context fusion operations.
    pub fusion_count: u64,

    /// Total cognitive cycles completed.
    pub cycle_count: u64,

    /// Total errors encountered.
    pub error_count: u64,

    /// Current memory usage estimate (bytes).
    pub memory_estimate: u64,

    /// Active goals count.
    pub active_goals: usize,

    /// Context staleness events.
    pub staleness_events: u64,
}

impl Default for SystemSnapshot {
    fn default() -> Self {
        Self {
            latencies: HashMap::new(),
            health: HealthStatus::Healthy,
            fusion_count: 0,
            cycle_count: 0,
            error_count: 0,
            memory_estimate: 0,
            active_goals: 0,
            staleness_events: 0,
        }
    }
}

/// Trait for self-monitoring — tracks latencies, health, and diagnostics.
#[async_trait]
pub trait SelfMonitor: Send + Sync {
    /// Record latency for a subsystem.
    async fn record_latency(&self, subsystem: &str, latency: Duration) -> Result<()>;

    /// Get latency record for a subsystem.
    async fn get_latency(&self, subsystem: &str) -> Result<Option<LatencyRecord>>;

    /// Get all latency records.
    async fn all_latencies(&self) -> Result<HashMap<String, LatencyRecord>>;

    /// Increment the fusion counter.
    async fn record_fusion(&self) -> Result<()>;

    /// Increment the cycle counter.
    async fn record_cycle(&self) -> Result<()>;

    /// Increment the error counter.
    async fn record_error(&self) -> Result<()>;

    /// Record a staleness event.
    async fn record_staleness(&self) -> Result<()>;

    /// Update active goals count.
    async fn set_active_goals(&self, count: usize) -> Result<()>;

    /// Get a full system snapshot.
    async fn snapshot(&self) -> Result<SystemSnapshot>;

    /// Get current health status.
    async fn health(&self) -> Result<HealthStatus>;
}
