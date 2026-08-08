use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::{DatabaseError, Result};
use crate::storage::StorageProvider;

const REGISTRY_FILE: &str = "_backup_registry.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupEntry {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub filename: String,
    pub size: u64,
}

#[derive(Debug)]
pub struct BackupManager {
    backup_dir: PathBuf,
}

impl BackupManager {
    pub fn new(backup_dir: impl Into<PathBuf>) -> Self {
        Self {
            backup_dir: backup_dir.into(),
        }
    }

    fn registry_path(&self) -> PathBuf {
        self.backup_dir.join(REGISTRY_FILE)
    }

    fn read_registry(&self) -> Result<Vec<BackupEntry>> {
        let path = self.registry_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let data = std::fs::read_to_string(&path)
            .map_err(|e| DatabaseError::BackupFailed(format!("Failed to read registry: {}", e)))?;
        serde_json::from_str(&data)
            .map_err(|e| DatabaseError::BackupFailed(format!("Failed to parse registry: {}", e)))
    }

    fn write_registry(&self, entries: &[BackupEntry]) -> Result<()> {
        let data = serde_json::to_string_pretty(entries).map_err(|e| {
            DatabaseError::BackupFailed(format!("Failed to serialize registry: {}", e))
        })?;
        // SECURITY: Atomic write — write to temp file first, then rename.
        // Prevents registry corruption if crash occurs during write.
        let path = self.registry_path();
        let temp_path = path.with_extension("json.tmp");
        std::fs::write(&temp_path, &data).map_err(|e| {
            DatabaseError::BackupFailed(format!("Failed to write temp registry: {}", e))
        })?;
        std::fs::rename(&temp_path, &path).map_err(|e| {
            // Cleanup temp file on rename failure
            let _ = std::fs::remove_file(&temp_path);
            DatabaseError::BackupFailed(format!("Failed to rename registry: {}", e))
        })
    }

    pub async fn create_backup(
        &self,
        provider: &dyn StorageProvider,
        name: &str,
    ) -> Result<BackupEntry> {
        let backup_dir = self.backup_dir.clone();
        tokio::task::spawn_blocking(move || {
            if !backup_dir.exists() {
                std::fs::create_dir_all(&backup_dir).map_err(|e| {
                    DatabaseError::BackupFailed(format!("Failed to create backup dir: {}", e))
                })?;
            }
            Ok::<(), DatabaseError>(())
        })
        .await
        .map_err(|e| DatabaseError::BackupFailed(format!("Task join error: {}", e)))??;

        let timestamp = chrono::Utc::now();
        let filename = format!(
            "{}_{}.db",
            sanitize_name(name),
            timestamp.format("%Y%m%d_%H%M%S")
        );
        let backup_path = self.backup_dir.join(&filename);
        let path_str = backup_path
            .to_str()
            .ok_or_else(|| DatabaseError::BackupFailed("Invalid backup path".into()))?;

        provider.backup(path_str).await?;

        let size = {
            let path = backup_path.clone();
            tokio::task::spawn_blocking(move || {
                std::fs::metadata(&path)
                    .map_err(|e| {
                        DatabaseError::BackupFailed(format!("Failed to read backup size: {}", e))
                    })
                    .map(|m| m.len())
            })
            .await
            .map_err(|e| DatabaseError::BackupFailed(format!("Task join error: {}", e)))??
        };

        let entry = BackupEntry {
            name: name.to_string(),
            timestamp,
            filename,
            size,
        };

        let mut registry = self.read_registry()?;
        registry.push(entry.clone());
        self.write_registry(&registry)?;

        Ok(entry)
    }

    pub fn list_backups(&self) -> Result<Vec<BackupEntry>> {
        let mut registry = self.read_registry()?;
        registry.sort_by_key(|b| std::cmp::Reverse(b.timestamp));
        Ok(registry)
    }

