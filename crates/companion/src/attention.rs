use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Activity kinds the companion can detect or reason about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityKind {
    Coding,
    Research,
    Planning,
    Writing,
    Reading,
    Meeting,
    Gaming,
    Browsing,
    Communication,
    Designing,
    Debugging,
    Testing,
    Documenting,
    休息,
    Unknown,
}

impl ActivityKind {
    /// How much focus this activity typically requires (0.0 - 1.0).
    pub fn focus_demand(&self) -> f64 {
        match self {
            Self::Coding => 0.9,
            Self::Research => 0.7,
            Self::Planning => 0.8,
            Self::Writing => 0.85,
            Self::Reading => 0.6,
            Self::Meeting => 0.75,
            Self::Gaming => 0.5,
            Self::Browsing => 0.3,
            Self::Communication => 0.5,
            Self::Designing => 0.85,
            Self::Debugging => 0.95,
            Self::Testing => 0.7,
            Self::Documenting => 0.65,
            Self::休息 => 0.1,
            Self::Unknown => 0.4,
        }
    }

    /// Whether the user is likely in deep focus for this activity.
    pub fn is_deep_focus(&self) -> bool {
        self.focus_demand() >= 0.8
    }
}

/// Attention model output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionState {
    /// Current detected activity.
    pub activity: ActivityKind,
    /// Focus level (0.0 - 1.0).
    pub focus_level: f64,
    /// Whether the user is in deep focus.
    pub deep_focus: bool,
    /// Whether we should interrupt.
    pub can_interrupt: bool,
    /// Stress estimate (0.0 - 1.0).
    pub stress_estimate: f64,
    /// How long the user has been in this state.
    pub state_duration: Duration,
    /// Confidence in activity detection.
    pub detection_confidence: f64,
}

/// Detects and tracks user attention state.
#[derive(Debug)]
pub struct AttentionModel {
    current_activity: ActivityKind,
    activity_start: std::time::Instant,
    focus_level: f64,
    stress_accumulator: f64,
    interruption_count: usize,
    last_interruption: Option<std::time::Instant>,
}

impl AttentionModel {
    pub fn new() -> Self {
        Self {
            current_activity: ActivityKind::Unknown,
            activity_start: std::time::Instant::now(),
            focus_level: 0.5,
            stress_accumulator: 0.0,
            interruption_count: 0,
            last_interruption: None,
        }
    }

    /// Update attention state from input signals.
    pub fn update(
        &mut self,
        activity: Option<ActivityKind>,
        idle_duration: Duration,
        stress: f64,
        focus_override: Option<f64>,
        now: std::time::Instant,
    ) -> AttentionState {
        let state_duration = now.duration_since(self.activity_start);

        if let Some(act) = activity {
            if act != self.current_activity {
                self.current_activity = act;
                self.activity_start = now;
            }
        }

        let base_focus = self.current_activity.focus_demand();
        let focus = focus_override.unwrap_or_else(|| {
            let idle_penalty = if idle_duration > Duration::from_secs(300) {
                0.2
            } else if idle_duration > Duration::from_secs(60) {
                0.1
            } else {
                0.0
            };
            (base_focus - idle_penalty - stress * 0.15).clamp(0.0, 1.0)
        });
        self.focus_level = focus;

        self.stress_accumulator = (self.stress_accumulator * 0.9 + stress * 0.1).clamp(0.0, 1.0);

        let deep_focus = self.current_activity.is_deep_focus() && focus >= 0.8;

        let since_last_interrupt = self
            .last_interruption
            .map(|t| now.duration_since(t))
            .unwrap_or(Duration::from_secs(u64::MAX));

        let can_interrupt =
            !deep_focus && since_last_interrupt > Duration::from_secs(30) && focus < 0.9;

        AttentionState {
            activity: self.current_activity,
            focus_level: self.focus_level,
            deep_focus,
            can_interrupt,
            stress_estimate: self.stress_accumulator,
            state_duration,
            detection_confidence: if state_duration > Duration::from_secs(10) {
                0.8
            } else {
                0.5
            },
        }
    }

    pub fn record_interruption(&mut self, now: std::time::Instant) {
        self.interruption_count += 1;
        self.last_interruption = Some(now);
    }

    pub fn interruption_count(&self) -> usize {
        self.interruption_count
    }
}

impl Default for AttentionModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_focus_demand() {
        assert!(ActivityKind::Coding.focus_demand() > 0.8);
        assert!(ActivityKind::Browsing.focus_demand() < 0.5);
    }

    #[test]
    fn test_deep_focus_detection() {
        assert!(ActivityKind::Coding.is_deep_focus());
        assert!(ActivityKind::Debugging.is_deep_focus());
        assert!(!ActivityKind::Browsing.is_deep_focus());
    }

    #[test]
    fn test_attention_model_update() {
        let mut model = AttentionModel::new();
        let now = std::time::Instant::now();
        let state = model.update(Some(ActivityKind::Coding), Duration::ZERO, 0.0, None, now);
        assert!(state.focus_level > 0.7);
        assert!(state.deep_focus);
        assert!(!state.can_interrupt);
    }

    #[test]
    fn test_attention_model_idle_penalty() {
        let mut model = AttentionModel::new();
        let now = std::time::Instant::now();
        let state = model.update(
            Some(ActivityKind::Coding),
            Duration::from_secs(120),
            0.0,
            None,
            now,
        );
        assert!(state.focus_level < 0.9);
    }

    #[test]
    fn test_can_interrupt_low_focus() {
        let mut model = AttentionModel::new();
        let now = std::time::Instant::now();
        let state = model.update(Some(ActivityKind::Browsing), Duration::ZERO, 0.0, None, now);
        assert!(state.can_interrupt);
    }

    #[test]
    fn test_stress_accumulates() {
        let mut model = AttentionModel::new();
        let now = std::time::Instant::now();
        model.update(Some(ActivityKind::Coding), Duration::ZERO, 0.8, None, now);
        let state = model.update(Some(ActivityKind::Coding), Duration::ZERO, 0.8, None, now);
        assert!(state.stress_estimate > 0.1);
    }
}
