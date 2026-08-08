//! Hot reload for settings files.
//!
//! Watches config file changes and triggers reload.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use tracing::info;

use crate::error::Result;

/// File watcher for hot-reloading settings.
pub struct HotReloader {
    path: PathBuf,
    watch_tx: broadcast::Sender<PathBuf>,
}

impl HotReloader {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let (watch_tx, _) = broadcast::channel(16);
        Ok(Self {
            path: path.into(),
            watch_tx,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PathBuf> {
        self.watch_tx.subscribe()
    }

    pub async fn watch_loop(&self, interval: Duration, shutdown_rx: &mut broadcast::Receiver<()>) {
        let mut last_modified = self.get_modified_time();
        let mut ticker = tokio::time::interval(interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let current = self.get_modified_time();
                    if let (Some(cur), Some(last)) = (current, last_modified) {
                        if cur > last {
                            info!("Settings file changed: {}", self.path.display());
                            last_modified = Some(cur);
                            let _ = self.watch_tx.send(self.path.clone());
                        }
                    } else if current.is_some() {
                        last_modified = current;
                    }
                }
                _ = shutdown_rx.recv() => break,
            }
        }
    }

    fn get_modified_time(&self) -> Option<std::time::SystemTime> {
        std::fs::metadata(&self.path)
            .ok()
            .and_then(|m| m.modified().ok())
    }
}

/// Coordinates multiple file watchers.
pub struct HotReloadManager {
    reloaders: Vec<Arc<HotReloader>>,
}

impl HotReloadManager {
    pub fn new() -> Self {
        Self {
            reloaders: Vec::new(),
        }
    }

    pub fn watch(&mut self, path: impl Into<PathBuf>) -> Result<()> {
        self.reloaders.push(Arc::new(HotReloader::new(path)?));
        Ok(())
    }

    pub async fn start_all(&self, interval: Duration, shutdown_rx: &mut broadcast::Receiver<()>) {
        for reloader in &self.reloaders {
            let r = reloader.clone();
            let mut rx = shutdown_rx.resubscribe();
            tokio::spawn(async move { r.watch_loop(interval, &mut rx).await });
        }
        info!("Hot reload manager started with {} watchers", self.reloaders.len());
    }
}

impl Default for HotReloadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hot_reloader_creation() {
        assert!(HotReloader::new("test.toml").is_ok());
    }

    #[test]
    fn hot_reloader_subscribe() {
        let r = HotReloader::new("test.toml").unwrap();
        let _rx = r.subscribe();
    }

    #[test]
    fn hot_reload_manager_default() {
        let m = HotReloadManager::new();
        assert!(m.reloaders.is_empty());
    }
}
