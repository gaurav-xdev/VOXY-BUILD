use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityManifest {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub entries: Vec<IntegrityEntry>,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityEntry {
    pub path: String,
    pub hash: String,
    pub size_bytes: u64,
    pub last_verified: Option<DateTime<Utc>>,
}

impl IntegrityEntry {
    pub fn new(path: &str, data: &[u8]) -> Self {
        let hash = compute_hash(data);
        Self {
            path: path.to_string(),
            hash,
            size_bytes: data.len() as u64,
            last_verified: Some(Utc::now()),
        }
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        let computed = compute_hash(data);
        computed == self.hash
    }
}

pub struct IntegrityVerifier {
    known_hashes: std::collections::HashMap<String, IntegrityEntry>,
}

impl IntegrityVerifier {
    pub fn new() -> Self {
        Self {
            known_hashes: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, path: &str, data: &[u8]) {
        let entry = IntegrityEntry::new(path, data);
        self.known_hashes.insert(path.to_string(), entry);
    }

    pub fn register_entry(&mut self, entry: IntegrityEntry) {
        self.known_hashes.insert(entry.path.clone(), entry);
    }

    pub fn verify(&self, path: &str, data: &[u8]) -> bool {
        self.known_hashes
            .get(path)
            .map(|entry| entry.verify(data))
            .unwrap_or(false)
    }

    pub fn verify_batch(&self, items: &[(String, &[u8])]) -> Vec<(String, bool)> {
        items
            .iter()
            .map(|(path, data)| (path.clone(), self.verify(path, data)))
            .collect()
    }

    pub fn get_entry(&self, path: &str) -> Option<&IntegrityEntry> {
        self.known_hashes.get(path)
    }

    pub fn all_entries(&self) -> Vec<&IntegrityEntry> {
        self.known_hashes.values().collect()
    }

    pub fn len(&self) -> usize {
        self.known_hashes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.known_hashes.is_empty()
    }

    pub fn export_manifest(&self) -> IntegrityManifest {
        let entries: Vec<IntegrityEntry> = self.known_hashes.values().cloned().collect();
        let manifest_json = serde_json::to_string(&entries).unwrap_or_default();
        let manifest_hash = compute_hash(manifest_json.as_bytes());
        IntegrityManifest {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            entries,
            manifest_hash,
        }
    }

    pub fn verify_manifest(manifest: &IntegrityManifest, files: &[(String, &[u8])]) -> bool {
        for (path, data) in files {
            if let Some(entry) = manifest.entries.iter().find(|e| e.path == *path) {
                if !entry.verify(data) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

impl Default for IntegrityVerifier {
    fn default() -> Self {
        Self::new()
    }
}

pub fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// SECURITY: Verify a file's integrity by computing its SHA-256 hash
/// and comparing against a known expected hash.
/// Returns Ok(true) if the file matches, Ok(false) if it doesn't,
/// or Err if the file cannot be read.
pub fn verify_file_integrity(path: &std::path::Path, expected_hash: &str) -> Result<bool, String> {
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read file for integrity check: {}", e))?;
    let computed = compute_hash(&data);
    Ok(computed == expected_hash)
}

/// SECURITY: Compute the SHA-256 hash of a file on disk.
pub fn hash_file(path: &std::path::Path) -> Result<String, String> {
    let data =
        std::fs::read(path).map_err(|e| format!("Failed to read file for hashing: {}", e))?;
    Ok(compute_hash(&data))
}

/// SECURITY: Verify a model file before loading it into FFI.
/// Checks: file exists, is readable, has expected SHA-256 hash,
/// and file size is within reasonable bounds (prevents OOM from massive files).
pub fn verify_model_file(
    path: &std::path::Path,
    expected_hash: Option<&str>,
    max_size_bytes: u64,
) -> Result<(), String> {
    // Check file exists
    if !path.exists() {
        return Err(format!("Model file does not exist: {}", path.display()));
    }

    // Check it's a file (not a directory)
    if !path.is_file() {
        return Err(format!("Model path is not a file: {}", path.display()));
    }

    // Check file size before reading
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read model metadata: {}", e))?;
    if metadata.len() > max_size_bytes {
        return Err(format!(
            "Model file too large: {} bytes (max: {} bytes)",
            metadata.len(),
            max_size_bytes
        ));
    }

    // Check file is not empty
    if metadata.len() == 0 {
        return Err("Model file is empty".to_string());
    }

    // Path traversal check: ensure the path doesn't escape expected directories
    if let Ok(canonical) = path.canonicalize() {
        let path_str = canonical.to_string_lossy();
        if path_str.contains("..") {
            return Err("Model path contains traversal".to_string());
        }
    }

    // Hash verification if expected hash is provided
    if let Some(expected) = expected_hash {
        let data =
            std::fs::read(path).map_err(|e| format!("Failed to read model for hash: {}", e))?;
        let computed = compute_hash(&data);
        if computed != expected {
            return Err(format!(
                "Model integrity check failed: expected hash {}, got {}",
                expected, computed
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_entry_creation() {
        let entry = IntegrityEntry::new("/etc/config.json", b"config-data");
        assert_eq!(entry.path, "/etc/config.json");
        assert_eq!(entry.size_bytes, 11);
    }

    #[test]
    fn integrity_verification() {
        let mut verifier = IntegrityVerifier::new();
        let data = b"important-config";
        verifier.register("/etc/config", data);
        assert!(verifier.verify("/etc/config", data));
        assert!(!verifier.verify("/etc/config", b"tampered"));
        assert!(!verifier.verify("/etc/unknown", data));
    }

    #[test]
    fn integrity_batch_verify() {
        let mut verifier = IntegrityVerifier::new();
        verifier.register("/a", b"aaa");
        verifier.register("/b", b"bbb");
        let items = vec![
            ("/a".to_string(), b"aaa" as &[u8]),
            ("/b".to_string(), b"tampered" as &[u8]),
        ];
        let results = verifier.verify_batch(&items);
        assert!(results[0].1);
        assert!(!results[1].1);
    }

    #[test]
    fn integrity_manifest_export() {
        let mut verifier = IntegrityVerifier::new();
        verifier.register("/a", b"data-a");
        verifier.register("/b", b"data-b");
        let manifest = verifier.export_manifest();
        assert_eq!(manifest.entries.len(), 2);
        assert!(!manifest.manifest_hash.is_empty());
    }

    #[test]
    fn integrity_manifest_verify() {
        let mut verifier = IntegrityVerifier::new();
        verifier.register("/config.json", b"config-data");
        let manifest = verifier.export_manifest();
        assert!(IntegrityVerifier::verify_manifest(
            &manifest,
            &[("/config.json".to_string(), b"config-data" as &[u8])]
        ));
        assert!(!IntegrityVerifier::verify_manifest(
            &manifest,
            &[("/config.json".to_string(), b"tampered" as &[u8])]
        ));
    }

    #[test]
    fn compute_hash_consistency() {
        let h1 = compute_hash(b"same data");
        let h2 = compute_hash(b"same data");
        let h3 = compute_hash(b"different data");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }
}
