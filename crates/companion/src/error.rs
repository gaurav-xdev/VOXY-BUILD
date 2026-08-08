use std::fmt;

#[derive(Debug, Clone)]
pub enum CompanionError {
    InvalidScore(f64),
    ConfigurationError(String),
    StateError(String),
}

impl fmt::Display for CompanionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScore(s) => write!(f, "Invalid score: {}", s),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Self::StateError(msg) => write!(f, "State error: {}", msg),
        }
    }
}

impl std::error::Error for CompanionError {}

pub type Result<T> = std::result::Result<T, CompanionError>;