    pub async fn restore(&self, provider: &dyn StorageProvider, name: &str) -> Result<()> {
        let registry = self.read_registry()?;
        let entries: Vec<&BackupEntry> = registry.iter().filter(|e| e.name == name).collect();

        let entry = entries
            .into_iter()
            .max_by_key(|e| e.timestamp)
            .ok_or(DatabaseError::NotFound)?;

        // SECURITY: Validate the filename from registry to prevent path traversal
        // in case the registry was tampered with.
        let filename = &entry.filename;
        if filename.contains("..") || filename.starts_with('/') || filename.starts_with('\\') {
            return Err(DatabaseError::BackupFailed(
                "Registry entry filename contains path traversal".into(),
            ));
        }
        // Only allow alphanumeric, underscore, hyphen, dot, and space in filenames
        if !filename
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ' ')
        {
            return Err(DatabaseError::BackupFailed(
                "Registry entry filename contains invalid characters".into(),
            ));
        }

        let backup_path = self.backup_dir.join(filename);
        let path_str = backup_path
            .to_str()
            .ok_or_else(|| DatabaseError::BackupFailed("Invalid backup path".into()))?;

        provider.restore(path_str).await
    }

    pub fn delete_backup(&self, name: &str) -> Result<()> {
        let mut registry = self.read_registry()?;
        let before = registry.len();
        registry.retain(|e| e.name != name);
        let removed = before - registry.len();

        if removed == 0 {
            return Err(DatabaseError::NotFound);
        }

        // Remove orphaned backup files
        let retained_names: std::collections::HashSet<&str> =
            registry.iter().map(|e| e.filename.as_str()).collect();
        if let Ok(entries) = std::fs::read_dir(&self.backup_dir) {
            for entry in entries.flatten() {
                if let Some(fname) = entry.file_name().to_str() {
                    if fname.ends_with(".db") && !retained_names.contains(fname) {
                        let path = entry.path();
                        let fname_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        if fname_stem.starts_with(&format!("{}_", sanitize_name(name))) {
                            std::fs::remove_file(&path).ok();
                        }
                    }
                }
            }
        }

        self.write_registry(&registry)
    }

    pub fn backup_dir(&self) -> &PathBuf {
        &self.backup_dir
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, DatabaseKind};
    use crate::sqlite::SqliteDatabase;

    #[tokio::test]
    async fn test_backup_manager_create_list() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let manager = BackupManager::new(tmp_dir.path().to_path_buf());

        let db = SqliteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Sqlite,
            path: Some(":memory:".into()),
            ..Default::default()
        };
        db.connect(&config).await.unwrap();
        db.set("key", b"val").await.unwrap();

        let entry = manager.create_backup(&db, "test-backup").await.unwrap();
        assert_eq!(entry.name, "test-backup");

        let backups = manager.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].name, "test-backup");
    }

    #[tokio::test]
    async fn test_backup_manager_restore() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let manager = BackupManager::new(tmp_dir.path().to_path_buf());

        let db = SqliteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Sqlite,
            path: Some(":memory:".into()),
            ..Default::default()
        };
        db.connect(&config).await.unwrap();
        db.set("restore_key", b"restore_val").await.unwrap();

        manager.create_backup(&db, "restore-test").await.unwrap();

        db.delete("restore_key").await.unwrap();
        assert!(!db.exists("restore_key").await.unwrap());

        manager.restore(&db, "restore-test").await.unwrap();
        assert_eq!(
            db.get("restore_key").await.unwrap(),
            Some(b"restore_val".to_vec())
        );
    }

    #[tokio::test]
    async fn test_backup_manager_delete() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let manager = BackupManager::new(tmp_dir.path().to_path_buf());

        let db = SqliteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Sqlite,
            path: Some(":memory:".into()),
            ..Default::default()
        };
        db.connect(&config).await.unwrap();
        db.set("k", b"v").await.unwrap();

        manager.create_backup(&db, "del-test").await.unwrap();
        assert_eq!(manager.list_backups().unwrap().len(), 1);

        manager.delete_backup("del-test").unwrap();
        assert_eq!(manager.list_backups().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_backup_manager_restore_nonexistent() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let manager = BackupManager::new(tmp_dir.path().to_path_buf());

        let db = SqliteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Sqlite,
            path: Some(":memory:".into()),
            ..Default::default()
        };
        db.connect(&config).await.unwrap();

        let result = manager.restore(&db, "nonexistent").await;
        assert!(matches!(result, Err(DatabaseError::NotFound)));
    }
}
