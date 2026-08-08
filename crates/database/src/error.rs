use thiserror::Error;

pub type Result<T> = std::result::Result<T, DatabaseError>;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Backup failed: {0}")]
    BackupFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Not found")]
    NotFound,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

#[cfg(feature = "sqlite")]
impl From<rusqlite::Error> for DatabaseError {
    fn from(e: rusqlite::Error) -> Self {
        DatabaseError::QueryFailed(e.to_string())
    }
}
