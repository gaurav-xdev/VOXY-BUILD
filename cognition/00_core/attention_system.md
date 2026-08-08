# Attention System

## Purpose

The Attention System determines what the COS should focus on at any given moment. It allocates cognitive resources, filters incoming stimuli, manages focus, and prevents information overload. The system is modeled after human attention: selective, limited, and adaptable. The Attention System ensures that:
- Important stimuli are processed immediately
- Unimportant stimuli are filtered out
- Cognitive resources are allocated efficiently
- Focus is maintained on current tasks
- Distractions are minimized

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                       ATTENTION SYSTEM                               │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    STIMULUS FILTER                           │    │
│  │  Priority │ Relevance │ Novelty │ Urgency │ User Focus      │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    ATTENTION ALLOCATION                      │    │
│  │  Resources │ Focus │ Background │ Periphery │ Emergency      │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    FOCUS MANAGEMENT                          │    │
│  │  Current Focus │ Focus Stack │ Focus Transitions │ Defocus   │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    ATTENTION OUTPUT                          │    │
│  │  Process │ Filter │ Defer │ Reject │ Notify                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Stimulus

```rust
pub struct AttentionStimulus {
    /// Stimulus identifier
    pub id: StimulusId,
    
    /// Stimulus content
    pub content: StimulusContent,
    
    /// Source of stimulus
    pub source: StimulusSource,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Priority hint
    pub priority_hint: Option<Priority>,
    
    /// Attention metadata
    pub metadata: AttentionMetadata,
}

pub enum StimulusSource {
    User(UserId),
    Agent(AgentId),
    System(SystemComponent),
    Environment(EnvironmentSensor),
    Internal(InternalDrive),
}

pub struct AttentionMetadata {
    /// Expected processing time
    pub expected_duration: Option<Duration>,
    
    /// Required cognitive resources
    pub resource_requirements: ResourceRequirements,
    
    /// Attention category
    pub category: AttentionCategory,
    
    /// Attention tags
    pub tags: Vec<AttentionTag>,
}
```

## Outputs

### Attention Decision

```rust
pub enum AttentionDecision {
    /// Process immediately with full resources
    ProcessFull(AttentionAllocation),
    
    /// Process with reduced resources
    ProcessReduced(AttentionAllocation),
    
    /// Defer to background processing
    DeferBackground(DeferConfig),
    
    /// Defer to peripheral processing
    DeferPeripheral(DeferConfig),
    
    /// Reject (below threshold)
    Reject(RejectReason),
    
    /// Queue for later processing
    Queue(QueueConfig),
}

pub struct AttentionAllocation {
    /// Focus level
    pub focus_level: FocusLevel,
    
    /// Allocated resources
    pub resources: ResourceAllocation,
    
    /// Time budget
    pub time_budget: Duration,
    
    /// Priority
    pub priority: Priority,
    
    /// Attention context
    pub context: AttentionContext,
}

pub enum FocusLevel {
    /// Full attention (user is directly engaged)
    Full,
    
    /// Focused attention (actively processing)
    Focused,
    
    /// Background attention (low-priority processing)
    Background,
    
    /// Peripheral attention (monitoring only)
    Peripheral,
    
    /// No attention (filtered out)
    None,
}
```

## Internal State

### Attention State

