# Conversation Context

## Purpose

The Conversation Context module manages the state and history of the ongoing dialogue between the user and VOXY. It answers: *What are we talking about? What was said before? What is the current intent? What happens if we lose connection?* This module wraps the `conversation` crate's `ConversationContext` and `ContextTracker`, extending them with topic tracking, intent continuity, summarization, compression, and recovery capabilities. It is consumed by the `ContextAssembler` as a context source, and directly feeds into the orchestrator's `Conversation` pipeline stage.

## Responsibilities

1. **Turn management**: Track each user/system turn with metadata
2. **Context key-value store**: Store and retrieve arbitrary conversation context
3. **Topic tracking**: Identify and track the current conversation topic
4. **Intent continuity**: Maintain intent across turns and interruptions
5. **Conversation summarization**: Summarize long conversations for context window management
6. **Conversation compression**: Compress older turns to save memory
7. **Interruption handling**: Handle barge-in, topic switches, mid-sentence interruptions
8. **Recovery after restart**: Restore conversation state after process restart
9. **Multi-turn coherence**: Ensure responses are coherent across multiple turns

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                    CONVERSATION CONTEXT                              │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUTS                                     │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │  Voice  │  │  Text   │  │  System │  │  Interruption │   │   │
│  │  │  Input  │  │  Input  │  │  Events │  │  (barge-in)   │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              ConversationManager                              │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  TurnManager (conversation crate)                       │  │   │
│  │  │  - begin_turn(source)                                    │  │   │
│  │  │  - end_turn(output)                                      │  │   │
│  │  │  - interrupt_current()                                   │  │   │
│  │  │  - turn_history(n)                                       │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  ContextTracker (conversation crate)                    │  │   │
│  │  │  - set(key, value)                                       │  │   │
│  │  │  - get(key)                                              │  │   │
│  │  │  - set_current_topic(topic)                              │  │   │
│  │  │  - current_topic()                                       │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  TopicTracker (this module)                             │  │   │
│  │  │  - detect_topic_change()                                 │  │   │
│  │  │  - track_intent_continuity()                             │  │   │
│  │  │  - summarize_conversation()                              │  │   │
│  │  │  - compress_history()                                    │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              ConversationSnapshot                             │   │
│  │  Point-in-time view of all conversation state                 │   │
│  │  Consumed by: Orchestrator, Cognition, Memory                │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Conversation Events

```rust
pub struct ConversationEvent {
    /// Event identifier
    pub id: String,
    
    /// Event type
    pub event_type: ConversationEventType,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Session identifier
    pub session_id: String,
    
    /// User identifier
    pub user_id: Option<String>,
    
    /// Device identifier
    pub device_id: Option<String>,
}

pub enum ConversationEventType {
    /// User sent a message
    UserMessage {
        text: String,
        source: InputSource,
        confidence: f64,
    },
    
    /// System generated a response
    SystemResponse {
        text: String,
        latency_ms: u64,
        model_used: String,
    },
    
    /// Wake word detected
    WakeWord {
        confidence: f64,
        keyword: String,
    },
    
    /// User interrupted system (barge-in)
    Interruption {
        partial_text: Option<String>,
        interrupt_point: Option<usize>,
    },
    
    /// Conversation started
    SessionStarted {
        user_id: Option<String>,
        device_id: Option<String>,
    },
    
    /// Conversation ended
    SessionEnded {
        reason: SessionEndReason,
        duration_ms: u64,
    },
    
    /// Topic changed
    TopicChanged {
        old_topic: Option<String>,
        new_topic: String,
        confidence: f64,
    },
    
    /// Context key set
    ContextSet {
        key: String,
        value: String,
    },
    
    /// Context key removed
    ContextRemoved {
        key: String,
    },
    
    /// Conversation summarized
    Summarized {
        original_turns: usize,
        summary: String,
        compressed_turns: usize,
    },
}

pub enum InputSource {
    Voice,
    Text,
    WakeWord,
    System,
}

pub enum SessionEndReason {
    UserExplicit,
    IdleTimeout,
    SystemShutdown,
    Error,
}
```

