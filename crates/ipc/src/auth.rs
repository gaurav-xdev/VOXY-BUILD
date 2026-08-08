use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: Uuid,
    pub capabilities: Vec<CapabilityClaim>,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub issuer: String,
    pub subject: String,
    /// SECURITY: Signature is required. Tokens without a signature are invalid.
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityClaim {
    pub capability: String,
    pub resource: Option<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub method: AuthMethod,
    pub credentials: serde_json::Value,
    pub requested_capabilities: Vec<String>,
    pub client_version: String,
    pub client_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Token,
    Certificate,
    ApiKey,
    OAuth2,
    MutualTls,
    Anonymous,
}

impl std::fmt::Display for AuthMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Token => write!(f, "token"),
            Self::Certificate => write!(f, "certificate"),
            Self::ApiKey => write!(f, "api_key"),
            Self::OAuth2 => write!(f, "oauth2"),
            Self::MutualTls => write!(f, "mutual_tls"),
            Self::Anonymous => write!(f, "anonymous"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub token: Option<CapabilityToken>,
    pub session_id: Option<Uuid>,
    pub error: Option<String>,
    pub granted_capabilities: Vec<String>,
}

impl CapabilityToken {
    pub fn new(
        subject: impl Into<String>,
        issuer: impl Into<String>,
        capabilities: Vec<CapabilityClaim>,
        ttl_secs: u64,
        signature: Vec<u8>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            capabilities,
            issued_at: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs as i64),
            issuer: issuer.into(),
            subject: subject.into(),
            signature,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// SECURITY: Token is only valid if it has a signature and is not expired.
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.signature.is_empty()
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| {
            if c.capability.ends_with(":*") {
                let prefix = &c.capability[..c.capability.len() - 2];
                capability.starts_with(prefix)
                    && capability.as_bytes().get(prefix.len()) == Some(&b':')
            } else {
                c.capability == capability
            }
        })
    }

    pub fn remaining_ttl_secs(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds().max(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_token_creation() {
        let claims = vec![CapabilityClaim {
            capability: "voice:transcribe".to_string(),
            resource: None,
            constraints: vec![],
        }];
        let token = CapabilityToken::new("user-1", "system", claims, 3600, vec![0xAB]);
        assert_eq!(token.subject, "user-1");
        assert_eq!(token.issuer, "system");
        assert!(!token.is_expired());
        assert!(token.is_valid());
        assert!(token.has_capability("voice:transcribe"));
        assert!(!token.has_capability("admin:*"));
    }

    #[test]
    fn capability_token_expiry() {
        let claims = vec![CapabilityClaim {
            capability: "test".to_string(),
            resource: None,
            constraints: vec![],
        }];
        let token = CapabilityToken::new("user", "system", claims, 0, vec![0xAB]);
        assert!(token.is_expired());
        assert!(!token.is_valid());
    }

    #[test]
    fn capability_token_unsigned_invalid() {
        let claims = vec![CapabilityClaim {
            capability: "test".to_string(),
            resource: None,
            constraints: vec![],
        }];
        let token = CapabilityToken::new("user", "system", claims, 3600, vec![]);
        assert!(!token.is_valid(), "Unsigned token must be invalid");
    }

    #[test]
    fn auth_method_display() {
        assert_eq!(AuthMethod::Token.to_string(), "token");
        assert_eq!(AuthMethod::MutualTls.to_string(), "mutual_tls");
    }

    #[test]
    fn auth_response_success() {
        let response = AuthResponse {
            success: true,
            token: Some(CapabilityToken::new(
                "user",
                "system",
                vec![],
                3600,
                vec![0xAB],
            )),
            session_id: Some(Uuid::new_v4()),
            error: None,
            granted_capabilities: vec!["voice:transcribe".to_string()],
        };
        assert!(response.success);
        assert!(response.error.is_none());
        assert_eq!(response.granted_capabilities.len(), 1);
    }

    #[test]
    fn auth_response_failure() {
        let response = AuthResponse {
            success: false,
            token: None,
            session_id: None,
            error: Some("Invalid credentials".to_string()),
            granted_capabilities: vec![],
        };
        assert!(!response.success);
        assert_eq!(response.error.unwrap(), "Invalid credentials");
    }

    #[test]
    fn capability_token_remaining_ttl() {
        let claims = vec![CapabilityClaim {
            capability: "test".to_string(),
            resource: None,
            constraints: vec![],
        }];
        let token = CapabilityToken::new("user", "system", claims, 3600, vec![0xAB]);
        let remaining = token.remaining_ttl_secs();
        assert!(remaining > 3590 && remaining <= 3600);
    }

    #[test]
    fn capability_claim_constraints() {
        let claim = CapabilityClaim {
            capability: "storage:write".to_string(),
            resource: Some("namespace:user-data".to_string()),
            constraints: vec!["max_size:10MB".to_string()],
        };
        assert_eq!(claim.resource.unwrap(), "namespace:user-data");
        assert!(claim.constraints.contains(&"max_size:10MB".to_string()));
    }

    #[test]
    fn auth_request_serialization() {
        let req = AuthRequest {
            method: AuthMethod::ApiKey,
            credentials: serde_json::json!({"key": "sk-123"}),
            requested_capabilities: vec!["voice:transcribe".to_string()],
            client_version: "1.0.0".to_string(),
            client_name: "voxy-cli".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let restored: AuthRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.method, AuthMethod::ApiKey);
        assert_eq!(restored.client_name, "voxy-cli");
    }
}
