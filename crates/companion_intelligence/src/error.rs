use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Emotional state error: {0}")]
    Emotional(String),

    #[error("Presence engine error: {0}")]
    Presence(String),

    #[error("Conversation error: {0}")]
    Conversation(String),

    #[error("Memory importance error: {0}")]
    MemoryImportance(String),

    #[error("Proactive engine error: {0}")]
    Proactive(String),

    #[error("Decision engine error: {0}")]
    Decision(String),

    #[error("Personality dynamics error: {0}")]
    Personality(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

pub type Result<T> = std::result::Result<T, IntelligenceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IntelligenceError::Emotional("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        assert!(json_err.is_err());
        let err: IntelligenceError = json_err.unwrap_err().into();
        assert!(matches!(err, IntelligenceError::Serialization(_)));
    }
}
