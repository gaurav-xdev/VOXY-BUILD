//! Capability tokens and permission grants.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A token granting specific capabilities.
/// SECURITY: All tokens must have a signature to be valid. Unsigned tokens
/// are rejected to prevent capability spoofing.
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    pub id: Uuid,
    pub subject: String,
    pub capabilities: Vec<String>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub signature: Vec<u8>,
}

impl CapabilityToken {
    /// Create a signed token.
    pub fn new(subject: impl Into<String>, capabilities: Vec<String>, signature: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject: subject.into(),
            capabilities,
            issued_at: Utc::now(),
            expires_at: None,
            signature,
        }
    }

    /// Set expiration.
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if the token is valid (not expired AND has a signature).
    pub fn is_valid(&self) -> bool {
        let not_expired = self.expires_at.map(|exp| Utc::now() < exp).unwrap_or(true);
        not_expired && !self.signature.is_empty()
    }

    /// Check if the token has a specific capability.
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| {
            c == capability || (c.ends_with(":*") && capability.starts_with(&c[..c.len() - 1]))
        })
    }
}

/// A record of a granted permission.
#[derive(Debug, Clone)]
pub struct PermissionGrant {
    pub capability: String,
    pub granted: bool,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
    pub expires_at: Option<DateTime<Utc>>,
}

impl PermissionGrant {
    pub fn new(
        capability: impl Into<String>,
        granted: bool,
        granted_by: impl Into<String>,
    ) -> Self {
        Self {
            capability: capability.into(),
            granted,
            granted_at: Utc::now(),
            granted_by: granted_by.into(),
            expires_at: None,
        }
    }

    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.granted && self.expires_at.map(|exp| Utc::now() < exp).unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_creation() {
        let token = CapabilityToken::new(
            "plugin-1",
            vec!["audio:capture".to_string()],
            vec![0xAB, 0xCD],
        );
        assert!(token.is_valid());
        assert!(token.has_capability("audio:capture"));
        assert!(!token.has_capability("file:read"));
    }

    #[test]
    fn token_wildcard() {
        let token = CapabilityToken::new("plugin-1", vec!["audio:*".to_string()], vec![0xAB]);
        assert!(token.has_capability("audio:capture"));
        assert!(token.has_capability("audio:playback"));
        assert!(!token.has_capability("file:read"));
    }

    #[test]
    fn token_unsigned_is_invalid() {
        let token = CapabilityToken::new("plugin-1", vec!["audio:capture".to_string()], vec![]);
        assert!(!token.is_valid(), "Unsigned token must be invalid");
    }

    #[test]
    fn grant_creation() {
        let grant = PermissionGrant::new("audio:capture", true, "user");
        assert!(grant.is_valid());
    }
}
