use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::RelationshipConfig;
use crate::types::{RelationshipLevel, TrustEvent};

/// Snapshot of relationship state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSnapshot {
    pub level: RelationshipLevel,
    pub score: f64,
    pub total_interactions: usize,
    pub days_known: i64,
    pub consistency: f64,
}

/// Tracks relationship evolution over time.
pub struct RelationshipEngine {
    config: RelationshipConfig,
    score: f64,
    total_interactions: usize,
    first_seen: DateTime<Utc>,
    last_interaction: DateTime<Utc>,
    consistency_window: Vec<bool>,
}

impl RelationshipEngine {
    pub fn new(config: RelationshipConfig) -> Self {
        let now = Utc::now();
        Self {
            config,
            score: 0.1,
            total_interactions: 0,
            first_seen: now,
            last_interaction: now,
            consistency_window: Vec::new(),
        }
    }

    /// Record an interaction and update relationship score.
    pub fn record_interaction(&mut self, positive: bool, now: DateTime<Utc>) {
        self.total_interactions += 1;
        self.last_interaction = now;

        self.consistency_window.push(positive);
        if self.consistency_window.len() > 50 {
            self.consistency_window.remove(0);
        }

        let growth = if positive {
            self.config.growth_rate
        } else {
            -self.config.decay_rate
        };

        let consistency_bonus = self.calculate_consistency() * 0.01;
        self.score = (self.score + growth + consistency_bonus).clamp(0.0, self.config.max_score);
    }

    /// Process trust events that affect relationship.
    pub fn process_trust_events(&mut self, events: &[TrustEvent]) {
        for event in events {
            let impact = match event.kind {
                crate::types::TrustEventKind::SuccessfulMission => 0.02,
                crate::types::TrustEventKind::TaskCompleted => 0.01,
                crate::types::TrustEventKind::Correction => -0.01,
                crate::types::TrustEventKind::FalseAlarm => -0.02,
                crate::types::TrustEventKind::UserReturned => 0.01,
                _ => 0.0,
            };
            self.score = (self.score + impact).clamp(0.0, self.config.max_score);
        }
    }

    /// Get current relationship level.
    pub fn level(&self) -> RelationshipLevel {
        RelationshipLevel::from_score(self.score)
    }

    /// Get relationship snapshot.
    pub fn snapshot(&self, now: DateTime<Utc>) -> RelationshipSnapshot {
        let days = (now - self.first_seen).num_days();
        RelationshipSnapshot {
            level: self.level(),
            score: self.score,
            total_interactions: self.total_interactions,
            days_known: days,
            consistency: self.calculate_consistency(),
        }
    }

    /// Calculate consistency (ratio of positive interactions).
    fn calculate_consistency(&self) -> f64 {
        if self.consistency_window.is_empty() {
            return 0.5;
        }
        let positive = self.consistency_window.iter().filter(|&&p| p).count();
        positive as f64 / self.consistency_window.len() as f64
    }

    pub fn score(&self) -> f64 {
        self.score
    }
}

impl Default for RelationshipEngine {
    fn default() -> Self {
        Self::new(RelationshipConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_starts_professional() {
        let engine = RelationshipEngine::new(RelationshipConfig::default());
        assert_eq!(engine.level(), RelationshipLevel::Professional);
    }

    #[test]
    fn test_relationship_grows() {
        let mut engine = RelationshipEngine::new(RelationshipConfig::default());
        for _ in 0..50 {
            engine.record_interaction(true, Utc::now());
        }
        assert!(engine.score() > 0.5);
    }

    #[test]
    fn test_relationship_decays() {
        let mut engine = RelationshipEngine::new(RelationshipConfig::default());
        for _ in 0..50 {
            engine.record_interaction(true, Utc::now());
        }
        let high_score = engine.score();
        for _ in 0..50 {
            engine.record_interaction(false, Utc::now());
        }
        assert!(engine.score() < high_score);
    }

    #[test]
    fn test_consistency_tracking() {
        let mut engine = RelationshipEngine::new(RelationshipConfig::default());
        for _ in 0..10 {
            engine.record_interaction(true, Utc::now());
        }
        let snap = engine.snapshot(Utc::now());
        assert!(snap.consistency > 0.9);
    }
}
