use crate::types::{ContextSnapshot, ContextSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The final output of the fusion engine — a single coherent context
/// consumed by the cognition pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledContext {
    /// Unique identifier for this assembly.
    pub id: String,

    /// When this context was assembled.
    pub assembled_at: chrono::DateTime<chrono::Utc>,

    /// Merged context data from all sources.
    pub data: serde_json::Value,

    /// Individual source snapshots that contributed.
    pub sources: HashMap<ContextSource, ContextSnapshot>,

    /// Ordered list of sources by priority (highest first).
    pub source_order: Vec<ContextSource>,

    /// Overall confidence in this assembled context.
    pub overall_confidence: f64,

    /// Total size in bytes.
    pub total_size_bytes: usize,

    /// Number of sources that contributed.
    pub source_count: usize,

    /// Sources that were excluded (stale, low confidence, etc.).
    pub excluded_sources: Vec<ContextSource>,
}

impl AssembledContext {
    /// Get a snapshot by source.
    pub fn get(&self, source: &ContextSource) -> Option<&ContextSnapshot> {
        self.sources.get(source)
    }

    /// Check if a source is included.
    pub fn has_source(&self, source: &ContextSource) -> bool {
        self.sources.contains_key(source)
    }

    /// Get all included sources.
    pub fn included_sources(&self) -> Vec<&ContextSource> {
        self.sources.keys().collect()
    }

    /// Get the highest-priority source.
    pub fn primary_source(&self) -> Option<&ContextSource> {
        self.source_order.first()
    }

    /// Extract a specific field from the merged data.
    pub fn field(&self, path: &str) -> Option<&serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.data;

        for part in parts {
            current = current.get(part)?;
        }

        Some(current)
    }
}

/// Builder for constructing an AssembledContext.
pub struct AssembledContextBuilder {
    id: String,
    data: serde_json::Value,
    sources: HashMap<ContextSource, ContextSnapshot>,
    source_order: Vec<ContextSource>,
    excluded_sources: Vec<ContextSource>,
}

impl AssembledContextBuilder {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            data: serde_json::Value::Null,
            sources: HashMap::new(),
            source_order: Vec::new(),
            excluded_sources: Vec::new(),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the merged data.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Add a contributing source snapshot.
    pub fn add_source(mut self, snapshot: ContextSnapshot) -> Self {
        let source = snapshot.source.clone();
        self.sources.insert(source.clone(), snapshot);
        if !self.source_order.contains(&source) {
            self.source_order.push(source);
        }
        self
    }

    /// Mark a source as excluded.
    pub fn exclude_source(mut self, source: ContextSource) -> Self {
        if !self.excluded_sources.contains(&source) {
            self.excluded_sources.push(source);
        }
        self
    }

    /// Build the assembled context.
    pub fn build(self) -> AssembledContext {
        let total_size: usize = self.sources.values().map(|s| s.size_bytes).sum();
        let source_count = self.sources.len();

        let overall_confidence = if source_count == 0 {
            0.0
        } else {
            self.sources.values().map(|s| s.confidence).sum::<f64>() / source_count as f64
        };

        AssembledContext {
            id: self.id,
            assembled_at: chrono::Utc::now(),
            data: self.data,
            sources: self.sources,
            source_order: self.source_order,
            overall_confidence,
            total_size_bytes: total_size,
            source_count,
            excluded_sources: self.excluded_sources,
        }
    }
}

impl Default for AssembledContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(source: ContextSource, confidence: f64) -> ContextSnapshot {
        let mut s = ContextSnapshot::new(source, serde_json::json!({"test": true}));
        s.confidence = confidence;
        s
    }

    #[test]
    fn test_build_assembled_context() {
        let ctx = AssembledContextBuilder::new()
            .with_data(serde_json::json!({"key": "value"}))
            .add_source(make_snapshot(ContextSource::Environment, 0.9))
            .add_source(make_snapshot(ContextSource::Activity, 0.7))
            .build();

        assert_eq!(ctx.source_count, 2);
        assert!(ctx.overall_confidence > 0.7);
        assert!(ctx.has_source(&ContextSource::Environment));
        assert!(ctx.has_source(&ContextSource::Activity));
    }

    #[test]
    fn test_get_field() {
        let ctx = AssembledContextBuilder::new()
            .with_data(serde_json::json!({"user": {"name": "Alice"}}))
            .build();

        assert_eq!(ctx.field("user.name"), Some(&serde_json::json!("Alice")));
        assert!(ctx.field("user.age").is_none());
    }

    #[test]
    fn test_primary_source() {
        let ctx = AssembledContextBuilder::new()
            .add_source(make_snapshot(ContextSource::SystemState, 0.9))
            .add_source(make_snapshot(ContextSource::Environment, 0.8))
            .build();

        assert_eq!(ctx.primary_source(), Some(&ContextSource::SystemState));
    }

    #[test]
    fn test_excluded_sources() {
        let ctx = AssembledContextBuilder::new()
            .add_source(make_snapshot(ContextSource::Environment, 0.9))
            .exclude_source(ContextSource::Activity)
            .build();

        assert_eq!(ctx.excluded_sources.len(), 1);
    }

    #[test]
    fn test_empty_context() {
        let ctx = AssembledContextBuilder::new().build();
        assert_eq!(ctx.source_count, 0);
        assert_eq!(ctx.overall_confidence, 0.0);
    }
}
