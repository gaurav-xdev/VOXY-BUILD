use thiserror::Error;

#[derive(Error, Debug)]
pub enum CognitiveError {
    #[error("Orchestrator error: {0}")]
    Orchestrator(String),

    #[error("Reflection error: {0}")]
    Reflection(String),

    #[error("Experience replay error: {0}")]
    ExperienceReplay(String),

    #[error("Knowledge validation error: {0}")]
    KnowledgeValidation(String),

    #[error("Skill discovery error: {0}")]
    SkillDiscovery(String),

    #[error("Workflow learning error: {0}")]
    WorkflowLearning(String),

    #[error("Goal manager error: {0}")]
    GoalManager(String),

    #[error("Curiosity engine error: {0}")]
    Curiosity(String),

    #[error("Planning error: {0}")]
    Planning(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Task cancelled")]
    Cancelled,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CognitiveError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CognitiveError::Orchestrator("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_timeout_error() {
        let err = CognitiveError::Timeout(5000);
        assert!(err.to_string().contains("5000"));
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        assert!(json_err.is_err());
        let err: CognitiveError = json_err.unwrap_err().into();
        assert!(matches!(err, CognitiveError::Serialization(_)));
    }
}
