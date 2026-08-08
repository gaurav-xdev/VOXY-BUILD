use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Provider error: {provider}: {message}")]
    ProviderError { provider: String, message: String },

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Assembly error: {0}")]
    AssemblyError(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Context not found: {0}")]
    NotFound(String),

    #[error("Stale context: {0}")]
    Stale(String),

    #[error("Cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, ContextError>;
