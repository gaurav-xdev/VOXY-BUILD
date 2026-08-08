use crate::error::Result;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use crate::types::VersionInfo;

#[async_trait::async_trait]
pub trait MemoryVersioning: Send + Sync {
    async fn create_version(&self, item: &MemoryItem, changes: Vec<String>) -> Result<MemoryItem>;
    async fn get_version(&self, id: &MemoryId, version: u64) -> Result<MemoryItem>;
    async fn list_versions(&self, id: &MemoryId) -> Result<Vec<VersionInfo>>;
    async fn revert(&self, id: &MemoryId, version: u64) -> Result<MemoryItem>;
    async fn diff(&self, id: &MemoryId, v1: u64, v2: u64) -> Result<Vec<String>>;
    async fn latest_version(&self, id: &MemoryId) -> Result<u64>;
}
