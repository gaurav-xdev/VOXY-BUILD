# Memory Context

## Purpose

The Memory Context module bridges the gap between the context system and the memory system. It determines which memories are relevant to the current context, prioritizes memory retrieval, manages working memory, and ensures that memory interacts coherently with all context sources. It answers: *What memories are relevant right now? What should be remembered? What can be forgotten? How does context influence memory retrieval?*

## Responsibilities

1. **Context-driven retrieval**: Retrieve memories relevant to current context
2. **Memory prioritization**: Prioritize which memories to retrieve based on context
3. **Working memory management**: Manage the limited-capacity working memory
4. **Memory-context synchronization**: Keep memory and context consistent
5. **Memory compression**: Compress older memories to save resources
6. **Memory decay**: Implement natural forgetting of less relevant memories
7. **Memory consolidation**: Move important short-term memories to long-term storage

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      MEMORY CONTEXT                                  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    CONTEXT INPUTS                             │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │Current  │  │ Topic   │  │ Activity│  │ Emotional    │   │   │
│  │  │Context  │  │ Context │  │ Context │  │ Context      │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              MemoryContextManager                             │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Retrieval Pipeline                                     │  │   │
│  │  │  Context → Query Generation → Memory Search → Ranking   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Working Memory Pipeline                               │  │   │
│  │  │  Current Context → Working Memory Update → Eviction     │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Consolidation Pipeline                                │  │   │
│  │  │  Short-term → Importance Scoring → Long-term Storage    │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              MemoryContextSnapshot                            │   │
│  │  Point-in-time view of memory-context interaction            │   │
│  │  Consumed by: Cognition, Reflection, Learning               │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Memory Signals

```rust
pub struct MemorySignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: MemorySignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum MemorySignalType {
    /// New memory to store
    StoreMemory {
        content: String,
        memory_type: MemoryType,
        importance: f64,
        context: Option<String>,
    },
    
    /// Query for relevant memories
    QueryMemories {
        query: String,
        context: Option<String>,
        limit: usize,
    },
    
    /// Memory accessed (for importance scoring)
    MemoryAccessed {
        memory_id: String,
        access_type: AccessType,
    },
    
    /// Memory outdated (for decay)
    MemoryOutdated {
        memory_id: String,
        reason: OutdatedReason,
    },
    
    /// Working memory full
    WorkingMemoryFull {
        current_count: usize,
        max_count: usize,
    },
    
    /// Context changed (triggers memory re-evaluation)
    ContextChanged {
        old_context: Option<String>,
        new_context: String,
    },
}

pub enum MemoryType {
    Episodic,
    Semantic,
    Procedural,
    Working,
}

pub enum AccessType {
    Read,
    Write,
    Update,
    Delete,
}

pub enum OutdatedReason {
    TimeDecay,
    RelevanceDecay,
    UserExplicit,
    Consolidation,
}
```

## Outputs

### Memory Context Snapshot

