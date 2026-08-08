//! Capability manifest types.

use serde::{Deserialize, Serialize};

/// Machine-readable capability manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityManifest {
    /// Unique identifier (e.g., "automation.openclaw").
    pub id: String,
    /// Manifest version.
    pub version: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Required permissions.
    pub permissions: Vec<ManifestPermission>,
    /// Dependencies on other capabilities.
    pub dependencies: Vec<ManifestDependency>,
    /// Whether this capability supports health checks.
    pub health_check: bool,
    /// Resource requirements.
    pub resources: ManifestResources,
    /// Capability category.
    pub category: ManifestCategory,
}

/// A permission required by a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPermission {
    /// Permission identifier (e.g., "mouse.click").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Risk level (low, medium, high, critical).
    pub risk_level: String,
    /// Whether user consent is required.
    pub requires_consent: bool,
}

/// A dependency on another capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestDependency {
    /// Capability ID.
    pub id: String,
    /// Required version (semver).
    pub version: String,
    /// Whether this dependency is optional.
    pub optional: bool,
}

/// Resource requirements for a capability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestResources {
    /// Maximum memory in MB.
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU percentage.
    pub max_cpu_percent: Option<f64>,
    /// Maximum IPC message size in bytes.
    pub max_ipc_message_size: Option<usize>,
    /// Maximum number of event handlers.
    pub max_event_handlers: Option<usize>,
    /// Timeout in seconds.
    pub timeout_seconds: Option<u64>,
}

/// Capability category.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManifestCategory {
    Runtime,
    Provider,
    Plugin,
    Tool,
    Service,
}

impl CapabilityManifest {
    /// Validate the manifest.
    pub fn validate(&self) -> crate::Result<()> {
        if self.id.is_empty() {
            return Err(crate::ManifestError::ValidationError(
                "Manifest ID is required".to_string(),
            ));
        }
        if self.version.is_empty() {
            return Err(crate::ManifestError::ValidationError(
                "Manifest version is required".to_string(),
            ));
        }
        if self.name.is_empty() {
            return Err(crate::ManifestError::ValidationError(
                "Manifest name is required".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_validation() {
        let manifest = CapabilityManifest {
            id: "test.capability".to_string(),
            version: "1.0.0".to_string(),
            name: "Test Capability".to_string(),
            description: None,
            permissions: vec![],
            dependencies: vec![],
            health_check: true,
            resources: ManifestResources::default(),
            category: ManifestCategory::Runtime,
        };
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn manifest_validation_fails_without_id() {
        let manifest = CapabilityManifest {
            id: String::new(),
            version: "1.0.0".to_string(),
            name: "Test".to_string(),
            description: None,
            permissions: vec![],
            dependencies: vec![],
            health_check: false,
            resources: ManifestResources::default(),
            category: ManifestCategory::Runtime,
        };
        assert!(manifest.validate().is_err());
    }
}
