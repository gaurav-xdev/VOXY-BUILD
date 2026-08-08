use crate::error::Result;
use crate::storage::{Migration, StorageProvider};

#[derive(Debug)]
pub struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            migrations: Vec::new(),
        }
    }

    pub fn register(&mut self, migration: Migration) {
        self.migrations.push(migration);
    }

    pub fn register_all(&mut self, migrations: Vec<Migration>) {
        self.migrations.extend(migrations);
    }

    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }

    pub async fn run(&self, provider: &dyn StorageProvider) -> Result<()> {
        provider.run_migrations(&self.migrations).await
    }

    pub async fn status(&self, provider: &dyn StorageProvider) -> Result<MigrationStatus> {
        let applied = provider.applied_migrations().await?;
        let applied_versions: std::collections::HashSet<i64> =
            applied.iter().map(|m| m.version).collect();

        let mut pending = Vec::new();
        let mut completed = Vec::new();

        for migration in &self.migrations {
            if applied_versions.contains(&migration.version) {
                completed.push(migration.version);
            } else {
                pending.push(migration.version);
            }
        }

        Ok(MigrationStatus {
            total: self.migrations.len(),
            applied: completed.len(),
            pending: pending.len(),
            pending_versions: pending,
            applied_versions: completed,
        })
    }
}

impl Default for MigrationRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MigrationStatus {
    pub total: usize,
    pub applied: usize,
    pub pending: usize,
    pub pending_versions: Vec<i64>,
    pub applied_versions: Vec<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DatabaseConfig, DatabaseKind};
    use crate::sqlite::SqliteDatabase;

    #[tokio::test]
    async fn test_register_migration() {
        let mut runner = MigrationRunner::new();
        assert_eq!(runner.migrations().len(), 0);

        runner.register(Migration {
            version: 1,
            name: "test",
            sql: "CREATE TABLE IF NOT EXISTS t (id INTEGER)",
        });
        assert_eq!(runner.migrations().len(), 1);
    }

    #[tokio::test]
    async fn test_register_all() {
        let mut runner = MigrationRunner::new();
        runner.register_all(vec![
            Migration {
                version: 1,
                name: "v1",
                sql: "CREATE TABLE IF NOT EXISTS t1 (id INTEGER)",
            },
            Migration {
                version: 2,
                name: "v2",
                sql: "CREATE TABLE IF NOT EXISTS t2 (id INTEGER)",
            },
        ]);
        assert_eq!(runner.migrations().len(), 2);
    }

    #[tokio::test]
    async fn test_migration_status() {
        let db = SqliteDatabase::new();
        let config = DatabaseConfig {
            kind: DatabaseKind::Sqlite,
            path: Some(":memory:".into()),
            ..Default::default()
        };
        db.connect(&config).await.unwrap();

        let mut runner = MigrationRunner::new();
        runner.register(Migration {
            version: 1,
            name: "v1",
            sql: "CREATE TABLE IF NOT EXISTS t (id INTEGER)",
        });
        runner.register(Migration {
            version: 2,
            name: "v2",
            sql: "CREATE TABLE IF NOT EXISTS u (id INTEGER)",
        });

        let status = runner.status(&db).await.unwrap();
        assert_eq!(status.total, 2);
        assert_eq!(status.applied, 0);
        assert_eq!(status.pending, 2);

        runner.run(&db).await.unwrap();

        let status = runner.status(&db).await.unwrap();
        assert_eq!(status.total, 2);
        assert_eq!(status.applied, 2);
        assert_eq!(status.pending, 0);
    }
}
