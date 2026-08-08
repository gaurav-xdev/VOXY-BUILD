use std::time::{Duration, Instant};

use crate::config::BehaviorConfig;
use crate::types::BehaviorState;

/// State transition record.
#[derive(Debug, Clone)]
pub struct TransitionRecord {
    pub from: BehaviorState,
    pub to: BehaviorState,
    pub timestamp: Instant,
    pub reason: String,
}

/// Behavior engine — manages VOXY's behavioral state machine.
pub struct BehaviorEngine {
    config: BehaviorConfig,
    current: BehaviorState,
    state_entered: Instant,
    last_transition: Instant,
    history: Vec<TransitionRecord>,
    transition_count: usize,
}

impl BehaviorEngine {
    pub fn new(config: BehaviorConfig) -> Self {
        let now = Instant::now();
        Self {
            current: config.default_state,
            state_entered: now,
            last_transition: now,
            history: Vec::new(),
            transition_count: 0,
            config,
        }
    }

    /// Attempt a state transition. Returns true if successful.
    pub fn transition(&mut self, target: BehaviorState, reason: &str, now: Instant) -> bool {
        if now.duration_since(self.last_transition) < self.config.transition_cooldown {
            return false;
        }

        if !self.current.can_transition_to(&target) {
            return false;
        }

        let record = TransitionRecord {
            from: self.current,
            to: target,
            timestamp: now,
            reason: reason.to_string(),
        };

        self.history.push(record);
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        self.current = target;
        self.state_entered = now;
        self.last_transition = now;
        self.transition_count += 1;
        true
    }

    /// Check if state has exceeded max time.
    pub fn should_timeout(&self, now: Instant) -> bool {
        now.duration_since(self.state_entered) > self.config.max_time_in_state
    }

    /// Get suggested state based on context.
    pub fn suggest_state(
        &self,
        focus_level: f64,
        user_present: bool,
        is_meeting: bool,
        idle_duration: Duration,
        has_pending_action: bool,
    ) -> BehaviorState {
        if !user_present {
            return if idle_duration > self.config.sleeping_after {
                BehaviorState::Sleeping
            } else {
                BehaviorState::Waiting
            };
        }

        if is_meeting {
            return BehaviorState::Observing;
        }

        if focus_level >= self.config.deep_focus_threshold {
            return BehaviorState::DeepFocus;
        }

        if has_pending_action {
            return BehaviorState::Working;
        }

        if idle_duration > Duration::from_secs(300) {
            return BehaviorState::Observing;
        }

        self.current
    }

    pub fn current(&self) -> BehaviorState {
        self.current
    }

    pub fn time_in_state(&self, now: Instant) -> Duration {
        now.duration_since(self.state_entered)
    }

    pub fn transition_count(&self) -> usize {
        self.transition_count
    }

    pub fn history(&self) -> &[TransitionRecord] {
        &self.history
    }
}

impl Default for BehaviorEngine {
    fn default() -> Self {
        Self::new(BehaviorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let engine = BehaviorEngine::new(BehaviorConfig::default());
        assert_eq!(engine.current(), BehaviorState::Observing);
    }

    #[test]
    fn test_valid_transition() {
        let mut engine = BehaviorEngine::new(BehaviorConfig::default());
        let now = Instant::now() + Duration::from_secs(3);
        assert!(engine.transition(BehaviorState::Listening, "user spoke", now));
        assert_eq!(engine.current(), BehaviorState::Listening);
    }

    #[test]
    fn test_invalid_transition() {
        let mut engine = BehaviorEngine::new(BehaviorConfig::default());
        let now = Instant::now();
        engine.transition(BehaviorState::DeepFocus, "focus", now);
        assert!(!engine.transition(BehaviorState::Celebrating, "test", now));
    }

    #[test]
    fn test_suggest_deep_focus() {
        let engine = BehaviorEngine::new(BehaviorConfig::default());
        let state = engine.suggest_state(0.9, true, false, Duration::ZERO, false);
        assert_eq!(state, BehaviorState::DeepFocus);
    }

    #[test]
    fn test_suggest_sleeping() {
        let engine = BehaviorEngine::new(BehaviorConfig::default());
        let state = engine.suggest_state(0.0, false, false, Duration::from_secs(2000), false);
        assert_eq!(state, BehaviorState::Sleeping);
    }

    #[test]
    fn test_transition_cooldown() {
        let mut engine = BehaviorEngine::new(BehaviorConfig::default());
        let now = Instant::now();
        engine.transition(BehaviorState::Listening, "test", now);
        assert!(!engine.transition(BehaviorState::Thinking, "test", now));
    }
}
