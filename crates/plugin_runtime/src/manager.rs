//! Plugin lifecycle management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{PluginContext, PluginError, PluginManifest, PluginState};

struct PluginEntry {
    _manifest: PluginManifest,
    state: PluginState,
    _context: PluginContext,
}

/// Plugin manager handling lifecycle.
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, PluginEntry>>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn load_plugin(&self, manifest: PluginManifest) -> crate::Result<()> {
        manifest.validate().map_err(PluginError::InvalidManifest)?;

        let context = PluginContext::new(&manifest.id);
        let entry = PluginEntry {
            _manifest: manifest.clone(),
            state: PluginState::Running,
            _context: context,
        };

        self.plugins
            .write()
            .await
            .insert(manifest.id.clone(), entry);
        tracing::info!("Plugin loaded: {}", manifest.id);
        Ok(())
    }

    pub async fn unload_plugin(&self, id: &str) -> crate::Result<()> {
        self.plugins
            .write()
            .await
            .remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        tracing::info!("Plugin unloaded: {}", id);
        Ok(())
    }

    pub async fn get_state(&self, id: &str) -> Option<PluginState> {
        self.plugins.read().await.get(id).map(|e| e.state)
    }

    pub async fn list_plugins(&self) -> Vec<String> {
        self.plugins.read().await.keys().cloned().collect()
    }

    pub async fn is_loaded(&self, id: &str) -> bool {
        self.plugins.read().await.contains_key(id)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_plugins_empty() {
        let mgr = PluginManager::new();
        assert!(mgr.list_plugins().await.is_empty());
    }
}
