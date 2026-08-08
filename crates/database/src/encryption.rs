use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;

use crate::error::{DatabaseError, Result};

pub struct DatabaseEncryption {
    key: [u8; 32],
}

impl DatabaseEncryption {
    pub fn new(key: [u8; 32]) -> Self {
        Self { key }
    }

    pub fn from_slice(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(DatabaseError::EncryptionFailed(
                "Key must be exactly 32 bytes".into(),
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(key);
        Ok(Self { key: arr })
    }

    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| DatabaseError::EncryptionFailed(format!("Invalid hex key: {}", e)))?;
        Self::from_slice(&bytes)
    }

    pub fn from_base64(b64_str: &str) -> Result<Self> {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::STANDARD;
        let bytes = engine
            .decode(b64_str)
            .map_err(|e| DatabaseError::EncryptionFailed(format!("Invalid base64 key: {}", e)))?;
        Self::from_slice(&bytes)
    }

    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self { key }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| DatabaseError::EncryptionFailed(e.to_string()))?;
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(DatabaseError::EncryptionFailed("Data too short".into()));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&self.key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Nonce::from_slice(nonce_bytes);
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DatabaseError::EncryptionFailed(e.to_string()))
    }

    pub fn encrypt_str(&self, plaintext: &str) -> Result<Vec<u8>> {
        self.encrypt(plaintext.as_bytes())
    }

    pub fn decrypt_str(&self, data: &[u8]) -> Result<String> {
        let bytes = self.decrypt(data)?;
        String::from_utf8(bytes)
            .map_err(|e| DatabaseError::EncryptionFailed(format!("Invalid UTF-8: {}", e)))
    }
}

impl std::fmt::Debug for DatabaseEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseEncryption")
            .field("key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0x42u8; 32];
        let encryption = DatabaseEncryption::new(key);
        let plaintext = b"Hello, World!";
        let encrypted = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_empty() {
        let encryption = DatabaseEncryption::generate();
        let encrypted = encryption.encrypt(b"").unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt_large() {
        let encryption = DatabaseEncryption::generate();
        let large: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
        let encrypted = encryption.encrypt(&large).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted.len(), 1024);
        assert_eq!(decrypted, large);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let encryption = DatabaseEncryption::generate();
        let result = encryption.decrypt(b"too short");
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let enc1 = DatabaseEncryption::new([1u8; 32]);
        let enc2 = DatabaseEncryption::new([2u8; 32]);
        let encrypted = enc1.encrypt(b"secret").unwrap();
        let result = enc2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_base64_key() {
        use base64::Engine as _;
        let engine = base64::engine::general_purpose::STANDARD;
        let original_key = [0xABu8; 32];
        let encoded = engine.encode(&original_key);
        let encryption = DatabaseEncryption::from_base64(&encoded).unwrap();
        let plaintext = b"base64 test";
        let encrypted = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_from_hex_key() {
        let original_key = [0xABu8; 32];
        let encoded = hex::encode(&original_key);
        let encryption = DatabaseEncryption::from_hex(&encoded).unwrap();
        let plaintext = b"hex test";
        let encrypted = encryption.encrypt(plaintext).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_str_methods() {
        let encryption = DatabaseEncryption::generate();
        let original = "Hello, 世界!";
        let encrypted = encryption.encrypt_str(original).unwrap();
        let decrypted = encryption.decrypt_str(&encrypted).unwrap();
        assert_eq!(decrypted, original);
    }

    #[test]
    fn test_invalid_key_length() {
        let result = DatabaseEncryption::from_slice(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_and_use() {
        let encryption = DatabaseEncryption::generate();
        let encrypted = encryption.encrypt(b"generate test").unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, b"generate test");
    }

    #[test]
    fn test_nonce_uniqueness() {
        let encryption = DatabaseEncryption::new([0u8; 32]);
        let e1 = encryption.encrypt(b"data").unwrap();
        let e2 = encryption.encrypt(b"data").unwrap();
        // Nonces should differ (first 12 bytes)
        assert_ne!(&e1[..12], &e2[..12]);
    }
}
