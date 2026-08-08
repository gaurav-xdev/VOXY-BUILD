//! Plugin manifest types.

use serde::{Deserialize, Serialize};

/// Plugin manifest defining metadata and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub capabilities: Vec<String>,
    pub entry_point: String,
    pub min_voxy_version: Option<String>,
    pub max_voxy_version: Option<String>,
    pub dependencies: Vec<PluginDependency>,
    pub tools: Vec<ToolDefinition>,
}

impl PluginManifest {
    /// Validate the manifest.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Plugin ID is required".to_string());
        }
        if self.name.is_empty() {
            return Err("Plugin name is required".to_string());
        }
        if self.version.is_empty() {
            return Err("Plugin version is required".to_string());
        }
        if self.entry_point.contains("..") {
            return Err("entry_point must not contain '..' (path traversal blocked)".to_string());
        }
        if std::path::Path::new(&self.entry_point).is_absolute() {
            return Err(
                "entry_point must be a relative path (absolute paths not allowed)".to_string(),
            );
        }
        if !self.entry_point.ends_with(".dll")
            && !self.entry_point.ends_with(".so")
            && !self.entry_point.ends_with(".dylib")
        {
            return Err("entry_point must be a shared library (.dll, .so, or .dylib)".to_string());
        }
        Ok(())
    }
}

/// Plugin dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub id: String,
    pub version: String,
    pub optional: bool,
}

/// Tool definition provided by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub required_permissions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_validation() {
        let manifest = PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            description: None,
            permissions: vec![],
            capabilities: vec![],
            entry_point: "libtest.so".to_string(),
            min_voxy_version: None,
            max_voxy_version: None,
            dependencies: vec![],
            tools: vec![],
        };
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn manifest_validation_fails_without_id() {
        let manifest = PluginManifest {
            id: String::new(),
            name: "Test".to_string(),
            version: "0.1.0".to_string(),
            author: None,
            description: None,
            permissions: vec![],
            capabilities: vec![],
            entry_point: "libtest.so".to_string(),
            min_voxy_version: None,
            max_voxy_version: None,
            dependencies: vec![],
            tools: vec![],
        };
        assert!(manifest.validate().is_err());
    }
}
