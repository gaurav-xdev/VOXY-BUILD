//! Runtime feature flag system.

pub mod error;

pub use error::{FeatureFlagError, Result};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Feature flag value.
#[derive(Debug, Clone)]
pub enum FeatureFlagValue {
    Bool(bool),
    String(String),
    Integer(i64),
    Float(f64),
}

impl FeatureFlagValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }
}

/// Feature flag store.
pub struct FeatureFlags {
    flags: Arc<RwLock<HashMap<String, FeatureFlagValue>>>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        Self {
            flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set a feature flag.
    pub async fn set(&self, key: &str, value: FeatureFlagValue) {
        self.flags.write().await.insert(key.to_string(), value);
    }

    /// Get a feature flag.
    pub async fn get(&self, key: &str) -> Option<FeatureFlagValue> {
        self.flags.read().await.get(key).cloned()
    }

    /// Check if a boolean flag is enabled.
    pub async fn is_enabled(&self, key: &str) -> bool {
        self.get(key)
            .await
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Get a string flag value.
    pub async fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)
            .await
            .and_then(|v| v.as_string().map(|s| s.to_string()))
    }

    /// Load flags from a TOML string.
    pub async fn load_from_toml(&self, toml: &str) -> Result<()> {
        let parsed: HashMap<String, toml::Value> =
            toml::from_str(toml).map_err(|e| FeatureFlagError::ParseError(e.to_string()))?;

        for (key, value) in parsed {
            let flag_value = match value {
                toml::Value::Boolean(b) => FeatureFlagValue::Bool(b),
                toml::Value::String(s) => FeatureFlagValue::String(s),
                toml::Value::Integer(i) => FeatureFlagValue::Integer(i),
                toml::Value::Float(f) => FeatureFlagValue::Float(f),
                _ => continue,
            };
            self.set(&key, flag_value).await;
        }
        Ok(())
    }

    /// List all flag keys.
    pub async fn list_keys(&self) -> Vec<String> {
        self.flags.read().await.keys().cloned().collect()
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn feature_flag_operations() {
        let flags = FeatureFlags::new();
        flags.set("test.bool", FeatureFlagValue::Bool(true)).await;
        flags
            .set("test.string", FeatureFlagValue::String("hello".to_string()))
            .await;

        assert!(flags.is_enabled("test.bool").await);
        assert!(!flags.is_enabled("nonexistent").await);
        assert_eq!(
            flags.get_string("test.string").await,
            Some("hello".to_string())
        );
    }
}
