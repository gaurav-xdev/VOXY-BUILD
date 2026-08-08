# Cognitive Loop

## Purpose

The Cognitive Loop is the central executive of the COS. It orchestrates the continuous cycle of perception, understanding, decision-making, action, and learning. Every interaction with the user or environment flows through this loop. The loop is designed to be:
- **Deterministic**: Same input produces same cognitive state progression
- **Interruptible**: Any cycle can be paused, resumed, or cancelled
- **Observable**: Every stage produces telemetry for debugging
- **Replaceable**: Any stage can be swapped without affecting others

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        COGNITIVE LOOP                                │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    INPUT MULTIPLEXER                         │    │
│  │  User Input │ System Event │ Agent Msg │ Timer │ Internal   │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    STIMULUS BUFFER                           │    │
│  │  Priority Queue │ Deduplication │ Batching │ Ordering        │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                 COGNITIVE PIPELINE                           │    │
│  │                                                              │    │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │ OBSERVE │─▶│UNDERSTAND│─▶│ RETRIEVE │─▶│ DETERMINE│    │    │
│  │  │  (5ms)  │  │  (10ms)  │  │  (15ms)  │  │   GOAL   │    │    │
│  │  └─────────┘  └──────────┘  └──────────┘  │  (5ms)   │    │    │
│  │                                            └──────────┘    │    │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │ ASSESS  │◀─│  PLAN    │◀─│  SELECT  │◀─│  RISK    │    │    │
│  │  │  RISK   │  │  (10ms)  │  │ SKILLS   │  │  (3ms)   │    │    │
│  │  │  (3ms)  │  └──────────┘  │  (5ms)   │  └──────────┘    │    │
│  │  └─────────┘                └──────────┘                    │    │
│  │       │                                                      │    │
│  │       ▼                                                      │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │    │
│  │  │ EXECUTE  │─▶│  VERIFY  │─▶│ REFLECT  │─▶│  LEARN   │    │    │
│  │  │  (50ms)  │  │  (10ms)  │  │  (10ms)  │  │  (10ms)  │    │    │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │    │
│  │                                              │               │    │
│  │                                              ▼               │    │
│  │                                        ┌──────────┐          │    │
│  │                                        │  UPDATE  │          │    │
│  │                                        │  MEMORY  │          │    │
│  │                                        │  (5ms)   │          │    │
│  │                                        └──────────┘          │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    OUTPUT DEMULTIPLEXER                      │    │
│  │  Response │ Action │ Memory Update │ Goal Update │ Notify    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Stimulus Types

```rust
pub enum Stimulus {
    /// Direct user input (voice, text, gesture)
    UserInput(UserInput),
    
    /// System events (file changes, network, timers)
    SystemEvent(SystemEvent),
    
    /// Messages from other agents
    AgentMessage(AgentMessage),
    
    /// Periodic timer ticks for background processing
    TimerTick(TimerTick),
    
    /// Environment changes (location, time, weather)
    EnvironmentChange(EnvironmentChange),
    
    /// Internal drives (curiosity, boredom, hunger for information)
    InternalDrive(InternalDrive),
}

pub struct UserInput {
    pub modality: InputModality,      // Voice, Text, Vision, Gesture
    pub content: InputContent,        // Raw content
    pub timestamp: DateTime<Utc>,     // When received
    pub session_id: SessionId,        // Which session
    pub turn_id: TurnId,              // Which turn
    pub metadata: InputMetadata,      // Confidence, language, etc.
}

pub enum InputModality {
    Voice,
    Text,
    Vision,
    Gesture,
    Hybrid(Vec<InputModality>),
}
```

### Stimulus Processing

1. **Ingestion**: Raw input is normalized to `Stimulus` format
2. **Deduplication**: Identical stimuli within time window are dropped
3. **Priority Assignment**: Based on modality, recency, and user focus
4. **Batching**: Multiple rapid stimuli are batched for efficiency
5. **Ordering**: Stimuli are ordered by priority and timestamp

## Outputs

### Cognitive Result

