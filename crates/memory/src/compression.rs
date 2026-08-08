use crate::error::Result;
use crate::types::CompressionPolicy;
use crate::types::MemoryItem;

#[async_trait::async_trait]
pub trait CompressionEngine: Send + Sync {
    async fn compress(&self, item: &MemoryItem, policy: &CompressionPolicy) -> Result<MemoryItem>;
    async fn should_compress(&self, item: &MemoryItem, policy: &CompressionPolicy) -> bool;
    async fn get_compression_candidates(
        &self,
        policy: &CompressionPolicy,
    ) -> Result<Vec<MemoryItem>>;
    async fn compress_markdown(&self, content: &serde_json::Value) -> Result<serde_json::Value>;
    async fn run_scheduled(&self) -> Result<usize>;
}
