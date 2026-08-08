# Task Manager

## Purpose

The Task Manager executes individual units of work derived from goals and plans. It manages task lifecycle, scheduling, execution, and completion. The Task Manager ensures that:
- Tasks are created, scheduled, and executed
- Task dependencies are resolved
- Task failures are handled gracefully
- Task progress is tracked and reported
- Task resources are allocated efficiently

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TASK MANAGER                                  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    TASK QUEUE                                │    │
│  │  Pending │ Ready │ Running │ Completed │ Failed │ Cancelled  │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    TASK SCHEDULER                            │    │
│  │  Priority │ Dependencies │ Resources │ Fairness │ Deadline    │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    TASK EXECUTOR                             │    │
│  │  Agent │ Tool │ API │ File │ Network │ Computation            │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    TASK TRACKER                              │    │
│  │  Progress │ Metrics │ History │ Rollback │ Recovery           │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Task

```rust
pub struct Task {
    /// Task identifier
    pub id: TaskId,
    
    /// Task description
    pub description: String,
    
    /// Task type
    pub task_type: TaskType,
    
    /// Task status
    pub status: TaskStatus,
    
    /// Task priority
    pub priority: TaskPriority,
    
    /// Task owner
    pub owner: TaskOwner,
    
    /// Task dependencies
    pub dependencies: Vec<TaskDependency>,
    
    /// Task resources
    pub resources: TaskResources,
    
    /// Task context
    pub context: TaskContext,
    
    /// Task execution state
    pub execution: TaskExecution,
    
    /// Task metrics
    pub metrics: TaskMetrics,
    
    /// Task history
    pub history: Vec<TaskEvent>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
    
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
    
    /// Timeout
    pub timeout: Option<Duration>,
    
    /// Retry configuration
    pub retry_config: RetryConfig,
}

pub enum TaskType {
    /// Agent-based task
    Agent(AgentTask),
    
    /// Tool-based task
    Tool(ToolTask),
    
    /// API call task
    Api(ApiTask),
    
    /// File operation task
    File(FileTask),
    
    /// Network request task
    Network(NetworkTask),
    
    /// Computation task
    Computation(ComputationTask),
    
    /// User interaction task
    UserInteraction(UserInteractionTask),
    
    /// System task
    System(SystemTask),
}

pub enum TaskStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Cancelled,
    Deferred,
    Blocked,
}

pub enum TaskPriority {
    Critical,
    High,
    Medium,
    Low,
    Background,
}

pub enum TaskOwner {
    Goal(GoalId),
    Plan(PlanId),
    Agent(AgentId),
    User(UserId),
    System(SystemComponent),
}

pub struct TaskResources {
    /// Required CPU time
    pub cpu_time: Option<Duration>,
    
    /// Required memory
    pub memory_bytes: Option<usize>,
    
    /// Required I/O
    pub io_required: bool,
    
    /// Required network
    pub network_required: bool,
    
    /// Required tools
    pub tools: Vec<ToolId>,
    
    /// Required permissions
    pub permissions: Vec<Permission>,
}

pub struct TaskContext {
    /// Why this task exists
    pub reason: String,
    
    /// Expected outcome
    pub expected_outcome: String,
    
    /// Success criteria
    pub success_criteria: Vec<SuccessCriterion>,
    
    /// Constraints
    pub constraints: Vec<TaskConstraint>,
    
    /// Related tasks
    pub related_tasks: Vec<TaskId>,
}

pub struct TaskExecution {
    /// Execution state
    pub state: ExecutionState,
    
    /// Execution progress
    pub progress: f32,
    
    /// Execution result
    pub result: Option<TaskResult>,
    
    /// Execution error
    pub error: Option<TaskError>,
    
    /// Execution attempts
    pub attempts: u32,
    
    /// Execution history
    pub execution_history: Vec<ExecutionAttempt>,
}

pub enum ExecutionState {
    NotStarted,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

pub struct TaskResult {
    /// Result type
    pub result_type: ResultType,
    
    /// Result data
    pub data: serde_json::Value,
    
    /// Result metadata
    pub metadata: ResultMetadata,
}

pub enum ResultType {
    Success,
    PartialSuccess,
    Failure,
    Cancelled,
}

pub struct TaskError {
    /// Error code
    pub code: ErrorCode,
    
    /// Error message
    pub message: String,
    
    /// Error source
    pub source: ErrorSource,
    
    /// Recovery suggestion
    pub recovery: Option<RecoverySuggestion>,
}

pub struct RetryConfig {
    /// Maximum retries
    pub max_retries: u32,
    
    /// Retry delay
    pub retry_delay: Duration,
    
    /// Backoff multiplier
    pub backoff_multiplier: f32,
    
    /// Maximum retry delay
    pub max_retry_delay: Duration,
    
    /// Retry conditions
    pub retry_conditions: Vec<RetryCondition>,
}
```

