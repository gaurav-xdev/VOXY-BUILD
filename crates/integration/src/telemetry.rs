//! Central Telemetry — aggregated metrics from all subsystems.
//!
//! Every subsystem reports metrics into this central system.
//! Provides a unified view of the entire system's health and performance.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// Metric Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemMetrics {
    pub name: String,
    pub latency_ms: f64,
    pub error_count: u64,
    pub warning_count: u64,
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub queue_size: u32,
    pub events_per_sec: f64,
    pub uptime_seconds: u64,
    pub last_error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl SubsystemMetrics {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            latency_ms: 0.0,
            error_count: 0,
            warning_count: 0,
            cpu_percent: 0.0,
            memory_mb: 0,
            queue_size: 0,
            events_per_sec: 0.0,
            uptime_seconds: 0,
            last_error: None,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTelemetrySnapshot {
    pub subsystems: HashMap<String, SubsystemMetrics>,
    pub total_errors: u64,
    pub total_warnings: u64,
    pub total_memory_mb: u64,
    pub aggregate_cpu_percent: f32,
    pub total_events_per_sec: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryAlert {
    pub id: String,
    pub subsystem: String,
    pub severity: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
}

// ============================================================================
// Central Telemetry
// ============================================================================

/// Aggregates metrics from all subsystems into one monitoring system.
pub struct CentralTelemetry {
    metrics: RwLock<HashMap<String, SubsystemMetrics>>,
    alerts: RwLock<Vec<TelemetryAlert>>,
    max_alerts: usize,
    /// Thresholds for auto-alerting.
    latency_threshold_ms: f64,
    error_rate_threshold: u64,
    memory_threshold_mb: u64,
    cpu_threshold_percent: f32,
}

impl CentralTelemetry {
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
            alerts: RwLock::new(Vec::new()),
            max_alerts: 100,
            latency_threshold_ms: 1000.0,
            error_rate_threshold: 10,
            memory_threshold_mb: 2048,
            cpu_threshold_percent: 90.0,
        }
    }

    pub fn with_thresholds(
        latency_ms: f64,
        error_rate: u64,
        memory_mb: u64,
        cpu_percent: f32,
    ) -> Self {
        Self {
            latency_threshold_ms: latency_ms,
            error_rate_threshold: error_rate,
            memory_threshold_mb: memory_mb,
            cpu_threshold_percent: cpu_percent,
            ..Self::new()
        }
    }

    /// Report metrics for a subsystem.
    pub fn report(&self, metrics: SubsystemMetrics) {
        self.check_thresholds(&metrics);
        self.metrics.write().insert(metrics.name.clone(), metrics);
    }

    /// Get metrics for a specific subsystem.
    pub fn get_metrics(&self, name: &str) -> Option<SubsystemMetrics> {
        self.metrics.read().get(name).cloned()
    }

    /// Get a snapshot of all metrics.
    pub fn snapshot(&self) -> SystemTelemetrySnapshot {
        let metrics = self.metrics.read().clone();
        let total_errors = metrics.values().map(|m| m.error_count).sum();
        let total_warnings = metrics.values().map(|m| m.warning_count).sum();
        let total_memory_mb = metrics.values().map(|m| m.memory_mb).sum();
        let count = metrics.len().max(1) as f32;
        let aggregate_cpu: f32 = metrics.values().map(|m| m.cpu_percent).sum::<f32>() / count;
        let total_eps: f64 = metrics.values().map(|m| m.events_per_sec).sum();

        SystemTelemetrySnapshot {
            subsystems: metrics,
            total_errors,
            total_warnings,
            total_memory_mb,
            aggregate_cpu_percent: aggregate_cpu,
            total_events_per_sec: total_eps,
            timestamp: Utc::now(),
        }
    }

    /// Get all alerts.
    pub fn alerts(&self) -> Vec<TelemetryAlert> {
        self.alerts.read().clone()
    }

    /// Get unacknowledged alerts.
    pub fn unacknowledged_alerts(&self) -> Vec<TelemetryAlert> {
        self.alerts
            .read()
            .iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    /// Acknowledge an alert.
    pub fn acknowledge_alert(&self, alert_id: &str) -> bool {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            true
        } else {
            false
        }
    }

    /// Get total error count across all subsystems.
    pub fn total_errors(&self) -> u64 {
        self.metrics.read().values().map(|m| m.error_count).sum()
    }

    /// Get total warning count across all subsystems.
    pub fn total_warnings(&self) -> u64 {
        self.metrics.read().values().map(|m| m.warning_count).sum()
    }

    /// Get subsystem names.
    pub fn subsystem_names(&self) -> Vec<String> {
        self.metrics.read().keys().cloned().collect()
    }

    /// Remove a subsystem's metrics.
    pub fn remove_subsystem(&self, name: &str) {
        self.metrics.write().remove(name);
    }

    /// Clear all metrics.
    pub fn clear(&self) {
        self.metrics.write().clear();
        self.alerts.write().clear();
    }

    fn check_thresholds(&self, metrics: &SubsystemMetrics) {
        if metrics.latency_ms > self.latency_threshold_ms {
            self.add_alert(
                &metrics.name,
                "warning",
                &format!("High latency: {:.1}ms", metrics.latency_ms),
            );
        }
        if metrics.error_count > self.error_rate_threshold {
            self.add_alert(
                &metrics.name,
                "error",
                &format!("High error count: {}", metrics.error_count),
            );
        }
        if metrics.memory_mb > self.memory_threshold_mb {
            self.add_alert(
                &metrics.name,
                "warning",
                &format!("High memory: {}MB", metrics.memory_mb),
            );
        }
        if metrics.cpu_percent > self.cpu_threshold_percent {
            self.add_alert(
                &metrics.name,
                "critical",
                &format!("High CPU: {:.1}%", metrics.cpu_percent),
            );
        }
    }

    fn add_alert(&self, subsystem: &str, severity: &str, message: &str) {
        let alert = TelemetryAlert {
            id: uuid::Uuid::new_v4().to_string(),
            subsystem: subsystem.to_string(),
            severity: severity.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            acknowledged: false,
        };
        let mut alerts = self.alerts.write();
        if alerts.len() >= self.max_alerts {
            alerts.remove(0);
        }
        alerts.push(alert);
    }
}

