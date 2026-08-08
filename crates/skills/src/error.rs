#[derive(Debug, thiserror::Error)]
pub enum SkillsError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Skill not found: {0}")]
    SkillNotFound(String),
    #[error("Skill execution failed: {0}")]
    SkillExecutionFailed(String),
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Skill execution timed out: {0}")]
    Timeout(String),
    #[error("Skill execution cancelled: {0}")]
    ExecutionCancelled(String),
}

pub type Result<T> = std::result::Result<T, SkillsError>;
