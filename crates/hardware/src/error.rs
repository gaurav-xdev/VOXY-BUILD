#[derive(Debug, thiserror::Error)]
pub enum HardwareError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Device busy: {0}")]
    DeviceBusy(String),
    #[error("Device error: {0}")]
    DeviceError(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    #[error("Audio error: {0}")]
    AudioError(String),
    #[error("Video error: {0}")]
    VideoError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    IO(std::io::Error),
}

pub type Result<T> = std::result::Result<T, HardwareError>;
