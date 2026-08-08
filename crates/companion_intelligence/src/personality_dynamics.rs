use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mood {
    Cheerful,
    Calm,
    Focused,
    Serious,
    Tired,
    Excited,
    Thoughtful,
    Playful,
}

impl Mood {
    pub fn voice_speed_modifier(&self) -> f64 {
        match self {
            Self::Cheerful => 1.1,
            Self::Calm => 0.95,
            Self::Focused => 1.0,
            Self::Serious => 0.9,
            Self::Tired => 0.85,
            Self::Excited => 1.2,
            Self::Thoughtful => 0.9,
            Self::Playful => 1.15,
        }
    }

    pub fn pause_duration_modifier(&self) -> f64 {
        match self {
            Self::Cheerful => 0.8,
            Self::Calm => 1.1,
            Self::Focused => 0.9,
            Self::Serious => 1.2,
            Self::Tired => 1.3,
            Self::Excited => 0.7,
            Self::Thoughtful => 1.4,
            Self::Playful => 0.85,
        }
    }

    pub fn thinking_delay_modifier(&self) -> f64 {
        match self {
            Self::Cheerful => 0.8,
            Self::Calm => 1.0,
            Self::Focused => 0.9,
            Self::Serious => 1.1,
            Self::Tired => 1.3,
            Self::Excited => 0.7,
            Self::Thoughtful => 1.5,
            Self::Playful => 0.85,
        }
    }

    pub fn response_length_modifier(&self) -> f64 {
        match self {
            Self::Cheerful => 1.2,
            Self::Calm => 1.0,
            Self::Focused => 0.9,
            Self::Serious => 1.1,
            Self::Tired => 0.8,
            Self::Excited => 1.3,
            Self::Thoughtful => 1.4,
            Self::Playful => 1.25,
        }
    }