## Outputs

### Conversation Snapshot

```rust
pub struct ConversationSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Session identifier
    pub session_id: String,
    
    /// Current turn
    pub current_turn: Option<TurnInfo>,
    
    /// Recent turn history (last N turns)
    pub recent_turns: Vec<TurnInfo>,
    
    /// Current topic
    pub current_topic: Option<TopicInfo>,
    
    /// Topic history
    pub topic_history: Vec<TopicInfo>,
    
    /// Active intents
    pub active_intents: Vec<IntentInfo>,
    
    /// Context key-value pairs
    pub context_entries: HashMap<String, String>,
    
    /// Session metadata
    pub session: SessionInfo,
    
    /// Conversation summary (if compressed)
    pub summary: Option<String>,
    
    /// Conversation metrics
    pub metrics: ConversationMetrics,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct TurnInfo {
    /// Turn identifier
    pub id: String,
    
    /// Turn source
    pub source: TurnSource,
    
    /// Turn state
    pub state: TurnState,
    
    /// Input text
    pub input_text: Option<String>,
    
    /// Output text
    pub output_text: Option<String>,
    
    /// Turn start time
    pub started_at: DateTime<Utc>,
    
    /// Turn end time
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Turn duration (ms)
    pub duration_ms: Option<f64>,
    
    /// Was interrupted
    pub was_interrupted: bool,
    
    /// Confidence in transcription/input
    pub confidence: f32,
}

pub struct TopicInfo {
    /// Topic identifier
    pub id: String,
    
    /// Topic name
    pub name: String,
    
    /// Topic description
    pub description: Option<String>,
    
    /// Topic start time
    pub started_at: DateTime<Utc>,
    
    /// Topic end time
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Turn count for this topic
    pub turn_count: u32,
    
    /// Topic confidence
    pub confidence: f64,
    
    /// Related topics
    pub related_topics: Vec<String>,
}

pub struct IntentInfo {
    /// Intent identifier
    pub id: String,
    
    /// Intent type
    pub intent_type: String,
    
    /// Intent description
    pub description: String,
    
    /// Intent confidence
    pub confidence: f64,
    
    /// Intent created at
    pub created_at: DateTime<Utc>,
    
    /// Intent status
    pub status: IntentStatus,
    
    /// Related intents
    pub related_intents: Vec<String>,
}

pub enum IntentStatus {
    Active,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

pub struct SessionInfo {
    /// Session identifier
    pub id: String,
    
    /// Session start time
    pub started_at: DateTime<Utc>,
    
    /// Session duration
    pub duration: Duration,
    
    /// Total turns
    pub total_turns: u32,
    
    /// User turns
    pub user_turns: u32,
    
    /// System turns
    pub system_turns: u32,
    
    /// Interrupted turns
    pub interrupted_turns: u32,
    
    /// Session state
    pub state: SessionState,
    
    /// User identifier
    pub user_id: Option<String>,
    
    /// Device identifier
    pub device_id: Option<String>,
}

pub enum SessionState {
    Active,
    Paused,
    Ended,
}

pub struct ConversationMetrics {
    /// Average turn duration (ms)
    pub avg_turn_duration_ms: f64,
    
    /// Average user message length (chars)
    pub avg_user_message_length: f64,
    
    /// Average system response length (chars)
    pub avg_system_response_length: f64,
    
    /// Interruption rate (interruptions per turn)
    pub interruption_rate: f64,
    
    /// Topic switch rate (switches per turn)
    pub topic_switch_rate: f64,
    
    /// Context key count
    pub context_key_count: u32,
    
    /// Total tokens used (estimate)
    pub total_tokens: u64,
    
    /// Estimated context window usage (0.0-1.0)
    pub context_window_usage: f64,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                CONVERSATION CONTEXT STATE MACHINE                     │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   NO_SESSION     │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (session start)                                 │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   IDLE           │◀──────────────────────┐              │        │
│  └────────┬─────────┘                        │              │        │
│           │ (user input)                     │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                        │              │        │
│  │   PROCESSING     │                        │              │        │
│  └────────┬─────────┘                        │              │        │
│           │                                  │              │        │
│     ┌─────┴─────┐                           │              │        │
│     │           │                           │              │        │
│     ▼           ▼                           │              │        │
│  ┌──────┐  ┌──────────┐                    │              │        │
│  │INTERRUPTED│  │RESPONDING│                  │              │        │
│  └──────┘  └────┬─────┘                    │              │        │
│     │           │ (response complete)       │              │        │
│     └───────────┴──────────────────────────┘              │        │
│           │ (response complete)                            │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   IDLE           │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │ (session end)                                   │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   ENDED          │──────────────────────────────────────┘        │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Topic Detection

```rust
fn detect_topic_change(
    turns: &[TurnInfo],
    current_topic: &Option<TopicInfo>,
) -> Option<TopicChange> {
    // Need at least 2 turns to detect topic change
    if turns.len() < 2 {
        return None;
    }
    
    let recent_turns = &turns[turns.len().saturating_sub(3)..];
    
    // Check for explicit topic markers
    if let Some(explicit_topic) = detect_explicit_topic_marker(recent_turns) {
        return Some(TopicChange {
            new_topic: explicit_topic.name,
            confidence: explicit_topic.confidence,
            reason: TopicChangeReason::ExplicitMarker,
        });
    }
    
    // Check for semantic shift
    if let Some(semantic_shift) = detect_semantic_shift(recent_turns) {
        if semantic_shift.significance > TOPIC_CHANGE_THRESHOLD {
            return Some(TopicChange {
                new_topic: semantic_shift.suggested_topic,
                confidence: semantic_shift.significance,
                reason: TopicChangeReason::SemanticShift,
            });
        }
    }
    
    // Check for question pattern change
    if let Some(pattern_change) = detect_question_pattern_change(recent_turns) {
        return Some(TopicChange {
            new_topic: pattern_change.suggested_topic,
            confidence: pattern_change.confidence,
            reason: TopicChangeReason::QuestionPattern,
        });
    }
    
    None
}