```rust
pub struct CognitiveResult {
    /// Action to execute (if any)
    pub action: Option<Action>,
    
    /// Response to user (if any)
    pub response: Option<Response>,
    
    /// Memory updates to apply
    pub memory_updates: Vec<MemoryUpdate>,
    
    /// Goal updates to apply
    pub goal_updates: Vec<GoalUpdate>,
    
    /// Notifications to other agents
    pub agent_notifications: Vec<AgentNotification>,
    
    /// Telemetry for this cycle
    pub telemetry: CognitiveTelemetry,
    
    /// Decision trace for explainability
    pub trace: DecisionTrace,
}

pub struct DecisionTrace {
    /// Why this action was chosen
    pub reasoning: String,
    
    /// What alternatives were considered
    pub alternatives: Vec<Alternative>,
    
    /// What risks were identified
    pub risks: Vec<Risk>,
    
    /// What memories influenced the decision
    pub influencing_memories: Vec<MemoryId>,
    
    /// What goals were active
    pub active_goals: Vec<GoalId>,
    
    /// Confidence in this decision
    pub confidence: f32,
}
```

## Internal State

### Cognitive State

```rust
pub struct CognitiveState {
    /// Current attention focus
    pub attention: AttentionState,
    
    /// Active goals
    pub goals: GoalStack,
    
    /// Working memory contents
    pub working_memory: WorkingMemory,
    
    /// Current context
    pub context: CognitiveContext,
    
    /// Emotional state (for empathetic responses)
    pub affect: AffectState,
    
    /// Energy level (for resource allocation)
    pub energy: EnergyLevel,
    
    /// History of recent decisions
    pub decision_history: DecisionHistory,
    
    /// Current cognitive load
    pub load: CognitiveLoad,
}

pub enum EnergyLevel {
    High,    // Full cognitive processing
    Medium,  // Reduced reasoning depth
    Low,     // Minimal processing, defer to heuristics
    Critical, // Emergency mode, safety only
}
```

### Working Memory