    pub fn word_choice(&self) -> &'static [&'static str] {
        match self {
            Self::Cheerful => &["great", "awesome", "love", "nice", "cool"],
            Self::Calm => &["sure", "alright", "okay", "understood", "noted"],
            Self::Focused => &["right", "proceeding", "executing", "working", "done"],
            Self::Serious => &[
                "acknowledged",
                "proceeding",
                "correct",
                "precisely",
                "indeed",
            ],
            Self::Tired => &["mm-hmm", "yep", "sure", "will do", "okay"],
            Self::Excited => &["wow", "amazing", "fantastic", "brilliant", "excellent"],
            Self::Thoughtful => &["hmm", "interesting", "considering", "perhaps", "maybe"],
            Self::Playful => &["ooh", "hey", "guess what", "by the way", "psst"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodEntry {
    pub mood: Mood,
    pub intensity: f64,
    pub timestamp: DateTime<Utc>,
    pub trigger: String,
}

pub struct PersonalityDynamics {
    current_mood: Mood,
    mood_intensity: f64,
    mood_history: VecDeque<MoodEntry>,
    base_personality: BasePersonality,
    mood_transitions: Vec<(Mood, Mood, f64)>,
    max_history: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasePersonality {
    pub name: String,
    pub traits: Vec<String>,
    pub default_mood: Mood,
    pub mood_volatility: f64,
    pub enthusiasm_base: f64,
    pub formality_base: f64,
    pub humor_base: f64,
}

impl Default for BasePersonality {
    fn default() -> Self {
        Self {
            name: "Voxy".to_string(),
            traits: vec![
                "helpful".to_string(),
                "attentive".to_string(),
                "friendly".to_string(),
                "intelligent".to_string(),
            ],
            default_mood: Mood::Calm,
            mood_volatility: 0.3,
            enthusiasm_base: 0.6,
            formality_base: 0.5,
            humor_base: 0.4,
        }
    }
}

impl PersonalityDynamics {
    pub fn new(base_personality: BasePersonality) -> Self {
        Self {
            current_mood: base_personality.default_mood,
            mood_intensity: 0.5,
            mood_history: VecDeque::with_capacity(50),
            base_personality,
            mood_transitions: vec![
                (Mood::Calm, Mood::Focused, 0.7),
                (Mood::Focused, Mood::Calm, 0.5),
                (Mood::Calm, Mood::Cheerful, 0.6),
                (Mood::Cheerful, Mood::Calm, 0.4),
                (Mood::Focused, Mood::Serious, 0.6),
                (Mood::Serious, Mood::Calm, 0.5),
                (Mood::Calm, Mood::Tired, 0.3),
                (Mood::Tired, Mood::Calm, 0.4),
                (Mood::Calm, Mood::Excited, 0.5),
                (Mood::Excited, Mood::Calm, 0.4),
                (Mood::Calm, Mood::Thoughtful, 0.6),
                (Mood::Thoughtful, Mood::Calm, 0.5),
                (Mood::Calm, Mood::Playful, 0.4),
                (Mood::Playful, Mood::Calm, 0.5),
            ],
            max_history: 50,
        }
    }

    pub fn update_mood(&mut self, trigger: &str, context_mood: Option<Mood>) {
        let new_mood = if let Some(mood) = context_mood {
            mood
        } else {
            self.infer_mood_from_trigger(trigger)
        };

        let should_transition = if context_mood.is_some() {
            true
        } else {
            let transition_probability = self
                .mood_transitions
                .iter()
                .find(|(from, _, _prob)| {
                    *from == self.current_mood && new_mood != self.current_mood
                })
                .map(|(_, _, prob)| *prob)
                .unwrap_or(0.5);

            let random_value = (chrono::Utc::now().timestamp_millis() % 100) as f64 / 100.0;
            random_value < transition_probability
        };

        if should_transition {
            self.current_mood = new_mood;
            self.mood_intensity =
                (self.mood_intensity + self.base_personality.mood_volatility).min(1.0);
        } else {
            self.mood_intensity =
                (self.mood_intensity - self.base_personality.mood_volatility * 0.5).max(0.0);
        }

        let entry = MoodEntry {
            mood: self.current_mood,
            intensity: self.mood_intensity,
            timestamp: Utc::now(),
            trigger: trigger.to_string(),
        };

        self.mood_history.push_back(entry);
        if self.mood_history.len() > self.max_history {
            self.mood_history.pop_front();
        }
    }

    pub fn current_mood(&self) -> Mood {
        self.current_mood
    }

    pub fn mood_intensity(&self) -> f64 {
        self.mood_intensity
    }

    pub fn voice_speed(&self) -> f64 {
        self.current_mood.voice_speed_modifier()
    }

    pub fn pause_duration(&self) -> f64 {
        self.current_mood.pause_duration_modifier()
    }

    pub fn thinking_delay(&self) -> f64 {
        self.current_mood.thinking_delay_modifier()
    }

    pub fn response_length(&self) -> f64 {
        self.current_mood.response_length_modifier()
    }

    pub fn word_choice(&self) -> &'static [&'static str] {
        self.current_mood.word_choice()
    }

    pub fn mood_history(&self) -> &VecDeque<MoodEntry> {
        &self.mood_history
    }

    pub fn base_personality(&self) -> &BasePersonality {
        &self.base_personality
    }

    pub fn adapt_to_user(&mut self, user_sentiment: f64) {
        let mood = if user_sentiment > 0.7 {
            Mood::Cheerful
        } else if user_sentiment < 0.3 {
            Mood::Thoughtful
        } else {
            Mood::Calm
        };

        self.update_mood("user_sentiment", Some(mood));
    }

    pub fn handle_task_completion(&mut self, success: bool) {
        let mood = if success {
            Mood::Cheerful
        } else {
            Mood::Serious
        };

        self.update_mood("task_completion", Some(mood));
    }

    pub fn handle_long_session(&mut self, duration_ms: u64) {
        if duration_ms > 3_600_000 {
            self.update_mood("long_session", Some(Mood::Tired));
        } else if duration_ms > 1_800_000 {
            self.update_mood("medium_session", Some(Mood::Focused));
        }
    }

    fn infer_mood_from_trigger(&self, trigger: &str) -> Mood {
        let lower = trigger.to_lowercase();

        if lower.contains("error") || lower.contains("fail") {
            Mood::Serious
        } else if lower.contains("success") || lower.contains("complete") {
            Mood::Cheerful
        } else if lower.contains("question") || lower.contains("think") {
            Mood::Thoughtful
        } else if lower.contains("joke") || lower.contains("fun") {
            Mood::Playful
        } else if lower.contains("urgent") || lower.contains("critical") {
            Mood::Serious
        } else {
            self.base_personality.default_mood
        }
    }
}

impl Default for PersonalityDynamics {
    fn default() -> Self {
        Self::new(BasePersonality::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mood_modifiers() {
        assert!(Mood::Excited.voice_speed_modifier() > 1.0);
        assert!(Mood::Tired.voice_speed_modifier() < 1.0);
        assert!(Mood::Thoughtful.thinking_delay_modifier() > 1.0);
    }

    #[test]
    fn test_personality_dynamics_creation() {
        let dynamics = PersonalityDynamics::default();
        assert_eq!(dynamics.current_mood(), Mood::Calm);
    }

    #[test]
    fn test_update_mood() {
        let mut dynamics = PersonalityDynamics::default();
        dynamics.update_mood("success", Some(Mood::Cheerful));
        assert_eq!(dynamics.current_mood(), Mood::Cheerful);
    }

    #[test]
    fn test_adapt_to_user() {
        let mut dynamics = PersonalityDynamics::default();
        dynamics.adapt_to_user(0.9);
        assert_eq!(dynamics.current_mood(), Mood::Cheerful);
    }

    #[test]
    fn test_handle_task_completion() {
        let mut dynamics = PersonalityDynamics::default();
        dynamics.handle_task_completion(true);
        assert_eq!(dynamics.current_mood(), Mood::Cheerful);
        dynamics.handle_task_completion(false);
        assert_eq!(dynamics.current_mood(), Mood::Serious);
    }

    #[test]
    fn test_mood_history() {
        let mut dynamics = PersonalityDynamics::default();
        dynamics.update_mood("test1", None);
        dynamics.update_mood("test2", None);
        assert_eq!(dynamics.mood_history().len(), 2);
    }

    #[test]
    fn test_voice_params() {
        let dynamics = PersonalityDynamics::default();
        assert!(dynamics.voice_speed() > 0.0);
        assert!(dynamics.pause_duration() > 0.0);
        assert!(dynamics.thinking_delay() > 0.0);
        assert!(dynamics.response_length() > 0.0);
    }
}
