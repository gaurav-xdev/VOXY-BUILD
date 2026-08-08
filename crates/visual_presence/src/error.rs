use thiserror::Error;

#[derive(Error, Debug)]
pub enum VisualPresenceError {
    #[error("GPU error: {0}")]
    Gpu(String),

    #[error("Window error: {0}")]
    Window(String),

    #[error("Shader error: {0}")]
    Shader(String),

    #[error("Particle engine error: {0}")]
    ParticleEngine(String),

    #[error("Animation error: {0}")]
    Animation(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Head tracking error: {0}")]
    HeadTracking(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Rendering error: {0}")]
    Rendering(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, VisualPresenceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VisualPresenceError::Gpu("test error".to_string());
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        assert!(json_err.is_err());
        let err: VisualPresenceError = json_err.unwrap_err().into();
        assert!(matches!(err, VisualPresenceError::Serialization(_)));
    }
}
