#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseKind {
    Sqlite,
    Postgres,
    DuckDb,
    Remote,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub kind: DatabaseKind,
    pub path: Option<String>,
    pub url: Option<String>,
    pub remote_url: Option<String>,
    pub pool_size: u32,
    /// SECURITY: Encryption key stored as Zeroizing to prevent memory exposure on drop.
    pub encryption_key: Option<zeroize::Zeroizing<String>>,
    pub backup_dir: Option<String>,
    pub auto_migrate: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            kind: DatabaseKind::Sqlite,
            path: Some("voxy.db".to_string()),
            url: None,
            remote_url: None,
            pool_size: 4,
            encryption_key: None,
            backup_dir: None,
            auto_migrate: true,
        }
    }
}
