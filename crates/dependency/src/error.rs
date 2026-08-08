#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    #[error("Dependency cycle detected: {0}")]
    CycleDetected(String),
    #[error("Dependency not found: {0}")]
    NotFound(String),
    #[error("Resolution failed: {0}")]
    ResolutionFailed(String),
    #[error("Invalid dependency spec: {0}")]
    InvalidSpec(String),
    #[error("Scheduling error: {0}")]
    SchedulingError(String),
}

pub type Result<T> = std::result::Result<T, DependencyError>;
