#[derive(Debug, thiserror::Error)]
pub enum HomeError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Scene not found: {0}")]
    SceneNotFound(String),
    #[error("Project not found: {0}")]
    ProjectNotFound(String),
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    #[error("Home not initialized")]
    NotInitialized,
    #[error("Device already registered: {0}")]
    DeviceAlreadyRegistered(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error(transparent)]
    Hardware(#[from] voxy_hardware::HardwareError),
}

pub type Result<T> = std::result::Result<T, HomeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        let err = HomeError::RoomNotFound("living_room".into());
        assert_eq!(err.to_string(), "Room not found: living_room");

        let err = HomeError::NotInitialized;
        assert_eq!(err.to_string(), "Home not initialized");
    }

    #[test]
    fn test_error_trait() {
        let err = HomeError::InvalidConfig("bad config".into());
        let err_ref: &dyn std::error::Error = &err;
        assert_eq!(err_ref.to_string(), "Invalid configuration: bad config");
    }

    #[test]
    fn test_all_variants() {
        let variants: Vec<HomeError> = vec![
            HomeError::RoomNotFound("a".into()),
            HomeError::DeviceNotFound("b".into()),
            HomeError::SceneNotFound("c".into()),
            HomeError::ProjectNotFound("d".into()),
            HomeError::EnvironmentNotFound("e".into()),
            HomeError::NotInitialized,
            HomeError::DeviceAlreadyRegistered("f".into()),
            HomeError::InvalidConfig("g".into()),
        ];
        for v in variants {
            let _ = v.to_string();
        }
    }
}
