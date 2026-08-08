use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::Notify;

/// Health status of a pipeline stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failed,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Health state for a single stage.
#[derive(Debug, Clone)]
pub struct StageHealth {
    pub name: String,
    pub status: HealthStatus,
    pub last_heartbeat: Instant,
    pub consecutive_failures: u32,
    pub total_failures: u64,
    pub recovery_attempts: u32,
}

impl StageHealth {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: HealthStatus::Healthy,
            last_heartbeat: Instant::now(),
            consecutive_failures: 0,
            total_failures: 0,
            recovery_attempts: 0,
        }
    }
}

/// Circuit breaker for a pipeline stage.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_attempts: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_attempts: 3,
        }
    }
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            half_open_max_attempts: 3,
        }
    }

    pub fn should_attempt_recovery(&self, health: &StageHealth) -> bool {
        if health.status == HealthStatus::Healthy {
            return false;
        }
        if health.consecutive_failures < self.failure_threshold {
            return true;
        }
        health.last_heartbeat.elapsed() >= self.recovery_timeout
            && health.recovery_attempts < self.half_open_max_attempts
    }
}

/// Health watchdog that monitors all pipeline stages.
pub struct HealthWatchdog {
    stages: RwLock<Vec<StageHealth>>,
    circuit_breakers: RwLock<Vec<CircuitBreaker>>,
    check_interval: Duration,
    max_stages: usize,
    restart_signal: Arc<Notify>,
    is_monitoring: Arc<AtomicBool>,
    check_count: Arc<AtomicU64>,
}

impl HealthWatchdog {
    pub fn new() -> Self {
        Self {
            stages: RwLock::new(Vec::new()),
            circuit_breakers: RwLock::new(Vec::new()),
            check_interval: Duration::from_secs(5),
            max_stages: 16,
            restart_signal: Arc::new(Notify::new()),
            is_monitoring: Arc::new(AtomicBool::new(false)),
            check_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn with_check_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Register a stage to monitor.
    pub fn register_stage(&self, name: &str) {
        let mut stages = self.stages.write();
        if stages.len() >= self.max_stages {
            return;
        }
        if !stages.iter().any(|s| s.name == name) {
            stages.push(StageHealth::new(name));
        }
    }

    /// Record a heartbeat for a stage (called when it's working).
    pub fn heartbeat(&self, name: &str) {
        let mut stages = self.stages.write();
        if let Some(stage) = stages.iter_mut().find(|s| s.name == name) {
            stage.last_heartbeat = Instant::now();
            stage.consecutive_failures = 0;
            stage.status = HealthStatus::Healthy;
        }
    }

    /// Record a failure for a stage.
    pub fn record_failure(&self, name: &str) {
        let mut stages = self.stages.write();
        if let Some(stage) = stages.iter_mut().find(|s| s.name == name) {
            stage.consecutive_failures += 1;
            stage.total_failures += 1;
            stage.status = if stage.consecutive_failures >= 5 {
                HealthStatus::Failed
            } else if stage.consecutive_failures >= 2 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };
        }
    }

    /// Start background health monitoring.
    pub fn start_monitoring(&self) -> Arc<Notify> {
        self.is_monitoring.store(true, Ordering::SeqCst);
        self.restart_signal.clone()
    }

    /// Stop monitoring.
    pub fn stop_monitoring(&self) {
        self.is_monitoring.store(false, Ordering::SeqCst);
    }

    /// Check all stages and return any that need recovery.
    pub fn check_stages(&self) -> Vec<String> {
        self.check_count.fetch_add(1, Ordering::Relaxed);

        let stages = self.stages.read();
        let breakers = self.circuit_breakers.read();
        let mut needs_recovery = Vec::new();

        for stage in stages.iter() {
            if stage.status == HealthStatus::Healthy {
                continue;
            }

            // Find matching circuit breaker (by index or use default)
            let breaker = breakers
                .get(
                    stages
                        .iter()
                        .position(|s| s.name == stage.name)
                        .unwrap_or(0),
                )
                .or_else(|| breakers.first())
                .cloned()
                .unwrap_or_default();

            if breaker.should_attempt_recovery(stage) {
                needs_recovery.push(stage.name.clone());
            }
        }

        needs_recovery
    }

    pub fn stage_health(&self, name: &str) -> Option<StageHealth> {
        self.stages.read().iter().find(|s| s.name == name).cloned()
    }

    pub fn all_stages(&self) -> Vec<StageHealth> {
        self.stages.read().clone()
    }

    pub fn is_healthy(&self) -> bool {
        self.stages
            .read()
            .iter()
            .all(|s| s.status == HealthStatus::Healthy)
    }

    pub fn check_count(&self) -> u64 {
        self.check_count.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.stages.write().clear();
        self.circuit_breakers.write().clear();
        self.check_count.store(0, Ordering::Relaxed);
    }
}

impl Default for HealthWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watchdog_creation() {
        let w = HealthWatchdog::new();
        assert!(w.is_healthy());
        assert_eq!(w.check_count(), 0);
    }

