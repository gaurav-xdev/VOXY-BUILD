#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Task failed: {0}")]
    TaskFailed(String),
    #[error("Task cancelled: {0}")]
    TaskCancelled(String),
    #[error("Scheduler error: {0}")]
    SchedulerError(String),
    #[error("Worker pool exhausted")]
    WorkerPoolExhausted,
    #[error("Runtime not initialized")]
    NotInitialized,
    #[error("Runtime already running")]
    AlreadyRunning,
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RuntimeError::TaskNotFound("test".into());
        assert_eq!(format!("{}", err), "Task not found: test");

        let err = RuntimeError::TaskFailed("crash".into());
        assert_eq!(format!("{}", err), "Task failed: crash");

        let err = RuntimeError::TaskCancelled("user".into());
        assert_eq!(format!("{}", err), "Task cancelled: user");

        let err = RuntimeError::SchedulerError("full".into());
        assert_eq!(format!("{}", err), "Scheduler error: full");

        let err = RuntimeError::WorkerPoolExhausted;
        assert_eq!(format!("{}", err), "Worker pool exhausted");

        let err = RuntimeError::NotInitialized;
        assert_eq!(format!("{}", err), "Runtime not initialized");

        let err = RuntimeError::AlreadyRunning;
        assert_eq!(format!("{}", err), "Runtime already running");

        let err = RuntimeError::InvalidConfig("bad field".into());
        assert_eq!(format!("{}", err), "Invalid configuration: bad field");
    }
}
