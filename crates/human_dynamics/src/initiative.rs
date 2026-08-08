use std::time::{Duration, Instant};

use crate::config::InitiativeConfig;
use crate::types::{BehaviorState, InitiativeDecision, RelationshipLevel};

/// Initiative engine — decides when VOXY may speak first.
pub struct InitiativeEngine {
    config: InitiativeConfig,
    last_initiative: Option<Instant>,
    initiative_count_hour: usize,
    hour_start: Instant,
}

impl InitiativeEngine {
    pub fn new(config: InitiativeConfig) -> Self {
        Self {
            config,
            last_initiative: None,
            initiative_count_hour: 0,
            hour_start: Instant::now(),
        }
    }

    /// Decide whether VOXY may take initiative.
    pub fn decide(
        &mut self,
        trust_score: f64,
        relationship: RelationshipLevel,
        current_behavior: BehaviorState,
        has_reason: bool,
        now: Instant,
    ) -> InitiativeDecision {
        if !self.config.enabled {
            return InitiativeDecision {
                may_speak: false,
                reason: "Initiative disabled".to_string(),
                priority: 0.0,
            };
        }

        if self.config.deep_focus_respect && current_behavior == BehaviorState::DeepFocus {
            return InitiativeDecision {
                may_speak: false,
                reason: "User in deep focus".to_string(),
                priority: 0.0,
            };
        }

        if current_behavior == BehaviorState::Sleeping {
            return InitiativeDecision {
                may_speak: false,
                reason: "User sleeping".to_string(),
                priority: 0.0,
            };
        }

        if current_behavior.requires_silence() {
            return InitiativeDecision {
                may_speak: false,
                reason: "Current state requires silence".to_string(),
                priority: 0.0,
            };
        }

        if now.duration_since(self.hour_start) > Duration::from_secs(3600) {
            self.initiative_count_hour = 0;
            self.hour_start = now;
        }

        if self.initiative_count_hour >= self.config.max_per_hour {
            return InitiativeDecision {
                may_speak: false,
                reason: "Hourly initiative limit reached".to_string(),
                priority: 0.0,
            };
        }

        if let Some(last) = self.last_initiative {
            if now.duration_since(last) < self.config.cooldown {
                return InitiativeDecision {
                    may_speak: false,
                    reason: "Initiative cooldown active".to_string(),
                    priority: 0.0,
                };
            }
        }

        if trust_score < self.config.min_trust_for_initiative {
            return InitiativeDecision {
                may_speak: false,
                reason: "Trust level insufficient for initiative".to_string(),
                priority: 0.0,
            };
        }

        if relationship < self.config.min_relationship_for_initiative {
            return InitiativeDecision {
                may_speak: false,
                reason: "Relationship level insufficient for initiative".to_string(),
                priority: 0.0,
            };
        }

        if !has_reason {
            return InitiativeDecision {
                may_speak: false,
                reason: "No valid reason for initiative".to_string(),
                priority: 0.0,
            };
        }

        let priority = trust_score * 0.5 + relationship.trust_multiplier() * 0.3 + 0.2;

        self.last_initiative = Some(now);
        self.initiative_count_hour += 1;

        InitiativeDecision {
            may_speak: true,
            reason: "All conditions met for initiative".to_string(),
            priority: priority.clamp(0.0, 1.0),
        }
    }

    pub fn reset_hour(&mut self) {
        self.initiative_count_hour = 0;
        self.hour_start = Instant::now();
    }
}

impl Default for InitiativeEngine {
    fn default() -> Self {
        Self::new(InitiativeConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initiative_disabled() {
        let config = InitiativeConfig {
            enabled: false,
            ..InitiativeConfig::default()
        };
        let mut engine = InitiativeEngine::new(config);
        let decision = engine.decide(
            0.8,
            RelationshipLevel::Trusted,
            BehaviorState::Waiting,
            true,
            Instant::now(),
        );
        assert!(!decision.may_speak);
    }

    #[test]
    fn test_initiative_deep_focus_blocks() {
        let mut engine = InitiativeEngine::new(InitiativeConfig::default());
        let decision = engine.decide(
            0.9,
            RelationshipLevel::Trusted,
            BehaviorState::DeepFocus,
            true,
            Instant::now(),
        );
        assert!(!decision.may_speak);
    }

    #[test]
    fn test_initiative_low_trust_blocks() {
        let mut engine = InitiativeEngine::new(InitiativeConfig::default());
        let decision = engine.decide(
            0.3,
            RelationshipLevel::Trusted,
            BehaviorState::Waiting,
            true,
            Instant::now(),
        );
        assert!(!decision.may_speak);
    }

    #[test]
    fn test_initiative_allowed() {
        let mut engine = InitiativeEngine::new(InitiativeConfig::default());
        let decision = engine.decide(
            0.8,
            RelationshipLevel::Trusted,
            BehaviorState::Waiting,
            true,
            Instant::now(),
        );
        assert!(decision.may_speak);
    }

    #[test]
    fn test_initiative_cooldown() {
        let mut engine = InitiativeEngine::new(InitiativeConfig::default());
        let now = Instant::now();
        engine.decide(
            0.8,
            RelationshipLevel::Trusted,
            BehaviorState::Waiting,
            true,
            now,
        );
        let decision = engine.decide(
            0.8,
            RelationshipLevel::Trusted,
            BehaviorState::Waiting,
            true,
            now,
        );
        assert!(!decision.may_speak);
    }
}
