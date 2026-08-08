use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresenceState {
    Sleeping,
    Idle,
    Listening,
    Thinking,
    Speaking,
    Celebrating,
    FocusMode,
    EmergencyMode,
}

impl PresenceState {
    pub fn transition_latency_ms(&self) -> u64 {
        match self {
            Self::Sleeping => 5000,
            Self::Idle => 1000,
            Self::Listening => 100,
            Self::Thinking => 200,
            Self::Speaking => 150,
            Self::Celebrating => 300,
            Self::FocusMode => 500,
            Self::EmergencyMode => 50,
        }
    }

    pub fn power_level(&self) -> f64 {
        match self {
            Self::Sleeping => 0.1,
            Self::Idle => 0.2,
            Self::Listening => 0.5,
            Self::Thinking => 0.6,
            Self::Speaking => 0.7,
            Self::Celebrating => 0.8,
            Self::FocusMode => 0.9,
            Self::EmergencyMode => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceEvent {
    pub event_type: PresenceEventType,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PresenceEventType {
    UserSpoke,
    UserTyped,
    UserIdle,
    UserReturned,
    VoiceActivated,
    WakeWordDetected,
    EmergencyAlert,
    TaskCompleted,
    AchievementUnlocked,
    SystemEvent,
    SleepTimerExpired,
    ActivityDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceSnapshot {
    pub state: PresenceState,
    pub duration_ms: u64,
    pub power_level: f64,
    pub last_transition: DateTime<Utc>,
    pub event_count: usize,
}

pub struct PresenceEngine {
    current_state: PresenceState,
    state_entered_at: DateTime<Utc>,
    event_history: VecDeque<PresenceEvent>,
    state_history: VecDeque<PresenceSnapshot>,
    max_history: usize,
}

impl PresenceEngine {
    pub fn new() -> Self {
        Self {
            current_state: PresenceState::Sleeping,
            state_entered_at: Utc::now(),
            event_history: VecDeque::with_capacity(100),
            state_history: VecDeque::with_capacity(50),
            max_history: 50,
        }
    }

    pub fn process_event(&mut self, event: PresenceEvent) -> &PresenceState {
        let old_state = self.current_state;

        self.event_history.push_back(event.clone());
        if self.event_history.len() > 100 {
            self.event_history.pop_front();
        }

        self.current_state = self.determine_next_state(&event);

        if old_state != self.current_state {
            let snapshot = PresenceSnapshot {
                state: old_state,
                duration_ms: (Utc::now() - self.state_entered_at).num_milliseconds() as u64,
                power_level: old_state.power_level(),
                last_transition: self.state_entered_at,
                event_count: self
                    .event_history
                    .iter()
                    .filter(|e| e.timestamp >= self.state_entered_at)
                    .count(),
            };

            self.state_history.push_back(snapshot);
            if self.state_history.len() > self.max_history {
                self.state_history.pop_front();
            }

            self.state_entered_at = Utc::now();
        }

        &self.current_state
    }

    pub fn current_state(&self) -> PresenceState {
        self.current_state
    }

    pub fn state_duration_ms(&self) -> u64 {
        (Utc::now() - self.state_entered_at).num_milliseconds() as u64
    }

    pub fn power_level(&self) -> f64 {
        self.current_state.power_level()
    }

    pub fn get_snapshot(&self) -> PresenceSnapshot {
        PresenceSnapshot {
            state: self.current_state,
            duration_ms: self.state_duration_ms(),
            power_level: self.power_level(),
            last_transition: self.state_entered_at,
            event_count: self.event_history.len(),
        }
    }

    pub fn state_history(&self) -> &VecDeque<PresenceSnapshot> {
        &self.state_history
    }

    pub fn event_history(&self) -> &VecDeque<PresenceEvent> {
        &self.event_history
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.current_state,
            PresenceState::Listening
                | PresenceState::Thinking
                | PresenceState::Speaking
                | PresenceState::FocusMode
                | PresenceState::EmergencyMode
        )
    }

    pub fn can_interrupt(&self) -> bool {
        !matches!(
            self.current_state,
            PresenceState::EmergencyMode | PresenceState::Sleeping
        )
    }

    fn determine_next_state(&self, event: &PresenceEvent) -> PresenceState {
        match event.event_type {
            PresenceEventType::UserSpoke => PresenceState::Listening,
            PresenceEventType::UserTyped => PresenceState::Thinking,
            PresenceEventType::UserIdle => {
                if self.current_state == PresenceState::Listening {
                    PresenceState::Thinking
                } else {
                    PresenceState::Idle
                }
            }
            PresenceEventType::UserReturned => PresenceState::Idle,
            PresenceEventType::VoiceActivated => PresenceState::Listening,
            PresenceEventType::WakeWordDetected => PresenceState::Listening,
            PresenceEventType::EmergencyAlert => PresenceState::EmergencyMode,
            PresenceEventType::TaskCompleted => PresenceState::Celebrating,
            PresenceEventType::AchievementUnlocked => PresenceState::Celebrating,
            PresenceEventType::SystemEvent => match self.current_state {
                PresenceState::Sleeping => PresenceState::Idle,
                _ => self.current_state,
            },
            PresenceEventType::SleepTimerExpired => PresenceState::Sleeping,
            PresenceEventType::ActivityDetected => {
                if self.current_state == PresenceState::Sleeping {
                    PresenceState::Idle
                } else {
                    self.current_state
                }
            }
        }
    }
}

impl Default for PresenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_engine_creation() {
        let engine = PresenceEngine::new();
        assert_eq!(engine.current_state(), PresenceState::Sleeping);
        assert!(!engine.is_active());
    }

    #[test]
    fn test_process_voice_event() {
        let mut engine = PresenceEngine::new();
        let event = PresenceEvent {
            event_type: PresenceEventType::VoiceActivated,
            timestamp: Utc::now(),
            source: "whisper".to_string(),
            data: None,
        };
        engine.process_event(event);
        assert_eq!(engine.current_state(), PresenceState::Listening);
        assert!(engine.is_active());
    }

    #[test]
    fn test_emergency_override() {
        let mut engine = PresenceEngine::new();
        engine.process_event(PresenceEvent {
            event_type: PresenceEventType::VoiceActivated,
            timestamp: Utc::now(),
            source: "test".to_string(),
            data: None,
        });
        assert_eq!(engine.current_state(), PresenceState::Listening);

        engine.process_event(PresenceEvent {
            event_type: PresenceEventType::EmergencyAlert,
            timestamp: Utc::now(),
            source: "system".to_string(),
            data: Some("critical".to_string()),
        });
        assert_eq!(engine.current_state(), PresenceState::EmergencyMode);
    }

    #[test]
    fn test_state_duration() {
        let mut engine = PresenceEngine::new();
        engine.process_event(PresenceEvent {
            event_type: PresenceEventType::VoiceActivated,
            timestamp: Utc::now(),
            source: "test".to_string(),
            data: None,
        });
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(engine.state_duration_ms() >= 10);
    }

    #[test]
    fn test_snapshot() {
        let engine = PresenceEngine::new();
        let snapshot = engine.get_snapshot();
        assert_eq!(snapshot.state, PresenceState::Sleeping);
        assert!(snapshot.power_level < 0.2);
    }
}
