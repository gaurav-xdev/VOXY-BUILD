use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorldModelError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Desktop error: {0}")]
    DesktopError(String),

    #[error("Environment error: {0}")]
    EnvironmentError(String),
}

pub type Result<T> = std::result::Result<T, WorldModelError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_model_error_display() {
        let err = WorldModelError::InvalidConfig("test".to_string());
        assert_eq!(format!("{}", err), "Invalid configuration: test");

        let err = WorldModelError::SnapshotError("snap failed".to_string());
        assert_eq!(format!("{}", err), "Snapshot error: snap failed");

        let err = WorldModelError::DeviceNotFound("dev1".to_string());
        assert_eq!(format!("{}", err), "Device not found: dev1");

        let err = WorldModelError::TaskNotFound("task1".to_string());
        assert_eq!(format!("{}", err), "Task not found: task1");

        let err = WorldModelError::DesktopError("desktop crash".to_string());
        assert_eq!(format!("{}", err), "Desktop error: desktop crash");

        let err = WorldModelError::EnvironmentError("env fail".to_string());
        assert_eq!(format!("{}", err), "Environment error: env fail");
    }
}
