use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Store error: {0}")]
    StoreError(String),

    #[error("Retrieval error: {0}")]
    RetrievalError(String),

    #[error("Consolidation error: {0}")]
    ConsolidationError(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Forgetting error: {0}")]
    ForgettingError(String),

    #[error("Graph error: {0}")]
    GraphError(String),

    #[error("Hermes error: {0}")]
    HermesError(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Version conflict: {0}")]
    VersionConflict(String),

    #[error("Storage full: {0}")]
    StorageFull(String),

    #[error("Importance error: {0}")]
    ImportanceError(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("SQLite error: {0}")]
    SqliteError(String),
}

impl From<rusqlite::Error> for MemoryError {
    fn from(err: rusqlite::Error) -> Self {
        MemoryError::SqliteError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MemoryError>;
