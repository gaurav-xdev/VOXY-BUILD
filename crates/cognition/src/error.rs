#[derive(Debug, thiserror::Error)]
pub enum CognitionError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Intent analysis failed: {0}")]
    IntentAnalysisFailed(String),
    #[error("Goal decomposition failed: {0}")]
    GoalDecompositionFailed(String),
    #[error("Planning failed: {0}")]
    PlanningFailed(String),
    #[error("Reasoning failed: {0}")]
    ReasoningFailed(String),
    #[error("Context assembly failed: {0}")]
    ContextAssemblyFailed(String),
    #[error("Tool selection failed: {0}")]
    ToolSelectionFailed(String),
    #[error("Confidence estimation failed: {0}")]
    ConfidenceEstimationFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),
    #[error("Reflection failed: {0}")]
    ReflectionFailed(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Operation timed out: {0}")]
    Timeout(String),
    #[error("State error: {0}")]
    StateError(String),
}

pub type Result<T> = std::result::Result<T, CognitionError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognition_error_display() {
        let err = CognitionError::InvalidConfig("missing field".to_string());
        assert_eq!(format!("{}", err), "Invalid configuration: missing field");

        let err = CognitionError::IntentAnalysisFailed("no intent".to_string());
        assert_eq!(format!("{}", err), "Intent analysis failed: no intent");

        let err = CognitionError::GoalDecompositionFailed("too complex".to_string());
        assert_eq!(format!("{}", err), "Goal decomposition failed: too complex");

        let err = CognitionError::PlanningFailed("no plan".to_string());
        assert_eq!(format!("{}", err), "Planning failed: no plan");

        let err = CognitionError::Timeout("took too long".to_string());
        assert_eq!(format!("{}", err), "Operation timed out: took too long");
    }
}
