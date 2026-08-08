use thiserror::Error;

#[derive(Error, Debug)]
pub enum LearningError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Preference error: {0}")]
    PreferenceError(String),

    #[error("Behavior error: {0}")]
    BehaviorError(String),

    #[error("Reinforcement error: {0}")]
    ReinforcementError(String),

    #[error("Calibration error: {0}")]
    CalibrationError(String),

    #[error("Feedback error: {0}")]
    FeedbackError(String),

    #[error("Evolution error: {0}")]
    EvolutionError(String),

    #[error("Policy error: {0}")]
    PolicyError(String),

    #[error("Threshold error: {0}")]
    ThresholdError(String),

    #[error("Memory access error: {0}")]
    MemoryAccessError(String),

    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, LearningError>;
