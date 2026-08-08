use std::collections::HashMap;

use crate::traits::{CommunicationStyle, MoodState};

#[derive(Debug, Clone)]
pub struct PersonalityConfig {
    pub profile_id: String,
    pub profile_name: String,
    pub traits: HashMap<String, f64>,
    pub default_mood: MoodState,
    pub allow_mood_transitions: bool,
    pub mood_transition_interval_seconds: u64,
    pub communication_style: CommunicationStyle,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            profile_id: "default".to_string(),
            profile_name: "Default Profile".to_string(),
            traits: HashMap::new(),
            default_mood: MoodState::Neutral,
            allow_mood_transitions: true,
            mood_transition_interval_seconds: 300,
            communication_style: CommunicationStyle::Casual,
        }
    }
}
