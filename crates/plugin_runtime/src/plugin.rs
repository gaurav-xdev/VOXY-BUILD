//! Plugin context and lifecycle trait.

/// Plugin state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Installed,
    Loading,
    Running,
    Paused,
    Error,
    Unloaded,
}

/// Plugin context providing services.
pub struct PluginContext {
    pub plugin_id: String,
}

impl PluginContext {
    pub fn new(plugin_id: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
        }
    }
}

/// Plugin lifecycle trait.
#[async_trait::async_trait]
pub trait PluginLifecycle: Send + Sync {
    async fn on_load(&self) -> crate::Result<()>;
    async fn on_activate(&self) -> crate::Result<()>;
    async fn on_deactivate(&self) -> crate::Result<()>;
    async fn on_unload(&self) -> crate::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_state_variants() {
        assert_eq!(PluginState::Installed, PluginState::Installed);
        assert_ne!(PluginState::Running, PluginState::Paused);
    }
}
