# Goal Manager

## Purpose

The Goal Manager maintains, prioritizes, and resolves goals across the COS. Goals are the highest-level abstraction of what the COS is trying to achieve. They drive planning, task creation, and decision-making. The Goal Manager ensures that:
- Goals are created, tracked, and resolved
- Goal conflicts are detected and resolved
- Goal priority is maintained across the system
- Goal decomposition produces actionable plans
- Goal completion is verified and reported

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        GOAL MANAGER                                  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    GOAL REGISTRY                             │    │
│  │  Active │ Completed │ Failed │ Deferred │ Cancelled          │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    GOAL PRIORITIZER                          │    │
│  │  Urgency │ Importance │ Difficulty │ Dependencies │ Time     │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    GOAL DECOMPOSER                           │    │
│  │  Sub-goals │ Milestones │ Dependencies │ Estimation          │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    GOAL TRACKER                              │    │
│  │  Progress │ Blockers │ Metrics │ History │ Completion         │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Goal

```rust
pub struct Goal {
    /// Goal identifier
    pub id: GoalId,
    
    /// Goal description
    pub description: String,
    
    /// Goal priority
    pub priority: GoalPriority,
    
    /// Goal status
    pub status: GoalStatus,
    
    /// Goal owner
    pub owner: GoalOwner,
    
    /// Goal context
    pub context: GoalContext,
    
    /// Goal dependencies
    pub dependencies: Vec<GoalDependency>,
    
    /// Goal milestones
    pub milestones: Vec<Milestone>,
    
    /// Goal metrics
    pub metrics: GoalMetrics,
    
    /// Goal history
    pub history: Vec<GoalEvent>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
    
    /// Estimated completion time
    pub estimated_duration: Option<Duration>,
}

pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

pub enum GoalStatus {
    Active,
    Completed,
    Failed,
    Deferred,
    Cancelled,
    Blocked,
}

pub enum GoalOwner {
    User(UserId),
    Agent(AgentId),
    System(SystemComponent),
    Shared(Vec<GoalOwner>),
}

pub struct GoalContext {
    /// Reason for goal
    pub reason: GoalReason,
    
    /// Expected outcome
    pub expected_outcome: String,
    
    /// Success criteria
    pub success_criteria: Vec<SuccessCriterion>,
    
    /// Constraints
    pub constraints: Vec<GoalConstraint>,
    
    /// Related goals
    pub related_goals: Vec<GoalId>,
}

pub enum GoalReason {
    UserRequest,
    SystemNeed,
    AgentInitiative,
    Scheduled,
    Emergency,
}

pub struct Milestone {
    /// Milestone identifier
    pub id: MilestoneId,
    
    /// Milestone description
    pub description: String,
    
    /// Milestone status
    pub status: MilestoneStatus,
    
    /// Milestone dependencies
    pub dependencies: Vec<MilestoneId>,
    
    /// Milestone deadline
    pub deadline: Option<DateTime<Utc>>,
    
    /// Milestone progress (0.0 to 1.0)
    pub progress: f32,
}
```

## Outputs

### Goal Decision

```rust
pub enum GoalDecision {
    /// Goal is ready to proceed
    Proceed(GoalAction),
    
    /// Goal is blocked
    Blocked(GoalBlocker),
    
    /// Goal should be decomposed
    Decompose(DecompositionRequest),
    
    /// Goal should be prioritized
    Reprioritize(PriorityChange),
    
    /// Goal should be deferred
    Defer(DeferConfig),
    
    /// Goal should be cancelled
    Cancel(CancelReason),
    
    /// Goal is complete
    Complete(GoalCompletion),
}

pub enum GoalAction {
    /// Create tasks for goal
    CreateTasks(Vec<TaskCreationRequest>),
    
    /// Update goal progress
    UpdateProgress(GoalId, f32),
    
    /// Resolve goal blocker
    ResolveBlocker(GoalId, BlockerId),
    
    /// Request resources for goal
    RequestResources(GoalId, ResourceRequest),
    
    /// Notify stakeholders
    Notify(GoalId, Notification),
}

pub struct GoalCompletion {
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
    
    /// Actual outcome
    pub actual_outcome: String,
    
    /// Deviation from expected
    pub deviation: Option<String>,
    
    /// Lessons learned
    pub lessons_learned: Vec<String>,
    
    /// Metrics
    pub metrics: CompletionMetrics,
}
```

