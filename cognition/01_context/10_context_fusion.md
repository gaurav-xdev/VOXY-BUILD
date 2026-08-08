# Context Fusion

## Purpose

Context Fusion is the brain of the context system. It combines all context sources — environment, user, conversation, visual, audio, memory, device, activity, and emotional — into a single unified `AssembledContext` object that the cognitive system consumes. It answers: *Given everything we know about the user's situation, what is the most relevant context for the current cognitive cycle?* This module implements the `ContextAssembler` trait from the `cognition` crate and is the primary integration point between the context system and the cognitive system.

## Responsibilities

1. **Context aggregation**: Collect context from all context sources
2. **Priority assignment**: Assign priority to each context source
3. **Conflict resolution**: Resolve conflicts between context sources
4. **Confidence scoring**: Score confidence in combined context
5. **Freshness management**: Track and manage context freshness
6. **Context decay**: Implement natural decay of stale context
7. **Context merging**: Merge overlapping context information
8. **Context compression**: Compress context for efficiency
9. **Context caching**: Cache context for repeated access
10. **Context synchronization**: Synchronize context across sources

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       CONTEXT FUSION                                 │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    CONTEXT SOURCES                            │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │Environment│ │  User   │  │Convers- │  │   Visual     │   │   │
│  │  │ Context  │  │ Context │  │ation    │  │   Context    │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  │  ┌────┴────┐  ┌────┴────┐  ┌────┴────┐  ┌──────┴───────┐   │   │
│  │  │  Audio  │  │ Memory  │  │ Device  │  │  Activity    │   │   │
│  │  │ Context │  │ Context │  │ Context │  │  Context     │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  │       │       ┌────┴────┐       │               │            │   │
│  │       │       │Emotional│       │               │            │   │
│  │       │       │ Context │       │               │            │   │
│  │       │       └────┬────┘       │               │            │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              ContextFusionEngine                              │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Priority System                                         │  │   │
│  │  │  - Assign priority to each source                       │  │   │
│  │  │  - Weight by relevance to current intent                │  │   │
│  │  │  - Adjust for recency and freshness                     │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Conflict Resolution                                    │  │   │
│  │  │  - Detect conflicts between sources                     │  │   │
│  │  │  - Resolve by priority, confidence, recency             │  │   │
│  │  │  - Log conflicts for debugging                          │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Confidence Scoring                                     │  │   │
│  │  │  - Score each context piece                             │  │   │
│  │  │  - Aggregate confidence                                 │  │   │
│  │  │  - Disclose uncertainty                                 │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Freshness Management                                   │  │   │
│  │  │  - Track freshness per source                           │  │   │
│  │  │  - Decay stale context                                  │  │   │
│  │  │  - Refresh when stale                                   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Context Merging                                        │  │   │
│  │  │  - Merge overlapping information                        │  │   │
│  │  │  - Deduplicate                                          │  │   │
│  │  │  - Compress for efficiency                              │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Caching                                                │  │   │
│  │  │  - Cache assembled context                              │  │   │
│  │  │  - Invalidate on source change                          │  │   │
│  │  │  - TTL-based expiration                                 │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              AssembledContext                                  │   │
│  │  Unified context object consumed by cognitive system          │   │
│  │  Implements: cognition::ContextAssembler                      │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Context Source Snapshots

```rust
pub struct ContextSourceInput {
    /// Source identifier
    pub source_id: String,
    
    /// Source type
    pub source_type: ContextSourceType,
    
    /// Source snapshot
    pub snapshot: Box<dyn ContextSnapshot>,
    
    /// Source priority
    pub priority: ContextPriority,
    
    /// Source confidence
    pub confidence: f64,
    
    /// Source freshness (seconds since capture)
    pub freshness: u64,
    
    /// Source relevance to current intent
    pub relevance: f64,
    
    /// Source timestamp
    pub captured_at: DateTime<Utc>,
}

pub enum ContextSourceType {
    Environment,
    User,
    Conversation,
    Visual,
    Audio,
    Memory,
    Device,
    Activity,
    Emotional,
    Personality,
    SystemState,
    ExternalService(String),
}

pub enum ContextPriority {
    /// Critical - always included
    Critical,
    
    /// High - included unless conflicting
    High,
    
    /// Medium - included if relevant
    Medium,
    
    /// Low - included if space allows
    Low,
    
    /// Background - included only if needed
    Background,
}

pub trait ContextSnapshot: Send + Sync {
    /// Get source type
    fn source_type(&self) -> ContextSourceType;
    
    /// Get freshness (seconds since capture)
    fn freshness(&self) -> u64;
    
    /// Get confidence (0.0-1.0)
    fn confidence(&self) -> f64;
    
    /// Get relevance to current intent
    fn relevance(&self, intent: &IntentAnalysis) -> f64;
    
    /// Get size estimate (bytes)
    fn size_estimate(&self) -> usize;
    
    /// Convert to JSON for merging
    fn to_json(&self) -> serde_json::Value;
    
    /// Merge with another snapshot of same type
    fn merge(&mut self, other: &dyn ContextSnapshot);
}
```