```rust
pub struct WorkingMemory {
    /// Current conversation context
    pub conversation: ConversationContext,
    
    /// Active task context
    pub task: Option<TaskContext>,
    
    /// Recently accessed memories
    pub recent_memories: VecDeque<MemoryId>,
    
    /// Current sensory buffer
    pub sensory: SensoryBuffer,
    
    /// Pending decisions
    pub pending_decisions: Vec<Decision>,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                     COGNITIVE LOOP STATE MACHINE                     │
│                                                                      │
│  ┌──────────┐                                                       │
│  │  IDLE    │◀──────────────────────────────────────────────┐       │
│  └────┬─────┘                                               │       │
│       │ (stimulus received)                                  │       │
│       ▼                                                      │       │
│  ┌──────────┐                                                       │
│  │ PERCEIVE │◀──────────────┐                                     │
│  └────┬─────┘               │                                     │
│       │ (perception complete)│                                     │
│       ▼                      │                                     │
│  ┌──────────┐               │                                     │
│  │ UNDERSTAND│              │                                     │
│  └────┬─────┘               │                                     │
│       │ (understanding      │                                     │
│       │  complete)          │                                     │
│       ▼                     │                                     │
│  ┌──────────┐               │                                     │
│  │ RETRIEVE │               │                                     │
│  └────┬─────┘               │                                     │
│       │ (retrieval          │                                     │
│       │  complete)          │                                     │
│       ▼                     │                                     │
│  ┌──────────┐               │                                     │
│  │ DECIDE   │               │                                     │
│  └────┬─────┘               │                                     │
│       │ (decision           │                                     │
│       │  made)              │                                     │
│       ▼                     │                                     │
│  ┌──────────┐               │                                     │
│  │  PLAN    │               │                                     │
│  └────┬─────┘               │                                     │
│       │ (plan               │                                     │
│       │  ready)             │                                     │
│       ▼                     │                                     │
│  ┌──────────┐               │                                     │
│  │ EXECUTE  │               │                                     │
│  └────┬─────┘               │                                     │
│       │ (execution          │                                     │
│       │  complete)          │                                     │
│       ▼                     │                                     │
│  ┌──────────┐               │                                     │
│  │ VERIFY   │───────────────┘                                     │
│  └────┬─────┘ (verification                                       │
│       │       failed: retry)                                      │
│       │ (verification                                               │
│       │  passed)                                                   │
│       ▼                                                            │
│  ┌──────────┐                                                      │
│  │ REFLECT  │                                                      │
│  └────┬─────┘                                                      │
│       │ (reflection                                                │
│       │  complete)                                                 │
│       ▼                                                            │
│  ┌──────────┐                                                      │
│  │  LEARN   │                                                      │
│  └────┬─────┘                                                      │
│       │ (learning                                                  │
│       │  complete)                                                 │
│       ▼                                                            │
│  ┌──────────┐                                                      │
│  │  SLEEP   │──────────────────────────────────────────────────────┘
│  └──────────┘ (cycle complete, wait for next stimulus)
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### State Transitions

| From | To | Trigger | Guard |
|------|-----|---------|-------|
| IDLE | PERCEIVE | Stimulus received | None |
| PERCEIVE | UNDERSTAND | Perception complete | None |
| UNDERSTAND | RETRIEVE | Understanding complete | None |
| RETRIEVE | DECIDE | Retrieval complete | None |
| DECIDE | PLAN | Decision made | Risk assessment passed |
| DECIDE | IDLE | Decision: no action | None |
| PLAN | EXECUTE | Plan ready | Safety gate passed |
| EXECUTE | VERIFY | Execution complete | None |
| VERIFY | REFLECT | Verification passed | None |
| VERIFY | EXECUTE | Verification failed | Retry count < max |
| VERIFY | REFLECT | Verification failed | Retry count >= max |
| REFLECT | LEARN | Reflection complete | None |
| LEARN | SLEEP | Learning complete | None |
| SLEEP | IDLE | Cycle complete | None |
| Any | IDLE | Cancel/Timeout | None |

## Algorithms

### Stimulus Priority Calculation

```rust
fn calculate_priority(stimulus: &Stimulus, state: &CognitiveState) -> Priority {
    let base = match stimulus {
        Stimulus::UserInput(_) => 100,
        Stimulus::AgentMessage(_) => 80,
        Stimulus::SystemEvent(_) => 60,
        Stimulus::EnvironmentChange(_) => 40,
        Stimulus::TimerTick(_) => 20,
        Stimulus::InternalDrive(_) => 10,
    };
    
    let recency_bonus = calculate_recency_bonus(stimulus.timestamp());
    let attention_bonus = if stimulus.matches_attention(&state.attention) {
        50
    } else {
        0
    };
    let urgency_penalty = if stimulus.is_urgent() { 100 } else { 0 };
    
    Priority::new(base + recency_bonus + attention_bonus + urgency_penalty)
}
```

### Cognitive Load Balancing

```rust
fn balance_load(state: &mut CognitiveState) {
    let load = calculate_current_load(state);
    
    match load {
        CognitiveLoad::Low => {
            // Full processing
            state.energy = EnergyLevel::High;
        }
        CognitiveLoad::Medium => {
            // Reduce reasoning depth
            state.energy = EnergyLevel::Medium;
            state.max_reasoning_depth = 3;
        }
        CognitiveLoad::High => {
            // Minimal processing
            state.energy = EnergyLevel::Low;
            state.max_reasoning_depth = 1;
            defer_non_urgent_tasks(state);
        }
        CognitiveLoad::Critical => {
            // Safety only
            state.energy = EnergyLevel::Critical;
            cancel_all_non_essential(state);
        }
    }
}
```

### Decision Timeout

```rust
async fn execute_with_timeout<F, T>(future: F, timeout: Duration) -> Result<T, CognitiveError>
where
    F: Future<Output = T>,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => Ok(result),
        Err(_) => {
            tracing::warn!("Cognitive stage timed out after {:?}", timeout);
            Err(CognitiveError::Timeout)
        }
    }
}
```

## Decision Logic

### When to Respond

```rust
fn should_respond(stimulus: &Stimulus, state: &CognitiveState) -> ResponseDecision {
    // Rule 1: Always respond to direct user input
    if matches!(stimulus, Stimulus::UserInput(_)) {
        return ResponseDecision::Respond;
    }
    
    // Rule 2: Respond to agent messages if relevant
    if let Stimulus::AgentMessage(msg) = stimulus {
        if msg.is_relevant_to(&state.goals) {
            return ResponseDecision::Respond;
        }
    }
    
    // Rule 3: Don't respond to timer ticks unless there's work to do
    if matches!(stimulus, Stimulus::TimerTick(_)) {
        if state.has_pending_work() {
            return ResponseDecision::Respond;
        } else {
            return ResponseDecision::Skip;
        }
    }
    
    // Rule 4: Respond to system events only if they affect active goals
    if let Stimulus::SystemEvent(event) = stimulus {
        if event.affects_goals(&state.goals) {
            return ResponseDecision::Respond;
        }
    }
    
    // Default: skip
    ResponseDecision::Skip
}

