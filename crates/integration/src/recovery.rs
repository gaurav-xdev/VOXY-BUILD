//! Subsystem Recovery — crash isolation and auto-restart.
//!
//! If one subsystem crashes, restart only that subsystem.
//! Never crash the entire assistant.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub max_restarts: u32,
    pub restart_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_backoff_ms: u64,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_reset_ms: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_backoff_ms: 30000,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_ms: 60000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubsystemState {
    Running,
    Failed { reason: String },
    Restarting { attempt: u32 },
    CircuitOpen { opened_at: DateTime<Utc> },
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub name: String,
    pub state: SubsystemState,
    pub restart_count: u32,
    pub last_restart: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub consecutive_failures: u32,
}

// ============================================================================
// Subsystem Recovery
// ============================================================================

/// Manages crash isolation and auto-restart for subsystems.
pub struct SubsystemRecovery {
    config: RecoveryConfig,
    subsystems: RwLock<HashMap<String, SubsystemHealth>>,
}

impl SubsystemRecovery {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            subsystems: RwLock::new(HashMap::new()),
        }
    }

    pub fn default_config() -> Self {
        Self::new(RecoveryConfig::default())
    }

    /// Register a subsystem for monitoring.
    pub fn register(&self, name: impl Into<String>) {
        let name = name.into();
        self.subsystems.write().insert(
            name.clone(),
            SubsystemHealth {
                name,
                state: SubsystemState::Stopped,
                restart_count: 0,
                last_restart: None,
                last_error: None,
                consecutive_failures: 0,
            },
        );
    }

    /// Mark a subsystem as running.
    pub fn mark_running(&self, name: &str) {
        let mut subs = self.subsystems.write();
        if let Some(sub) = subs.get_mut(name) {
            sub.state = SubsystemState::Running;
            sub.consecutive_failures = 0;
        }
    }

    /// Report a failure for a subsystem.
    pub fn report_failure(&self, name: &str, reason: &str) -> RecoveryAction {
        let mut subs = self.subsystems.write();
        let sub = match subs.get_mut(name) {
            Some(s) => s,
            None => return RecoveryAction::None,
        };

        sub.last_error = Some(reason.to_string());
        sub.consecutive_failures += 1;

        // Check circuit breaker
        if sub.consecutive_failures >= self.config.circuit_breaker_threshold {
            sub.state = SubsystemState::CircuitOpen {
                opened_at: Utc::now(),
            };
            return RecoveryAction::CircuitOpen {
                subsystem: name.to_string(),
            };
        }

        // Check restart limit
        if sub.restart_count >= self.config.max_restarts {
            sub.state = SubsystemState::Failed {
                reason: reason.to_string(),
            };
            return RecoveryAction::GiveUp {
                subsystem: name.to_string(),
                reason: format!("Exceeded max restarts ({})", self.config.max_restarts),
            };
        }

        // Schedule restart
        sub.restart_count += 1;
        sub.last_restart = Some(Utc::now());
        sub.state = SubsystemState::Restarting {
            attempt: sub.restart_count,
        };

        let delay = self.calculate_backoff(sub.restart_count);
        RecoveryAction::Restart {
            subsystem: name.to_string(),
            delay_ms: delay,
            attempt: sub.restart_count,
        }
    }

    /// Check if a subsystem is in circuit-open state and should be reset.
    pub fn check_circuit_reset(&self, name: &str) -> bool {
        let mut subs = self.subsystems.write();
        if let Some(sub) = subs.get_mut(name) {
            if let SubsystemState::CircuitOpen { opened_at } = sub.state {
                let elapsed = (Utc::now() - opened_at).num_milliseconds() as u64;
                if elapsed >= self.config.circuit_breaker_reset_ms {
                    sub.state = SubsystemState::Stopped;
                    sub.consecutive_failures = 0;
                    return true;
                }
            }
        }
        false
    }

    /// Get health of a subsystem.
    pub fn health(&self, name: &str) -> Option<SubsystemHealth> {
        self.subsystems.read().get(name).cloned()
    }

    /// Get health of all subsystems.
    pub fn all_health(&self) -> HashMap<String, SubsystemHealth> {
        self.subsystems.read().clone()
    }

    /// Get subsystems that need restart.
    pub fn needs_restart(&self) -> Vec<String> {
        self.subsystems
            .read()
            .iter()
            .filter(|(_, sub)| matches!(sub.state, SubsystemState::Restarting { .. }))
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get failed subsystems.
    pub fn failed_subsystems(&self) -> Vec<String> {
        self.subsystems
            .read()
            .iter()
            .filter(|(_, sub)| {
                matches!(
                    sub.state,
                    SubsystemState::Failed { .. } | SubsystemState::CircuitOpen { .. }
                )
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Reset a subsystem's state.
    pub fn reset(&self, name: &str) {
        let mut subs = self.subsystems.write();
        if let Some(sub) = subs.get_mut(name) {
            sub.state = SubsystemState::Stopped;
            sub.restart_count = 0;
            sub.consecutive_failures = 0;
            sub.last_error = None;
        }
    }

    /// Get total restart count across all subsystems.
    pub fn total_restarts(&self) -> u32 {
        self.subsystems
            .read()
            .values()
            .map(|s| s.restart_count)
            .sum()
    }

    fn calculate_backoff(&self, attempt: u32) -> u64 {
        let delay = self.config.restart_delay_ms as f64
            * self.config.backoff_multiplier.powi(attempt as i32 - 1);
        (delay as u64).min(self.config.max_backoff_ms)
    }
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    None,
    Restart {
        subsystem: String,
        delay_ms: u64,
        attempt: u32,
    },
    CircuitOpen {
        subsystem: String,
    },
    GiveUp {
        subsystem: String,
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_creation() {
        let r = SubsystemRecovery::default_config();
        assert_eq!(r.total_restarts(), 0);
    }

    #[test]
    fn register_and_mark_running() {
        let r = SubsystemRecovery::default_config();
        r.register("voice");
        r.mark_running("voice");
        let h = r.health("voice").unwrap();
        assert!(matches!(h.state, SubsystemState::Running));
    }

    #[test]
    fn single_failure_triggers_restart() {
        let r = SubsystemRecovery::default_config();
        r.register("voice");
        r.mark_running("voice");

        let action = r.report_failure("voice", "timeout");
        assert!(matches!(action, RecoveryAction::Restart { .. }));
    }

    #[test]
    fn max_restarts_triggers_giveup() {
        let config = RecoveryConfig {
            max_restarts: 2,
            ..Default::default()
        };
        let r = SubsystemRecovery::new(config);
        r.register("svc");
        r.mark_running("svc");

        r.report_failure("svc", "err1");
        r.mark_running("svc");
        r.report_failure("svc", "err2");
        r.mark_running("svc");
        let action = r.report_failure("svc", "err3");

        assert!(matches!(action, RecoveryAction::GiveUp { .. }));
    }

    #[test]
    fn circuit_breaker() {
        let config = RecoveryConfig {
            max_restarts: 100,
            circuit_breaker_threshold: 3,
            ..Default::default()
        };
        let r = SubsystemRecovery::new(config);
        r.register("svc");
        r.mark_running("svc");

        r.report_failure("svc", "e1");
        r.report_failure("svc", "e2");
        let action = r.report_failure("svc", "e3");

        assert!(matches!(action, RecoveryAction::CircuitOpen { .. }));
    }

    #[test]
    fn backoff_calculation() {
        let config = RecoveryConfig {
            restart_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_backoff_ms: 10000,
            ..Default::default()
        };
        let r = SubsystemRecovery::new(config);
        r.register("svc");
        r.mark_running("svc");

        // First failure: 1000ms
        let action = r.report_failure("svc", "e1");
        if let RecoveryAction::Restart { delay_ms, .. } = action {
            assert_eq!(delay_ms, 1000);
        }

        r.mark_running("svc");
        // Second failure: 2000ms
        let action = r.report_failure("svc", "e2");
        if let RecoveryAction::Restart { delay_ms, .. } = action {
            assert_eq!(delay_ms, 2000);
        }
    }

    #[test]
    fn needs_restart() {
        let r = SubsystemRecovery::default_config();
        r.register("a");
        r.register("b");
        r.mark_running("a");
        r.mark_running("b");

        r.report_failure("a", "crash");
        let needs = r.needs_restart();
        assert_eq!(needs.len(), 1);
        assert_eq!(needs[0], "a");
    }

    #[test]
    fn failed_subsystems() {
        let config = RecoveryConfig {
            max_restarts: 1,
            circuit_breaker_threshold: 100,
            ..Default::default()
        };
        let r = SubsystemRecovery::new(config);
        r.register("a");
        r.mark_running("a");
        r.report_failure("a", "e1");
        r.mark_running("a");
        r.report_failure("a", "e2"); // Exceeds max_restarts

        let failed = r.failed_subsystems();
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn reset_subsystem() {
        let r = SubsystemRecovery::default_config();
        r.register("svc");
        r.mark_running("svc");
        r.report_failure("svc", "err");
        r.reset("svc");

        let h = r.health("svc").unwrap();
        assert!(matches!(h.state, SubsystemState::Stopped));
        assert_eq!(h.restart_count, 0);
    }

    #[test]
    fn total_restarts_count() {
        let r = SubsystemRecovery::default_config();
        r.register("a");
        r.register("b");
        r.mark_running("a");
        r.mark_running("b");
        r.report_failure("a", "e1");
        r.report_failure("b", "e2");
        assert_eq!(r.total_restarts(), 2);
    }

    #[test]
    fn unregister_existing() {
        let r = SubsystemRecovery::default_config();
        r.register("svc");
        r.mark_running("svc");
        assert!(r.health("svc").is_some());
    }

    #[test]
    fn failure_on_unregistered_is_noop() {
        let r = SubsystemRecovery::default_config();
        let action = r.report_failure("nonexistent", "err");
        assert!(matches!(action, RecoveryAction::None));
    }
}