    #[test]
    fn watchdog_register_and_heartbeat() {
        let w = HealthWatchdog::new();
        w.register_stage("stt");
        w.heartbeat("stt");
        let health = w.stage_health("stt").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn watchdog_record_failure_degraded() {
        let w = HealthWatchdog::new();
        w.register_stage("tts");
        w.record_failure("tts");
        let health = w.stage_health("tts").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy); // 1 failure is still healthy
        w.record_failure("tts");
        let health = w.stage_health("tts").unwrap();
        assert_eq!(health.status, HealthStatus::Degraded);
    }

    #[test]
    fn watchdog_record_failure_failed() {
        let w = HealthWatchdog::new();
        w.register_stage("llm");
        for _ in 0..5 {
            w.record_failure("llm");
        }
        let health = w.stage_health("llm").unwrap();
        assert_eq!(health.status, HealthStatus::Failed);
        assert_eq!(health.total_failures, 5);
    }

    #[test]
    fn watchdog_heartbeat_resets_failures() {
        let w = HealthWatchdog::new();
        w.register_stage("stt");
        w.record_failure("stt");
        w.record_failure("stt");
        w.heartbeat("stt");
        let health = w.stage_health("stt").unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn watchdog_check_stages() {
        let w = HealthWatchdog::new();
        w.register_stage("stt");
        w.register_stage("tts");
        w.record_failure("tts");
        w.record_failure("tts");
        let needs_recovery = w.check_stages();
        assert!(needs_recovery.contains(&"tts".to_string()));
        assert!(!needs_recovery.contains(&"stt".to_string()));
    }

    #[test]
    fn watchdog_circuit_breaker_default() {
        let cb = CircuitBreaker::default();
        assert_eq!(cb.failure_threshold, 5);
    }

    #[test]
    fn watchdog_circuit_breaker_should_recover() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(0));
        let mut health = StageHealth::new("test");
        health.consecutive_failures = 3;
        health.status = HealthStatus::Failed;
        // Set heartbeat far in the past so elapsed >= recovery_timeout
        health.last_heartbeat = Instant::now() - Duration::from_secs(10);
        assert!(cb.should_attempt_recovery(&health));
    }

    #[test]
    fn watchdog_circuit_breaker_no_recover_if_healthy() {
        let cb = CircuitBreaker::default();
        let health = StageHealth::new("test");
        assert!(!cb.should_attempt_recovery(&health));
    }

    #[test]
    fn watchdog_all_stages() {
        let w = HealthWatchdog::new();
        w.register_stage("a");
        w.register_stage("b");
        let all = w.all_stages();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn watchdog_max_stages() {
        let w = HealthWatchdog::new().with_check_interval(Duration::from_secs(1));
        for i in 0..20 {
            w.register_stage(&format!("stage-{i}"));
        }
        assert_eq!(w.all_stages().len(), 16);
    }

    #[test]
    fn watchdog_reset() {
        let w = HealthWatchdog::new();
        w.register_stage("stt");
        w.record_failure("stt");
        w.reset();
        assert!(w.all_stages().is_empty());
        assert_eq!(w.check_count(), 0);
    }

    #[test]
    fn watchdog_nonexistent_stage() {
        let w = HealthWatchdog::new();
        assert!(w.stage_health("nope").is_none());
    }
}
