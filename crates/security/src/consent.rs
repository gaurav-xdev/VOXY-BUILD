use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRequest {
    pub id: Uuid,
    pub subject: String,
    pub capability: String,
    pub resource: Option<String>,
    pub reason: String,
    pub requested_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub duration_secs: Option<u64>,
    pub metadata: std::collections::HashMap<String, String>,
}

impl ConsentRequest {
    pub fn new(
        subject: impl Into<String>,
        capability: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            subject: subject.into(),
            capability: capability.into(),
            resource: None,
            reason: reason.into(),
            requested_at: Utc::now(),
            expires_at: None,
            duration_secs: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_duration(mut self, duration_secs: u64) -> Self {
        self.duration_secs = Some(duration_secs);
        self.expires_at = Some(Utc::now() + chrono::Duration::seconds(duration_secs as i64));
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentGrant {
    pub request_id: Uuid,
    pub granted: bool,
    pub granted_by: String,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reason: Option<String>,
    pub is_revoked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentState {
    Pending,
    Granted,
    Denied,
    Expired,
    Revoked,
}

pub struct ConsentManager {
    requests: std::collections::HashMap<Uuid, ConsentRequest>,
    grants: std::collections::HashMap<Uuid, ConsentGrant>,
}

impl ConsentManager {
    pub fn new() -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            grants: std::collections::HashMap::new(),
        }
    }

    pub fn create_request(&mut self, request: ConsentRequest) -> Uuid {
        let id = request.id;
        self.requests.insert(id, request);
        id
    }

    pub fn grant(
        &mut self,
        request_id: Uuid,
        granted_by: impl Into<String>,
        reason: Option<String>,
    ) -> crate::Result<()> {
        let request =
            self.requests
                .get(&request_id)
                .ok_or(crate::SecurityError::ConsentRequestNotFound(
                    request_id.to_string(),
                ))?;

        let grant = ConsentGrant {
            request_id,
            granted: true,
            granted_by: granted_by.into(),
            granted_at: Utc::now(),
            expires_at: request.expires_at,
            reason,
            is_revoked: false,
        };
        self.grants.insert(request_id, grant);
        Ok(())
    }

    pub fn deny(
        &mut self,
        request_id: Uuid,
        denied_by: impl Into<String>,
        reason: Option<String>,
    ) -> crate::Result<()> {
        if !self.requests.contains_key(&request_id) {
            return Err(crate::SecurityError::ConsentRequestNotFound(
                request_id.to_string(),
            ));
        }
        let grant = ConsentGrant {
            request_id,
            granted: false,
            granted_by: denied_by.into(),
            granted_at: Utc::now(),
            expires_at: None,
            reason,
            is_revoked: false,
        };
        self.grants.insert(request_id, grant);
        Ok(())
    }

    pub fn revoke(&mut self, request_id: Uuid) -> crate::Result<()> {
        let grant = self.grants.get_mut(&request_id).ok_or(
            crate::SecurityError::ConsentRequestNotFound(request_id.to_string()),
        )?;
        grant.is_revoked = true;
        Ok(())
    }

    pub fn check(&self, request_id: Uuid) -> ConsentState {
        if let Some(grant) = self.grants.get(&request_id) {
            if grant.is_revoked {
                return ConsentState::Revoked;
            }
            if let Some(expires) = grant.expires_at {
                if Utc::now() > expires {
                    return ConsentState::Expired;
                }
            }
            if grant.granted {
                return ConsentState::Granted;
            }
            return ConsentState::Denied;
        }
        if self.requests.contains_key(&request_id) {
            return ConsentState::Pending;
        }
        ConsentState::Denied
    }

    pub fn is_granted(&self, subject: &str, capability: &str) -> bool {
        self.grants.values().any(|g| {
            if g.is_revoked || !g.granted {
                return false;
            }
            if let Some(expires) = g.expires_at {
                if Utc::now() > expires {
                    return false;
                }
            }
            if let Some(request) = self.requests.get(&g.request_id) {
                return request.subject == subject && request.capability == capability;
            }
            false
        })
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_request_lifecycle() {
        let mut manager = ConsentManager::new();
        let request = ConsentRequest::new("user-1", "screen:capture", "Need screen for automation");
        let id = manager.create_request(request);
        assert_eq!(manager.check(id), ConsentState::Pending);
        manager.grant(id, "admin", None).unwrap();
        assert_eq!(manager.check(id), ConsentState::Granted);
    }

    #[test]
    fn consent_deny() {
        let mut manager = ConsentManager::new();
        let request = ConsentRequest::new("user-1", "admin:delete", "Delete system file");
        let id = manager.create_request(request);
        manager
            .deny(id, "admin", Some("Too dangerous".to_string()))
            .unwrap();
        assert_eq!(manager.check(id), ConsentState::Denied);
    }

    #[test]
    fn consent_revoke() {
        let mut manager = ConsentManager::new();
        let request = ConsentRequest::new("user-1", "file:write", "Write to data dir");
        let id = manager.create_request(request);
        manager.grant(id, "admin", None).unwrap();
        assert_eq!(manager.check(id), ConsentState::Granted);
        manager.revoke(id).unwrap();
        assert_eq!(manager.check(id), ConsentState::Revoked);
    }

    #[test]
    fn consent_expiry() {
        let mut manager = ConsentManager::new();
        let request = ConsentRequest::new("user-1", "file:read", "Read config").with_duration(1);
        let id = manager.create_request(request);
        manager.grant(id, "admin", None).unwrap();
        assert_eq!(manager.check(id), ConsentState::Granted);
    }

    #[test]
    fn is_granted_check() {
        let mut manager = ConsentManager::new();
        let request = ConsentRequest::new("user-1", "audio:capture", "Voice input");
        let id = manager.create_request(request);
        assert!(!manager.is_granted("user-1", "audio:capture"));
        manager.grant(id, "admin", None).unwrap();
        assert!(manager.is_granted("user-1", "audio:capture"));
        assert!(!manager.is_granted("user-1", "file:write"));
    }

    #[test]
    fn consent_request_with_resource_and_duration() {
        let request = ConsentRequest::new("user", "file:write", "Write to tmp")
            .with_resource("/tmp")
            .with_duration(3600);
        assert_eq!(request.resource, Some("/tmp".to_string()));
        assert_eq!(request.duration_secs, Some(3600));
        assert!(request.expires_at.is_some());
    }
}
