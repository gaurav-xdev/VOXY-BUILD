use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::Result;
use crate::report::{ComponentType, HealthReport};
use crate::state::StateTracker;
use voxy_shared::HealthStatus;

pub type HealthCheckFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = HealthReport> + Send + Sync>> + Send + Sync>;

pub struct HealthMonitor {
    checks: Arc<RwLock<HashMap<String, HealthCheckFn>>>,
    interval_ms: u64,
    state_tracker: Arc<RwLock<Option<StateTracker>>>,
}

impl HealthMonitor {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            interval_ms,
            state_tracker: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn register_check(&self, name: &str, check: HealthCheckFn) {
        let mut checks = self.checks.write().await;
        checks.insert(name.to_string(), check);
    }

    pub async fn check_all(&self) -> HashMap<String, HealthReport> {
        let checks = self.checks.read().await;
        let mut results = HashMap::new();

        for (name, check) in checks.iter() {
            let report = check().await;
            results.insert(name.clone(), report);
        }

        results
    }

    pub async fn check(&self, name: &str) -> Result<HealthReport> {
        let checks = self.checks.read().await;
        let check = checks
            .get(name)
            .ok_or_else(|| crate::HealthError::NotFound(name.to_string()))?;
        Ok(check().await)
    }

    pub fn interval_ms(&self) -> u64 {
        self.interval_ms
    }

    pub async fn set_state_tracker(&self, tracker: StateTracker) {
        *self.state_tracker.write().await = Some(tracker);
    }

    pub async fn add_memory_check(&self, name: &str) {
        let check_name = name.to_string();
        self.register_check(
            name,
            Box::new(move || {
                let n = check_name.clone();
                Box::pin(async move {
                    let mut sys = sysinfo::System::new();
                    sys.refresh_memory();

                    let total_mb = sys.total_memory() / (1024 * 1024);
                    let used_mb = sys.used_memory() / (1024 * 1024);
                    let usage_pct = if total_mb > 0 {
                        (used_mb as f64 / total_mb as f64) * 100.0
                    } else {
                        0.0
                    };

                    let status = if usage_pct > 90.0 {
                        HealthStatus::Unhealthy(format!(
                            "Memory usage critical: {:.1}% ({}MB / {}MB)",
                            usage_pct, used_mb, total_mb
                        ))
                    } else if usage_pct > 75.0 {
                        HealthStatus::Degraded(format!(
                            "Memory usage high: {:.1}% ({}MB / {}MB)",
                            usage_pct, used_mb, total_mb
                        ))
                    } else {
                        HealthStatus::Healthy
                    };

                    HealthReport::new(n, status)
                        .with_component_type(ComponentType::Memory)
                        .with_details(format!(
                            "{:.1}% used ({}MB / {}MB)",
                            usage_pct, used_mb, total_mb
                        ))
                })
            }),
        )
        .await;
    }

    pub async fn add_cpu_check(&self, name: &str) {
        let check_name = name.to_string();
        self.register_check(
            name,
            Box::new(move || {
                let n = check_name.clone();
                Box::pin(async move {
                    let mut sys = sysinfo::System::new();
                    sys.refresh_cpu_all();
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    sys.refresh_cpu_all();

                    let cpu_count = sys.cpus().len();
                    let global_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>()
                        / cpu_count.max(1) as f32;

                    let status = if global_usage > 90.0 {
                        HealthStatus::Unhealthy(format!("CPU usage critical: {:.1}%", global_usage))
                    } else if global_usage > 75.0 {
                        HealthStatus::Degraded(format!("CPU usage high: {:.1}%", global_usage))
                    } else {
                        HealthStatus::Healthy
                    };

                    HealthReport::new(n, status)
                        .with_component_type(ComponentType::Cpu)
                        .with_details(format!(
                            "{:.1}% global usage across {} cores",
                            global_usage, cpu_count
                        ))
                })
            }),
        )
        .await;
    }

    pub async fn add_event_bus_check(&self, name: &str, event_bus: Arc<crate::EventBus>) {
        let check_name = name.to_string();
        self.register_check(
            name,
            Box::new(move || {
                let n = check_name.clone();
                let bus = event_bus.clone();
                Box::pin(async move {
                    let topic_count = bus.topic_count().await;

                    let status = if topic_count == 0 {
                        HealthStatus::Degraded("No topics registered on event bus".into())
                    } else {
                        HealthStatus::Healthy
                    };

                    HealthReport::new(n, status)
                        .with_component_type(ComponentType::EventBus)
                        .with_details(format!("{} topics", topic_count))
                })
            }),
        )
        .await;
    }

    pub async fn add_ipc_check<F, Fut>(&self, name: &str, health_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HealthReport> + Send + Sync + 'static,
    {
        let check_name = name.to_string();
        self.register_check(
            name,
            Box::new(move || {
                let n = check_name.clone();
                let fut = health_fn();
                Box::pin(async move {
                    let mut report = fut.await;
                    report.name = n;
                    report.component_type = ComponentType::Ipc;
                    report
                })
            }),
        )
        .await;
    }

    pub async fn add_database_check<F, Fut>(&self, name: &str, health_fn: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = HealthReport> + Send + Sync + 'static,
    {
        let check_name = name.to_string();
        self.register_check(
            name,
            Box::new(move || {
                let n = check_name.clone();
                let fut = health_fn();
                Box::pin(async move {
                    let mut report = fut.await;
                    report.name = n;
                    report.component_type = ComponentType::Database;
                    report
                })
            }),
        )
        .await;
    }

    pub async fn checks(&self) -> Vec<String> {
        self.checks.read().await.keys().cloned().collect()
    }

    pub async fn remove_check(&self, name: &str) {
        self.checks.write().await.remove(name);
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new(30000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn monitor_creation() {
        let monitor = HealthMonitor::new(5000);
        assert_eq!(monitor.interval_ms(), 5000);
    }

    #[tokio::test]
    async fn check_all_empty() {
        let monitor = HealthMonitor::new(5000);
        let results = monitor.check_all().await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn register_and_check() {
        let monitor = HealthMonitor::new(5000);
        monitor
            .register_check(
                "test",
                Box::new(|| {
                    Box::pin(async {
                        HealthReport::new("test", HealthStatus::Healthy).with_latency(0.1)
                    })
                }),
            )
            .await;
        let report = monitor.check("test").await.unwrap();
        assert!(report.is_healthy());
    }

    #[tokio::test]
    async fn check_not_found() {
        let monitor = HealthMonitor::new(5000);
        let result = monitor.check("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn set_state_tracker() {
        let monitor = HealthMonitor::new(5000);
        let tracker = StateTracker::new();
        tracker.register("comp").await;
        monitor.set_state_tracker(tracker).await;
    }

    #[tokio::test]
    async fn remove_check() {
        let monitor = HealthMonitor::new(5000);
        monitor
            .register_check(
                "temp",
                Box::new(|| Box::pin(async { HealthReport::new("temp", HealthStatus::Healthy) })),
            )
            .await;
        assert!(!monitor.checks().await.is_empty());
        monitor.remove_check("temp").await;
        assert!(monitor.check("temp").await.is_err());
    }
}