```rust
pub struct AttentionState {
    /// Current focus
    pub current_focus: Option<FocusTarget>,
    
    /// Focus stack (for nested attention)
    pub focus_stack: VecDeque<FocusTarget>,
    
    /// Attention resources
    pub resources: AttentionResources,
    
    /// Stimulus history
    pub history: AttentionHistory,
    
    /// Attention filters
    pub filters: AttentionFilters,
    
    /// Attention thresholds
    pub thresholds: AttentionThresholds,
    
    /// Attention metrics
    pub metrics: AttentionMetrics,
}

pub struct FocusTarget {
    /// What we're focused on
    pub target: FocusTargetType,
    
    /// When we started focusing
    pub started_at: DateTime<Utc>,
    
    /// Expected duration
    pub expected_duration: Option<Duration>,
    
    /// Focus level
    pub level: FocusLevel,
    
    /// Resources allocated
    pub resources: ResourceAllocation,
}

pub enum FocusTargetType {
    /// User interaction
    UserInteraction(UserId, InteractionType),
    
    /// Task execution
    TaskExecution(TaskId),
    
    /// Agent communication
    AgentCommunication(AgentId),
    
    /// System monitoring
    SystemMonitoring(SystemComponent),
    
    /// Background processing
    BackgroundProcessing(BackgroundTask),
}

pub struct AttentionResources {
    /// Total cognitive capacity
    pub total_capacity: CognitiveCapacity,
    
    /// Currently allocated
    pub allocated: CognitiveCapacity,
    
    /// Available
    pub available: CognitiveCapacity,
    
    /// Reserved for emergencies
    pub emergency_reserve: CognitiveCapacity,
}

pub struct CognitiveCapacity {
    /// CPU time allocation (relative)
    pub cpu_weight: f32,
    
    /// Memory allocation (bytes)
    pub memory_bytes: usize,
    
    /// I/O bandwidth allocation
    pub io_weight: f32,
    
    /// Network bandwidth allocation
    pub network_weight: f32,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                    ATTENTION STATE MACHINE                            │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │      IDLE        │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (stimulus received)                             │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │    FILTERING     │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │                                                  │        │
│     ┌─────┴─────┐                                           │        │
│     │           │                                           │        │
│     ▼           ▼                                           │        │
│  ┌──────┐  ┌──────────┐                                    │        │
│  │REJECT│  │  QUEUE   │                                    │        │
│  └──────┘  └────┬─────┘                                    │        │
│                 │ (queue has capacity)                      │        │
│                 ▼                                            │        │
│  ┌──────────────────┐                                       │        │
│  │    ALLOCATING    │                                       │        │
│  └────────┬─────────┘                                       │        │
│           │ (resources allocated)                           │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   FOCUSED        │◀──────────────────┐                  │        │
│  └────────┬─────────┘                    │                  │        │
│           │ (processing complete)        │                  │        │
│           ▼                              │                  │        │
│  ┌──────────────────┐                    │                  │        │
│  │   DEFOCUSSING    │────────────────────┘                  │        │
│  └──────────────────┘ (higher priority stimulus)            │        │
│           │                                                  │        │
│           │ (no more stimuli)                               │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   RETURNING TO   │──────────────────────────────────────┘        │
│  │       IDLE       │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Stimulus Filtering

```rust
fn filter_stimulus(
    stimulus: &AttentionStimulus,
    state: &AttentionState,
) -> FilterResult {
    // Check attention filters
    for filter in &state.filters {
        match filter.apply(stimulus) {
            FilterAction::Accept => continue,
            FilterAction::Reject(reason) => {
                return FilterResult::Rejected(reason);
            }
            FilterAction::Modify(modified) => {
                return filter_stimulus(&modified, state);
            }
        }
    }
    
    // Check priority threshold
    let priority = calculate_priority(stimulus, state);
    if priority < state.thresholds.minimum_priority {
        return FilterResult::Rejected(RejectReason::BelowThreshold);
    }
    
    // Check relevance to current focus
    if let Some(focus) = &state.current_focus {
        let relevance = calculate_relevance(stimulus, focus);
        if relevance < state.thresholds.minimum_relevance {
            return FilterResult::Deferred(DeferReason::LowRelevance);
        }
    }
    
    // Check novelty (avoid processing duplicates)
    if state.history.is_duplicate(stimulus) {
        return FilterResult::Rejected(RejectReason::Duplicate);
    }
    
    FilterResult::Accepted(priority)
}
```

### Attention Allocation

```rust
fn allocate_attention(
    stimulus: &AttentionStimulus,
    priority: Priority,
    state: &mut AttentionState,
) -> Option<AttentionAllocation> {
    // Check available resources
    let required = estimate_resource_requirements(stimulus);
    let available = state.resources.available;
    
    if required.exceeds(available) {
        // Try to free resources
        if !try_free_resources(required, state) {
            return None;
        }
    }
    
    // Calculate focus level based on priority and context
    let focus_level = calculate_focus_level(priority, stimulus, state);
    
    // Allocate resources
    let allocation = ResourceAllocation {
        cpu_weight: required.cpu_weight,
        memory_bytes: required.memory_bytes,
        io_weight: required.io_weight,
        network_weight: required.network_weight,
    };
    
    // Update state
    state.resources.allocated += &allocation;
    state.resources.available -= &allocation;
    
    Some(AttentionAllocation {
        focus_level,
        resources: allocation,
        time_budget: calculate_time_budget(priority, focus_level),
        priority,
        context: AttentionContext::from_stimulus(stimulus),
    })
}
```

### Focus Management

```rust
fn manage_focus(
    new_focus: FocusTarget,
    state: &mut AttentionState,
) -> FocusTransition {
    // If we have a current focus, decide whether to interrupt
    if let Some(current) = &state.current_focus {
        if new_focus.priority <= current.priority {
            // Queue the new focus
            state.focus_stack.push_back(new_focus);
            return FocusTransition::Queued;
        }
        
        // Interrupt current focus
        let interrupted = state.focus_stack.pop_front();
        state.focus_stack.push_front(current.clone());
        state.focus_stack.push_front(new_focus);
        
        return FocusTransition::Interrupted {
            interrupted: current.clone(),
            reason: InterruptReason::HigherPriority,
        };
    }
    
    // No current focus, start new focus
    state.current_focus = Some(new_focus);
    FocusTransition::Started
}
```

### Priority Calculation

```rust
fn calculate_priority(
    stimulus: &AttentionStimulus,
    state: &AttentionState,
) -> Priority {
    let mut score = 0.0;
    
    // Base priority from source
    score += match stimulus.source {
        StimulusSource::User(_) => 100.0,
        StimulusSource::Agent(_) => 80.0,
        StimulusSource::System(_) => 60.0,
        StimulusSource::Environment(_) => 40.0,
        StimulusSource::Internal(_) => 20.0,
    };
    
    // Urgency bonus
    if stimulus.metadata.category == AttentionCategory::Urgent {
        score += 50.0;
    }
    
    // Relevance to current focus
    if let Some(focus) = &state.current_focus {
        let relevance = calculate_relevance(stimulus, focus);
        score += relevance * 30.0;
    }
    
    // Novelty bonus
    if !state.history.contains(stimulus) {
        score += 20.0;
    }
    
    // Recency penalty (older stimuli are less important)
    let age = Utc::now() - stimulus.timestamp;
    let age_penalty = age.num_seconds() as f64 * 0.1;
    score -= age_penalty;
    
    // User preference bonus
    score += calculate_user_preference_bonus(stimulus, state);
    
    Priority::new(score)
}
```

## Decision Logic

### When to Focus

```rust
fn should_focus(
    stimulus: &AttentionStimulus,
    state: &AttentionState,
) -> bool {
    // Always focus on user input
    if matches!(stimulus.source, StimulusSource::User(_)) {
        return true;
    }
    
    // Focus on urgent system events
    if stimulus.metadata.category == AttentionCategory::Urgent {
        return true;
    }
    
    // Focus if relevant to current task
    if let Some(focus) = &state.current_focus {
        if calculate_relevance(stimulus, focus) > HIGH_RELEVANCE {
            return true;
        }
    }
    
    // Focus if priority exceeds threshold
    let priority = calculate_priority(stimulus, state);
    if priority > state.thresholds.focus_threshold {
        return true;
    }
    
    false
}
```

### When to Filter Out

```rust
fn should_filter_out(
    stimulus: &AttentionStimulus,
    state: &AttentionState,
) -> bool {
    // Filter out low-priority stimuli
    let priority = calculate_priority(stimulus, state);
    if priority < state.thresholds.minimum_priority {
        return true;
    }
    
    // Filter out duplicates
    if state.history.is_duplicate(stimulus) {
        return true;
    }
    
    // Filter out stimuli below attention threshold
    if priority < state.thresholds.attention_threshold {
        return true;
    }
    
    // Filter out if cognitive load is critical
    if state.resources.available.cpu_weight < CRITICAL_THRESHOLD {
        return true;
    }
    
    false
}
```

## Failure Modes

### 1. Attention Overflow

**Symptom**: Too many stimuli queued
**Detection**: Queue length exceeds threshold
**Recovery**: Drop lowest priority items, log warning
**Prevention**: Adaptive thresholds, load shedding

### 2. Focus Starvation

**Symptom**: Current focus never gets resources
**Detection**: Focus duration exceeds expected time
**Recovery**: Force focus transition, log warning
**Prevention**: Time budgets, priority inheritance

### 3. Priority Inversion

**Symptom**: Low-priority stimulus blocks high-priority
**Detection**: Priority inversion detected
**Recovery**: Priority inheritance, preempt if needed
**Prevention**: Careful priority assignment, lock-free design

### 4. Attention Fragmentation

**Symptom**: Frequent focus switches reduce efficiency
**Detection**: Focus switch rate exceeds threshold
**Recovery**: Increase focus hysteresis, batch stimuli
**Prevention**: Hysteresis, batching, debouncing

## Recovery Strategy

```rust
impl AttentionSystem {
    async fn recover_from_overflow(&self, state: &mut AttentionState) {
        // Drop lowest priority items
        let to_drop = state.queue.len() - MAX_QUEUE_SIZE;
        for _ in 0..to_drop {
            if let Some(dropped) = state.queue.pop_lowest_priority() {
                tracing::warn!(
                    stimulus_id = ?dropped.id,
                    priority = dropped.priority,
                    "Dropped stimulus due to overflow"
                );
            }
        }
    }
    
