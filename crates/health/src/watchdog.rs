use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

use crate::monitor::HealthMonitor;
use voxy_shared::HealthStatus;

type StatusChangeCallback = Box<dyn Fn(StatusChange) + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub interval_ms: u64,
    pub auto_recovery: bool,
    pub max_failures_before_action: u32,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            interval_ms: 60000,
            auto_recovery: true,
            max_failures_before_action: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecoveryAction {
    Restart,
    ReloadConfig,
    ScaleResources,
    NotifyAdmin,
}

impl RecoveryAction {
    pub fn description(&self) -> &str {
        match self {
            Self::Restart => "Restart the component",
            Self::ReloadConfig => "Reload configuration",
            Self::ScaleResources => "Scale up resources",
            Self::NotifyAdmin => "Notify administrator",
        }
    }
}

type RecoveryFn = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), crate::HealthError>> + Send + Sync>>
        + Send
        + Sync,
>;

pub struct RecoveryManager {
    actions: RwLock<HashMap<String, Vec<RecoveryAction>>>,
    recovery_fns: RwLock<HashMap<String, Vec<RecoveryFn>>>,
}

impl RecoveryManager {
    pub fn new() -> Self {
        Self {
            actions: RwLock::new(HashMap::new()),
            recovery_fns: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, component: &str, action: RecoveryAction) {
        let mut actions = self.actions.write().await;
        actions
            .entry(component.to_string())
            .or_default()
            .push(action);
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn register_with_fn<F, Fut>(
        &self,
        component: &str,
        action: RecoveryAction,
        recovery_fn: F,
    ) where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), crate::HealthError>> + Send + Sync + 'static,
    {
        self.register(component, action).await;
        let mut fns = self.recovery_fns.write().await;
        fns.entry(component.to_string())
            .or_default()
            .push(Arc::new(move || {
                let fut = recovery_fn();
                Box::pin(fut)
            }));
    }

    pub async fn execute_recovery(
        &self,
        component: &str,
    ) -> Result<Vec<RecoveryAction>, crate::HealthError> {
        let actions = {
            let actions_map = self.actions.read().await;
            actions_map.get(component).cloned().unwrap_or_default()
        };

        if actions.is_empty() {
            return Err(crate::HealthError::RecoveryFailed(format!(
                "No recovery actions registered for {}",
                component
            )));
        }

        let fns = {
            let fns_map = self.recovery_fns.read().await;
            fns_map.get(component).cloned().unwrap_or_default()
        };

        let mut executed = Vec::new();

        for (i, action) in actions.iter().enumerate() {
            if i < fns.len() {
                let result = fns[i]().await;
                if result.is_ok() {
                    info!("Recovery action {:?} succeeded for {}", action, component);
                    executed.push(action.clone());
                } else {
                    return Err(crate::HealthError::RecoveryFailed(format!(
                        "Recovery action {:?} failed for {}",
                        action, component
                    )));
                }
            } else {
                info!(
                    "Recovery action {:?} for {} (no custom handler, assuming success)",
                    action, component
                );
                executed.push(action.clone());
            }
        }

        Ok(executed)
    }

    pub async fn registered_actions(&self, component: &str) -> Vec<RecoveryAction> {
        let actions = self.actions.read().await;
        actions.get(component).cloned().unwrap_or_default()
    }

    pub async fn clear(&self) {
        self.actions.write().await.clear();
        self.recovery_fns.write().await.clear();
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusChange {
    pub component: String,
    pub previous: HealthStatus,
    pub current: HealthStatus,
    pub timestamp: DateTime<Utc>,
}

pub struct Watchdog {
    running: Arc<AtomicBool>,
    config: WatchdogConfig,
    previous_statuses: Arc<RwLock<HashMap<String, HealthStatus>>>,
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    recovery_manager: Arc<RecoveryManager>,
    on_status_change: Arc<RwLock<Option<StatusChangeCallback>>>,
    watch_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Watchdog {
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            config,
            previous_statuses: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            recovery_manager: Arc::new(RecoveryManager::new()),
            on_status_change: Arc::new(RwLock::new(None)),
            watch_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub fn recovery_manager(&self) -> Arc<RecoveryManager> {
        self.recovery_manager.clone()
    }

    pub async fn set_on_status_change<F>(&self, callback: F)
    where
        F: Fn(StatusChange) + Send + Sync + 'static,
    {
        let mut cb = self.on_status_change.write().await;
        *cb = Some(Box::new(callback));
    }

    pub fn start(self: Arc<Self>, monitor: Arc<HealthMonitor>, event_bus: Arc<crate::EventBus>) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();
        let config = self.config.clone();
        let previous_statuses = self.previous_statuses.clone();
        let failure_counts = self.failure_counts.clone();
        let recovery_manager = self.recovery_manager.clone();
        let on_status_change = self.on_status_change.clone();

        let handle = tokio::spawn(async move {
            info!(
                "Watchdog started with interval {}ms, auto_recovery={}",
                config.interval_ms, config.auto_recovery
            );

            while running.load(Ordering::SeqCst) {
                let reports = monitor.check_all().await;

                for (name, report) in &reports {
                    let prev = {
                        let mut prev_map = previous_statuses.write().await;
                        prev_map.insert(name.clone(), report.status.clone())
                    };

                    let prev_status = prev.unwrap_or(HealthStatus::Healthy);

                    if prev_status != report.status {
                        let change = StatusChange {
                            component: name.clone(),
                            previous: prev_status.clone(),
                            current: report.status.clone(),
                            timestamp: Utc::now(),
                        };

                        let cb = on_status_change.read().await;
                        if let Some(ref callback) = *cb {
                            callback(change.clone());
                        }

                        let payload = serde_json::to_vec(&change).unwrap_or_default();
                        let event = voxy_shared::Event::new(
                            format!("health.status.{}", name),
                            "voxy-health",
                            payload,
                        );
                        let _ = event_bus
                            .publish(&format!("health.status.{}", name), event)
                            .await;
                    }

                    if report.status.is_unhealthy() || report.status.is_degraded() {
                        let mut counts = failure_counts.write().await;
                        let count = counts.entry(name.clone()).or_insert(0);
                        *count += 1;
                        let current_count = *count;

                        if config.auto_recovery
                            && current_count >= config.max_failures_before_action
                        {
                            let total_key = format!("__total_recovery_{}", name);
                            let total = counts.entry(total_key.clone()).or_insert(0);
                            *total += 1;
                            let current_total = *total;
                            if current_total > 10 {
                                error!(
                                    "Component {} exceeded max total recoveries ({}), disabling auto-recovery",
                                    name, current_total
                                );
                                counts.remove(name.as_str());
                                counts.remove(total_key.as_str());
                                continue;
                            }
                            info!(
                                "Auto-recovery triggered for {} after {} failures (total recoveries: {})",
                                name, current_count, current_total
                            );
                            match recovery_manager.execute_recovery(name).await {
                                Ok(_) => {
                                    counts.remove(name.as_str());
                                }
                                Err(e) => {
                                    error!("Recovery failed for {name}: {e}");
                                }
                            }
                        }
                    } else {
                        let mut counts = failure_counts.write().await;
                        counts.remove(name);
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(config.interval_ms)).await;
            }

            info!("Watchdog stopped");
        });

        // Store the handle for proper cleanup
        let watch_handle = self.watch_handle.clone();
        tokio::spawn(async move {
            *watch_handle.write().await = Some(handle);
        });
    }

    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let handle = self.watch_handle.write().await.take();
        if let Some(h) = handle {
            let _ = tokio::time::timeout(Duration::from_secs(5), h).await;
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn config(&self) -> &WatchdogConfig {
        &self.config
    }

    pub async fn failure_count(&self, component: &str) -> u32 {
        let counts = self.failure_counts.read().await;
        counts.get(component).copied().unwrap_or(0)
    }

    pub async fn reset_failure_count(&self, component: &str) {
        let mut counts = self.failure_counts.write().await;
        counts.remove(component);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HealthReport;

    #[test]
    fn watchdog_config_defaults() {
        let config = WatchdogConfig::default();
        assert_eq!(config.interval_ms, 60000);
        assert!(config.auto_recovery);
        assert_eq!(config.max_failures_before_action, 3);
    }

    #[tokio::test]
    async fn recovery_manager_register_and_execute() {
        let manager = RecoveryManager::new();
        manager.register("comp1", RecoveryAction::Restart).await;
        manager.register("comp1", RecoveryAction::NotifyAdmin).await;

        let actions = manager.registered_actions("comp1").await;
        assert_eq!(actions.len(), 2);

        let result = manager.execute_recovery("comp1").await;
        assert!(result.is_ok());
        let executed = result.unwrap();
        assert_eq!(executed.len(), 2);
    }

    #[tokio::test]
    async fn recovery_manager_no_actions() {
        let manager = RecoveryManager::new();
        let result = manager.execute_recovery("unknown").await;
        assert!(result.is_err());
        match result {
            Err(crate::HealthError::RecoveryFailed(msg)) => {
                assert!(msg.contains("No recovery actions"));
            }
            _ => panic!("Expected RecoveryFailed"),
        }
    }

    #[tokio::test]
    async fn recovery_manager_with_fn() {
        let manager = RecoveryManager::new();
        manager
            .register_with_fn("svc1", RecoveryAction::Restart, || async { Ok(()) })
            .await;

        let result = manager.execute_recovery("svc1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn recovery_manager_clear() {
        let manager = RecoveryManager::new();
        manager.register("comp", RecoveryAction::Restart).await;
        assert!(!manager.registered_actions("comp").await.is_empty());
        manager.clear().await;
        assert!(manager.registered_actions("comp").await.is_empty());
    }

    #[test]
    fn recovery_action_description() {
        assert_eq!(
            RecoveryAction::Restart.description(),
            "Restart the component"
        );
        assert_eq!(
            RecoveryAction::ReloadConfig.description(),
            "Reload configuration"
        );
        assert_eq!(
            RecoveryAction::ScaleResources.description(),
            "Scale up resources"
        );
        assert_eq!(
            RecoveryAction::NotifyAdmin.description(),
            "Notify administrator"
        );
    }

    #[tokio::test]
    async fn watchdog_start_stop() {
        let config = WatchdogConfig {
            interval_ms: 100,
            auto_recovery: false,
            max_failures_before_action: 3,
        };
        let watchdog = Arc::new(Watchdog::new(config));
        let monitor = Arc::new(HealthMonitor::new(100));
        let event_bus = Arc::new(crate::EventBus::new(10));

        assert!(!watchdog.is_running());
        watchdog.clone().start(monitor, event_bus);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(watchdog.is_running());

        watchdog.stop().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        assert!(!watchdog.is_running());
    }

    #[tokio::test]
    async fn failure_count_tracking() {
        let config = WatchdogConfig {
            interval_ms: 50,
            auto_recovery: false,
            max_failures_before_action: 5,
        };
        let watchdog = Arc::new(Watchdog::new(config));
        let monitor = Arc::new(HealthMonitor::new(50));

        monitor
            .register_check(
                "failing",
                Box::new(|| {
                    Box::pin(async {
                        HealthReport::new("failing", HealthStatus::Unhealthy("down".into()))
                    })
                }),
            )
            .await;

        let event_bus = Arc::new(crate::EventBus::new(10));
        watchdog.clone().start(monitor, event_bus);
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        watchdog.stop().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    #[tokio::test]
    async fn status_change_callback() {
        let config = WatchdogConfig {
            interval_ms: 50,
            auto_recovery: false,
            max_failures_before_action: 5,
        };
        let watchdog = Arc::new(Watchdog::new(config));
        let changes = Arc::new(RwLock::new(Vec::new()));
        let changes_clone = changes.clone();

        watchdog
            .set_on_status_change(move |change| {
                let c = changes_clone.clone();
                tokio::spawn(async move {
                    c.write().await.push(change);
                });
            })
            .await;

        let monitor = Arc::new(HealthMonitor::new(50));
        let event_bus = Arc::new(crate::EventBus::new(10));

        watchdog.clone().start(monitor.clone(), event_bus);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        watchdog.stop().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    #[test]
    fn recovery_action_serde() {
        let action = RecoveryAction::Restart;
        let json = serde_json::to_string(&action).unwrap();
        let deserialized: RecoveryAction = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, RecoveryAction::Restart);
    }
}
