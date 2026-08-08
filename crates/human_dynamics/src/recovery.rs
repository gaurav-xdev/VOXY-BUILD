use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::RecoveryConfig;
use crate::types::RecoveryAction;

/// Recovery engine — handles mistakes gracefully.
pub struct RecoveryEngine {
    config: RecoveryConfig,
    recovery_count: usize,
    last_recovery: Option<Instant>,
    history: Vec<RecoveryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecord {
    error: String,
    recovery: String,
    timestamp: DateTime<Utc>,
    resolved: bool,
}

impl RecoveryEngine {
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            recovery_count: 0,
            last_recovery: None,
            history: Vec::new(),
        }
    }

    /// Generate a recovery action for a mistake.
    pub fn recover(
        &mut self,
        error_description: &str,
        correction: Option<&str>,
        now: Instant,
    ) -> Option<RecoveryAction> {
        if self.recovery_count >= self.config.max_recovery_attempts {
            return None;
        }

        if let Some(last) = self.last_recovery {
            if now.duration_since(last) < self.config.cooldown {
                return None;
            }
        }

        let recovery = RecoveryAction {
            acknowledge: self.config.acknowledgment_required,
            correct: self.config.auto_correct && correction.is_some(),
            description: correction
                .map(|c| format!("Acknowledged: {}. Correcting: {}", error_description, c))
                .unwrap_or_else(|| format!("Acknowledged: {}. Investigating.", error_description)),
        };

        self.history.push(RecoveryRecord {
            error: error_description.to_string(),
            recovery: recovery.description.clone(),
            timestamp: Utc::now(),
            resolved: true,
        });

        self.recovery_count += 1;
        self.last_recovery = Some(now);

        Some(recovery)
    }

    pub fn recovery_count(&self) -> usize {
        self.recovery_count
    }

    pub fn history(&self) -> &[RecoveryRecord] {
        &self.history
    }

    pub fn reset(&mut self) {
        self.recovery_count = 0;
        self.last_recovery = None;
    }
}

impl Default for RecoveryEngine {
    fn default() -> Self {
        Self::new(RecoveryConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_generated() {
        let mut engine = RecoveryEngine::new(RecoveryConfig::default());
        let recovery = engine.recover("Wrong answer", Some("Corrected"), Instant::now());
        assert!(recovery.is_some());
        assert!(recovery.unwrap().acknowledge);
    }

    #[test]
    fn test_recovery_max_attempts() {
        let mut engine = RecoveryEngine::new(RecoveryConfig::default());
        for _ in 0..3 {
            engine.recover("Error", None, Instant::now());
        }
        let recovery = engine.recover("Error", None, Instant::now());
        assert!(recovery.is_none());
    }

    #[test]
    fn test_recovery_cooldown() {
        let mut engine = RecoveryEngine::new(RecoveryConfig::default());
        let now = Instant::now();
        engine.recover("Error", None, now);
        let recovery = engine.recover("Error", None, now);
        assert!(recovery.is_none());
    }

    #[test]
    fn test_recovery_reset() {
        let mut engine = RecoveryEngine::new(RecoveryConfig::default());
        for _ in 0..3 {
            engine.recover("Error", None, Instant::now());
        }
        engine.reset();
        let recovery = engine.recover("Error", None, Instant::now());
        assert!(recovery.is_some());
    }
}