## Outputs

### Assembled Context

```rust
pub struct AssembledContext {
    /// Context identifier
    pub id: ContextId,
    
    /// Assembly timestamp
    pub assembled_at: DateTime<Utc>,
    
    /// Context sources used
    pub sources: Vec<ContextSource>,
    
    /// World snapshot (from world_model crate)
    pub world_snapshot: Option<WorldSnapshot>,
    
    /// Personality context
    pub personality_context: Option<serde_json::Value>,
    
    /// Relevant history
    pub relevant_history: Vec<String>,
    
    /// Constraints
    pub constraints: Vec<String>,
    
    /// Priority hints
    pub priority_hints: Vec<String>,
    
    /// Assembly time (ms)
    pub assembly_time_ms: u64,
    
    /// Overall confidence
    pub confidence: f64,
    
    /// Overall freshness
    pub freshness: u64,
    
    /// Size estimate (bytes)
    pub size_estimate: usize,
    
    /// Conflict log
    pub conflicts: Vec<ContextConflict>,
    
    /// Compression info
    pub compression: Option<CompressionInfo>,
}

pub struct ContextSource {
    /// Source type
    pub source_type: ContextSourceType,
    
    /// Source priority
    pub priority: ContextPriority,
    
    /// Source confidence
    pub confidence: f64,
    
    /// Source freshness
    pub freshness: u64,
    
    /// Source relevance
    pub relevance: f64,
    
    /// Source data (merged)
    pub data: serde_json::Value,
}

pub struct ContextConflict {
    /// Conflict identifier
    pub id: String,
    
    /// Conflicting sources
    pub sources: Vec<ContextSourceType>,
    
    /// Conflict description
    pub description: String,
    
    /// Resolution method
    pub resolution: ConflictResolution,
    
    /// Resolution confidence
    pub confidence: f64,
    
    /// Conflict timestamp
    pub timestamp: DateTime<Utc>,
}

pub enum ConflictResolution {
    /// Used highest priority source
    HighestPriority,
    
    /// Used most confident source
    MostConfident,
    
    /// Used most recent source
    MostRecent,
    
    /// Merged information
    Merged,
    
    /// Used user-explicit value
    UserExplicit,
    
    /// Deferred to user
    DeferredToUser,
}

pub struct CompressionInfo {
    /// Original size (bytes)
    pub original_size: usize,
    
    /// Compressed size (bytes)
    pub compressed_size: usize,
    
    /// Compression ratio
    pub ratio: f64,
    
    /// Compression method
    pub method: CompressionMethod,
    
    /// Information lost
    pub information_lost: bool,
}

pub enum CompressionMethod {
    Deduplication,
    Truncation,
    Summarization,
    Sampling,
    None,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  CONTEXT FUSION STATE MACHINE                        │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (all sources registered)                                │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   COLLECTING     │────▶│   DEGRADED       │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (all sources collected) │ (source failure)               │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   FUSING         │◀────│   COLLECTING     │                     │
│  └────────┬─────────┘     └──────────────────┘                     │
│           │ (fusion complete)                                        │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   CACHING        │                                               │
│  └────────┬─────────┘                                               │
│           │ (cached)                                                 │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   READY          │                                               │
│  └────────┬─────────┘                                               │
│           │ (source change)                                          │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   INVALIDATING   │──────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (invalidated)                                   │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   COLLECTING     │──────────────────────────────────────┘        │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Context Priority System

```rust
fn calculate_source_priority(
    source: &ContextSourceInput,
    intent: &IntentAnalysis,
    current_context: &Option<AssembledContext>,
) -> f64 {
    let mut priority = 0.0;
    
    // Base priority from source type
    priority += match source.source_type {
        ContextSourceType::Conversation => 100.0,
        ContextSourceType::User => 90.0,
        ContextSourceType::Environment => 80.0,
        ContextSourceType::Visual => 70.0,
        ContextSourceType::Audio => 60.0,
        ContextSourceType::Activity => 50.0,
        ContextSourceType::Emotional => 40.0,
        ContextSourceType::Memory => 30.0,
        ContextSourceType::Device => 20.0,
        ContextSourceType::Personality => 10.0,
        ContextSourceType::SystemState => 5.0,
        ContextSourceType::ExternalService(_) => 15.0,
    };
    
    // Relevance bonus
    priority += source.relevance * 50.0;
    
    // Freshness bonus (fresh context is more valuable)
    let freshness_factor = 1.0 / (1.0 + source.freshness as f64 / 60.0);
    priority *= freshness_factor;
    
    // Confidence bonus
    priority *= source.confidence;
    
    // Intent relevance bonus
    let intent_relevance = calculate_intent_relevance(source, intent);
    priority += intent_relevance * 30.0;
    
    // Priority level modifier
    priority += match source.priority {
        ContextPriority::Critical => 200.0,
        ContextPriority::High => 100.0,
        ContextPriority::Medium => 0.0,
        ContextPriority::Low => -50.0,
        ContextPriority::Background => -100.0,
    };
    
    priority
}
```

### Conflict Resolution

```rust
fn resolve_conflicts(
    sources: &[ContextSourceInput],
) -> (Vec<ContextSourceInput>, Vec<ContextConflict>) {
    let mut resolved = Vec::new();
    let mut conflicts = Vec::new();
    
    // Group sources by information type
    let grouped = group_by_information_type(sources);
    
    for (info_type, group) in &grouped {
        if group.len() <= 1 {
            resolved.extend(group.clone());
            continue;
        }
        
        // Check for conflicts
        if has_conflict(&group) {
            let resolution = resolve_conflict(&group);
            
            conflicts.push(ContextConflict {
                id: Uuid::new_v4().to_string(),
                sources: group.iter().map(|s| s.source_type.clone()).collect(),
                description: format!("Conflict in {} information", info_type),
                resolution: resolution.method,
                confidence: resolution.confidence,
                timestamp: Utc::now(),
            });
            
            resolved.push(resolution.resolved_source);
        } else {
            // No conflict, merge information
            let merged = merge_sources(&group);
            resolved.push(merged);
        }
    }
    
    (resolved, conflicts)
}

