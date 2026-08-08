use crate::trust::TrustLevel;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    User,
    Device,
    Agent,
    Plugin,
    Service,
    Home,
    Organization,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Device => write!(f, "device"),
            Self::Agent => write!(f, "agent"),
            Self::Plugin => write!(f, "plugin"),
            Self::Service => write!(f, "service"),
            Self::Home => write!(f, "home"),
            Self::Organization => write!(f, "organization"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityId(pub Uuid);

impl IdentityId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for IdentityId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: IdentityId,
    pub name: String,
    pub entity_type: EntityType,
    pub trust_level: TrustLevel,
    pub parent: Option<IdentityId>,
    pub children: Vec<IdentityId>,
    pub metadata: std::collections::HashMap<String, String>,
    pub is_active: bool,
}

impl Identity {
    pub fn new(name: impl Into<String>, entity_type: EntityType) -> Self {
        Self {
            id: IdentityId::new(),
            name: name.into(),
            entity_type,
            trust_level: TrustLevel::Unknown,
            parent: None,
            children: Vec::new(),
            metadata: std::collections::HashMap::new(),
            is_active: true,
        }
    }

    pub fn acts_for(&self, target: &Identity) -> bool {
        if self.id.0 == target.id.0 {
            return true;
        }
        self.children.contains(&target.id)
    }
}

pub struct IdentityHierarchy {
    pub root: Identity,
    pub ancestors: Vec<Identity>,
    pub descendants: Vec<Identity>,
}

pub struct IdentityManager {
    identities: std::collections::HashMap<Uuid, Identity>,
}

impl IdentityManager {
    pub fn new() -> Self {
        Self {
            identities: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, identity: Identity) -> crate::Result<()> {
        let id = identity.id.0;
        if self.identities.contains_key(&id) {
            return Err(crate::SecurityError::IdentityAlreadyRegistered(
                identity.name,
            ));
        }
        self.identities.insert(id, identity);
        Ok(())
    }

    pub fn get(&self, id: &IdentityId) -> Option<&Identity> {
        self.identities.get(&id.0)
    }

    pub fn get_mut(&mut self, id: &IdentityId) -> Option<&mut Identity> {
        self.identities.get_mut(&id.0)
    }

    pub fn deactivate(&mut self, id: &IdentityId, _reason: &str) -> crate::Result<()> {
        if let Some(identity) = self.identities.get_mut(&id.0) {
            identity.is_active = false;
            Ok(())
        } else {
            Err(crate::SecurityError::IdentityNotFound(id.0.to_string()))
        }
    }

    pub fn find_by_type(&self, entity_type: EntityType) -> Vec<&Identity> {
        self.identities
            .values()
            .filter(|i| i.entity_type == entity_type && i.is_active)
            .collect()
    }

    pub fn find_by_trust(&self, min_trust: TrustLevel) -> Vec<&Identity> {
        self.identities
            .values()
            .filter(|i| i.trust_level >= min_trust && i.is_active)
            .collect()
    }

    pub fn find_children(&self, parent: &IdentityId) -> Vec<&Identity> {
        self.identities
            .values()
            .filter(|i| i.parent.as_ref().map(|p| p.0) == Some(parent.0))
            .collect()
    }

    pub fn set_parent(&mut self, child: &IdentityId, parent: &IdentityId) -> crate::Result<()> {
        if !self.identities.contains_key(&parent.0) {
            return Err(crate::SecurityError::IdentityNotFound(parent.0.to_string()));
        }
        if !self.identities.contains_key(&child.0) {
            return Err(crate::SecurityError::IdentityNotFound(child.0.to_string()));
        }
        if let Some(child_identity) = self.identities.get_mut(&child.0) {
            child_identity.parent = Some(parent.clone());
            child_identity.children.clear();
        }
        if let Some(parent_identity) = self.identities.get_mut(&parent.0) {
            parent_identity.children.push(child.clone());
        }
        Ok(())
    }

    pub fn get_hierarchy(&self, id: &IdentityId) -> Option<IdentityHierarchy> {
        let root = self.identities.get(&id.0)?.clone();
        let descendants: Vec<Identity> = self
            .identities
            .values()
            .filter(|i| i.parent.as_ref().map(|p| p.0) == Some(id.0))
            .cloned()
            .collect();
        let ancestors: Vec<Identity> = self
            .identities
            .values()
            .filter(|i| root.parent.as_ref().map(|p| p.0) == Some(i.id.0))
            .cloned()
            .collect();
        Some(IdentityHierarchy {
            root,
            ancestors,
            descendants,
        })
    }
}

impl Default for IdentityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_creation() {
        let identity = Identity::new("test-device", EntityType::Device);
        assert_eq!(identity.name, "test-device");
        assert_eq!(identity.entity_type, EntityType::Device);
        assert_eq!(identity.trust_level, TrustLevel::Unknown);
        assert!(identity.is_active);
    }

    #[test]
    fn identity_registration() {
        let mut manager = IdentityManager::new();
        let identity = Identity::new("user-1", EntityType::User);
        let id = identity.id.clone();
        assert!(manager.register(identity).is_ok());
        assert!(manager.get(&id).is_some());
    }

    #[test]
    fn identity_acts_for() {
        let user = Identity::new("alice", EntityType::User);
        let device = Identity::new("alice-phone", EntityType::Device);
        assert!(user.acts_for(&user));
        assert!(!user.acts_for(&device));
    }

    #[test]
    fn find_by_type() {
        let mut manager = IdentityManager::new();
        manager
            .register(Identity::new("alice", EntityType::User))
            .unwrap();
        manager
            .register(Identity::new("bob", EntityType::User))
            .unwrap();
        manager
            .register(Identity::new("desktop", EntityType::Device))
            .unwrap();
        assert_eq!(manager.find_by_type(EntityType::User).len(), 2);
        assert_eq!(manager.find_by_type(EntityType::Device).len(), 1);
    }

    #[test]
    fn deactivate_identity() {
        let mut manager = IdentityManager::new();
        let identity = Identity::new("temp", EntityType::Service);
        let id = identity.id.clone();
        manager.register(identity).unwrap();
        assert!(manager.get(&id).unwrap().is_active);
        manager.deactivate(&id, "expired").unwrap();
        assert!(!manager.get(&id).unwrap().is_active);
    }

    #[test]
    fn entity_type_display() {
        assert_eq!(EntityType::User.to_string(), "user");
        assert_eq!(EntityType::Organization.to_string(), "organization");
    }

    #[test]
    fn parent_child_relationship() {
        let mut manager = IdentityManager::new();
        let org = Identity::new("voxy-inc", EntityType::Organization);
        let org_id = org.id.clone();
        let user = Identity::new("alice", EntityType::User);
        let user_id = user.id.clone();
        manager.register(org).unwrap();
        manager.register(user).unwrap();
        manager.set_parent(&user_id, &org_id).unwrap();
        let children = manager.find_children(&org_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "alice");
    }

    #[test]
    fn hierarchy_lookup() {
        let mut manager = IdentityManager::new();
        let org = Identity::new("voxy-inc", EntityType::Organization);
        let org_id = org.id.clone();
        let user = Identity::new("alice", EntityType::User);
        let user_id = user.id.clone();
        manager.register(org).unwrap();
        manager.register(user).unwrap();
        manager.set_parent(&user_id, &org_id).unwrap();
        let hierarchy = manager.get_hierarchy(&org_id);
        assert!(hierarchy.is_some());
        assert_eq!(hierarchy.unwrap().descendants.len(), 1);
    }
}