## Outputs

### Task Decision

```rust
pub enum TaskDecision {
    /// Task is ready to execute
    ReadyToExecute(TaskExecutionRequest),
    
    /// Task is blocked
    Blocked(TaskBlocker),
    
    /// Task should be retried
    Retry(RetryConfig),
    
    /// Task should be deferred
    Defer(DeferConfig),
    
    /// Task should be cancelled
    Cancel(CancelReason),
    
    /// Task is complete
    Complete(TaskCompletion),
    
    /// Task needs input
    NeedsInput(InputRequest),
    
    /// Task needs approval
    NeedsApproval(ApprovalRequest),
}

pub struct TaskExecutionRequest {
    /// Task to execute
    pub task_id: TaskId,
    
    /// Execution strategy
    pub strategy: ExecutionStrategy,
    
    /// Resource allocation
    pub resources: ResourceAllocation,
    
    /// Time budget
    pub time_budget: Duration,
    
    /// Execution context
    pub context: ExecutionContext,
}

pub enum ExecutionStrategy {
    /// Execute immediately
    Immediate,
    
    /// Execute with batching
    Batch(BatchConfig),
    
    /// Execute with throttling
    Throttle(ThrottleConfig),
    
    /// Execute with priority
    Prioritized(PriorityConfig),
    
    /// Execute with fallback
    Fallback(FallbackConfig),
}

pub struct TaskCompletion {
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
    
    /// Result
    pub result: TaskResult,
    
    /// Duration
    pub duration: Duration,
    
    /// Attempts
    pub attempts: u32,
    
    /// Rollback information
    pub rollback: Option<RollbackInfo>,
}
```

## Internal State

### Task Manager State

