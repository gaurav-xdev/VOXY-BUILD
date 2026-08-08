//! Plugin lifecycle, sandboxing, and SDK.

pub mod error;
pub mod manager;
pub mod manifest;
pub mod plugin;

pub use error::{PluginError, Result};
pub use manager::PluginManager;
pub use manifest::{PluginDependency, PluginManifest, ToolDefinition};
pub use plugin::{PluginContext, PluginLifecycle, PluginState};
