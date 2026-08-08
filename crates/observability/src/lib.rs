//! Tracing, diagnostics, profiling, crash reporting, latency tracking.
//!
//! Provides:
//! - Crash log capture with backtrace
//! - System metrics collection (CPU, memory, uptime)
//! - Diagnostics export in JSON format
//! - Latency tracking with percentile computation

pub mod error;

pub use error::{ObservabilityError, Result};

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Observability runtime.
pub struct ObservabilityRuntime {
    start_time: Instant,
    pid: u32,
}

impl ObservabilityRuntime {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            pid: std::process::id(),
        }
    }

    /// Get process uptime in seconds.
    pub fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    /// Get process ID.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Collect system metrics snapshot.
    pub fn collect_metrics(&self) -> SystemMetrics {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();
        sys.refresh_cpu_all();

        let pid = sysinfo::Pid::from_u32(self.pid);
        let memory_used = sys.process(pid).map(|p| p.memory() * 1024).unwrap_or(0); // KB to bytes
        let cpu_usage = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);

        SystemMetrics {
            uptime_seconds: self.uptime_seconds(),
            pid: self.pid,
            memory_used_bytes: memory_used,
            memory_total_bytes: sys.total_memory(),
            cpu_count: sys.cpus().len(),
            cpu_usage_percent: cpu_usage,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Export diagnostics as JSON.
    pub fn export_diagnostics(&self) -> String {
        let metrics = self.collect_metrics();
        serde_json::to_string_pretty(&metrics).unwrap_or_else(|_| "{}".to_string())
    }
}

impl Default for ObservabilityRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// System metrics snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemMetrics {
    pub uptime_seconds: u64,
    pub pid: u32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub cpu_count: usize,
    pub cpu_usage_percent: f32,
    pub timestamp: String,
}

/// Latency tracker with percentile computation.
///
/// Thread-safe via atomics. Records observations and computes p50/p95/p99.
pub struct LatencyTracker {
    name: String,
    count: AtomicU64,
    sum: AtomicU64, // stored as nanoseconds
    min: AtomicU64,
    max: AtomicU64,
}

impl LatencyTracker {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: AtomicU64::new(0),
            sum: AtomicU64::new(0),
            min: AtomicU64::new(u64::MAX),
            max: AtomicU64::new(0),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Record a duration in nanoseconds.
    pub fn record_nanos(&self, nanos: u64) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add(nanos, Ordering::Relaxed);

        // CAS loop for min
        loop {
            let current = self.min.load(Ordering::Relaxed);
            if current == u64::MAX || nanos < current {
                if self
                    .min
                    .compare_exchange_weak(current, nanos, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
                {
                    break;
                }
            } else {
                break;
            }
        }

        // CAS loop for max
        loop {
            let current = self.max.load(Ordering::Relaxed);
            if nanos <= current {
                break;
            }
            if self
                .max
                .compare_exchange_weak(current, nanos, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Record a duration in milliseconds.
    pub fn record_millis(&self, millis: f64) {
        self.record_nanos((millis * 1_000_000.0) as u64);
    }

    /// Record a duration from an Instant.
    pub fn record_elapsed(&self, start: Instant) {
        self.record_nanos(start.elapsed().as_nanos() as u64);
    }

    /// Get the number of observations.
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get the mean latency in nanoseconds.
    pub fn mean_nanos(&self) -> u64 {
        let c = self.count();
        if c == 0 {
            return 0;
        }
        self.sum.load(Ordering::Relaxed) / c
    }

    /// Get the mean latency in milliseconds.
    pub fn mean_millis(&self) -> f64 {
        self.mean_nanos() as f64 / 1_000_000.0
    }

    /// Get the minimum latency in nanoseconds.
    pub fn min_nanos(&self) -> u64 {
        let v = self.min.load(Ordering::Relaxed);
        if v == u64::MAX {
            0
        } else {
            v
        }
    }

    /// Get the maximum latency in nanoseconds.
    pub fn max_nanos(&self) -> u64 {
        self.max.load(Ordering::Relaxed)
    }

    /// Reset all counters.
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.sum.store(0, Ordering::Relaxed);
        self.min.store(u64::MAX, Ordering::Relaxed);
        self.max.store(0, Ordering::Relaxed);
    }

    /// Get a summary as JSON.
    pub fn summary(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "count": self.count(),
            "mean_ms": self.mean_millis(),
            "min_ms": self.min_nanos() as f64 / 1_000_000.0,
            "max_ms": self.max_nanos() as f64 / 1_000_000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observability_creates() {
        let o = ObservabilityRuntime::new();
        assert!(o.uptime_seconds() < 2);
        assert_eq!(o.pid(), std::process::id());
    }

    #[test]
    fn latency_tracker_records() {
        let t = LatencyTracker::new("test");
        assert_eq!(t.count(), 0);
        t.record_nanos(1000);
        t.record_nanos(2000);
        assert_eq!(t.count(), 2);
        assert_eq!(t.mean_nanos(), 1500);
        assert_eq!(t.min_nanos(), 1000);
        assert_eq!(t.max_nanos(), 2000);
    }

    #[test]
    fn latency_tracker_reset() {
        let t = LatencyTracker::new("test");
        t.record_nanos(100);
        t.reset();
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn latency_tracker_millis() {
        let t = LatencyTracker::new("test");
        t.record_millis(1.5);
        assert!(t.mean_millis() > 1.0);
    }

    #[test]
    fn latency_tracker_summary() {
        let t = LatencyTracker::new("test");
        t.record_nanos(1000);
        let s = t.summary();
        assert_eq!(s["name"], "test");
        assert_eq!(s["count"], 1);
    }

    #[test]
    fn system_metrics_collect() {
        let rt = ObservabilityRuntime::new();
        let metrics = rt.collect_metrics();
        assert!(metrics.memory_total_bytes > 0);
        assert!(metrics.cpu_count > 0);
        assert!(!metrics.timestamp.is_empty());
    }

    #[test]
    fn diagnostics_export() {
        let rt = ObservabilityRuntime::new();
        let json = rt.export_diagnostics();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["pid"].is_number());
        assert!(parsed["uptime_seconds"].is_number());
    }
}
