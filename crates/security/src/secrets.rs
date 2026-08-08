use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zeroize::{Zeroize, Zeroizing};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretEntry {
    pub id: Uuid,
    pub key: String,
    pub value: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub version: u32,
    pub is_active: bool,
}

impl Zeroize for SecretEntry {
    fn zeroize(&mut self) {
        self.value.zeroize();
    }
}

impl Drop for SecretEntry {
    fn drop(&mut self) {
        self.zeroize();
    }
}

pub struct SecretVault {
    secrets: std::collections::HashMap<String, Vec<SecretEntry>>,
    master_key: Option<Zeroizing<Vec<u8>>>,
}

impl SecretVault {
    pub fn new() -> Self {
        Self {
            secrets: std::collections::HashMap::new(),
            master_key: None,
        }
    }

    pub fn with_master_key(mut self, key: Vec<u8>) -> Self {
        self.master_key = Some(Zeroizing::new(key));
        self
    }

    pub fn set_master_key(&mut self, key: Vec<u8>) {
        self.master_key = Some(Zeroizing::new(key));
    }

    pub fn has_master_key(&self) -> bool {
        self.master_key.is_some()
    }

    pub fn store(&mut self, key: &str, value: Vec<u8>) -> crate::Result<()> {
        if !self.has_master_key() {
            return Err(crate::SecurityError::NoMasterKey);
        }
        let entry = SecretEntry {
            id: Uuid::new_v4(),
            key: key.to_string(),
            value,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            expires_at: None,
            version: 1,
            is_active: true,
        };
        self.secrets.insert(key.to_string(), vec![entry]);
        Ok(())
    }

    pub fn get(&self, key: &str) -> crate::Result<&SecretEntry> {
        let entries = self
            .secrets
            .get(key)
            .ok_or(crate::SecurityError::SecretNotFound(key.to_string()))?;
        entries
            .iter()
            .find(|e| e.is_active)
            .ok_or(crate::SecurityError::SecretNotFound(key.to_string()))
    }

    pub fn rotate(&mut self, key: &str, new_value: Vec<u8>) -> crate::Result<()> {
        if let Some(entries) = self.secrets.get_mut(key) {
            for entry in entries.iter_mut() {
                entry.is_active = false;
            }
            let version = entries.last().map(|e| e.version + 1).unwrap_or(1);
            let new_entry = SecretEntry {
                id: Uuid::new_v4(),
                key: key.to_string(),
                value: new_value,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                expires_at: None,
                version,
                is_active: true,
            };
            entries.push(new_entry);
            Ok(())
        } else {
            self.store(key, new_value)
        }
    }

    pub fn delete(&mut self, key: &str) {
        if let Some(mut entries) = self.secrets.remove(key) {
            for entry in &mut entries {
                entry.zeroize();
            }
        }
    }

    pub fn list_keys(&self) -> Vec<&str> {
        self.secrets.keys().map(|k| k.as_str()).collect()
    }

    pub fn list_all(&self) -> Vec<&SecretEntry> {
        self.secrets
            .values()
            .filter_map(|entries| entries.iter().find(|e| e.is_active))
            .collect()
    }

    pub fn clear(&mut self) {
        for (_, mut entries) in self.secrets.drain() {
            for entry in &mut entries {
                entry.zeroize();
            }
        }
    }
}

impl Drop for SecretVault {
    fn drop(&mut self) {
        self.clear();
        if let Some(ref mut mk) = self.master_key {
            mk.zeroize();
        }
    }
}

impl Default for SecretVault {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_requires_master_key() {
        let mut vault = SecretVault::new();
        assert!(vault.store("api_key", b"sk-123".to_vec()).is_err());
        vault.set_master_key(b"master-key-123".to_vec());
        assert!(vault.store("api_key", b"sk-123".to_vec()).is_ok());
    }

    #[test]
    fn vault_store_and_retrieve() {
        let mut vault = SecretVault::new().with_master_key(b"mk".to_vec());
        vault.store("api_key", b"sk-secret".to_vec()).unwrap();
        let entry = vault.get("api_key").unwrap();
        assert_eq!(entry.value, b"sk-secret");
        assert_eq!(entry.version, 1);
    }

    #[test]
    fn vault_rotation() {
        let mut vault = SecretVault::new().with_master_key(b"mk".to_vec());
        vault.store("api_key", b"v1".to_vec()).unwrap();
        vault.rotate("api_key", b"v2".to_vec()).unwrap();
        let entry = vault.get("api_key").unwrap();
        assert_eq!(entry.value, b"v2");
        assert_eq!(entry.version, 2);
    }

    #[test]
    fn vault_delete() {
        let mut vault = SecretVault::new().with_master_key(b"mk".to_vec());
        vault.store("temp", b"value".to_vec()).unwrap();
        assert!(vault.get("temp").is_ok());
        vault.delete("temp");
        assert!(vault.get("temp").is_err());
    }

    #[test]
    fn vault_list_keys() {
        let mut vault = SecretVault::new().with_master_key(b"mk".to_vec());
        vault.store("key1", b"v1".to_vec()).unwrap();
        vault.store("key2", b"v2".to_vec()).unwrap();
        let keys = vault.list_keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1"));
    }

    #[test]
    fn vault_clear() {
        let mut vault = SecretVault::new().with_master_key(b"mk".to_vec());
        vault.store("key1", b"v1".to_vec()).unwrap();
        vault.clear();
        assert!(vault.get("key1").is_err());
    }
}
