use std::collections::VecDeque;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::config::TrustConfig;
use crate::types::{TrustEvent, TrustEventKind};

/// Trust score breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBreakdown {
    pub total: f64,
    pub successes: f64,
    pub failures: f64,
    pub permissions: f64,
    pub consistency: f64,
    pub recency: f64,
}

/// Tracks and calculates user trust.
pub struct TrustEngine {
    config: TrustConfig,
    score: f64,
    events: VecDeque<TrustEvent>,
    success_count: usize,
    failure_count: usize,
    correction_count: usize,
    permission_count: usize,
    override_count: usize,
}

impl TrustEngine {
    pub fn new(config: TrustConfig) -> Self {
        Self {
            score: config.initial_score,
            events: VecDeque::new(),
            success_count: 0,
            failure_count: 0,
            correction_count: 0,
            permission_count: 0,
            override_count: 0,
            config,
        }
    }

    /// Process a trust event.
    pub fn process_event(&mut self, event: TrustEvent) {
        let impact = match event.kind {
            TrustEventKind::SuccessfulMission => {
                self.success_count += 1;
                self.config.growth_per_success
            }
            TrustEventKind::TaskCompleted => {
                self.success_count += 1;
                self.config.growth_per_success * 0.5
            }
            TrustEventKind::TaskFailed => {
                self.failure_count += 1;
                -self.config.penalty_per_failure
            }
            TrustEventKind::Correction => {
                self.correction_count += 1;
                -self.config.penalty_per_correction
            }
            TrustEventKind::FalseAlarm => -self.config.penalty_per_false_alarm,
            TrustEventKind::PermissionGranted => {
                self.permission_count += 1;
                self.config.bonus_per_permission
            }
            TrustEventKind::PermissionDenied => -self.config.bonus_per_permission,
            TrustEventKind::ManualOverride => {
                self.override_count += 1;
                -self.config.penalty_per_correction * 0.5
            }
            TrustEventKind::UserReturned => 0.01,
            TrustEventKind::UserAbsent => -self.config.decay_per_absence,
        };

        self.score = (self.score + impact).clamp(self.config.min_score, self.config.max_score);

        self.events.push_back(event);
        while self.events.len() > self.config.event_history_limit {
            self.events.pop_front();
        }
    }

    /// Calculate trust breakdown.
    pub fn breakdown(&self) -> TrustBreakdown {
        let total_events = self.events.len() as f64;
        let successes = self.success_count as f64 / total_events.max(1.0);
        let failures = self.failure_count as f64 / total_events.max(1.0);
        let permissions = self.permission_count as f64 / total_events.max(1.0);

        let consistency = if total_events > 0.0 {
            1.0 - (self.failure_count as f64 / total_events).min(1.0)
        } else {
            0.5
        };

        let recency = self
            .events
            .back()
            .map(|e| {
                let age = (Utc::now() - e.timestamp).num_seconds() as f64;
                (1.0 - (age / 86400.0)).max(0.0)
            })
            .unwrap_or(0.5);

        TrustBreakdown {
            total: self.score,
            successes,
            failures,
            permissions,
            consistency,
            recency,
        }
    }

    pub fn score(&self) -> f64 {
        self.score
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Derive autonomy level from trust.
    pub fn autonomy_level(&self) -> f64 {
        (self.score * 1.2).clamp(0.0, 1.0)
    }

    /// Derive confirmation level from trust (inverse — lower trust = more confirmation).
    pub fn confirmation_level(&self) -> f64 {
        (1.0 - self.score * 0.8).clamp(0.0, 1.0)
    }

    /// Derive initiative level from trust.
    pub fn initiative_level(&self) -> f64 {
        (self.score * 0.9).clamp(0.0, 1.0)
    }
}

impl Default for TrustEngine {
    fn default() -> Self {
        Self::new(TrustConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_initial() {
        let engine = TrustEngine::new(TrustConfig::default());
        assert_eq!(engine.score(), 0.5);
    }

    #[test]
    fn test_trust_grows() {
        let mut engine = TrustEngine::new(TrustConfig::default());
        engine.process_event(TrustEvent {
            kind: TrustEventKind::SuccessfulMission,
            impact: 0.0,
            timestamp: Utc::now(),
            context: "test".to_string(),
        });
        assert!(engine.score() > 0.5);
    }

    #[test]
    fn test_trust_decays() {
        let mut engine = TrustEngine::new(TrustConfig::default());
        let initial = engine.score();
        engine.process_event(TrustEvent {
            kind: TrustEventKind::TaskFailed,
            impact: 0.0,
            timestamp: Utc::now(),
            context: "test".to_string(),
        });
        assert!(engine.score() < initial);
    }

    #[test]
    fn test_autonomy_derived() {
        let mut engine = TrustEngine::new(TrustConfig::default());
        for _ in 0..20 {
            engine.process_event(TrustEvent {
                kind: TrustEventKind::SuccessfulMission,
                impact: 0.0,
                timestamp: Utc::now(),
                context: "test".to_string(),
            });
        }
        assert!(engine.autonomy_level() > 0.5);
    }

    #[test]
    fn test_breakdown() {
        let mut engine = TrustEngine::new(TrustConfig::default());
        engine.process_event(TrustEvent {
            kind: TrustEventKind::SuccessfulMission,
            impact: 0.0,
            timestamp: Utc::now(),
            context: "test".to_string(),
        });
        let breakdown = engine.breakdown();
        assert!(breakdown.total > 0.0);
    }
}