pub enum ResponseDecision {
    Respond,
    Skip,
    Defer(Duration),
    Cancel,
}
```

### When NOT to Respond

```rust
fn should_not_respond(stimulus: &Stimulus, state: &CognitiveState) -> bool {
    // Don't respond if in critical mode
    if state.energy == EnergyLevel::Critical {
        return true;
    }
    
    // Don't respond if stimulus is below attention threshold
    if stimulus.priority() < state.attention.threshold {
        return true;
    }
    
    // Don't respond if already processing similar stimulus
    if state.is_duplicate(stimulus) {
        return true;
    }
    
    // Don't respond if user is in "do not disturb" mode
    if state.context.user_do_not_disturb {
        return true;
    }
    
    // Don't respond if cognitive load is too high
    if state.load > CognitiveLoad::High {
        return true;
    }
    
    false
}
```

## Failure Modes

### 1. Stage Timeout

**Symptom**: Cognitive stage takes too long
**Detection**: `tokio::time::timeout` triggers
**Recovery**: Skip stage, use default behavior, log warning
**Prevention**: Cache results, use heuristics for common cases

### 2. Memory Exhaustion

**Symptom**: Working memory full
**Detection**: Memory allocation fails
**Recovery**: Flush oldest memories, compress context
**Prevention**: Monitor memory usage, proactive consolidation

### 3. Decision Conflict

**Symptom**: Multiple conflicting decisions generated
**Detection**: Conflict detector identifies contradictions
**Recovery**: Use priority rules, defer to user
**Prevention**: Clear goal hierarchy, consistent rules

### 4. Resource Starvation

**Symptom**: No CPU/memory available for processing
**Detection**: System resource monitors
**Recovery**: Defer non-critical processing
**Prevention**: Resource budgets, load shedding

### 5. State Corruption

**Symptom**: Internal state inconsistent
**Detection**: State validation checks
**Recovery**: Reset to last known good state
**Prevention**: Immutable state, transactional updates

## Recovery Strategy

```rust
impl CognitiveLoop {
    async fn recover_from_error(&self, error: &CognitiveError, state: &mut CognitiveState) {
        match error {
            CognitiveError::Timeout => {
                // Skip current cycle, preserve state
                tracing::warn!("Cognitive cycle timed out, skipping");
                state.record_timeout();
            }
            CognitiveError::MemoryExhausted => {
                // Flush working memory
                tracing::warn!("Memory exhausted, flushing working memory");
                state.working_memory.flush();
            }
            CognitiveError::StateCorruption => {
                // Reset to last checkpoint
                tracing::error!("State corruption detected, resetting");
                *state = self.last_checkpoint.clone();
            }
            CognitiveError::ResourceStarvation => {
                // Enter low-energy mode
                tracing::warn!("Resource starvation, entering low-energy mode");
                state.energy = EnergyLevel::Low;
            }
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Stage | Target | Maximum | Measurement |
|-------|--------|---------|-------------|
| Stimulus Processing | 2ms | 5ms | Per stimulus |
| Perceive | 3ms | 5ms | Per cycle |
| Understand | 5ms | 10ms | Per cycle |
| Retrieve | 10ms | 15ms | Per cycle |
| Decide | 3ms | 5ms | Per cycle |
| Plan | 5ms | 10ms | Per cycle |
| Execute | 20ms | 50ms | Per action |
| Verify | 5ms | 10ms | Per cycle |
| Reflect | 5ms | 10ms | Per cycle |
| Learn | 5ms | 10ms | Per cycle |
| **Total** | **63ms** | **130ms** | **Per cycle** |

### Optimization Strategies

1. **Lazy Evaluation**: Only compute what's needed
2. **Caching**: Cache frequently accessed data
3. **Batching**: Process multiple stimuli together
4. **Parallelism**: Run independent stages concurrently
5. **Precomputation**: Precompute common decisions
6. **Approximation**: Use fast approximations when precision isn't critical

## Security Considerations

### Input Validation

```rust
fn validate_stimulus(stimulus: &Stimulus) -> Result<(), SecurityError> {
    // Validate input size
    if stimulus.size() > MAX_STIMULUS_SIZE {
        return Err(SecurityError::InputTooLarge);
    }
    
    // Validate input content
    if stimulus.contains_malicious_content() {
        return Err(SecurityError::MaliciousInput);
    }
    
    // Validate input source
    if !stimulus.from_trusted_source() {
        return Err(SecurityError::UntrustedSource);
    }
    
    Ok(())
}
```

### State Protection

- Working memory is encrypted at rest
- Decision traces are signed to prevent tampering
- State transitions are validated before execution
- Rollback capability for failed transitions

## Privacy Rules

1. **No Cloud Processing**: All cognitive processing occurs locally
2. **Memory Encryption**: Sensitive memories are encrypted
3. **Decision Anonymization**: Decision traces are anonymized for debugging
4. **User Control**: Users can view and delete any cognitive data
5. **Data Minimization**: Only necessary data is retained

## Examples

### Example 1: Simple User Query

```
Stimulus: UserInput("What's the weather?")
Cycle:
  1. Perceive: Extract text, detect language (English)
  2. Understand: Parse intent (weather_query), extract entities (none)
  3. Retrieve: Get current location, get weather skill
  4. Decide: Use weather skill
  5. Plan: Call weather API with current location
  6. Execute: Call weather API
  7. Verify: Response received, valid format
  8. Reflect: User asked simple question, got simple answer
  9. Learn: User asks about weather, remember location preference
  10. Update Memory: Store location preference
Response: "It's 72°F and sunny in your location."
```

### Example 2: Complex Multi-Step Task

```
Stimulus: UserInput("Book me a flight to NYC for next week")
Cycle:
  1. Perceive: Extract text, detect language (English)
  2. Understand: Parse intent (flight_booking), entities (NYC, next_week)
  3. Retrieve: Get user preferences, get calendar, get flight skill
  4. Decide: Need more information (exact dates, preferences)
  5. Plan: Ask clarifying questions
  6. Execute: Generate clarifying question
  7. Verify: Question is clear and complete
  8. Reflect: User needs guidance for complex task
  9. Learn: User books flights, remember preferences
  10. Update Memory: Store flight preferences
Response: "I'd be happy to help book a flight to NYC. When exactly next week? Do you have any airline or time preferences?"
```

### Example 3: Ignored Stimulus

```
Stimulus: TimerTick(every 5 minutes)
Cycle:
  1. Perceive: Timer tick received
  2. Understand: Background maintenance check
  3. Retrieve: Check if any pending tasks
  4. Decide: No pending tasks, skip
  5. Plan: None
  6. Execute: None
  7. Verify: N/A
  8. Reflect: No action needed
  9. Learn: N/A
  10. Update Memory: N/A
Response: None (cycle skipped)
```

## Edge Cases

### 1. Rapid Successive Inputs
**Scenario**: User sends multiple messages rapidly
**Handling**: Batch messages, process together, respond to combined intent

### 2. Interrupted Mid-Cycle
**Scenario**: New stimulus arrives while processing
**Handling**: If higher priority, interrupt current cycle; otherwise queue

### 3. Resource Exhaustion Mid-Cycle
**Scenario**: System runs out of memory during processing
**Handling**: Save checkpoint, free resources, resume from checkpoint

### 4. Contradictory Instructions
**Scenario**: User says "do X" then immediately "do Y"
**Handling**: Cancel pending X, execute Y, explain cancellation

### 5. Ambiguous Input
**Scenario**: Input can be interpreted multiple ways
**Handling**: Use context to disambiguate, ask for clarification if needed

## Future Extensions

1. **Predictive Processing**: Anticipate user needs before they ask
2. **Emotional Awareness**: Adapt responses based on user emotional state
3. **Creative Synthesis**: Generate novel solutions to unprecedented problems
4. **Social Cognition**: Understand social dynamics in multi-user scenarios
5. **Self-Improvement**: Meta-cognitive optimization of the loop itself

## Engineering Notes

- The cognitive loop runs on a dedicated tokio runtime to prevent I/O interference
- State is immutable between stages; mutations are explicit and logged
- All timestamps use `chrono::DateTime<Utc>` for consistency
- Duration measurements use `std::time::Instant` for monotonicity
- Errors are hierarchical: `CognitiveError` contains stage-specific variants
- Telemetry is collected via `tracing` crate with structured fields
- The loop supports graceful shutdown via `CancellationToken`
- Test coverage target: 90%+ for all public APIs
