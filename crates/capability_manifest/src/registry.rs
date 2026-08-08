//! Capability manifest registry.

use crate::{CapabilityManifest, ManifestError};
use std::collections::HashMap;

/// Registry for capability manifests.
pub struct ManifestRegistry {
    manifests: HashMap<String, CapabilityManifest>,
}

impl ManifestRegistry {
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
        }
    }

    /// Register a manifest.
    pub fn register(&mut self, manifest: CapabilityManifest) -> crate::Result<()> {
        manifest.validate()?;
        if self.manifests.contains_key(&manifest.id) {
            return Err(ManifestError::DuplicateId(manifest.id));
        }
        self.manifests.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    /// Get a manifest by ID.
    pub fn get(&self, id: &str) -> Option<&CapabilityManifest> {
        self.manifests.get(id)
    }

    /// Check if a manifest exists.
    pub fn has(&self, id: &str) -> bool {
        self.manifests.contains_key(id)
    }

    /// List all manifest IDs.
    pub fn list_ids(&self) -> Vec<&str> {
        self.manifests.keys().map(|s| s.as_str()).collect()
    }

    /// Check if all dependencies of a manifest are satisfied.
    pub fn dependencies_satisfied(&self, id: &str) -> bool {
        if let Some(manifest) = self.manifests.get(id) {
            manifest
                .dependencies
                .iter()
                .all(|dep| dep.optional || self.manifests.contains_key(&dep.id))
        } else {
            false
        }
    }
}

impl Default for ManifestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ManifestCategory, ManifestResources};

    fn test_manifest(id: &str) -> CapabilityManifest {
        CapabilityManifest {
            id: id.to_string(),
            version: "1.0.0".to_string(),
            name: format!("Test {}", id),
            description: None,
            permissions: vec![],
            dependencies: vec![],
            health_check: false,
            resources: ManifestResources::default(),
            category: ManifestCategory::Runtime,
        }
    }

    #[test]
    fn registry_operations() {
        let mut registry = ManifestRegistry::new();
        let m = test_manifest("test.cap");
        assert!(registry.register(m).is_ok());
        assert!(registry.has("test.cap"));
        assert_eq!(registry.list_ids().len(), 1);
    }

    #[test]
    fn duplicate_rejected() {
        let mut registry = ManifestRegistry::new();
        let m1 = test_manifest("test.cap");
        let m2 = test_manifest("test.cap");
        registry.register(m1).unwrap();
        assert!(registry.register(m2).is_err());
    }

    #[test]
    fn dependencies_satisfied() {
        let mut registry = ManifestRegistry::new();
        let m1 = test_manifest("dep.cap");
        registry.register(m1).unwrap();

        let mut m2 = test_manifest("test.cap");
        m2.dependencies.push(crate::ManifestDependency {
            id: "dep.cap".to_string(),
            version: "1.0.0".to_string(),
            optional: false,
        });
        registry.register(m2).unwrap();

        assert!(registry.dependencies_satisfied("test.cap"));
    }
}
