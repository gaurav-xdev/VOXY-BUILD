use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionType {
    Focused,
    Happy,
    Frustrated,
    Confused,
    Tired,
    Busy,
    Calm,
    Excited,
    Bored,
    Satisfied,
}

impl std::fmt::Display for EmotionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Focused => write!(f, "Focused"),
            Self::Happy => write!(f, "Happy"),
            Self::Frustrated => write!(f, "Frustrated"),
            Self::Confused => write!(f, "Confused"),
            Self::Tired => write!(f, "Tired"),
            Self::Busy => write!(f, "Busy"),
            Self::Calm => write!(f, "Calm"),
            Self::Excited => write!(f, "Excited"),
            Self::Bored => write!(f, "Bored"),
            Self::Satisfied => write!(f, "Satisfied"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalSignal {
    pub signal_type: SignalType,
    pub intensity: f64,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalType {
    RapidTyping,
    LongPause,
    RepeatedActions,
    ErrorCount,
    TaskCompletion,
    IdleTime,
    VoiceTone,
    WindowSwitching,
    ScrollSpeed,
    CodeCompilation,
    TestFailure,
    TestSuccess,
    LongCodingSession,
    ShortBreak,
    MeetingActive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionState {
    pub emotion: EmotionType,
    pub confidence: f64,
    pub valence: f64,
    pub arousal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalSnapshot {
    pub primary: EmotionState,
    pub secondary: Option<EmotionState>,
    pub timestamp: DateTime<Utc>,
}

pub struct EmotionalStateMachine {
    current_emotion: EmotionState,
    emotion_history: VecDeque<EmotionalSnapshot>,
    signal_history: VecDeque<EmotionalSignal>,
    config: EmotionalConfig,
}

struct EmotionalConfig {
    #[allow(dead_code)]
    signal_decay_rate: f64,
    #[allow(dead_code)]
    emotion_persistence_ms: u64,
    confidence_threshold: f64,
    max_history: usize,
}

impl Default for EmotionalStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionalStateMachine {
    pub fn new() -> Self {
        Self {
            current_emotion: EmotionState {
                emotion: EmotionType::Calm,
                confidence: 0.5,
                valence: 0.5,
                arousal: 0.3,
            },
            emotion_history: VecDeque::with_capacity(50),
            signal_history: VecDeque::with_capacity(100),
            config: EmotionalConfig {
                signal_decay_rate: 0.1,
                emotion_persistence_ms: 30_000,
                confidence_threshold: 0.3,
                max_history: 50,
            },
        }
    }

    pub fn process_signal(&mut self, signal: EmotionalSignal) -> &EmotionState {
        self.signal_history.push_back(signal.clone());
        if self.signal_history.len() > 100 {
            self.signal_history.pop_front();
        }

        let (new_emotion, new_valence, new_arousal) = self.infer_emotion_from_signal(&signal);

        let confidence = self.calculate_confidence(&signal, &new_emotion);

        if confidence >= self.config.confidence_threshold {
            let old_emotion = self.current_emotion.emotion;
            self.current_emotion = EmotionState {
                emotion: new_emotion,
                confidence,
                valence: self.blend_value(self.current_emotion.valence, new_valence, 0.3),
                arousal: self.blend_value(self.current_emotion.arousal, new_arousal, 0.3),
            };

            if old_emotion != new_emotion {
                let snapshot = EmotionalSnapshot {
                    primary: self.current_emotion.clone(),
                    secondary: None,
                    timestamp: Utc::now(),
                };
                self.emotion_history.push_back(snapshot);
                if self.emotion_history.len() > self.config.max_history {
                    self.emotion_history.pop_front();
                }
            }
        }

        &self.current_emotion
    }

    pub fn current_state(&self) -> &EmotionState {
        &self.current_emotion
    }

    pub fn get_snapshot(&self) -> EmotionalSnapshot {
        EmotionalSnapshot {
            primary: self.current_emotion.clone(),
            secondary: self.emotion_history.back().map(|s| s.primary.clone()),
            timestamp: Utc::now(),
        }
    }

    pub fn emotion_history(&self) -> &VecDeque<EmotionalSnapshot> {
        &self.emotion_history
    }

    fn infer_emotion_from_signal(&self, signal: &EmotionalSignal) -> (EmotionType, f64, f64) {
        match signal.signal_type {
            SignalType::RapidTyping => {
                if signal.intensity > 0.7 {
                    (EmotionType::Focused, 0.6, 0.7)
                } else {
                    (EmotionType::Busy, 0.5, 0.5)
                }
            }
            SignalType::LongPause => {
                if signal.intensity > 0.8 {
                    (EmotionType::Tired, 0.3, 0.2)
                } else {
                    (EmotionType::Bored, 0.4, 0.2)
                }
            }
            SignalType::RepeatedActions => (EmotionType::Frustrated, 0.3, 0.6),
            SignalType::ErrorCount => {
                if signal.intensity > 0.7 {
                    (EmotionType::Frustrated, 0.2, 0.8)
                } else {
                    (EmotionType::Confused, 0.4, 0.5)
                }
            }
            SignalType::TaskCompletion => (EmotionType::Satisfied, 0.8, 0.5),
            SignalType::IdleTime => {
                if signal.intensity > 0.6 {
                    (EmotionType::Tired, 0.3, 0.1)
                } else {
                    (EmotionType::Calm, 0.6, 0.2)
                }
            }
            SignalType::VoiceTone => {
                if signal.intensity > 0.7 {
                    (EmotionType::Excited, 0.7, 0.8)
                } else {
                    (EmotionType::Happy, 0.6, 0.5)
                }
            }
            SignalType::WindowSwitching => {
                if signal.intensity > 0.5 {
                    (EmotionType::Busy, 0.5, 0.6)
                } else {
                    (EmotionType::Bored, 0.4, 0.3)
                }
            }
            SignalType::ScrollSpeed => (EmotionType::Focused, 0.5, 0.4),
            SignalType::CodeCompilation => {
                if signal.intensity > 0.8 {
                    (EmotionType::Satisfied, 0.7, 0.6)
                } else {
                    (EmotionType::Focused, 0.6, 0.5)
                }
            }
            SignalType::TestFailure => (EmotionType::Frustrated, 0.3, 0.7),
            SignalType::TestSuccess => (EmotionType::Satisfied, 0.8, 0.6),
            SignalType::LongCodingSession => {
                if signal.intensity > 0.7 {
                    (EmotionType::Tired, 0.4, 0.3)
                } else {
                    (EmotionType::Focused, 0.7, 0.5)
                }
            }
            SignalType::ShortBreak => (EmotionType::Calm, 0.6, 0.3),
            SignalType::MeetingActive => (EmotionType::Busy, 0.5, 0.4),
        }
    }

    fn calculate_confidence(&self, signal: &EmotionalSignal, _emotion: &EmotionType) -> f64 {
        let recency_factor = {
            let now = Utc::now();
            let elapsed = (now - signal.timestamp).num_milliseconds() as f64;
            1.0 / (1.0 + elapsed / 10_000.0)
        };

        let intensity_factor = signal.intensity;
        let consistency_factor = self.calculate_consistency(signal);

        (recency_factor * 0.3 + intensity_factor * 0.4 + consistency_factor * 0.3).min(1.0)
    }

    fn calculate_consistency(&self, signal: &EmotionalSignal) -> f64 {
        let recent_same_type = self
            .signal_history
            .iter()
            .rev()
            .take(10)
            .filter(|s| s.signal_type == signal.signal_type)
            .count();

        (recent_same_type as f64 / 10.0).min(1.0)
    }

    fn blend_value(&self, current: f64, target: f64, factor: f64) -> f64 {
        current * (1.0 - factor) + target * factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emotional_state_machine_creation() {
        let machine = EmotionalStateMachine::new();
        assert_eq!(machine.current_state().emotion, EmotionType::Calm);
    }

    #[test]
    fn test_process_rapid_typing_signal() {
        let mut machine = EmotionalStateMachine::new();
        let signal = EmotionalSignal {
            signal_type: SignalType::RapidTyping,
            intensity: 0.8,
            timestamp: Utc::now(),
            source: "desktop".to_string(),
        };
        machine.process_signal(signal);
        assert_eq!(machine.current_state().emotion, EmotionType::Focused);
    }

    #[test]
    fn test_process_frustration_signal() {
        let mut machine = EmotionalStateMachine::new();
        let signal = EmotionalSignal {
            signal_type: SignalType::RepeatedActions,
            intensity: 0.9,
            timestamp: Utc::now(),
            source: "desktop".to_string(),
        };
        machine.process_signal(signal);
        assert_eq!(machine.current_state().emotion, EmotionType::Frustrated);
    }

    #[test]
    fn test_emotion_history() {
        let mut machine = EmotionalStateMachine::new();
        for i in 0..5 {
            let signal = EmotionalSignal {
                signal_type: SignalType::TaskCompletion,
                intensity: 0.8,
                timestamp: Utc::now(),
                source: format!("source_{}", i),
            };
            machine.process_signal(signal);
        }
        assert!(!machine.emotion_history().is_empty());
    }

    #[test]
    fn test_snapshot_creation() {
        let machine = EmotionalStateMachine::new();
        let snapshot = machine.get_snapshot();
        assert_eq!(snapshot.primary.emotion, EmotionType::Calm);
    }
}
