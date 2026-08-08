use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user_id: String,
    pub session_id: String,
    pub trust_level: TrustLevel,
    pub device_id: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustLevel {
    Unknown,
    Local,
    Authenticated,
    Verified,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub success: bool,
    pub context: Option<AuthContext>,
    pub error: Option<String>,
    pub requires_mfa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub user_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub device_info: String,
    pub ip_address: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> AuthResult;
    async fn validate_token(&self, token: &str) -> Option<AuthContext>;
    async fn refresh_token(&self, refresh_token: &str) -> Option<TokenPair>;
    async fn revoke_token(&self, token: &str) -> bool;
    async fn revoke_all_sessions(&self, user_id: &str) -> u32;
    async fn get_sessions(&self, user_id: &str) -> Vec<SessionInfo>;
    async fn create_session(&self, user_id: &str, device_info: &str) -> Option<SessionInfo>;
    async fn destroy_session(&self, session_id: &str) -> bool;
    fn auth_method(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credentials {
    EmailOtp { email: String, otp: String },
    ApiKey { key: String },
    DeviceToken { device_id: String, token: String },
    LocalSession,
}

#[async_trait]
pub trait SyncProvider: Send + Sync {
    async fn push(&self, entity: &str, data: &[u8]) -> Result<(), String>;
    async fn pull(&self, entity: &str) -> Option<Vec<u8>>;
    async fn list_conflicts(&self) -> Vec<SyncConflict>;
    async fn resolve_conflict(&self, conflict_id: &str, resolution: ConflictResolution) -> bool;
    fn is_online(&self) -> bool;
    fn last_sync_time(&self) -> Option<chrono::DateTime<chrono::Utc>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: String,
    pub entity: String,
    pub local_version: Vec<u8>,
    pub remote_version: Vec<u8>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    Merge(Vec<u8>),
}

#[async_trait]
pub trait BillingProvider: Send + Sync {
    async fn get_subscription(&self, user_id: &str) -> Option<Subscription>;
    async fn check_feature_access(&self, user_id: &str, feature: &str) -> bool;
    async fn get_usage(&self, user_id: &str) -> UsageStats;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub plan: String,
    pub status: SubscriptionStatus,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Trial,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub voice_minutes_used: u64,
    pub voice_minutes_limit: u64,
    pub memory_items_used: u64,
    pub memory_items_limit: u64,
    pub api_calls_used: u64,
    pub api_calls_limit: u64,
}

pub struct LocalAuthProvider;

#[async_trait]
impl AuthProvider for LocalAuthProvider {
    async fn authenticate(&self, credentials: &Credentials) -> AuthResult {
        match credentials {
            Credentials::LocalSession => AuthResult {
                success: true,
                context: Some(AuthContext {
                    user_id: "local-user".to_string(),
                    session_id: uuid::Uuid::new_v4().to_string(),
                    trust_level: TrustLevel::Local,
                    device_id: None,
                    expires_at: None,
                    metadata: HashMap::new(),
                }),
                error: None,
                requires_mfa: false,
            },
            Credentials::ApiKey { key } => {
                if key.starts_with("voxy_") {
                    AuthResult {
                        success: true,
                        context: Some(AuthContext {
                            user_id: "local-user".to_string(),
                            session_id: uuid::Uuid::new_v4().to_string(),
                            trust_level: TrustLevel::Authenticated,
                            device_id: None,
                            expires_at: None,
                            metadata: HashMap::new(),
                        }),
                        error: None,
                        requires_mfa: false,
                    }
                } else {
                    AuthResult {
                        success: false,
                        context: None,
                        error: Some("Invalid API key format".to_string()),
                        requires_mfa: false,
                    }
                }
            }
            _ => AuthResult {
                success: false,
                context: None,
                error: Some("Local auth only supports session and API key credentials".to_string()),
                requires_mfa: false,
            },
        }
    }

    async fn validate_token(&self, _token: &str) -> Option<AuthContext> {
        Some(AuthContext {
            user_id: "local-user".to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            trust_level: TrustLevel::Local,
            device_id: None,
            expires_at: None,
            metadata: HashMap::new(),
        })
    }

    async fn refresh_token(&self, _refresh_token: &str) -> Option<TokenPair> {
        Some(TokenPair {
            access_token: uuid::Uuid::new_v4().to_string(),
            refresh_token: uuid::Uuid::new_v4().to_string(),
            expires_in: 86400,
            token_type: "Bearer".to_string(),
        })
    }

    async fn revoke_token(&self, _token: &str) -> bool {
        true
    }
    async fn revoke_all_sessions(&self, _user_id: &str) -> u32 {
        0
    }
    async fn get_sessions(&self, _user_id: &str) -> Vec<SessionInfo> {
        vec![]
    }
    async fn create_session(&self, user_id: &str, device_info: &str) -> Option<SessionInfo> {
        Some(SessionInfo {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            device_info: device_info.to_string(),
            ip_address: None,
            is_active: true,
        })
    }
    async fn destroy_session(&self, _session_id: &str) -> bool {
        true
    }
    fn auth_method(&self) -> &str {
        "local"
    }
}

pub struct OfflineSyncProvider;

#[async_trait]
impl SyncProvider for OfflineSyncProvider {
    async fn push(&self, _entity: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
    async fn pull(&self, _entity: &str) -> Option<Vec<u8>> {
        None
    }
    async fn list_conflicts(&self) -> Vec<SyncConflict> {
        vec![]
    }
    async fn resolve_conflict(&self, _conflict_id: &str, _resolution: ConflictResolution) -> bool {
        true
    }
    fn is_online(&self) -> bool {
        false
    }
    fn last_sync_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        None
    }
}

pub struct OfflineBillingProvider;

#[async_trait]
impl BillingProvider for OfflineBillingProvider {
    async fn get_subscription(&self, _user_id: &str) -> Option<Subscription> {
        Some(Subscription {
            plan: "local".to_string(),
            status: SubscriptionStatus::Active,
            expires_at: None,
            features: vec!["voice".into(), "memory".into(), "automation".into()],
        })
    }
    async fn check_feature_access(&self, _user_id: &str, _feature: &str) -> bool {
        true
    }
    async fn get_usage(&self, _user_id: &str) -> UsageStats {
        UsageStats {
            voice_minutes_used: 0,
            voice_minutes_limit: u64::MAX,
            memory_items_used: 0,
            memory_items_limit: u64::MAX,
            api_calls_used: 0,
            api_calls_limit: u64::MAX,
        }
    }
}
