#[derive(Debug, thiserror::Error)]
pub enum PersonalityError {
    #[error("Invalid personality configuration: {0}")]
    InvalidConfig(String),
    #[error("Trait value out of range: {0}")]
    TraitValueOutOfRange(String),
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),
    #[error("Mood transition failed: {0}")]
    MoodTransitionFailed(String),
}

pub type Result<T> = std::result::Result<T, PersonalityError>;
