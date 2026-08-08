use std::collections::HashMap;
use std::fmt;

use async_trait::async_trait;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub provider_hint: Option<String>,
}

#[async_trait]
pub trait CapabilityRegistry: Send + Sync {
    async fn register_capability(&self, capability: CapabilityDescriptor) -> Result<()>;
    async fn unregister_capability(&self, capability_id: &CapabilityId) -> Result<()>;
    async fn has_capability(&self, capability_id: &CapabilityId) -> bool;
    async fn list_capabilities(&self) -> Result<Vec<CapabilityDescriptor>>;
    async fn find_capabilities(&self, query: &str) -> Result<Vec<CapabilityDescriptor>>;
}

pub struct InMemoryCapabilityRegistry {
    capabilities: tokio::sync::Mutex<HashMap<CapabilityId, CapabilityDescriptor>>,
}

impl InMemoryCapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: tokio::sync::Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryCapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CapabilityRegistry for InMemoryCapabilityRegistry {
    async fn register_capability(&self, capability: CapabilityDescriptor) -> Result<()> {
        let mut caps = self.capabilities.lock().await;
        caps.insert(capability.id.clone(), capability);
        Ok(())
    }

    async fn unregister_capability(&self, capability_id: &CapabilityId) -> Result<()> {
        let mut caps = self.capabilities.lock().await;
        caps.remove(capability_id);
        Ok(())
    }

    async fn has_capability(&self, capability_id: &CapabilityId) -> bool {
        self.capabilities.lock().await.contains_key(capability_id)
    }

    async fn list_capabilities(&self) -> Result<Vec<CapabilityDescriptor>> {
        let caps = self.capabilities.lock().await;
        Ok(caps.values().cloned().collect())
    }

    async fn find_capabilities(&self, query: &str) -> Result<Vec<CapabilityDescriptor>> {
        let caps = self.capabilities.lock().await;
        let query_lower = query.to_lowercase();
        Ok(caps
            .values()
            .filter(|c| {
                c.name.to_lowercase().contains(&query_lower)
                    || c.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect())
    }
}