impl Default for CentralTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_creation() {
        let t = CentralTelemetry::new();
        assert_eq!(t.subsystem_names().len(), 0);
    }

    #[test]
    fn report_and_snapshot() {
        let t = CentralTelemetry::new();
        t.report(SubsystemMetrics {
            name: "voice".to_string(),
            latency_ms: 50.0,
            error_count: 2,
            warning_count: 1,
            cpu_percent: 15.0,
            memory_mb: 256,
            queue_size: 5,
            events_per_sec: 10.0,
            uptime_seconds: 3600,
            last_error: None,
            timestamp: Utc::now(),
        });

        let snap = t.snapshot();
        assert_eq!(snap.subsystems.len(), 1);
        assert_eq!(snap.total_errors, 2);
        assert_eq!(snap.total_warnings, 1);
        assert_eq!(snap.total_memory_mb, 256);
    }

    #[test]
    fn alert_on_high_latency() {
        let t = CentralTelemetry::with_thresholds(100.0, 10, 2048, 90.0);
        t.report(SubsystemMetrics {
            name: "slow_svc".to_string(),
            latency_ms: 200.0,
            ..SubsystemMetrics::new("slow_svc")
        });
        assert!(!t.unacknowledged_alerts().is_empty());
    }

    #[test]
    fn alert_on_high_errors() {
        let t = CentralTelemetry::with_thresholds(1000.0, 5, 2048, 90.0);
        t.report(SubsystemMetrics {
            name: "failing_svc".to_string(),
            error_count: 10,
            ..SubsystemMetrics::new("failing_svc")
        });
        assert!(!t.unacknowledged_alerts().is_empty());
    }

    #[test]
    fn no_alert_within_thresholds() {
        let t = CentralTelemetry::with_thresholds(1000.0, 10, 2048, 90.0);
        t.report(SubsystemMetrics {
            name: "healthy_svc".to_string(),
            latency_ms: 50.0,
            error_count: 1,
            memory_mb: 128,
            cpu_percent: 30.0,
            ..SubsystemMetrics::new("healthy_svc")
        });
        assert!(t.unacknowledged_alerts().is_empty());
    }

    #[test]
    fn acknowledge_alert() {
        let t = CentralTelemetry::with_thresholds(100.0, 10, 2048, 90.0);
        t.report(SubsystemMetrics {
            name: "svc".to_string(),
            latency_ms: 200.0,
            ..SubsystemMetrics::new("svc")
        });
        let alerts = t.unacknowledged_alerts();
        assert_eq!(alerts.len(), 1);
        t.acknowledge_alert(&alerts[0].id);
        assert!(t.unacknowledged_alerts().is_empty());
    }

    #[test]
    fn get_metrics() {
        let t = CentralTelemetry::new();
        t.report(SubsystemMetrics::new("svc"));
        assert!(t.get_metrics("svc").is_some());
        assert!(t.get_metrics("nonexistent").is_none());
    }

    #[test]
    fn remove_subsystem() {
        let t = CentralTelemetry::new();
        t.report(SubsystemMetrics::new("svc"));
        assert_eq!(t.subsystem_names().len(), 1);
        t.remove_subsystem("svc");
        assert_eq!(t.subsystem_names().len(), 0);
    }

    #[test]
    fn clear() {
        let t = CentralTelemetry::new();
        t.report(SubsystemMetrics::new("a"));
        t.report(SubsystemMetrics::new("b"));
        t.clear();
        assert_eq!(t.subsystem_names().len(), 0);
        assert!(t.alerts().is_empty());
    }

    #[test]
    fn snapshot_totals() {
        let t = CentralTelemetry::new();
        t.report(SubsystemMetrics {
            name: "a".to_string(),
            error_count: 3,
            warning_count: 2,
            memory_mb: 100,
            cpu_percent: 20.0,
            events_per_sec: 5.0,
            ..SubsystemMetrics::new("a")
        });
        t.report(SubsystemMetrics {
            name: "b".to_string(),
            error_count: 7,
            warning_count: 4,
            memory_mb: 200,
            cpu_percent: 40.0,
            events_per_sec: 15.0,
            ..SubsystemMetrics::new("b")
        });

        let snap = t.snapshot();
        assert_eq!(snap.total_errors, 10);
        assert_eq!(snap.total_warnings, 6);
        assert_eq!(snap.total_memory_mb, 300);
        assert!((snap.total_events_per_sec - 20.0).abs() < 0.01);
    }

    #[test]
    fn alert_max_limit() {
        let t = CentralTelemetry::with_thresholds(0.0, 0, 0, 0.0);
        for i in 0..150 {
            t.report(SubsystemMetrics {
                name: format!("svc_{}", i),
                latency_ms: 1.0,
                timestamp: Utc::now(),
                ..SubsystemMetrics::new(format!("svc_{}", i))
            });
        }
        assert!(t.alerts().len() <= 100);
    }
}
