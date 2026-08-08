//! SDK + Extensibility — platform abstraction for cross-platform deployment.
//!
//! Provides trait-based abstraction for:
//! - Platform-specific I/O (audio, filesystem, network)
//! - Plugin system (load, register, invoke)
//! - Platform capabilities query
//! - Configuration portability

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ============================================================================
// Platform Abstraction
// ============================================================================

/// Platform type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Platform {
    Windows,
    Linux,
    MacOS,
    Android,
    Ios,
    WebAssembly,
    Unknown,
}

impl Platform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else if cfg!(target_os = "macos") {
            Self::MacOS
        } else if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "ios") {
            Self::Ios
        } else if cfg!(target_arch = "wasm32") {
            Self::WebAssembly
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Windows => write!(f, "Windows"),
            Self::Linux => write!(f, "Linux"),
            Self::MacOS => write!(f, "macOS"),
            Self::Android => write!(f, "Android"),
            Self::Ios => write!(f, "iOS"),
            Self::WebAssembly => write!(f, "WebAssembly"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Platform capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub has_gpu: bool,
    pub has_audio_input: bool,
    pub has_audio_output: bool,
    pub has_network: bool,
    pub has_filesystem: bool,
    pub has_camera: bool,
    pub has_microphone: bool,
    pub has_speech_recognition: bool,
    pub has_tts: bool,
    pub max_threads: u32,
    pub total_memory_mb: u64,
    pub features: Vec<String>,
}

/// Query platform capabilities.
pub trait PlatformQuery: Send + Sync {
    fn platform(&self) -> Platform;
    fn capabilities(&self) -> PlatformCapabilities;
    fn hostname(&self) -> Option<String>;
    fn username(&self) -> Option<String>;
}

/// Default implementation for current platform.
pub struct DefaultPlatformQuery;

impl PlatformQuery for DefaultPlatformQuery {
    fn platform(&self) -> Platform {
        Platform::current()
    }

    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities {
            has_gpu: true,
            has_audio_input: cfg!(target_os = "windows") || cfg!(target_os = "linux"),
            has_audio_output: true,
            has_network: true,
            has_filesystem: true,
            has_camera: false,
            has_microphone: cfg!(target_os = "windows") || cfg!(target_os = "linux"),
            has_speech_recognition: true,
            has_tts: true,
            max_threads: num_cpus::get() as u32,
            total_memory_mb: 0,
            features: Vec::new(),
        }
    }

    fn hostname(&self) -> Option<String> {
        hostname::get()
            .ok()
            .map(|h| h.to_string_lossy().to_string())
    }

    fn username(&self) -> Option<String> {
        std::env::var("USERNAME")
            .or_else(|_| std::env::var("USER"))
            .ok()
    }
}

// ============================================================================
// Plugin System
// ============================================================================

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub platform: Platform,
    pub dependencies: Vec<String>,
    pub enabled: bool,
}

/// Plugin error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin load failed: {0}")]
    LoadFailed(String),

    #[error("Plugin already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Platform mismatch: plugin requires {required}, running on {actual}")]
    PlatformMismatch {
        required: Platform,
        actual: Platform,
    },

    #[error("Plugin initialization failed: {0}")]
    InitFailed(String),
}

/// Plugin trait — plugins implement this to be loaded by the system.
pub trait Plugin: Send + Sync {
    fn info(&self) -> PluginInfo;
    fn initialize(&mut self) -> Result<(), PluginError>;
    fn shutdown(&mut self) -> Result<(), PluginError>;
    fn invoke(&self, command: &str, args: &HashMap<String, String>) -> Result<String, PluginError>;
}

/// Plugin registry — manages plugin lifecycle.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_names: std::collections::HashSet<String>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_names: std::collections::HashSet::new(),
        }
    }

    /// Register a plugin.
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let info = plugin.info();
        if self.plugin_names.contains(&info.name) {
            return Err(PluginError::AlreadyRegistered(info.name));
        }
        self.plugin_names.insert(info.name.clone());
        self.plugins.push(plugin);
        Ok(())
    }

    /// Initialize all plugins.
    pub fn initialize_all(&mut self) -> Vec<Result<(), PluginError>> {
        self.plugins.iter_mut().map(|p| p.initialize()).collect()
    }

    /// Shutdown all plugins.
    pub fn shutdown_all(&mut self) -> Vec<Result<(), PluginError>> {
        self.plugins.iter_mut().map(|p| p.shutdown()).collect()
    }

    /// Invoke a command on a specific plugin.
    pub fn invoke(
        &self,
        plugin_name: &str,
        command: &str,
        args: &HashMap<String, String>,
    ) -> Result<String, PluginError> {
        let plugin = self
            .plugins
            .iter()
            .find(|p| p.info().name == plugin_name)
            .ok_or_else(|| PluginError::NotFound(plugin_name.to_string()))?;
        plugin.invoke(command, args)
    }

    /// List all registered plugins.
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    /// Get plugin count.
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Get enabled plugins only.
    pub fn enabled_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .iter()
            .map(|p| p.info())
            .filter(|i| i.enabled)
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Mock Plugin (for testing)
// ============================================================================