fn detect_explicit_topic_marker(turns: &[TurnInfo]) -> Option<TopicMarker> {
    let markers = [
        "let's talk about",
        "moving on to",
        "switching topics",
        "back to",
        "about that",
        "regarding",
        "concerning",
        "on the topic of",
        "about",
    ];
    
    for turn in turns {
        if let Some(text) = &turn.input_text {
            let text_lower = text.to_lowercase();
            for marker in &markers {
                if let Some(pos) = text_lower.find(marker) {
                    let topic_start = pos + marker.len();
                    let topic_text = text[topic_start..].trim().to_string();
                    if !topic_text.is_empty() && topic_text.len() < 100 {
                        return Some(TopicMarker {
                            name: topic_text,
                            confidence: 0.9,
                        });
                    }
                }
            }
        }
    }
    
    None
}
```

### Intent Continuity

```rust
fn track_intent_continuity(
    current_intents: &[IntentInfo],
    new_turn: &TurnInfo,
) -> Vec<IntentUpdate> {
    let mut updates = Vec::new();
    
    // Check if new turn continues existing intent
    for intent in current_intents {
        if is_intent_continuation(intent, new_turn) {
            updates.push(IntentUpdate {
                intent_id: intent.id.clone(),
                update_type: IntentUpdateType::Continued,
                new_status: IntentStatus::InProgress,
                confidence: 0.8,
            });
        }
        
        // Check if new turn completes existing intent
        if is_intent_completion(intent, new_turn) {
            updates.push(IntentUpdate {
                intent_id: intent.id.clone(),
                update_type: IntentUpdateType::Completed,
                new_status: IntentStatus::Completed,
                confidence: 0.9,
            });
        }
        
        // Check if new turn cancels existing intent
        if is_intent_cancellation(intent, new_turn) {
            updates.push(IntentUpdate {
                intent_id: intent.id.clone(),
                update_type: IntentUpdateType::Cancelled,
                new_status: IntentStatus::Cancelled,
                confidence: 0.85,
            });
        }
    }
    
    // Check if new turn creates new intent
    if let Some(new_intent) = detect_new_intent(new_turn) {
        updates.push(IntentUpdate {
            intent_id: new_intent.id,
            update_type: IntentUpdateType::Created,
            new_status: IntentStatus::Active,
            confidence: new_intent.confidence,
        });
    }
    
    updates
}
```

### Conversation Summarization

```rust
fn summarize_conversation(
    turns: &[TurnInfo],
    max_summary_length: usize,
) -> ConversationSummary {
    // Group turns by topic
    let topic_groups = group_turns_by_topic(turns);
    
    // Summarize each topic group
    let mut topic_summaries = Vec::new();
    for (topic, topic_turns) in &topic_groups {
        let summary = summarize_topic_group(topic, topic_turns);
        topic_summaries.push(summary);
    }
    
    // Combine summaries
    let combined_summary = combine_topic_summaries(&topic_summaries);
    
    // Truncate if needed
    let final_summary = if combined_summary.len() > max_summary_length {
        truncate_summary(&combined_summary, max_summary_length)
    } else {
        combined_summary
    };
    
    ConversationSummary {
        summary: final_summary,
        topic_count: topic_groups.len(),
        turn_count: turns.len(),
        compressed_turns: calculate_compressed_turns(turns),
    }
}
```

### Conversation Compression

```rust
fn compress_conversation(
    turns: &[TurnInfo],
    target_token_count: usize,
    current_token_count: usize,
) -> CompressionResult {
    if current_token_count <= target_token_count {
        return CompressionResult {
            compressed_turns: turns.to_vec(),
            removed_turns: 0,
            tokens_saved: 0,
        };
    }
    
    let tokens_to_remove = current_token_count - target_token_count;
    let mut removed_tokens = 0;
    let mut removed_turns = 0;
    let mut compressed_turns = turns.to_vec();
    
    // Remove oldest turns first (skip most recent N turns)
    let min_keep = 4; // Always keep at least 2 user + 2 system turns
    let mut i = 0;
    
    while removed_tokens < tokens_to_remove && i < compressed_turns.len() - min_keep {
        let turn_tokens = estimate_turn_tokens(&compressed_turns[i]);
        removed_tokens += turn_tokens;
        compressed_turns.remove(i);
        removed_turns += 1;
    }
    
    CompressionResult {
        compressed_turns,
        removed_turns,
        tokens_saved: removed_tokens,
    }
}
```

## Decision Logic

### When to Summarize

```rust
fn should_summarize(
    conversation: &ConversationSnapshot,
    config: &ConversationConfig,
) -> bool {
    // Summarize when context window usage exceeds threshold
    if conversation.metrics.context_window_usage > config.summarize_threshold {
        return true;
    }
    
    // Summarize when turn count exceeds limit
    if conversation.session.total_turns > config.max_turns_before_summary {
        return true;
    }
    
    // Summarize when token count exceeds limit
    if conversation.metrics.total_tokens > config.max_tokens_before_summary {
        return true;
    }
    
    false
}
```

### When to Compress

```rust
fn should_compress(
    conversation: &ConversationSnapshot,
    config: &ConversationConfig,
) -> bool {
    // Compress when context window is getting full
    if conversation.metrics.context_window_usage > config.compress_threshold {
        return true;
    }
    
    // Compress when turn count is very high
    if conversation.session.total_turns > config.max_turns_before_compression {
        return true;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Session loss | No session ID | Create new session, inform user | Periodic session persistence |
| Turn corruption | Invalid turn state | Discard corrupted turn, continue | Atomic turn operations |
| Topic detection failure | Low confidence | Use last known topic | Conservative detection thresholds |
| Summarization failure | Summarization error | Use original turns | Fallback to truncation |
| Compression data loss | Important turn removed | Restore from backup | Protect recent turns from compression |
| Intent tracking failure | Conflicting intents | Reset intent state | Confidence thresholds |

### Recovery Strategy

```rust
impl ConversationManager {
    async fn recover_from_session_loss(&self) -> ConversationSnapshot {
        // Create new session
        let new_session = self.create_new_session().await;
        
        // Attempt to restore from persistence
        if let Some(restored) = self.restore_session().await {
            tracing::info!("Session restored from persistence");
            return restored;
        }
        
        // Start fresh
        tracing::warn!("Session loss, starting fresh conversation");
        ConversationSnapshot::new(new_session)
    }
}
```

## Privacy Considerations

1. **Conversation content**: All conversation content is stored locally, never transmitted to external services without explicit user consent.
2. **Topic tracking**: Topic detection uses pattern matching, not content analysis. Topics are not logged.
3. **Intent tracking**: Intent tracking is local, not used for profiling.
4. **Summarization**: Summaries are stored locally, not transmitted.
5. **Session data**: Session data is encrypted at rest using user-provided keys.
6. **Data retention**: Conversation data is retained according to user-configured retention policy.
7. **Export/delete**: Users can export or delete all conversation data at any time.
8. **No advertising**: Conversation data is never used for advertising or sold to third parties.

## Security Considerations

1. **Encryption at rest**: Conversation data is encrypted using AES-256-GCM.
2. **Encryption in transit**: If conversation data is synced between devices, it is encrypted in transit.
3. **Access control**: Only the user and authorized COS components can access conversation data.
4. **Integrity**: Conversation turns are immutable once created. Tampering is detectable.
5. **Authentication**: Session operations require user authentication.
6. **Audit logging**: Access to conversation data is auditable.

## Future Extensibility

1. **Multi-language support**: Topic detection and summarization for non-English conversations
2. **Sentiment tracking**: Track sentiment across conversation turns
3. **Conversation analytics**: Provide conversation insights to users
4. **Cross-session continuity**: Carry context across conversation sessions
5. **Agent-to-agent conversation**: Support multiple agents in one conversation
6. **Voice emotion detection**: Detect emotion from voice patterns
7. **Conversation branching**: Support conversation forks (alternative paths)

## Examples

### Example 1: Simple Topic Continuity

```
Turn 1: User: "What's the weather like?"
Turn 2: System: "It's 72°F and sunny."
Turn 3: User: "Will it rain tomorrow?"
Topic: Weather (continuous)
Intent: WeatherQuery (continued)
```

### Example 2: Topic Switch

```
Turn 1: User: "What's the weather like?"
Turn 2: System: "It's 72°F and sunny."
Turn 3: User: "Now, about that Rust project..."
Topic Change: Weather → RustProject
Confidence: 0.95
Reason: Explicit marker ("about that")
```

### Example 3: Interruption Handling

```
Turn 1: User: "Tell me about the history of"
Turn 2: System: "The history of computing began in the..."
Turn 3: User: (barge-in) "Actually, what's the capital of France?"
Interruption detected
Topic Change: HistoryOfComputing → Geography
Turn 2 marked as interrupted
```

### Example 4: Conversation Compression

```
Before compression: 50 turns, ~8000 tokens
Target: ~2000 tokens
Compression: Remove turns 1-40, keep turns 41-50
Summary: "Discussed weather, Rust project architecture, and debugging approaches."
Result: 10 turns + summary, ~2000 tokens
```

## Engineering Notes

- `ConversationContext` is the core state from `conversation` crate
- `TurnManager` handles turn lifecycle with atomic operations
- `ContextTracker` provides key-value storage for arbitrary context
- Turn history is stored in a `VecDeque` with configurable max size (default: 100)
- Topic detection uses keyword matching and semantic similarity
- Summarization uses extractive summarization (key sentence selection)
- Compression removes oldest turns while preserving recent context
- All operations are async for non-blocking behavior
- Session persistence uses the `database` crate's SQLite backend
- Conversation snapshots are produced on-demand, not cached
- Token estimation uses a simple heuristic (words * 1.3)
- Topic confidence is calculated using keyword density + semantic similarity
- Intent tracking uses keyword patterns + context key-value pairs