    async fn recover_from_starvation(&self, state: &mut AttentionState) {
        // Force focus transition
        if let Some(focus) = state.current_focus.take() {
            tracing::warn!(
                focus = ?focus.target,
                duration = ?focus.elapsed(),
                "Focus starvation detected, forcing transition"
            );
            
            // Move to background
            state.focus_stack.push_front(FocusTarget {
                target: focus.target,
                level: FocusLevel::Background,
                ..focus
            });
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Operation | Target | Maximum | Measurement |
|-----------|--------|---------|-------------|
| Stimulus Filtering | 1ms | 2ms | Per stimulus |
| Priority Calculation | 0.5ms | 1ms | Per stimulus |
| Resource Allocation | 1ms | 2ms | Per allocation |
| Focus Transition | 2ms | 5ms | Per transition |
| **Total** | **4.5ms** | **10ms** | **Per attention cycle** |

### Optimization Strategies

1. **Priority Caching**: Cache priority calculations for similar stimuli
2. **Resource Pooling**: Pre-allocate resource pools for common patterns
3. **Lazy Evaluation**: Only compute what's needed for current focus
4. **Batch Processing**: Process multiple stimuli in single allocation
5. **Predictive Allocation**: Pre-allocate resources for predicted stimuli

## Security Considerations

### Attention Manipulation Protection

```rust
fn protect_against_manipulation(
    stimulus: &AttentionStimulus,
    state: &AttentionState,
) -> bool {
    // Detect rapid-fire attacks
    if state.history.recent_count(stimulus.source) > RATE_LIMIT {
        return false;
    }
    
    // Detect priority escalation attacks
    if let Some(hint) = &stimulus.priority_hint {
        if *hint > MAXIMUM_USER_PRIORITY {
            return false;
        }
    }
    
    // Detect resource exhaustion attacks
    if state.resources.available.cpu_weight < EMERGENCY_THRESHOLD {
        return false;
    }
    
    true
}
```

### Resource Protection

- Attention resources are capped per source
- Emergency reserve is protected from normal allocation
- Resource allocation is logged for auditing
- Resource exhaustion triggers graceful degradation

## Privacy Rules

1. **Attention Privacy**: What the COS pays attention to is not logged
2. **Focus Privacy**: Current focus is not exposed to other agents
3. **Priority Privacy**: Priority calculations are not exposed
4. **User Control**: Users can view and modify attention settings
5. **Data Minimization**: Attention history is aggressively pruned

## Examples

### Example 1: User Input During Task

```
Current Focus: BackgroundTask(file_sync)
Incoming Stimulus: UserInput("What's the weather?")
Priority: 100 (user input)
Decision: Interrupt background task, focus on user input
Result: Background task deferred, weather query processed
```

### Example 2: Multiple System Events

```
Current Focus: UserInteraction(voice_chat)
Incoming Stimulus: SystemEvent(battery_low), SystemEvent(update_available), TimerTick
Priorities: battery_low=70, update_available=40, timer_tick=10
Decision: Queue battery_low, defer update_available, reject timer_tick
Result: Battery warning processed after voice chat, others queued
```

### Example 3: Attention Overflow

```
Current Focus: UserInteraction(voice_chat)
Queue: 15 stimuli (10 low priority, 5 medium priority)
Threshold: 10 stimuli maximum
Decision: Drop 5 lowest priority stimuli
Result: Queue reduced to 10, lowest priority items logged and dropped
```

## Edge Cases

### 1. Simultaneous User Inputs
**Scenario**: User sends voice and text simultaneously
**Handling**: Process most complete first, acknowledge other

### 2. Interrupt During Critical Operation
**Scenario**: User interrupts while system is updating firmware
**Handling**: Warn user of risk, allow override if confirmed

### 3. Attention Deadlock
**Scenario**: Two tasks waiting for each other's attention
**Handling**: Detect deadlock, break with priority rules

### 4. Attention Livelock
**Scenario**: Tasks keep interrupting each other without progress
**Handling**: Implement hysteresis, force focus on one task

### 5. Resource Exhaustion
**Scenario**: All cognitive resources allocated
**Handling**: Emergency mode, process only safety-critical stimuli

## Future Extensions

1. **Predictive Attention**: Anticipate what user will focus on
2. **Social Attention**: Coordinate attention across multiple users
3. **Emotional Attention**: Prioritize based on emotional state
4. **Creative Attention**: Allocate resources for creative tasks
5. **Meta-Attention**: Optimize attention allocation itself

## Engineering Notes

- Attention state is updated atomically to prevent race conditions
- Focus transitions are logged for debugging
- Attention metrics are collected via `tracing` crate
- Attention thresholds are configurable at runtime
- Attention system supports graceful shutdown
- Attention state can be serialized for debugging
- Attention system is testable with mock stimuli
- Attention system supports priority inheritance
