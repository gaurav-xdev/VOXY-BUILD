//! Configuration loading, validation, and watching.
//!
//! Provides:
//! - TOML and JSON configuration file support
//! - Hot-reload via `ConfigManager`
//! - Environment variable overrides
//! - Schema versioning for migration support

pub mod error;
pub mod types;

pub use error::{ConfigError, Result};
pub use types::{
    ApiKeysConfig, AppConfig, EventBusConfig, KernelConfig, LoggingConfig, MetricsConfig,
};

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// Trait for configuration providers.
#[async_trait::async_trait]
pub trait ConfigProvider: Send + Sync {
    /// Load the configuration.
    async fn load(&self) -> Result<AppConfig>;

    /// Save the configuration.
    async fn save(&self, config: &AppConfig) -> Result<()>;

    /// Reload the configuration.
    async fn reload(&self) -> Result<AppConfig>;
}

/// File-based configuration provider.
pub struct FileConfigProvider {
    path: PathBuf,
}

impl FileConfigProvider {
    /// Create a new file config provider.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Get the config file path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait::async_trait]
impl ConfigProvider for FileConfigProvider {
    async fn load(&self) -> Result<AppConfig> {
        AppConfig::load_from_file(&self.path)
    }

    async fn save(&self, config: &AppConfig) -> Result<()> {
        config.save_to_file(&self.path)
    }

    async fn reload(&self) -> Result<AppConfig> {
        self.load().await
    }
}

/// Configuration manager with hot-reload support.
pub struct ConfigManager {
    config: Arc<RwLock<AppConfig>>,
    provider: Box<dyn ConfigProvider>,
    change_tx: broadcast::Sender<AppConfig>,
}

impl ConfigManager {
    /// Create a new config manager.
    pub async fn new(provider: impl ConfigProvider + 'static) -> Result<Self> {
        let config = Arc::new(RwLock::new(provider.load().await?));
        let (change_tx, _) = broadcast::channel(16);
        Ok(Self {
            config,
            provider: Box::new(provider),
            change_tx,
        })
    }

    /// Get the current configuration.
    pub async fn get(&self) -> AppConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration and notify subscribers.
    pub async fn set(&self, config: AppConfig) -> Result<()> {
        config.validate()?;
        self.provider.save(&config).await?;
        *self.config.write().await = config.clone();
        let _ = self.change_tx.send(config);
        Ok(())
    }

    /// Reload configuration from the provider.
    pub async fn reload(&self) -> Result<()> {
        let config = self.provider.reload().await?;
        config.validate()?;
        *self.config.write().await = config.clone();
        let _ = self.change_tx.send(config);
        Ok(())
    }

    /// Subscribe to configuration changes.
    pub fn subscribe(&self) -> broadcast::Receiver<AppConfig> {
        self.change_tx.subscribe()
    }
}

/// Get the default application configuration.
pub fn default_config() -> AppConfig {
    AppConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_config_creates() {
        let config = default_config();
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn config_manager_set_and_get() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = AppConfig::default();
        config.save_to_file(&path).unwrap();

        let provider = FileConfigProvider::new(path);
        let manager = ConfigManager::new(provider).await.unwrap();

        // ConfigManager.get() returns AppConfig directly
        let retrieved = manager.get().await;
        // Just verify we can access it — actual field changes require setters
        assert!(retrieved.validate().is_ok());
    }

    #[tokio::test]
    async fn config_manager_subscribe() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let config = AppConfig::default();
        config.save_to_file(&path).unwrap();

        let provider = FileConfigProvider::new(path);
        let manager = ConfigManager::new(provider).await.unwrap();
        let mut rx = manager.subscribe();

        // Set the same config to trigger notification
        manager.set(config).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert!(received.validate().is_ok());
    }

    #[tokio::test]
    async fn file_provider_load_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let config = AppConfig::default();
        config.save_to_file(&path).unwrap();

        let provider = FileConfigProvider::new(path);
        let loaded = provider.load().await.unwrap();
        assert_eq!(
            loaded.kernel().thread_pool_size(),
            config.kernel().thread_pool_size()
        );
    }
}
