use std::fmt;

#[derive(Debug, Clone)]
pub enum HdrError {
    InvalidScore(String),
    ConfigurationError(String),
    StateError(String),
    TransitionDenied(String),
}

impl fmt::Display for HdrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScore(msg) => write!(f, "Invalid score: {}", msg),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Self::StateError(msg) => write!(f, "State error: {}", msg),
            Self::TransitionDenied(msg) => write!(f, "Transition denied: {}", msg),
        }
    }
}

impl std::error::Error for HdrError {}

pub type Result<T> = std::result::Result<T, HdrError>;