```rust
pub struct TaskManagerState {
    /// Pending tasks
    pub pending: VecDeque<Task>,
    
    /// Ready tasks
    pub ready: VecDeque<Task>,
    
    /// Running tasks
    pub running: HashMap<TaskId, Task>,
    
    /// Completed tasks
    pub completed: VecDeque<Task>,
    
    /// Failed tasks
    pub failed: VecDeque<Task>,
    
    /// Deferred tasks
    pub deferred: VecDeque<Task>,
    
    /// Blocked tasks
    pub blocked: VecDeque<Task>,
    
    /// Task dependencies
    pub dependencies: TaskDependencyGraph,
    
    /// Task scheduler
    pub scheduler: TaskScheduler,
    
    /// Task executor
    pub executor: TaskExecutor,
    
    /// Task tracker
    pub tracker: TaskTracker,
    
    /// Task metrics
    pub metrics: TaskMetrics,
    
    /// Resource allocation
    pub resources: ResourceAllocator,
}

pub struct TaskScheduler {
    /// Scheduling algorithm
    pub algorithm: SchedulingAlgorithm,
    
    /// Scheduling queue
    pub queue: PriorityQueue<TaskId>,
    
    /// Scheduling rules
    pub rules: Vec<SchedulingRule>,
    
    /// Fairness configuration
    pub fairness: FairnessConfig,
}

pub enum SchedulingAlgorithm {
    /// Priority-based scheduling
    Priority,
    
    /// Round-robin scheduling
    RoundRobin,
    
    /// Shortest job first
    ShortestJobFirst,
    
    /// Earliest deadline first
    EarliestDeadlineFirst,
    
    /// Fair scheduling
    Fair,
}

pub struct TaskExecutor {
    /// Execution strategies
    pub strategies: HashMap<TaskType, ExecutionStrategy>,
    
    /// Execution limits
    pub limits: ExecutionLimits,
    
    /// Execution monitoring
    pub monitoring: ExecutionMonitoring,
}

pub struct TaskTracker {
    /// Progress tracking
    pub progress: ProgressTracker,
    
    /// Metrics collection
    pub metrics: MetricsCollector,
    
    /// History recording
    pub history: HistoryRecorder,
    
    /// Rollback support
    pub rollback: RollbackManager,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                    TASK STATE MACHINE                                │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │     PENDING      │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (dependencies resolved)                         │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │      READY       │◀──────────────────────┐              │        │
│  └────────┬─────────┘                        │              │        │
│           │ (resources allocated)            │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                        │              │        │
│  │     RUNNING      │──────────┐            │              │        │
│  └────────┬─────────┘          │            │              │        │
│           │                    │            │              │        │
│     ┌─────┴─────┐             │            │              │        │
│     │           │             │            │              │        │
│     ▼           ▼             │            │              │        │
│  ┌──────┐  ┌──────────┐      │            │              │        │
│  │PAUSED│  │  BLOCKED │      │            │              │        │
│  └──────┘  └────┬─────┘      │            │              │        │
│     │           │ (unblocked) │            │              │        │
│     └───────────┴────────────┘            │              │        │
│           │                                │              │        │
│           │ (resume)                       │              │        │
│           └────────────────────────────────┘              │        │
│           │ (execution complete)                          │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │    COMPLETED     │                                       │        │
│  └──────────────────┘                                       │        │
│           │                                                  │        │
│           │ (execution failed)                              │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │      FAILED      │──────────┐                           │        │
│  └──────────────────┘          │                           │        │
│           │                    │ (max retries exceeded)    │        │
│           │ (retry)            │                           │        │
│           └────────────────────┴───────────────────────────┘        │
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

### Task Scheduling

```rust
fn schedule_tasks(state: &mut TaskManagerState) -> Vec<TaskExecutionRequest> {
    let mut scheduled = Vec::new();
    
    // Move tasks from pending to ready
    while let Some(task) = state.pending.pop_front() {
        if are_dependencies_met(&task, state) {
            state.ready.push_back(task);
        } else {
            state.pending.push_back(task);
            break; // Stop to prevent infinite loop
        }
    }
    
    // Sort ready tasks by priority
    state.ready.make_contiguous().sort_by(|a, b| {
        let score_a = calculate_scheduling_score(a, state);
        let score_b = calculate_scheduling_score(b, state);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
    });
    
    // Schedule tasks up to resource limits
    while let Some(task) = state.ready.pop_front() {
        if can_schedule(&task, state) {
            let request = create_execution_request(&task, state);
            scheduled.push(request);
            
            // Move to running
            state.running.insert(task.id.clone(), task);
        } else {
            state.ready.push_front(task);
            break;
        }
    }
    
    scheduled
}

