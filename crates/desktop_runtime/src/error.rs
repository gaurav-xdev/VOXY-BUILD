use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Tray error: {0}")]
    Tray(String),
    #[error("Registry error: {0}")]
    Registry(String),
    #[error("Clipboard error: {0}")]
    Clipboard(String),
    #[error("Window manager error: {0}")]
    WindowManager(String),
    #[error("Notification error: {0}")]
    Notification(String),
    #[error("Download error: {0}")]
    Download(String),
    #[error("Settings error: {0}")]
    Settings(String),
    #[error("File watcher error: {0}")]
    FileWatcher(String),
    #[error("Shortcut error: {0}")]
    Shortcut(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<voxy_shared::VoxyError> for RuntimeError {
    fn from(e: voxy_shared::VoxyError) -> Self {
        RuntimeError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

// VoxyError conversion will be added when integrated with main app
