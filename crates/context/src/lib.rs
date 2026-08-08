//! # voxy-context
//!
//! Context runtime for VOXY — provides context types, traits, cache, manager, and fusion engine.
//!
//! This crate is the foundation for VOXY's context-aware system. It defines:
//! - Core types: `ContextId`, `ContextSnapshot`, `ContextUpdate`, `ContextSource`
//! - Traits: `ContextProvider`, `ContextRegistry`
//! - `ContextCache` with LRU eviction
//! - `ContextManager` that orchestrates context collection and assembly
//! - `ContextFusionEngine` that merges, resolves conflicts, and produces `AssembledContext`

pub mod cache;
pub mod error;
pub mod fusion;
pub mod manager;
pub mod provider;
pub mod providers;
pub mod registry;
pub mod types;

pub use cache::{CacheConfig, CacheStats, ContextCache};
pub use error::{ContextError, Result};
pub use fusion::{
    AssembledContext, AssembledContextBuilder, ConfidenceEngine, ConflictResolution,
    ConflictResolutionResult, ContextConflict, ContextConflictResolver, ContextDelta,
    ContextDeltaGenerator, ContextFusionEngine, ContextInvalidation, ContextMerger,
    ContextPriorityResolver, FreshnessEngine, FreshnessStatus, FusionPolicy, InvalidationResult,
    MergeStrategy,
};
pub use manager::{ContextManager, ContextSnapshotSet, ManagerConfig};
pub use provider::ContextProvider;
pub use providers::{
    ActivityContextProvider, ConversationContextProvider, DeviceContextProvider,
    EnvironmentContextProvider, MemoryContextProvider,
};
pub use registry::ContextRegistry;
pub use types::{
    ContextId, ContextPriority, ContextSnapshot, ContextSource, ContextUpdate, FreshnessConfig,
};
