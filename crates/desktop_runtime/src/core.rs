use crate::error::Result;
use crate::settings::SettingsManager;
use crate::window_manager::WindowTracker;
use tracing::info;

pub struct RuntimeConfig {
    pub app_name: String,
    pub settings_path: Option<String>,
}

impl RuntimeConfig {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            settings_path: None,
        }
    }
}

pub struct DesktopRuntime {
    config: RuntimeConfig,
    settings: std::sync::Arc<SettingsManager>,
    window_tracker: std::sync::Arc<WindowTracker>,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl DesktopRuntime {
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        let settings = std::sync::Arc::new(SettingsManager::new()?);
        let window_tracker = std::sync::Arc::new(WindowTracker::new());
        Ok(Self {
            config,
            settings,
            window_tracker,
            shutdown_tx: None,
        })
    }

    pub fn settings(&self) -> &SettingsManager {
        &self.settings
    }
    pub fn window_tracker(&self) -> &WindowTracker {
        &self.window_tracker
    }

    pub async fn start(&mut self) -> Result<()> {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());
        info!("{} desktop runtime started", self.config.app_name);
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
        info!("{} desktop runtime shutdown", self.config.app_name);
        Ok(())
    }
}