## Internal State

### Goal Manager State

```rust
pub struct GoalManagerState {
    /// Active goals
    pub active_goals: HashMap<GoalId, Goal>,
    
    /// Completed goals
    pub completed_goals: VecDeque<Goal>,
    
    /// Failed goals
    pub failed_goals: VecDeque<Goal>,
    
    /// Goal dependencies
    pub dependencies: DependencyGraph,
    
    /// Goal priorities
    pub priorities: PriorityQueue<GoalId>,
    
    /// Goal metrics
    pub metrics: GoalMetrics,
    
    /// Goal history
    pub history: Vec<GoalEvent>,
    
    /// Goal constraints
    pub constraints: Vec<GoalConstraint>,
}

pub struct DependencyGraph {
    /// Adjacency list
    pub adjacency: HashMap<GoalId, Vec<GoalId>>,
    
    /// Reverse adjacency list
    pub reverse: HashMap<GoalId, Vec<GoalId>>,
    
    /// Cycle detection state
    pub cycle_state: CycleDetectionState,
}

pub struct PriorityQueue<T> {
    /// Priority buckets
    pub buckets: VecDeque<Vec<T>>,
    
    /// Item to priority mapping
    pub priorities: HashMap<T, usize>,
    
    /// Current priority level
    pub current_level: usize,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                    GOAL STATE MACHINE                                │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │    INACTIVE      │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (goal created)                                  │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │     ACTIVE       │◀──────────────────────┐              │        │
│  └────────┬─────────┘                        │              │        │
│           │                                  │              │        │
│     ┌─────┴─────┐                           │              │        │
│     │           │                           │              │        │
│     ▼           ▼                           │              │        │
│  ┌──────┐  ┌──────────┐                    │              │        │
│  │BLOCKED│  │ DEFERRED │                    │              │        │
│  └──────┘  └────┬─────┘                    │              │        │
│     │           │ (unblocked/resumed)       │              │        │
│     └───────────┴──────────────────────────┘              │        │
│           │ (all tasks complete)                          │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │    COMPLETED     │                                       │        │
│  └──────────────────┘                                       │        │
│           │                                                  │        │
│           │ (failure/unrecoverable)                         │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │     FAILED       │──────────────────────────────────────┘        │
│  └──────────────────┘                                               │
│           │                                                          │
│           │ (cancelled by user/system)                               │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │    CANCELLED     │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Goal Prioritization

```rust
fn prioritize_goals(goals: &mut Vec<Goal>, context: &GoalContext) {
    // Sort by priority score
    goals.sort_by(|a, b| {
        let score_a = calculate_priority_score(a, context);
        let score_b = calculate_priority_score(b, context);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn calculate_priority_score(goal: &Goal, context: &GoalContext) -> f64 {
    let mut score = 0.0;
    
    // Base priority
    score += match goal.priority {
        GoalPriority::Critical => 1000.0,
        GoalPriority::High => 100.0,
        GoalPriority::Medium => 10.0,
        GoalPriority::Low => 1.0,
        GoalPriority::Background => 0.1,
    };
    
    // Urgency (time pressure)
    if let Some(deadline) = goal.deadline {
        let time_remaining = deadline - Utc::now();
        let urgency = if time_remaining.num_seconds() < 0 {
            100.0 // Overdue
        } else {
            100.0 / (1.0 + time_remaining.num_seconds() as f64 / 3600.0)
        };
        score += urgency;
    }
    
    // Importance (user/system need)
    score += calculate_importance(goal, context);
    
    // Difficulty penalty
    let difficulty = estimate_difficulty(goal);
    score *= 1.0 / (1.0 + difficulty);
    
    // Dependency bonus (if blocking other goals)
    let dependency_bonus = calculate_dependency_bonus(goal);
    score += dependency_bonus;
    
    // Progress bonus (near completion)
    if goal.metrics.progress > 0.8 {
        score *= 1.5;
    }
    
    score
}
```

### Goal Decomposition

```rust
fn decompose_goal(goal: &Goal, context: &GoalContext) -> Vec<SubGoal> {
    let mut sub_goals = Vec::new();
    
    // Analyze goal requirements
    let requirements = analyze_requirements(goal, context);
    
    // Create sub-goals for each requirement
    for requirement in requirements {
        let sub_goal = SubGoal {
            id: SubGoalId::new(),
            description: requirement.description,
            priority: requirement.priority,
            dependencies: requirement.dependencies,
            estimated_duration: requirement.estimated_duration,
            success_criteria: requirement.success_criteria,
        };
        sub_goals.push(sub_goal);
    }
    
    // Add milestone sub-goals
    for milestone in &goal.milestones {
        let milestone_goal = SubGoal {
            id: SubGoalId::new(),
            description: format!("Milestone: {}", milestone.description),
            priority: GoalPriority::High,
            dependencies: milestone.dependencies.clone(),
            estimated_duration: None,
            success_criteria: vec![SuccessCriterion::MilestoneComplete(milestone.id.clone())],
        };
        sub_goals.push(milestone_goal);
    }
    
    // Optimize sub-goal order
    optimize_sub_goal_order(&mut sub_goals);
    
    sub_goals
}
```

### Goal Dependency Resolution

```rust
fn resolve_dependencies(
    goal: &Goal,
    state: &GoalManagerState,
) -> DependencyResolution {
    let mut resolved = Vec::new();
    let mut blocked = Vec::new();
    
    for dependency in &goal.dependencies {
        match dependency {
            GoalDependency::Requires(other_goal_id) => {
                if let Some(other_goal) = state.active_goals.get(other_goal_id) {
                    match other_goal.status {
                        GoalStatus::Completed => {
                            resolved.push(dependency.clone());
                        }
                        GoalStatus::Active => {
                            blocked.push(Blocker::DependencyPending(other_goal_id.clone()));
                        }
                        GoalStatus::Failed => {
                            blocked.push(Blocker::DependencyFailed(other_goal_id.clone()));
                        }
                        _ => {
                            blocked.push(Blocker::DependencyBlocked(other_goal_id.clone()));
                        }
                    }
                } else {
                    blocked.push(Blocker::DependencyMissing(other_goal_id.clone()));
                }
            }
            GoalDependency::BlockedBy(other_goal_id) => {
                // Similar logic for blocking dependencies
            }
            GoalDependency::ParallelWith(other_goal_id) => {
                // Can run in parallel, no blocking
            }
        }
    }
    
    if blocked.is_empty() {
        DependencyResolution::Resolved(resolved)
    } else {
        DependencyResolution::Blocked(blocked)
    }
}
```

### Goal Progress Tracking

```rust
fn track_progress(
    goal: &mut Goal,
    task_progress: &TaskProgress,
) {
    // Update goal metrics
    goal.metrics.tasks_completed += task_progress.completed;
    goal.metrics.tasks_total += task_progress.total;
    
    // Calculate overall progress
    let task_progress_ratio = if goal.metrics.tasks_total > 0 {
        goal.metrics.tasks_completed as f32 / goal.metrics.tasks_total as f32
    } else {
        0.0
    };
    
    // Update milestone progress
    let milestone_progress = calculate_milestone_progress(goal);
    
    // Weighted progress
    goal.metrics.progress = (task_progress_ratio * 0.7) + (milestone_progress * 0.3);
    
    // Update timestamp
    goal.updated_at = Utc::now();
    
    // Record progress event
    goal.history.push(GoalEvent::ProgressUpdated {
        timestamp: Utc::now(),
        progress: goal.metrics.progress,
        task_progress: task_progress.clone(),
    });
}
```

## Decision Logic

### When to Create a Goal

```rust
fn should_create_goal(
    trigger: &GoalTrigger,
    state: &GoalManagerState,
) -> bool {
    // User explicitly requests a goal
    if matches!(trigger, GoalTrigger::UserRequest(_)) {
        return true;
    }
    
    // System detects a need
    if matches!(trigger, GoalTrigger::SystemNeed(_)) {
        // Check if similar goal already exists
        let similar_exists = state.active_goals.values().any(|g| {
            g.description.contains(&trigger.description())
        });
        return !similar_exists;
    }
    
    // Agent initiates a goal
    if matches!(trigger, GoalTrigger::AgentInitiative(_)) {
        // Check agent has authority
        return agent_has_authority(trigger.agent_id(), state);
    }
    
    false
}
```

### When to Cancel a Goal

```rust
fn should_cancel_goal(
    goal: &Goal,
    reason: &CancelReason,
    state: &GoalManagerState,
) -> bool {
    // User cancels
    if matches!(reason, CancelReason::UserCancelled) {
        return true;
    }
    
    // Goal is blocked indefinitely
    if matches!(reason, CancelReason::PermanentlyBlocked) {
        return true;
    }
    
    // Goal is no longer relevant
    if matches!(reason, CancelReason::NoLongerRelevant) {
        return true;
    }
    
    // Goal has exceeded maximum attempts
    if goal.metrics.attempts > MAX_ATTEMPTS {
        return true;
    }
    
    // Goal has exceeded deadline significantly
    if let Some(deadline) = goal.deadline {
        let overdue = Utc::now() - deadline;
        if overdue.num_days() > MAX_OVERDUE_DAYS {
            return true;
        }
    }
    
    false
}
```

## Failure Modes

### 1. Goal Conflict

**Symptom**: Two goals require mutually exclusive resources
**Detection**: Dependency graph shows conflict
**Resolution**: Prioritize one goal, defer or cancel other
**Prevention**: Conflict detection during goal creation

### 2. Goal Starvation

**Symptom**: Goal never gets resources
**Detection**: Goal age exceeds threshold
**Resolution**: Increase priority, allocate minimum resources
**Prevention**: Minimum resource guarantee per goal

### 3. Goal Oscillation

**Symptom**: Goal repeatedly changes priority
**Detection**: Priority changes exceed threshold
**Resolution**: Hysteresis, minimum change duration
**Prevention**: Priority smoothing algorithm

### 4. Goal Deadlock

**Symptom**: Circular dependencies prevent progress
**Detection**: Cycle detection in dependency graph
**Resolution**: Break cycle by deferring one goal
**Prevention**: Dependency validation during creation

## Recovery Strategy

```rust
impl GoalManager {
    async fn recover_from_conflict(
        &self,
        goal_a: &GoalId,
        goal_b: &GoalId,
        state: &mut GoalManagerState,
    ) {
        // Compare priorities
        let priority_a = state.priorities.get(goal_a);
        let priority_b = state.priorities.get(goal_b);
        
        if priority_a > priority_b {
            // Defer goal_b
            if let Some(goal) = state.active_goals.get_mut(goal_b) {
                goal.status = GoalStatus::Deferred;
                goal.history.push(GoalEvent::Deferred {
                    timestamp: Utc::now(),
                    reason: DeferredReason::ConflictWith(goal_a.clone()),
                });
            }
        } else {
            // Defer goal_a
            if let Some(goal) = state.active_goals.get_mut(goal_a) {
                goal.status = GoalStatus::Deferred;
                goal.history.push(GoalEvent::Deferred {
                    timestamp: Utc::now(),
                    reason: DeferredReason::ConflictWith(goal_b.clone()),
                });
            }
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Operation | Target | Maximum | Measurement |
|-----------|--------|---------|-------------|
| Goal Creation | 5ms | 10ms | Per goal |
| Goal Prioritization | 2ms | 5ms | Per goal set |
| Goal Decomposition | 10ms | 20ms | Per goal |
| Dependency Resolution | 3ms | 8ms | Per goal |
| Progress Tracking | 1ms | 2ms | Per update |
| **Total** | **21ms** | **45ms** | **Per goal cycle** |

### Optimization Strategies

1. **Priority Caching**: Cache priority scores, recompute only when changes occur
2. **Incremental Decomposition**: Decompose goals lazily as needed
3. **Dependency Pre-computation**: Pre-compute dependency graphs
4. **Batch Progress Updates**: Batch progress updates for efficiency
5. **Goal Pruning**: Remove stale goals to reduce search space

## Security Considerations

### Goal Authorization

```rust
fn authorize_goal(
    goal: &Goal,
    owner: &GoalOwner,
    state: &GoalManagerState,
) -> bool {
    match owner {
        GoalOwner::User(user_id) => {
            // Users can create any goal
            true
        }
        GoalOwner::Agent(agent_id) => {
            // Agents have limited goal creation authority
            agent_has_authority(agent_id, &goal.description, state)
        }
        GoalOwner::System(_) => {
            // System goals are pre-authorized
            true
        }
        GoalOwner::Shared(owners) => {
            // At least one owner must be authorized
            owners.iter().any(|o| authorize_goal(goal, o, state))
        }
    }
}
```

### Goal Protection

- Goal priority cannot be escalated by unauthorized agents
- Goal deletion requires appropriate permissions
- Goal history is immutable
- Goal metrics are tamper-evident

## Privacy Rules

1. **Goal Privacy**: Goal details are only visible to owner and authorized agents
2. **Progress Privacy**: Progress is shared only with stakeholders
3. **History Privacy**: Goal history is private to owner
4. **User Control**: Users can view, modify, and delete their goals
5. **Data Minimization**: Completed goals are archived after retention period

## Examples

### Example 1: User Goal Creation

```
Trigger: User says "Help me learn Spanish"
Goal Created: LearnSpanish
Priority: High (user request)
Decomposition: [LearnVocabulary, PracticeGrammar, ConversationPractice]
Dependencies: None (parallel sub-goals)
Estimated Duration: 30 days
Success Criteria: [BasicConversation, 500Words, PresentTense]
```

### Example 2: System Goal

```
Trigger: System detects low battery
Goal Created: ConserveBattery
Priority: Critical (system need)
Decomposition: [ReduceBrightness, LimitBackgroundSync, AlertUser]
Dependencies: None
Estimated Duration: Until charged
Success Criteria: [BatteryAbove20%, UserAcknowledged]
```

### Example 3: Goal Conflict Resolution

```
Active Goals: [WatchMovie, LearnSpanish]
Conflict: Both require user attention at same time
Resolution: LearnSpanish is higher priority (user goal), WatchMovie deferred
Result: Learning session proceeds, movie suggested for later
```

## Edge Cases

### 1. Circular Dependencies
**Scenario**: Goal A depends on Goal B, Goal B depends on Goal A
**Handling**: Detect cycle, break by deferring one goal, notify user

### 2. Goal Without Owner
**Scenario**: System creates goal but no owner assigned
**Handling**: Assign to system, allow adoption by user/agent

### 3. Goal With Unrealistic Deadline
**Scenario**: User sets deadline of 1 minute for 1-hour task
**Handling**: Warn user, suggest realistic deadline, allow override

### 4. Goal Abandonment
**Scenario**: Owner stops interacting with goal
**Handling**: Periodic check-ins, auto-defer after timeout, notify owner

### 5. Goal Merge Request
**Scenario**: Two similar goals created independently
**Handling**: Detect similarity, suggest merge, preserve history

## Future Extensions

1. **Goal Learning**: Learn from past goal success/failure
2. **Goal Prediction**: Anticipate user goals before explicit request
3. **Goal Negotiation**: Negotiate goals between multiple users
4. **Goal Templates**: Reusable goal patterns
5. **Goal Visualization**: Visual goal trees and progress

## Engineering Notes

- Goal state is updated atomically
- Goal history is append-only
- Goal metrics are collected via `tracing` crate
- Goal priorities are configurable at runtime
- Goal manager supports graceful shutdown
- Goal state can be serialized for persistence
- Goal manager is testable with mock goals
- Goal manager supports concurrent goal processing
