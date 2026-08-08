use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum CharacterTrait {
    Warmth(f64),
    Formality(f64),
    Humor(f64),
    Empathy(f64),
    Assertiveness(f64),
    Creativity(f64),
    Conciseness(f64),
    Custom(String, f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoodState {
    Neutral,
    Happy,
    Sad,
    Anxious,
    Excited,
    Calm,
    Frustrated,
    Playful,
    Serious,
    Tired,
    Custom(String),
}

impl fmt::Display for MoodState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Neutral => write!(f, "Neutral"),
            Self::Happy => write!(f, "Happy"),
            Self::Sad => write!(f, "Sad"),
            Self::Anxious => write!(f, "Anxious"),
            Self::Excited => write!(f, "Excited"),
            Self::Calm => write!(f, "Calm"),
            Self::Frustrated => write!(f, "Frustrated"),
            Self::Playful => write!(f, "Playful"),
            Self::Serious => write!(f, "Serious"),
            Self::Tired => write!(f, "Tired"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommunicationStyle {
    Casual,
    Formal,
    Professional,
    Friendly,
    Authoritative,
    Empathetic,
    Playful,
    Custom(String),
}

impl fmt::Display for CommunicationStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Casual => write!(f, "Casual"),
            Self::Formal => write!(f, "Formal"),
            Self::Professional => write!(f, "Professional"),
            Self::Friendly => write!(f, "Friendly"),
            Self::Authoritative => write!(f, "Authoritative"),
            Self::Empathetic => write!(f, "Empathetic"),
            Self::Playful => write!(f, "Playful"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[async_trait]
pub trait PersonalityProfile: Send + Sync {
    fn profile_id(&self) -> &str;
    fn profile_name(&self) -> &str;
    fn get_trait(&self, name: &str) -> Option<f64>;
    fn set_trait(&mut self, name: &str, value: f64) -> Result<()>;
    fn all_traits(&self) -> HashMap<String, f64>;
    fn mood(&self) -> MoodState;
    fn set_mood(&mut self, mood: MoodState);
    fn communication_style(&self) -> CommunicationStyle;
    fn set_communication_style(&mut self, style: CommunicationStyle);
}

#[async_trait]
pub trait PersonalityManager: Send + Sync {
    async fn load_profile(&self, id: &str) -> Result<Box<dyn PersonalityProfile>>;
    async fn save_profile(&self, profile: Box<dyn PersonalityProfile>) -> Result<()>;
    async fn list_profiles(&self) -> Result<Vec<String>>;
    async fn delete_profile(&self, id: &str) -> Result<()>;
    async fn default_profile(&self) -> Result<Box<dyn PersonalityProfile>>;
}