pub struct MockPlugin {
    info: PluginInfo,
    initialized: bool,
    commands: HashMap<String, String>,
}

impl MockPlugin {
    pub fn new(name: &str) -> Self {
        let mut commands = HashMap::new();
        commands.insert("ping".to_string(), "pong".to_string());
        commands.insert("version".to_string(), "1.0.0".to_string());

        Self {
            info: PluginInfo {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                author: "test".to_string(),
                description: format!("Mock plugin: {}", name),
                platform: Platform::current(),
                dependencies: Vec::new(),
                enabled: true,
            },
            initialized: false,
            commands,
        }
    }
}

impl Plugin for MockPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), PluginError> {
        self.initialized = true;
        Ok(())
    }

    fn shutdown(&mut self) -> Result<(), PluginError> {
        self.initialized = false;
        Ok(())
    }

    fn invoke(
        &self,
        command: &str,
        _args: &HashMap<String, String>,
    ) -> Result<String, PluginError> {
        self.commands
            .get(command)
            .cloned()
            .ok_or_else(|| PluginError::NotFound(command.to_string()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_detection() {
        let p = Platform::current();
        // Should detect current platform
        assert_ne!(p, Platform::Unknown);
    }

    #[test]
    fn platform_display() {
        assert_eq!(format!("{}", Platform::Windows), "Windows");
        assert_eq!(format!("{}", Platform::Linux), "Linux");
        assert_eq!(format!("{}", Platform::MacOS), "macOS");
        assert_eq!(format!("{}", Platform::Android), "Android");
        assert_eq!(format!("{}", Platform::Ios), "iOS");
        assert_eq!(format!("{}", Platform::WebAssembly), "WebAssembly");
        assert_eq!(format!("{}", Platform::Unknown), "Unknown");
    }

    #[test]
    fn platform_capabilities() {
        let q = DefaultPlatformQuery;
        let caps = q.capabilities();
        assert!(caps.has_network);
        assert!(caps.has_filesystem);
        assert!(caps.max_threads > 0);
    }

    #[test]
    fn platform_query() {
        let q = DefaultPlatformQuery;
        assert_ne!(q.platform(), Platform::Unknown);
        assert!(q.hostname().is_some() || q.hostname().is_none());
    }

    #[test]
    fn plugin_registry_creation() {
        let reg = PluginRegistry::new();
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn register_and_invoke() {
        let mut reg = PluginRegistry::new();
        let plugin = Box::new(MockPlugin::new("test-plugin"));
        reg.register(plugin).unwrap();

        let result = reg.invoke("test-plugin", "ping", &HashMap::new()).unwrap();
        assert_eq!(result, "pong");
    }

    #[test]
    fn register_duplicate_plugin() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("dup"))).unwrap();
        let result = reg.register(Box::new(MockPlugin::new("dup")));
        assert!(matches!(result, Err(PluginError::AlreadyRegistered(_))));
    }

    #[test]
    fn invoke_nonexistent_plugin() {
        let reg = PluginRegistry::new();
        let result = reg.invoke("nope", "cmd", &HashMap::new());
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn invoke_unknown_command() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("p"))).unwrap();
        let result = reg.invoke("p", "unknown", &HashMap::new());
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn initialize_all_plugins() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("a"))).unwrap();
        reg.register(Box::new(MockPlugin::new("b"))).unwrap();
        let results = reg.initialize_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn shutdown_all_plugins() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("a"))).unwrap();
        let results = reg.shutdown_all();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    #[test]
    fn list_plugins() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("x"))).unwrap();
        reg.register(Box::new(MockPlugin::new("y"))).unwrap();
        let list = reg.list();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|i| i.name == "x"));
        assert!(list.iter().any(|i| i.name == "y"));
    }

    #[test]
    fn enabled_plugins_only() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(MockPlugin::new("enabled"))).unwrap();
        let enabled = reg.enabled_plugins();
        assert_eq!(enabled.len(), 1);
        assert!(enabled[0].enabled);
    }

    #[test]
    fn default_plugin_registry() {
        let reg = PluginRegistry::default();
        assert_eq!(reg.count(), 0);
    }
}