fn calculate_scheduling_score(task: &Task, state: &TaskManagerState) -> f64 {
    let mut score = 0.0;
    
    // Base priority
    score += match task.priority {
        TaskPriority::Critical => 1000.0,
        TaskPriority::High => 100.0,
        TaskPriority::Medium => 10.0,
        TaskPriority::Low => 1.0,
        TaskPriority::Background => 0.1,
    };
    
    // Deadline urgency
    if let Some(deadline) = task.deadline {
        let time_remaining = deadline - Utc::now();
        let urgency = if time_remaining.num_seconds() < 0 {
            1000.0 // Overdue
        } else {
            100.0 / (1.0 + time_remaining.num_seconds() as f64 / 3600.0)
        };
        score += urgency;
    }
    
    // Resource efficiency
    let efficiency = calculate_resource_efficiency(task);
    score *= efficiency;
    
    // Fairness bonus
    let fairness_bonus = calculate_fairness_bonus(task, state);
    score += fairness_bonus;
    
    score
}
```

### Task Dependency Resolution

```rust
fn are_dependencies_met(task: &Task, state: &TaskManagerState) -> bool {
    for dependency in &task.dependencies {
        match dependency {
            TaskDependency::Requires(other_task_id) => {
                if let Some(other_task) = state.running.get(other_task_id) {
                    if !matches!(other_task.status, TaskStatus::Completed) {
                        return false;
                    }
                } else if let Some(other_task) = state.completed.iter().find(|t| t.id == *other_task_id) {
                    if !matches!(other_task.status, TaskStatus::Completed) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            TaskDependency::BlockedBy(other_task_id) => {
                if let Some(other_task) = state.running.get(other_task_id) {
                    if matches!(other_task.status, TaskStatus::Running) {
                        return false;
                    }
                }
            }
            TaskDependency::ParallelWith(_) => {
                // Can run in parallel, no blocking
            }
        }
    }
    
    true
}
```

### Task Execution

```rust
async fn execute_task(
    task: &Task,
    executor: &TaskExecutor,
) -> Result<TaskResult, TaskError> {
    // Get execution strategy
    let strategy = executor.strategies.get(&task.task_type)
        .unwrap_or(&ExecutionStrategy::Immediate);
    
    // Execute based on strategy
    match strategy {
        ExecutionStrategy::Immediate => {
            execute_immediate(task, executor).await
        }
        ExecutionStrategy::Batch(config) => {
            execute_batch(task, executor, config).await
        }
        ExecutionStrategy::Throttle(config) => {
            execute_throttled(task, executor, config).await
        }
        ExecutionStrategy::Prioritized(config) => {
            execute_prioritized(task, executor, config).await
        }
        ExecutionStrategy::Fallback(config) => {
            execute_with_fallback(task, executor, config).await
        }
    }
}

async fn execute_immediate(
    task: &Task,
    executor: &TaskExecutor,
) -> Result<TaskResult, TaskError> {
    // Allocate resources
    let resources = allocate_resources(&task.resources, executor).await?;
    
    // Execute task
    let result = match &task.task_type {
        TaskType::Agent(agent_task) => {
            execute_agent_task(agent_task, &resources).await?
        }
        TaskType::Tool(tool_task) => {
            execute_tool_task(tool_task, &resources).await?
        }
        TaskType::Api(api_task) => {
            execute_api_task(api_task, &resources).await?
        }
        TaskType::File(file_task) => {
            execute_file_task(file_task, &resources).await?
        }
        TaskType::Network(network_task) => {
            execute_network_task(network_task, &resources).await?
        }
        TaskType::Computation(computation_task) => {
            execute_computation_task(computation_task, &resources).await?
        }
        TaskType::UserInteraction(user_interaction_task) => {
            execute_user_interaction_task(user_interaction_task, &resources).await?
        }
        TaskType::System(system_task) => {
            execute_system_task(system_task, &resources).await?
        }
    };
    
    // Release resources
    release_resources(&resources, executor).await;
    
    Ok(result)
}
```

### Task Failure Handling

```rust
fn handle_task_failure(
    task: &mut Task,
    error: &TaskError,
    state: &mut TaskManagerState,
) -> TaskDecision {
    // Increment attempt count
    task.metrics.attempts += 1;
    
    // Record failure
    task.history.push(TaskEvent::Failed {
        timestamp: Utc::now(),
        error: error.clone(),
        attempt: task.metrics.attempts,
    });
    
    // Check if we should retry
    if should_retry(task, error) {
        // Calculate retry delay
        let delay = calculate_retry_delay(task);
        
        // Schedule retry
        TaskDecision::Retry(RetryConfig {
            max_retries: task.retry_config.max_retries,
            retry_delay: delay,
            backoff_multiplier: task.retry_config.backoff_multiplier,
            max_retry_delay: task.retry_config.max_retry_delay,
            retry_conditions: task.retry_config.retry_conditions.clone(),
        })
    } else {
        // Mark as failed
        task.status = TaskStatus::Failed;
        
        // Notify stakeholders
        notify_stakeholders(task, error, state);
        
        TaskDecision::Cancel(CancelReason::Failed)
    }
}

fn should_retry(task: &Task, error: &TaskError) -> bool {
    // Check retry count
    if task.metrics.attempts >= task.retry_config.max_retries {
        return false;
    }
    
    // Check retry conditions
    for condition in &task.retry_config.retry_conditions {
        if condition.matches(error) {
            return true;
        }
    }
    
    // Default: retry on transient errors
    matches!(error.source, ErrorSource::Transient)
}
```

## Decision Logic

### When to Schedule a Task

```rust
fn should_schedule_task(
    task: &Task,
    state: &TaskManagerState,
) -> bool {
    // Task must be pending
    if !matches!(task.status, TaskStatus::Pending) {
        return false;
    }
    
    // Dependencies must be met
    if !are_dependencies_met(task, state) {
        return false;
    }
    
    // Resources must be available
    if !can_allocate_resources(&task.resources, state) {
        return false;
    }
    
    // Task must not be blocked by user/system
    if is_task_blocked(task, state) {
        return false;
    }
    
    true
}
```

### When to Cancel a Task

```rust
fn should_cancel_task(
    task: &Task,
    reason: &CancelReason,
    state: &TaskManagerState,
) -> bool {
    // User cancels
    if matches!(reason, CancelReason::UserCancelled) {
        return true;
    }
    
    // Task is permanently blocked
    if matches!(reason, CancelReason::PermanentlyBlocked) {
        return true;
    }
    
    // Task has exceeded maximum attempts
    if task.metrics.attempts > task.retry_config.max_retries {
        return true;
    }
    
    // Task has exceeded deadline significantly
    if let Some(deadline) = task.deadline {
        let overdue = Utc::now() - deadline;
        if overdue.num_seconds() > MAX_OVERDUE_SECONDS {
            return true;
        }
    }
    
    // Task is no longer relevant
    if matches!(reason, CancelReason::NoLongerRelevant) {
        return true;
    }
    
    false
}
```

## Failure Modes

### 1. Task Starvation

**Symptom**: Task never gets scheduled
**Detection**: Task age exceeds threshold
**Resolution**: Increase priority, allocate minimum resources
**Prevention**: Minimum resource guarantee, aging

### 2. Task Deadlock

**Symtask**: Circular dependencies prevent progress
**Detection**: Cycle detection in dependency graph
**Resolution**: Break cycle by deferring one task
**Prevention**: Dependency validation during creation

### 3. Resource Exhaustion

**Symptom**: No resources available for tasks
**Detection**: Resource availability below threshold
**Resolution**: Preempt low-priority tasks, emergency resources
**Prevention**: Resource limits, monitoring, scaling

### 4. Task Thrashing

**Symptom**: Tasks constantly being scheduled and unscheduled
**Detection**: High scheduling churn rate
**Resolution**: Increase hysteresis, batch scheduling
**Prevention**: Debouncing, smoothing algorithms

## Recovery Strategy

```rust
impl TaskManager {
    async fn recover_from_deadlock(
        &self,
        state: &mut TaskManagerState,
    ) {
        // Detect cycles in dependency graph
        if let Some(cycle) = detect_cycle(&state.dependencies) {
            // Break cycle by deferring one task
            let task_to_defer = select_task_to_defer(&cycle);
            
            if let Some(task) = state.running.get_mut(&task_to_defer) {
                task.status = TaskStatus::Deferred;
                task.history.push(TaskEvent::Deferred {
                    timestamp: Utc::now(),
                    reason: DeferredReason::DeadlockResolution,
                });
            }
            
            tracing::warn!(
                task_id = task_to_defer,
                "Task deferred to break deadlock"
            );
        }
    }
    
    async fn recover_from_resource_exhaustion(
        &self,
        state: &mut TaskManagerState,
    ) {
        // Find lowest priority task
        if let Some(lowest_task) = find_lowest_priority_task(state) {
            // Preempt it
            if let Some(task) = state.running.get_mut(&lowest_task) {
                task.status = TaskStatus::Deferred;
                task.history.push(TaskEvent::Preempted {
                    timestamp: Utc::now(),
                    reason: PreemptReason::ResourceExhaustion,
                });
                
                // Release its resources
                release_task_resources(task, state).await;
            }
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Operation | Target | Maximum | Measurement |
|-----------|--------|---------|-------------|
| Task Creation | 2ms | 5ms | Per task |
| Task Scheduling | 1ms | 3ms | Per task set |
| Task Execution | Varies | Depends on task type | Per task |
| Task Completion | 1ms | 2ms | Per task |
| Dependency Resolution | 2ms | 5ms | Per dependency |
| **Total Overhead** | **6ms** | **15ms** | **Per task cycle** |

### Optimization Strategies

1. **Priority Caching**: Cache priority scores, recompute only when changes occur
2. **Dependency Pre-computation**: Pre-compute dependency graphs
3. **Batch Scheduling**: Schedule multiple tasks in single pass
4. **Resource Pooling**: Pre-allocate resource pools for common patterns
5. **Lazy Execution**: Execute tasks only when results are needed

## Security Considerations

### Task Authorization

```rust
fn authorize_task(
    task: &Task,
    owner: &TaskOwner,
    state: &TaskManagerState,
) -> bool {
    match owner {
        TaskOwner::Goal(goal_id) => {
            // Tasks derived from goals inherit goal authorization
            authorize_goal_task(task, goal_id, state)
        }
        TaskOwner::Plan(plan_id) => {
            // Tasks derived from plans inherit plan authorization
            authorize_plan_task(task, plan_id, state)
        }
        TaskOwner::Agent(agent_id) => {
            // Agent tasks have limited authorization
            agent_has_task_authority(agent_id, &task.task_type, state)
        }
        TaskOwner::User(user_id) => {
            // User tasks are fully authorized
            true
        }
        TaskOwner::System(_) => {
            // System tasks are pre-authorized
            true
        }
    }
}
```

### Task Protection

- Task priority cannot be escalated by unauthorized agents
- Task deletion requires appropriate permissions
- Task history is immutable
- Task metrics are tamper-evident

## Privacy Rules

1. **Task Privacy**: Task details are only visible to owner and authorized agents
2. **Execution Privacy**: Execution details are private
3. **History Privacy**: Task history is private to owner
4. **User Control**: Users can view, modify, and delete their tasks
5. **Data Minimization**: Completed tasks are archived after retention period

## Examples

### Example 1: Simple Task Execution

```
Goal: LearnSpanish
Task: LearnVocabulary(words=50)
Priority: High
Resources: [Memory(100MB), Network(API)]
Dependencies: None
Execution: AgentTask(vocabulary_learning)
Result: Success(words_learned=50, duration=2h)
```

### Example 2: Complex Task with Dependencies

```
Goal: BuildWebsite
Task: DeployWebsite
Priority: Critical
Resources: [Network(SSH), Disk(1GB)]
Dependencies: [BuildWebsite, TestWebsite]
Execution: ToolTask(deployment)
Result: Success(url="https://example.com", duration=30m)
```

### Example 3: Task Failure and Retry

```
Goal: SendEmail
Task: DeliverEmail
Priority: High
Resources: [Network(SMTP)]
Dependencies: [ComposeEmail]
Execution: NetworkTask(smtp_delivery)
Error: ConnectionTimeout (transient)
Retry: Yes (attempt 1/3, delay=5s)
Result: Success(delivered=true, attempts=2, duration=10s)
```

## Edge Cases

### 1. Task Without Dependencies
**Scenario**: Task has no dependencies
**Handling**: Schedule immediately when resources available

### 2. Task With Circular Dependencies
**Scenario**: Task A depends on Task B, Task B depends on Task A
**Handling**: Detect cycle, break by deferring one task, notify user

### 3. Task With Unrealistic Deadline
**Scenario**: Task requires 1 hour but deadline is 5 minutes
**Handling**: Warn user, suggest realistic deadline, allow override

### 4. Task Abandonment
**Scenario**: Owner stops interacting with task
**Handling**: Periodic check-ins, auto-defer after timeout, notify owner

### 5. Task Merge Request
**Scenario**: Two similar tasks created independently
**Handling**: Detect similarity, suggest merge, preserve history

## Future Extensions

1. **Task Learning**: Learn from past task success/failure
2. **Task Prediction**: Anticipate task needs before explicit request
3. **Task Coordination**: Coordinate tasks across multiple agents
4. **Task Templates**: Reusable task patterns
5. **Task Visualization**: Visual task trees and progress

## Engineering Notes

- Task state is updated atomically
- Task history is append-only
- Task metrics are collected via `tracing` crate
- Task priorities are configurable at runtime
- Task manager supports graceful shutdown
- Task state can be serialized for persistence
- Task manager is testable with mock tasks
- Task manager supports concurrent task processing
