use std::time::Duration;

use chrono::Timelike;
use serde::{Deserialize, Serialize};

use crate::attention::AttentionState;
use crate::config::ScoreWeights;
use crate::types::{CompanionInput, UserPresence};

/// Detailed breakdown of the presence score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceScoreBreakdown {
    pub user_activity: f64,
    pub focus_level: f64,
    pub time_of_day: f64,
    pub conversation_frequency: f64,
    pub mission_state: f64,
    pub stress_estimate: f64,
    pub idle_time: f64,
    pub total: f64,
}

/// Computes the continuous presence score.
pub struct PresenceScoreEngine {
    weights: ScoreWeights,
    history: Vec<f64>,
    max_history: usize,
}

impl PresenceScoreEngine {
    pub fn new(weights: ScoreWeights) -> Self {
        Self {
            weights,
            history: Vec::new(),
            max_history: 60,
        }
    }

    /// Compute the presence score from current input.
    pub fn compute(
        &mut self,
        input: &CompanionInput,
        attention: &AttentionState,
    ) -> PresenceScoreBreakdown {
        let user_activity = match &input.user_presence {
            UserPresence::Active => 1.0,
            UserPresence::Focused => 0.9,
            UserPresence::InMeeting => 0.7,
            UserPresence::Gaming => 0.6,
            UserPresence::Browsing => 0.5,
            UserPresence::Idle { .. } => 0.3,
            UserPresence::Away { .. } => 0.1,
            UserPresence::Sleeping { .. } => 0.0,
        };

        let focus_level = attention.focus_level;

        let time_of_day = {
            let hour = input.now.hour();
            match hour {
                9..=17 => 0.9,
                7..=8 => 0.7,
                18..=19 => 0.7,
                20..=21 => 0.5,
                _ => 0.2,
            }
        };

        let conversation_frequency =
            (input.conversation_count_this_session as f64 / 10.0).clamp(0.0, 1.0);

        let mission_state = if input.mission_state.is_active() {
            0.8
        } else {
            0.3
        };

        let stress = 1.0 - input.stress_estimate;

        let idle = if input.idle_duration > Duration::from_secs(600) {
            0.1
        } else if input.idle_duration > Duration::from_secs(120) {
            0.4
        } else {
            0.8
        };

        let total = user_activity * self.weights.user_activity
            + focus_level * self.weights.focus_level
            + time_of_day * self.weights.time_of_day
            + conversation_frequency * self.weights.conversation_frequency
            + mission_state * self.weights.mission_state
            + stress * self.weights.stress_estimate
            + idle * self.weights.idle_time;

        let total = total.clamp(0.0, 1.0);

        self.history.push(total);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        PresenceScoreBreakdown {
            user_activity,
            focus_level,
            time_of_day,
            conversation_frequency,
            mission_state,
            stress_estimate: input.stress_estimate,
            idle_time: idle,
            total,
        }
    }

    /// Smoothed presence score over recent history.
    pub fn smoothed_score(&self) -> f64 {
        if self.history.is_empty() {
            return 0.5;
        }
        let sum: f64 = self.history.iter().sum();
        sum / self.history.len() as f64
    }

    pub fn reset(&mut self) {
        self.history.clear();
    }
}

impl Default for PresenceScoreEngine {
    fn default() -> Self {
        Self::new(ScoreWeights::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attention::ActivityKind;
    use crate::types::{MissionState, SessionId, WeatherContext};
    use chrono::Utc;

    fn make_input() -> CompanionInput {
        CompanionInput {
            now: Utc::now(),
            session_id: SessionId::new(),
            user_presence: UserPresence::Active,
            current_activity: Some(ActivityKind::Coding),
            time_since_last_interaction: Duration::from_secs(30),
            conversation_count_this_session: 3,
            total_session_duration: Duration::from_secs(3600),
            active_goals: vec![],
            recent_milestones: vec![],
            weather: WeatherContext::Clear,
            stress_estimate: 0.2,
            idle_duration: Duration::from_secs(10),
            pending_tasks: 2,
            completed_tasks_today: 5,
            last_greeting: None,
            last_micro_interaction: None,
            last_memory_reference: None,
            mission_state: MissionState::Idle,
            focus_level: 0.8,
        }
    }

    #[test]
    fn test_score_computation() {
        let mut engine = PresenceScoreEngine::new(ScoreWeights::default());
        let attention = AttentionState {
            activity: ActivityKind::Coding,
            focus_level: 0.8,
            deep_focus: true,
            can_interrupt: false,
            stress_estimate: 0.2,
            state_duration: Duration::from_secs(60),
            detection_confidence: 0.8,
        };
        let input = make_input();
        let breakdown = engine.compute(&input, &attention);
        assert!(breakdown.total > 0.0 && breakdown.total <= 1.0);
    }

    #[test]
    fn test_smoothed_score() {
        let mut engine = PresenceScoreEngine::new(ScoreWeights::default());
        let attention = AttentionState {
            activity: ActivityKind::Coding,
            focus_level: 0.8,
            deep_focus: true,
            can_interrupt: false,
            stress_estimate: 0.2,
            state_duration: Duration::from_secs(60),
            detection_confidence: 0.8,
        };
        let input = make_input();
        for _ in 0..10 {
            engine.compute(&input, &attention);
        }
        let smoothed = engine.smoothed_score();
        assert!(smoothed > 0.0 && smoothed <= 1.0);
    }
}