fn resolve_conflict(sources: &[ContextSourceInput]) -> ConflictResolution {
    // Strategy 1: User explicit value wins
    if let Some(user_source) = sources.iter().find(|s| s.source_type == ContextSourceType::User) {
        return ConflictResolution {
            method: ConflictResolution::UserExplicit,
            confidence: 1.0,
            resolved_source: user_source.clone(),
        };
    }
    
    // Strategy 2: Most confident source wins
    let most_confident = sources.iter()
        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    
    if most_confident.confidence > 0.8 {
        return ConflictResolution {
            method: ConflictResolution::MostConfident,
            confidence: most_confident.confidence,
            resolved_source: most_confident.clone(),
        };
    }
    
    // Strategy 3: Most recent source wins
    let most_recent = sources.iter()
        .max_by_key(|s| s.captured_at)
        .unwrap();
    
    ConflictResolution {
        method: ConflictResolution::MostRecent,
        confidence: 0.6,
        resolved_source: most_recent.clone(),
    }
}
```

### Context Freshness Management

```rust
fn manage_freshness(
    context: &mut AssembledContext,
    config: &FusionConfig,
) {
    let now = Utc::now();
    
    for source in &mut context.sources {
        // Calculate freshness factor
        let age_seconds = source.freshness as f64;
        let max_age = config.max_age_for_source(&source.source_type);
        
        let freshness_factor = if age_seconds > max_age {
            0.0 // Completely stale
        } else {
            1.0 - (age_seconds / max_age)
        };
        
        // Apply freshness decay to confidence
        source.confidence *= freshness_factor;
        
        // Apply freshness decay to priority
        if freshness_factor < 0.5 {
            // Stale context gets lower priority
            source.priority = ContextPriority::Low;
        }
    }
    
    // Update overall freshness
    context.freshness = context.sources.iter()
        .map(|s| s.freshness)
        .min()
        .unwrap_or(0);
    
    // Update overall confidence
    context.confidence = context.sources.iter()
        .map(|s| s.confidence)
        .sum::<f64>() / context.sources.len() as f64;
}
```

### Context Compression

```rust
fn compress_context(
    context: &mut AssembledContext,
    target_size: usize,
) {
    if context.size_estimate <= target_size {
        return;
    }
    
    // Strategy 1: Deduplicate sources
    deduplicate_sources(&mut context.sources);
    
    if context.size_estimate <= target_size {
        return;
    }
    
    // Strategy 2: Remove low-priority sources
    remove_low_priority_sources(&mut context.sources, target_size);
    
    if context.size_estimate <= target_size {
        return;
    }
    
    // Strategy 3: Truncate large sources
    truncate_large_sources(&mut context.sources, target_size);
    
    // Update compression info
    context.compression = Some(CompressionInfo {
        original_size: context.size_estimate,
        compressed_size: calculate_size(&context.sources),
        ratio: calculate_size(&context.sources) as f64 / context.size_estimate as f64,
        method: CompressionMethod::Deduplication,
        information_lost: true,
    });
}
```

## Decision Logic

### When to Reassemble Context

```rust
fn should_reassemble(
    current: &Option<AssembledContext>,
    source_changes: &[ContextSourceType],
    config: &FusionConfig,
) -> bool {
    // Always reassemble if no current context
    if current.is_none() {
        return true;
    }
    
    // Reassemble if critical source changed
    if source_changes.iter().any(|s| is_critical_source(s)) {
        return true;
    }
    
    // Reassemble if context is stale
    if let Some(ctx) = current {
        if ctx.freshness > config.max_staleness_seconds {
            return true;
        }
    }
    
    // Reassemble if confidence is low
    if let Some(ctx) = current {
        if ctx.confidence < config.reassemble_threshold {
            return true;
        }
    }
    
    // Reassemble periodically
    if should_periodic_reassembly(config) {
        return true;
    }
    
    false
}
```

### When to Invalidate Cache

```rust
fn should_invalidate_cache(
    source_change: &ContextSourceType,
    config: &FusionConfig,
) -> bool {
    // Always invalidate if critical source changed
    if is_critical_source(source_change) {
        return true;
    }
    
    // Invalidate if source is in invalidation list
    if config.invalidate_on_change.contains(source_change) {
        return true;
    }
    
    // Don't invalidate for low-priority sources
    if is_low_priority_source(source_change) {
        return false;
    }
    
    true
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Source failure | Missing source | Use cached/last-known | Multiple source fallbacks |
| Fusion timeout | Timeout exceeded | Use partial context | Timeout configuration |
| Cache corruption | Integrity check fails | Reassemble from sources | Cache validation |
| Conflict resolution failure | Unresolvable conflict | Log and use highest priority | Better conflict detection |
| Stale context | Freshness threshold exceeded | Force reassembly | Adaptive freshness thresholds |
| Memory pressure | Size threshold exceeded | Compress context | Size limits |

### Recovery Strategy

```rust
impl ContextFusionEngine {
    async fn recover_from_source_failure(
        &self,
        failed_source: &ContextSourceType,
    ) -> AssembledContext {
        // Try cached version
        if let Some(cached) = self.cache.get_by_source(failed_source).await {
            tracing::warn!(
                source = ?failed_source,
                "Using cached context due to source failure"
            );
            return cached;
        }
        
        // Try last-known good context
        if let Some(last_known) = self.last_known_context.read().await.as_ref() {
            tracing::warn!(
                source = ?failed_source,
                "Using last-known context due to source failure"
            );
            return last_known.clone();
        }
        
        // Assemble without failed source
        tracing::warn!(
            source = ?failed_source,
            "Assembling context without failed source"
        );
        self.assemble_without_source(failed_source).await
    }
}
```

## Performance Considerations

### Latency Budget

| Operation | Target | Maximum | Measurement |
|-----------|--------|---------|-------------|
| Source collection | 2ms | 5ms | Per source |
| Priority calculation | 0.5ms | 1ms | Per source |
| Conflict resolution | 1ms | 2ms | Per conflict |
| Context merging | 1ms | 3ms | Per merge |
| Freshness management | 0.5ms | 1ms | Per source |
| Compression | 1ms | 3ms | Per compression |
| Cache lookup | 0.1ms | 0.5ms | Per lookup |
| **Total** | **6ms** | **15ms** | **Per assembly** |

### Optimization Strategies

1. **Incremental assembly**: Only reassemble changed sources
2. **Parallel collection**: Collect sources in parallel
3. **Lazy evaluation**: Evaluate relevance only when needed
4. **Cache warming**: Pre-warm cache for common contexts
5. **Source pooling**: Pool similar sources for efficiency

## Privacy Considerations

1. **Context data**: All context data is processed locally, never transmitted.
2. **Fusion logs**: Conflict logs are stored locally, never transmitted.
3. **Cache data**: Cache is stored locally, never transmitted.
4. **User control**: Users can view and modify all context data.
5. **No profiling**: Context fusion is not used for advertising or profiling.
6. **Data retention**: Context data is retained according to user-configured policy.
7. **Sensitive context**: Emotional and health context are treated with extra care.

## Security Considerations

1. **Data storage**: All context data is stored in encrypted local database.
2. **Access control**: Only authorized COS components can access fused context.
3. **Integrity**: Fused context is tamper-evident.
4. **Audit logging**: Context fusion is auditable.
5. **No remote transmission**: Fused context never leaves the device without explicit consent.
6. **Secure caching**: Cache is encrypted and integrity-protected.

## Future Extensibility

1. **ML-based fusion**: Machine learning for optimal context weighting
2. **Predictive fusion**: Anticipate context needs before they arise
3. **Distributed fusion**: Fuse context across multiple devices
4. **Collaborative fusion**: Fuse context from multiple users
5. **Real-time fusion**: Sub-millisecond context fusion
6. **Adaptive fusion**: Learn optimal fusion strategies per user
7. **Context visualization**: Visualize fused context for debugging

## Examples

### Example 1: Simple Context Assembly

```
Sources: [
  Environment { freshness: 5s, confidence: 0.9, priority: High },
  User { freshness: 30s, confidence: 0.8, priority: High },
  Conversation { freshness: 1s, confidence: 0.95, priority: Critical },
  Visual { freshness: 10s, confidence: 0.7, priority: Medium },
]
AssembledContext {
  confidence: 0.85,
  freshness: 1s,
  sources: [Conversation, Environment, User, Visual],
  world_snapshot: Some(WorldSnapshot { ... }),
  constraints: ["user is in focus mode", "system under moderate load"],
  priority_hints: ["respond concisely", "avoid interruptions"],
}
```

### Example 2: Conflict Resolution

```
Conflict: Environment says "nighttime", User says "working hours"
Resolution: UserExplicit (user override)
Context: { is_night: true, is_working_hours: true, confidence: 0.9 }
Note: User-explicit values always win over inferred values
```

### Example 3: Context Compression

```
Original: 50KB context, 10 sources
Target: 20KB
Compression: Deduplicated 2 sources, truncated 3 sources, removed 1 low-priority source
Result: 18KB context, 8 sources, information_lost: true
Compression ratio: 0.36
```

## Engineering Notes

- Context fusion implements the `ContextAssembler` trait from `cognition` crate
- `AssembledContext` is the primary output type consumed by the cognitive system
- Fusion runs on a dedicated tokio task to avoid blocking the main pipeline
- Cache uses LRU eviction with configurable size limit (default: 10 entries)
- Conflict resolution is deterministic given the same inputs
- Freshness decay uses exponential decay with configurable half-life
- All timestamps use `chrono::DateTime<Utc>` for consistency
- Fusion metrics are collected via the `metrics` crate
- Fusion state is serialized for debugging and inspection