```rust
pub struct MemoryContextSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Working memory contents
    pub working_memory: WorkingMemoryState,
    
    /// Retrieved memories
    pub retrieved_memories: Vec<RetrievedMemory>,
    
    /// Memory retrieval query
    pub retrieval_query: Option<String>,
    
    /// Memory retrieval latency
    pub retrieval_latency_ms: u64,
    
    /// Memory relevance scores
    pub relevance_scores: HashMap<String, f64>,
    
    /// Memory consolidation status
    pub consolidation: ConsolidationStatus,
    
    /// Memory system health
    pub health: MemoryHealth,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct WorkingMemoryState {
    /// Current working memory items
    pub items: Vec<WorkingMemoryItem>,
    
    /// Current capacity usage (0.0-1.0)
    pub capacity_usage: f64,
    
    /// Maximum capacity
    pub max_capacity: usize,
    
    /// Items evicted since last cycle
    pub evicted_count: u32,
    
    /// Average item age
    pub avg_item_age: Duration,
}

pub struct WorkingMemoryItem {
    /// Item identifier
    pub id: String,
    
    /// Item content
    pub content: String,
    
    /// Item type
    pub item_type: WorkingMemoryItemType,
    
    /// Item importance (0.0-1.0)
    pub importance: f64,
    
    /// Item recency (0.0-1.0)
    pub recency: f64,
    
    /// Item context relevance (0.0-1.0)
    pub context_relevance: f64,
    
    /// Combined score
    pub score: f64,
    
    /// When item was added
    pub added_at: DateTime<Utc>,
    
    /// When item was last accessed
    pub last_accessed_at: DateTime<Utc>,
    
    /// Access count
    pub access_count: u32,
}

pub enum WorkingMemoryItemType {
    CurrentTask,
    CurrentTopic,
    RecentMessage,
    ImportantFact,
    UserPreference,
    SystemState,
    Temporary,
}

pub struct RetrievedMemory {
    /// Memory identifier
    pub id: String,
    
    /// Memory content
    pub content: String,
    
    /// Memory type
    pub memory_type: MemoryType,
    
    /// Relevance score
    pub relevance: f64,
    
    /// Memory source
    pub source: String,
    
    /// Memory timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Memory context
    pub context: Option<String>,
    
    /// Memory metadata
    pub metadata: HashMap<String, String>,
}

pub struct ConsolidationStatus {
    /// Short-term memory count
    pub short_term_count: u32,
    
    /// Long-term memory count
    pub long_term_count: u32,
    
    /// Pending consolidation
    pub pending_consolidation: u32,
    
    /// Last consolidation time
    pub last_consolidation: Option<DateTime<Utc>>,
    
    /// Consolidation health
    pub health: ConsolidationHealth,
}

pub enum ConsolidationHealth {
    Healthy,
    Backlog,
    Stalled,
    Failed,
}

pub struct MemoryHealth {
    /// Overall health
    pub overall: MemoryHealthLevel,
    
    /// Retrieval latency
    pub retrieval_latency_ms: u64,
    
    /// Storage usage
    pub storage_usage: f64,
    
    /// Fragmentation level
    pub fragmentation: f64,
    
    /// Error rate
    pub error_rate: f64,
}

pub enum MemoryHealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  MEMORY CONTEXT STATE MACHINE                        │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (memory system loaded)                                  │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   ACTIVE         │────▶│   DEGRADED       │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (context change)        │ (memory system restored)       │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   RETRIEVING     │────▶│   ACTIVE         │                     │
│  └────────┬─────────┘     └──────────────────┘                     │
│           │ (retrieval complete)                                    │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   CONSOLIDATING  │                                               │
│  └────────┬─────────┘                                               │
│           │ (consolidation complete)                                │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   ACTIVE         │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### When to Retrieve Memories

```rust
fn should_retrieve_memories(
    context: &AssembledContext,
    working_memory: &WorkingMemoryState,
    config: &MemoryConfig,
) -> bool {
    // Retrieve if working memory is low
    if working_memory.capacity_usage < config.retrieve_threshold {
        return true;
    }
    
    // Retrieve if context changed significantly
    if context_changed_significantly(context) {
        return true;
    }
    
    // Retrieve if explicit query
    if has_explicit_memory_query(context) {
        return true;
    }
    
    // Retrieve periodically
    if should_periodic_retrieval(config) {
        return true;
    }
    
    false
}
```

### When to Consolidate Memories

```rust
fn should_consolidate(
    status: &ConsolidationStatus,
    config: &MemoryConfig,
) -> bool {
    // Consolidate if backlog is large
    if status.pending_consolidation > config.consolidation_threshold {
        return true;
    }
    
    // Consolidate periodically
    if should_periodic_consolidation(status, config) {
        return true;
    }
    
    // Consolidate if short-term is full
    if status.short_term_count > config.max_short_term {
        return true;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Retrieval failure | Empty results, low confidence | Retry with broader query | Fallback retrieval strategies |
| Working memory overflow | Capacity exceeded | Evict lowest-score items | Adaptive capacity limits |
| Consolidation failure | Backlog growing | Retry consolidation | Robust consolidation pipeline |
| Memory corruption | Integrity check fails | Restore from backup | Regular integrity checks |
| Relevance drift | Low retrieval accuracy | Re-calibrate relevance scoring | Continuous learning |
| Storage exhaustion | Storage usage > threshold | Archive old memories | Proactive storage management |

### Recovery Strategy

```rust
impl MemoryContextManager {
    async fn recover_from_retrieval_failure(&self, query: &str) -> Vec<RetrievedMemory> {
        // Try broader query
        let broader_query = broaden_query(query);
        if let Some(results) = self.retry_retrieval(&broader_query).await {
            return results;
        }
        
        // Try semantic similarity
        if let Some(results) = self.semantic_retrieval(query).await {
            return results;
        }
        
        // Fall back to recent memories
        self.get_recent_memories().await
    }
}
```

## Privacy Considerations

1. **Memory content**: All memories are stored locally, never transmitted.
2. **Memory queries**: Memory queries are processed locally, never transmitted.
3. **Working memory**: Working memory is in-process only, never persisted.
4. **Consolidation**: Memory consolidation is local, never transmitted.
5. **Memory access**: Memory access is logged locally for debugging only.
6. **User control**: Users can view, edit, and delete any memory.
7. **Data retention**: Memory retention is user-configurable.
8. **No profiling**: Memories are not used for advertising or profiling.

## Security Considerations

1. **Encryption at rest**: Memories are encrypted using AES-256-GCM.
2. **Access control**: Only authorized COS components can access memories.
3. **Integrity**: Memories are tamper-evident using cryptographic hashes.
4. **Audit logging**: Memory access is auditable.
5. **Secure deletion**: Memories are securely deleted when removed.
6. **No remote storage**: Memories never leave the device without explicit consent.

## Future Extensibility

1. **Distributed memory**: Memory sync across user's devices
2. **Collaborative memory**: Shared memories for team contexts
3. **Memory visualization**: Visualize memory graphs and connections
4. **Memory search**: Advanced search through memories
5. **Memory analytics**: Analyze memory usage patterns
6. **Memory optimization**: AI-driven memory optimization
7. **Memory privacy**: Differential privacy for sensitive memories

## Examples

### Example 1: Context-Driven Retrieval

```
Current Context: { topic: "Rust async", activity: Coding, project: "voxy" }
Working Memory: { capacity_usage: 0.3 }
Retrieval Query: "Rust async patterns, tokio, pin"
Retrieved: [
  { content: "Rust async uses Pin for self-referential futures", relevance: 0.95 },
  { content: "Tokio runtime manages async task scheduling", relevance: 0.9 },
  { content: "VOXY uses tokio for async voice pipeline", relevance: 0.85 },
]
Working Memory Updated: Added retrieved items, evicted 2 low-score items
```

### Example 2: Working Memory Eviction

```
Working Memory: { items: 15, max_capacity: 15, capacity_usage: 1.0 }
Eviction Candidates: [
  { id: "item_12", score: 0.2, added_at: 2h_ago },
  { id: "item_13", score: 0.25, added_at: 1.5h_ago },
]
Evicted: item_12, item_13
New Item Added: { content: "Current task: fix auth bug", importance: 0.9 }
```

## Engineering Notes

- Working memory is in-process `VecDeque` with configurable max size (default: 15)
- Memory retrieval uses the `memory` crate's `MemoryManager`
- Relevance scoring uses cosine similarity between context and memory embeddings
- Consolidation runs periodically (default: every 5 minutes)
- Memory decay uses exponential decay with configurable half-life
- Memory health is monitored via the `health` crate
- All timestamps use `chrono::DateTime<Utc>` for consistency
