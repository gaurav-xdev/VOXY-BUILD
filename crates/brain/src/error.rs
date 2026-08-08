use std::fmt;

#[derive(Debug, Clone)]
pub enum BrainError {
    NotInitialized,
    AlreadyInitialized,
    ShutdownInProgress,
    PipelineTimeout(u64),
    CognitionError(String),
    ContextError(String),
    Interruption(String),
    Cancelled(String),
    EngineLock(String),
    InvalidConfig(String),
}

impl fmt::Display for BrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotInitialized => write!(f, "Brain not initialized"),
            Self::AlreadyInitialized => write!(f, "Brain already initialized"),
            Self::ShutdownInProgress => write!(f, "Shutdown in progress"),
            Self::PipelineTimeout(ms) => write!(f, "Pipeline timeout after {}ms", ms),
            Self::CognitionError(e) => write!(f, "Cognition error: {}", e),
            Self::ContextError(e) => write!(f, "Context error: {}", e),
            Self::Interruption(r) => write!(f, "Interrupted: {}", r),
            Self::Cancelled(r) => write!(f, "Cancelled: {}", r),
            Self::EngineLock(e) => write!(f, "Engine lock error: {}", e),
            Self::InvalidConfig(e) => write!(f, "Invalid config: {}", e),
        }
    }
}

impl std::error::Error for BrainError {}

pub type Result<T> = std::result::Result<T, BrainError>;
