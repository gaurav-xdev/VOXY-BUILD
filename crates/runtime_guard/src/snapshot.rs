use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use voxy_shared::HealthStatus;

/// Health status of an individual subsystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemStatus {
    pub name: String,
    pub health: HealthStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub restart_count: u32,
    pub last_error: Option<String>,
    pub latency_ms: Option<f64>,
    pub uptime_secs: u64,
}

/// Complete runtime snapshot for dashboard and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub timestamp: DateTime<Utc>,
    pub overall_health: HealthStatus,
    pub health_pct: f64,
    pub subsystems: HashMap<String, SubsystemStatus>,
    pub cpu_usage_pct: f64,
    pub ram_usage_mb: u64,
    pub ram_total_mb: u64,
    pub gpu_usage_pct: Option<f64>,
    pub thread_count: usize,
    pub active_tasks: usize,
    pub event_bus_backlog: usize,
    pub uptime_secs: u64,
    pub total_restarts: u32,
    pub current_mood: Option<String>,
    pub presence_state: Option<String>,
    pub current_goal: Option<String>,
    pub desktop_activity: Option<String>,
    pub current_plan: Option<String>,
    pub current_workflow: Option<String>,
}

impl RuntimeSnapshot {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            overall_health: HealthStatus::Healthy,
            health_pct: 100.0,
            subsystems: HashMap::new(),
            cpu_usage_pct: 0.0,
            ram_usage_mb: 0,
            ram_total_mb: 0,
            gpu_usage_pct: None,
            thread_count: 0,
            active_tasks: 0,
            event_bus_backlog: 0,
            uptime_secs: 0,
            total_restarts: 0,
            current_mood: None,
            presence_state: None,
            current_goal: None,
            desktop_activity: None,
            current_plan: None,
            current_workflow: None,
        }
    }

    /// Recalculate overall health from subsystem statuses.
    pub fn recalculate_overall_health(&mut self) {
        if self.subsystems.is_empty() {
            self.overall_health = HealthStatus::Healthy;
            self.health_pct = 100.0;
            return;
        }

        let total = self.subsystems.len() as f64;
        let healthy = self
            .subsystems
            .values()
            .filter(|s| matches!(s.health, HealthStatus::Healthy))
            .count() as f64;
        let degraded = self
            .subsystems
            .values()
            .filter(|s| s.health.is_degraded())
            .count();
        let unhealthy = self
            .subsystems
            .values()
            .filter(|s| s.health.is_unhealthy())
            .count();

        self.health_pct = (healthy / total) * 100.0;

        self.overall_health = if unhealthy > 0 {
            HealthStatus::Unhealthy(format!("{} subsystems unhealthy", unhealthy))
        } else if degraded > 0 {
            HealthStatus::Degraded(format!("{} subsystems degraded", degraded))
        } else {
            HealthStatus::Healthy
        };
    }

    /// Convert to JSON string for dashboard consumption.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".into())
    }

    /// Format a human-readable status summary.
    pub fn summary(&self) -> String {
        let health_icon = match &self.overall_health {
            HealthStatus::Healthy => "🟢",
            HealthStatus::Degraded(_) => "🟡",
            HealthStatus::Unhealthy(_) => "🔴",
        };
        let subsystem_lines: Vec<String> = self
            .subsystems
            .values()
            .map(|s| {
                let icon = match &s.health {
                    HealthStatus::Healthy => "  ✅",
                    HealthStatus::Degraded(_) => "  ⚠️ ",
                    HealthStatus::Unhealthy(_) => "  ❌",
                };
                format!(
                    "{} {} (restarts: {}, latency: {:.1}ms)",
                    icon,
                    s.name,
                    s.restart_count,
                    s.latency_ms.unwrap_or(0.0)
                )
            })
            .collect();

        format!(
            "{} Runtime Health: {:.0}% | CPU: {:.1}% | RAM: {}MB/{}MB | Threads: {} | Uptime: {}s | Restarts: {}\n\nSubsystems:\n{}",
            health_icon,
            self.health_pct,
            self.cpu_usage_pct,
            self.ram_usage_mb,
            self.ram_total_mb,
            self.thread_count,
            self.uptime_secs,
            self.total_restarts,
            subsystem_lines.join("\n"),
        )
    }
}

impl Default for RuntimeSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_creation() {
        let snap = RuntimeSnapshot::new();
        assert_eq!(snap.health_pct, 100.0);
        assert!(snap.subsystems.is_empty());
    }

    #[test]
    fn snapshot_all_healthy() {
        let mut snap = RuntimeSnapshot::new();
        snap.subsystems.insert(
            "audio".into(),
            SubsystemStatus {
                name: "audio".into(),
                health: HealthStatus::Healthy,
                last_heartbeat: Utc::now(),
                restart_count: 0,
                last_error: None,
                latency_ms: Some(1.5),
                uptime_secs: 100,
            },
        );
        snap.subsystems.insert(
            "whisper".into(),
            SubsystemStatus {
                name: "whisper".into(),
                health: HealthStatus::Healthy,
                last_heartbeat: Utc::now(),
                restart_count: 0,
                last_error: None,
                latency_ms: Some(150.0),
                uptime_secs: 100,
            },
        );
        snap.recalculate_overall_health();
        assert!(snap.health_pct > 99.0);
        assert!(snap.overall_health.is_healthy());
    }

    #[test]
    fn snapshot_one_unhealthy() {
        let mut snap = RuntimeSnapshot::new();
        snap.subsystems.insert(
            "a".into(),
            SubsystemStatus {
                name: "a".into(),
                health: HealthStatus::Healthy,
                last_heartbeat: Utc::now(),
                restart_count: 0,
                last_error: None,
                latency_ms: None,
                uptime_secs: 0,
            },
        );
        snap.subsystems.insert(
            "b".into(),
            SubsystemStatus {
                name: "b".into(),
                health: HealthStatus::Unhealthy("crashed".into()),
                last_heartbeat: Utc::now(),
                restart_count: 3,
                last_error: Some("crashed".into()),
                latency_ms: None,
                uptime_secs: 0,
            },
        );
        snap.recalculate_overall_health();
        assert!(snap.health_pct < 100.0);
        assert!(snap.overall_health.is_unhealthy());
    }

    #[test]
    fn snapshot_json_roundtrip() {
        let snap = RuntimeSnapshot::new();
        let json = snap.to_json();
        let deserialized: RuntimeSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.health_pct, 100.0);
    }

    #[test]
    fn snapshot_summary_not_empty() {
        let snap = RuntimeSnapshot::new();
        let summary = snap.summary();
        assert!(summary.contains("Runtime Health"));
    }
}
