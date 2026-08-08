#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Unsupported capability: {0}")]
    UnsupportedCapability(String),
    #[error("Provider is busy")]
    ProviderBusy,
    #[error("Capability not available: {0}")]
    CapabilityNotAvailable(String),
    #[error("Provider unavailable")]
    ProviderUnavailable,
}

pub type Result<T> = std::result::Result<T, ProviderError>;
