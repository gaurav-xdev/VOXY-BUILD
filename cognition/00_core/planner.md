# Planner

## Purpose

The Planner transforms goals into actionable plans with concrete tasks, timelines, and resource allocations. It creates, manages, and adapts plans based on changing conditions. The Planner ensures that:
- Plans are created from goals and context
- Plans are decomposed into actionable tasks
- Plans adapt to changing conditions
- Plans are monitored and adjusted
- Plans are completed successfully

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           PLANNER                                    │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PLAN REGISTRY                             │    │
│  │  Active │ Completed │ Failed │ Deferred │ Cancelled          │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PLAN CREATOR                              │    │
│  │  Templates │ Context │ Goals │ Resources │ Constraints       │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PLAN EXECUTOR                             │    │
│  │  Task Creation │ Scheduling │ Monitoring │ Adjustment         │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PLAN TRACKER                              │    │
│  │  Progress │ Deviations │ Adjustments │ Completion             │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Plan

```rust
pub struct Plan {
    /// Plan identifier
    pub id: PlanId,
    
    /// Plan description
    pub description: String,
    
    /// Plan type
    pub plan_type: PlanType,
    
    /// Plan status
    pub status: PlanStatus,
    
    /// Plan owner
    pub owner: PlanOwner,
    
    /// Plan context
    pub context: PlanContext,
    
    /// Plan steps
    pub steps: Vec<PlanStep>,
    
    /// Plan resources
    pub resources: PlanResources,
    
    /// Plan timeline
    pub timeline: PlanTimeline,
    
    /// Plan metrics
    pub metrics: PlanMetrics,
    
    /// Plan history
    pub history: Vec<PlanEvent>,
    
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    
    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
    
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
}

pub enum PlanType {
    /// Sequential plan (steps in order)
    Sequential,
    
    /// Parallel plan (steps in parallel)
    Parallel,
    
    /// Hybrid plan (mix of sequential and parallel)
    Hybrid,
    
    /// Adaptive plan (adjusts based on results)
    Adaptive,
    
    /// Template-based plan
    Template(TemplateId),
}

pub enum PlanStatus {
    Draft,
    Active,
    Completed,
    Failed,
    Deferred,
    Cancelled,
    Paused,
}

pub enum PlanOwner {
    Goal(GoalId),
    Agent(AgentId),
    User(UserId),
    System(SystemComponent),
}

pub struct PlanContext {
    /// Why this plan exists
    pub reason: String,
    
    /// Expected outcome
    pub expected_outcome: String,
    
    /// Constraints
    pub constraints: Vec<PlanConstraint>,
    
    /// Related plans
    pub related_plans: Vec<PlanId>,
    
    /// Plan assumptions
    pub assumptions: Vec<PlanAssumption>,
}

pub struct PlanStep {
    /// Step identifier
    pub id: StepId,
    
    /// Step description
    pub description: String,
    
    /// Step type
    pub step_type: StepType,
    
    /// Step status
    pub status: StepStatus,
    
    /// Step dependencies
    pub dependencies: Vec<StepDependency>,
    
    /// Step resources
    pub resources: StepResources,
    
    /// Step context
    pub context: StepContext,
    
    /// Step execution
    pub execution: StepExecution,
    
    /// Step metrics
    pub metrics: StepMetrics,
    
    /// Step history
    pub history: Vec<StepEvent>,
}

pub enum StepType {
    /// Task step
    Task(TaskCreationRequest),
    
    /// Decision step
    Decision(DecisionRequest),
    
    /// Checkpoint step
    Checkpoint(CheckpointRequest),
    
    /// Waiting step
    Waiting(WaitingRequest),
    
    /// Communication step
    Communication(CommunicationRequest),
    
    /// Review step
    Review(ReviewRequest),
    
    /// Approval step
    Approval(ApprovalRequest),
}

pub enum StepStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Deferred,
    Skipped,
}

pub struct PlanResources {
    /// Required resources
    pub required: ResourceRequirements,
    
    /// Allocated resources
    pub allocated: ResourceAllocation,
    
    /// Resource constraints
    pub constraints: Vec<ResourceConstraint>,
    
    /// Resource optimization
    pub optimization: ResourceOptimization,
}

pub struct PlanTimeline {
    /// Start time
    pub start_time: DateTime<Utc>,
    
    /// End time
    pub end_time: DateTime<Utc>,
    
    /// Milestones
    pub milestones: Vec<PlanMilestone>,
    
    /// Checkpoints
    pub checkpoints: Vec<PlanCheckpoint>,
    
    /// Buffer time
    pub buffer_time: Duration,
    
    /// Critical path
    pub critical_path: Vec<StepId>,
}

pub struct PlanMetrics {
    /// Overall progress (0.0 to 1.0)
    pub progress: f32,
    
    /// Steps completed
    pub steps_completed: u32,
    
    /// Steps total
    pub steps_total: u32,
    
    /// Time elapsed
    pub time_elapsed: Duration,
    
    /// Time remaining estimate
    pub time_remaining: Duration,
    
    /// Deviation from plan
    pub deviation: PlanDeviation,
    
    /// Quality metrics
    pub quality: QualityMetrics,
}

pub enum PlanDeviation {
    /// On track
    OnTrack,
    
    /// Slightly behind
    SlightlyBehind,
    
    /// Significantly behind
    SignificantlyBehind,
    
    /// Ahead of schedule
    AheadOfSchedule,
    
    /// Off track (needs adjustment)
    OffTrack,
}
```

## Outputs

### Plan Decision

```rust
pub enum PlanDecision {
    /// Plan is ready to execute
    ReadyToExecute(PlanExecutionRequest),
    
    /// Plan needs adjustment
    Adjust(PlanAdjustment),
    
    /// Plan is blocked
    Blocked(PlanBlocker),
    
    /// Plan should be deferred
    Defer(DeferConfig),
    
    /// Plan should be cancelled
    Cancel(CancelReason),
    
    /// Plan is complete
    Complete(PlanCompletion),
    
    /// Plan needs review
    NeedsReview(ReviewRequest),
    
    /// Plan needs approval
    NeedsApproval(ApprovalRequest),
}

pub struct PlanExecutionRequest {
    /// Plan to execute
    pub plan_id: PlanId,
    
    /// Execution strategy
    pub strategy: ExecutionStrategy,
    
    /// Resource allocation
    pub resources: ResourceAllocation,
    
    /// Time budget
    pub time_budget: Duration,
    
    /// Execution context
    pub context: ExecutionContext,
}

pub struct PlanAdjustment {
    /// Adjustment type
    pub adjustment_type: AdjustmentType,
    
    /// Affected steps
    pub affected_steps: Vec<StepId>,
    
    /// New configuration
    pub new_config: PlanConfig,
    
    /// Reason for adjustment
    pub reason: String,
    
    /// Expected impact
    pub expected_impact: AdjustmentImpact,
}

pub enum AdjustmentType {
    /// Reorder steps
    Reorder(Vec<StepId>),
    
    /// Add steps
    AddSteps(Vec<PlanStep>),
    
    /// Remove steps
    RemoveSteps(Vec<StepId>),
    
    /// Modify step
    ModifyStep(StepId, PlanStep),
    
    /// Change resources
    ChangeResources(ResourceRequirements),
    
    /// Change timeline
    ChangeTimeline(PlanTimeline),
    
    /// Change strategy
    ChangeStrategy(ExecutionStrategy),
}

pub struct PlanCompletion {
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
    
    /// Artifacts
    pub artifacts: Vec<PlanArtifact>,
}
```

## Internal State

### Planner State

```rust
pub struct PlannerState {
    /// Active plans
    pub active_plans: HashMap<PlanId, Plan>,
    
    /// Completed plans
    pub completed_plans: VecDeque<Plan>,
    
    /// Failed plans
    pub failed_plans: VecDeque<Plan>,
    
    /// Plan templates
    pub templates: HashMap<TemplateId, PlanTemplate>,
    
    /// Plan dependencies
    pub dependencies: PlanDependencyGraph,
    
    /// Plan metrics
    pub metrics: PlannerMetrics,
    
    /// Plan history
    pub history: Vec<PlanEvent>,
}

pub struct PlanTemplate {
    /// Template identifier
    pub id: TemplateId,
    
    /// Template name
    pub name: String,
    
    /// Template description
    pub description: String,
    
    /// Template steps
    pub steps: Vec<PlanStep>,
    
    /// Template resources
    pub resources: PlanResources,
    
    /// Template timeline
    pub timeline: PlanTimeline,
    
    /// Template usage count
    pub usage_count: u32,
    
    /// Template success rate
    pub success_rate: f32,
}

pub struct PlanDependencyGraph {
    /// Adjacency list
    pub adjacency: HashMap<PlanId, Vec<PlanId>>,
    
    /// Reverse adjacency list
    pub reverse: HashMap<PlanId, Vec<PlanId>>,
    
    /// Cycle detection state
    pub cycle_state: CycleDetectionState,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                      PLAN STATE MACHINE                              │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │      DRAFT       │◀─────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (plan approved)                                 │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │      ACTIVE      │◀──────────────────────┐              │        │
│  └────────┬─────────┘                        │              │        │
│           │                                  │              │        │
│     ┌─────┴─────┐                           │              │        │
│     │           │                           │              │        │
│     ▼           ▼                           │              │        │
│  ┌──────┐  ┌──────────┐                    │              │        │
│  │PAUSED│  │ DEFERRED │                    │              │        │
│  └──────┘  └────┬─────┘                    │              │        │
│     │           │ (unblocked/resumed)       │              │        │
│     └───────────┴──────────────────────────┘              │        │
│           │ (all steps complete)                          │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │    COMPLETED     │                                       │        │
│  └──────────────────┘                                       │        │
│           │                                                  │        │
│           │ (failure/unrecoverable)                         │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │      FAILED      │──────────────────────────────────────┘        │
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

### Plan Creation

```rust
fn create_plan(
    goal: &Goal,
    context: &PlanContext,
    state: &PlannerState,
) -> Plan {
    // Select plan template
    let template = select_template(goal, context, state);
    
    // Create plan from template
    let mut plan = if let Some(template) = template {
        create_plan_from_template(template, goal, context)
    } else {
        create_plan_from_scratch(goal, context)
    };
    
    // Optimize plan
    optimize_plan(&mut plan, state);
    
    // Validate plan
    validate_plan(&plan, state).unwrap_or_else(|e| {
        tracing::warn!(error = ?e, "Plan validation failed, using basic plan");
        create_basic_plan(goal, context)
    });
    
    plan
}

fn create_plan_from_template(
    template: &PlanTemplate,
    goal: &Goal,
    context: &PlanContext,
) -> Plan {
    Plan {
        id: PlanId::new(),
        description: format!("Plan for: {}", goal.description),
        plan_type: PlanType::Template(template.id.clone()),
        status: PlanStatus::Draft,
        owner: PlanOwner::Goal(goal.id.clone()),
        context: context.clone(),
        steps: template.steps.clone(),
        resources: template.resources.clone(),
        timeline: template.timeline.clone(),
        metrics: PlanMetrics::new(),
        history: Vec::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deadline: goal.deadline,
    }
}

fn create_plan_from_scratch(goal: &Goal, context: &PlanContext) -> Plan {
    // Analyze goal requirements
    let requirements = analyze_goal_requirements(goal, context);
    
    // Create steps based on requirements
    let steps = create_steps_from_requirements(&requirements);
    
    // Create timeline
    let timeline = create_timeline(&steps, goal.deadline);
    
    // Create resources
    let resources = create_resources(&steps, context);
    
    Plan {
        id: PlanId::new(),
        description: format!("Plan for: {}", goal.description),
        plan_type: PlanType::Sequential,
        status: PlanStatus::Draft,
        owner: PlanOwner::Goal(goal.id.clone()),
        context: context.clone(),
        steps,
        resources,
        timeline,
        metrics: PlanMetrics::new(),
        history: Vec::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deadline: goal.deadline,
    }
}
```

### Plan Optimization

```rust
fn optimize_plan(plan: &mut Plan, state: &PlannerState) {
    // Parallelize independent steps
    parallelize_independent_steps(plan);
    
    // Optimize resource allocation
    optimize_resource_allocation(plan);
    
    // Optimize timeline
    optimize_timeline(plan);
    
    // Remove redundant steps
    remove_redundant_steps(plan);
    
    // Add checkpoints
    add_checkpoints(plan);
}

fn parallelize_independent_steps(plan: &mut Plan) {
    // Find independent steps
    let independent_groups = find_independent_groups(&plan.steps);
    
    // Reorder steps to maximize parallelism
    plan.steps = independent_groups.into_iter().flatten().collect();
    
    // Update plan type if parallelism is possible
    if has_parallel_opportunities(&plan.steps) {
        plan.plan_type = PlanType::Hybrid;
    }
}
```

### Plan Execution

```rust
async fn execute_plan(
    plan: &Plan,
    executor: &PlanExecutor,
) -> Result<PlanCompletion, PlanError> {
    let mut completed_steps = Vec::new();
    let mut failed_steps = Vec::new();
    
    // Execute steps based on plan type
    match plan.plan_type {
        PlanType::Sequential => {
            for step in &plan.steps {
                match execute_step(step, executor).await {
                    Ok(result) => {
                        completed_steps.push((step.id.clone(), result));
                    }
                    Err(error) => {
                        failed_steps.push((step.id.clone(), error));
                        break; // Stop on failure for sequential plans
                    }
                }
            }
        }
        PlanType::Parallel => {
            let results = execute_steps_parallel(&plan.steps, executor).await;
            for (step_id, result) in results {
                match result {
                    Ok(result) => completed_steps.push((step_id, result)),
                    Err(error) => failed_steps.push((step_id, error)),
                }
            }
        }
        PlanType::Hybrid => {
            // Execute steps in parallel where possible, sequentially otherwise
            let execution_order = calculate_hybrid_execution_order(&plan.steps);
            for batch in execution_order {
                let results = execute_steps_parallel(&batch, executor).await;
                for (step_id, result) in results {
                    match result {
                        Ok(result) => completed_steps.push((step_id, result)),
                        Err(error) => failed_steps.push((step_id, error)),
                    }
                }
            }
        }
        PlanType::Adaptive => {
            // Execute steps adaptively based on results
            let mut context = ExecutionContext::new();
            for step in &plan.steps {
                let result = execute_step_with_context(step, executor, &context).await;
                match result {
                    Ok(result) => {
                        completed_steps.push((step.id.clone(), result.clone()));
                        context.update_from_result(&result);
                    }
                    Err(error) => {
                        failed_steps.push((step.id.clone(), error.clone()));
                        // Adaptive plans can skip failed steps
                        if !should_skip_step(step, &error, &context) {
                            break;
                        }
                    }
                }
            }
        }
        PlanType::Template(_) => {
            // Template plans execute like sequential plans
            for step in &plan.steps {
                match execute_step(step, executor).await {
                    Ok(result) => {
                        completed_steps.push((step.id.clone(), result));
                    }
                    Err(error) => {
                        failed_steps.push((step.id.clone(), error));
                        break;
                    }
                }
            }
        }
    }
    
    // Determine plan completion
    if failed_steps.is_empty() {
        Ok(PlanCompletion {
            completed_at: Utc::now(),
            actual_outcome: calculate_actual_outcome(&completed_steps),
            deviation: None,
            lessons_learned: extract_lessons_learned(&completed_steps),
            metrics: calculate_completion_metrics(&completed_steps),
            artifacts: extract_artifacts(&completed_steps),
        })
    } else {
        Err(PlanError::StepsFailed {
            completed: completed_steps,
            failed: failed_steps,
        })
    }
}
```

### Plan Adaptation

```rust
fn adapt_plan(
    plan: &mut Plan,
    deviation: &PlanDeviation,
    state: &PlannerState,
) -> PlanAdjustment {
    match deviation {
        PlanDeviation::OnTrack => {
            // No adjustment needed
            PlanAdjustment {
                adjustment_type: AdjustmentType::None,
                affected_steps: Vec::new(),
                new_config: PlanConfig::from_plan(plan),
                reason: "Plan is on track".to_string(),
                expected_impact: AdjustmentImpact::None,
            }
        }
        PlanDeviation::SlightlyBehind => {
            // Minor adjustment: increase resources slightly
            let adjustment = PlanAdjustment {
                adjustment_type: AdjustmentType::ChangeResources(
                    increase_resources(&plan.resources, 0.1)
                ),
                affected_steps: get_current_steps(plan),
                new_config: PlanConfig::from_plan(plan),
                reason: "Plan is slightly behind, increasing resources".to_string(),
                expected_impact: AdjustmentImpact::Minor,
            };
            adjustment
        }
        PlanDeviation::SignificantlyBehind => {
            // Major adjustment: reorder steps, add resources
            let adjustment = PlanAdjustment {
                adjustment_type: AdjustmentType::Reorder(
                    optimize_step_order(&plan.steps)
                ),
                affected_steps: get_remaining_steps(plan),
                new_config: PlanConfig::from_plan(plan),
                reason: "Plan is significantly behind, reordering steps".to_string(),
                expected_impact: AdjustmentImpact::Major,
            };
            adjustment
        }
        PlanDeviation::AheadOfSchedule => {
            // Positive adjustment: reduce resources, add quality checks
            let adjustment = PlanAdjustment {
                adjustment_type: AdjustmentType::ChangeResources(
                    decrease_resources(&plan.resources, 0.1)
                ),
                affected_steps: get_current_steps(plan),
                new_config: PlanConfig::from_plan(plan),
                reason: "Plan is ahead of schedule, reducing resources".to_string(),
                expected_impact: AdjustmentImpact::Minor,
            };
            adjustment
        }
        PlanDeviation::OffTrack => {
            // Critical adjustment: replan from scratch
            let adjustment = PlanAdjustment {
                adjustment_type: AdjustmentType::Reorder(
                    replan_steps(&plan.steps)
                ),
                affected_steps: get_remaining_steps(plan),
                new_config: PlanConfig::from_plan(plan),
                reason: "Plan is off track, replanning".to_string(),
                expected_impact: AdjustmentImpact::Critical,
            };
            adjustment
        }
    }
}
```

## Decision Logic

### When to Create a Plan

```rust
fn should_create_plan(
    goal: &Goal,
    state: &PlannerState,
) -> bool {
    // Always create plan for user goals
    if matches!(goal.owner, GoalOwner::User(_)) {
        return true;
    }
    
    // Create plan for complex system goals
    if is_complex_goal(goal) {
        return true;
    }
    
    // Create plan for agent goals with multiple steps
    if has_multiple_steps(goal) {
        return true;
    }
    
    // Don't create plan for simple tasks
    if is_simple_task(goal) {
        return false;
    }
    
    true
}
```

### When to Adapt a Plan

```rust
fn should_adapt_plan(
    plan: &Plan,
    deviation: &PlanDeviation,
    state: &PlannerState,
) -> bool {
    // Always adapt if plan is off track
    if matches!(deviation, PlanDeviation::OffTrack) {
        return true;
    }
    
    // Adapt if significantly behind
    if matches!(deviation, PlanDeviation::SignificantlyBehind) {
        return true;
    }
    
    // Don't adapt if on track
    if matches!(deviation, PlanDeviation::OnTrack) {
        return false;
    }
    
    // Adapt if behind and has adaptation options
    if matches!(deviation, PlanDeviation::SlightlyBehind) {
        return has_adaptation_options(plan, state);
    }
    
    false
}
```

## Failure Modes

### 1. Plan Stagnation

**Symptom**: Plan makes no progress
**Detection**: Progress unchanged for extended period
**Resolution**: Force adaptation, replan if necessary
**Prevention**: Regular progress checks, proactive adaptation

### 2. Plan Over-optimization

**Symptom**: Plan keeps changing without progress
**Detection**: High adaptation rate
**Resolution**: Freeze plan, force execution
**Prevention**: Adaptation cooldown, minimum execution time

### 3. Plan Under-estimation

**Symptom**: Plan consistently behind schedule
**Detection**: Repeated deadline misses
**Resolution**: Increase estimates, add buffer time
**Prevention**: Historical estimation, planning poker

### 4. Plan Complexity

**Symptom**: Plan too complex to execute
**Detection**: High step count, many dependencies
**Resolution**: Simplify plan, break into sub-plans
**Prevention**: Complexity limits, regular simplification

## Recovery Strategy

```rust
impl Planner {
    async fn recover_from_stagnation(
        &self,
        plan: &mut Plan,
        state: &mut PlannerState,
    ) {
        // Check if plan is blocked
        if let Some(blocker) = find_plan_blocker(plan, state) {
            // Try to resolve blocker
            match resolve_blocker(blocker, state).await {
                Ok(()) => {
                    tracing::info!("Plan blocker resolved");
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "Failed to resolve plan blocker");
                    // Force adaptation
                    let deviation = PlanDeviation::OffTrack;
                    let adjustment = adapt_plan(plan, &deviation, state);
                    apply_adjustment(plan, adjustment);
                }
            }
        } else {
            // Plan is not blocked, force execution
            plan.status = PlanStatus::Active;
            plan.history.push(PlanEvent::ForceExecuted {
                timestamp: Utc::now(),
                reason: "Plan stagnation detected".to_string(),
            });
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Operation | Target | Maximum | Measurement |
|-----------|--------|---------|-------------|
| Plan Creation | 10ms | 25ms | Per plan |
| Plan Optimization | 5ms | 15ms | Per plan |
| Plan Execution | Varies | Depends on steps | Per plan |
| Plan Adaptation | 5ms | 10ms | Per adaptation |
| Step Execution | Varies | Depends on step type | Per step |
| **Total Overhead** | **20ms** | **50ms** | **Per plan cycle** |

### Optimization Strategies

1. **Template Caching**: Cache plan templates for reuse
2. **Incremental Planning**: Plan only affected parts
3. **Parallel Execution**: Execute independent steps in parallel
4. **Resource Pooling**: Share resources across steps
5. **Predictive Adaptation**: Adapt before deviation occurs

## Security Considerations

### Plan Authorization

```rust
fn authorize_plan(
    plan: &Plan,
    owner: &PlanOwner,
    state: &PlannerState,
) -> bool {
    match owner {
        PlanOwner::Goal(goal_id) => {
            // Plans derived from goals inherit goal authorization
            authorize_goal_plan(plan, goal_id, state)
        }
        PlanOwner::Agent(agent_id) => {
            // Agent plans have limited authorization
            agent_has_plan_authority(agent_id, &plan.plan_type, state)
        }
        PlanOwner::User(user_id) => {
            // User plans are fully authorized
            true
        }
        PlanOwner::System(_) => {
            // System plans are pre-authorized
            true
        }
    }
}
```

### Plan Protection

- Plan priority cannot be escalated by unauthorized agents
- Plan deletion requires appropriate permissions
- Plan history is immutable
- Plan metrics are tamper-evident

## Privacy Rules

1. **Plan Privacy**: Plan details are only visible to owner and authorized agents
2. **Execution Privacy**: Execution details are private
3. **History Privacy**: Plan history is private to owner
4. **User Control**: Users can view, modify, and delete their plans
5. **Data Minimization**: Completed plans are archived after retention period

## Examples

### Example 1: Simple Sequential Plan

```
Goal: LearnSpanish
Plan: Sequential learning plan
Steps: [LearnVocabulary, PracticeGrammar, ConversationPractice]
Timeline: 30 days
Resources: [Memory(100MB), Network(API)]
Result: Success(progress=100%, duration=30d)
```

### Example 2: Parallel Plan

```
Goal: BuildWebsite
Plan: Parallel development plan
Steps: [DesignUI, DevelopBackend, CreateContent] (parallel)
       [IntegrateComponents, TestWebsite, DeployWebsite] (sequential)
Timeline: 14 days
Resources: [Memory(500MB), Network(API), Disk(10GB)]
Result: Success(progress=100%, duration=12d)
```

### Example 3: Adaptive Plan

```
Goal: SendEmail
Plan: Adaptive delivery plan
Steps: [ComposeEmail, ValidateEmail, SendEmail]
Context: Network unstable
Adaptation: Retry SendEmail with exponential backoff
Result: Success(progress=100%, duration=5m, attempts=3)
```

## Edge Cases

### 1. Plan Without Dependencies
**Scenario**: Plan has no step dependencies
**Handling**: Execute all steps in parallel

### 2. Plan With Circular Dependencies
**Scenario**: Step A depends on Step B, Step B depends on Step A
**Handling**: Detect cycle, break by deferring one step, notify user

### 3. Plan With Unrealistic Timeline
**Scenario**: Plan requires 1 week but deadline is 1 day
**Handling**: Warn user, suggest realistic timeline, allow override

### 4. Plan Abandonment
**Scenario**: Owner stops interacting with plan
**Handling**: Periodic check-ins, auto-defer after timeout, notify owner

### 5. Plan Merge Request
**Scenario**: Two similar plans created independently
**Handling**: Detect similarity, suggest merge, preserve history

## Future Extensions

1. **Plan Learning**: Learn from past plan success/failure
2. **Plan Prediction**: Anticipate plan needs before explicit request
3. **Plan Coordination**: Coordinate plans across multiple agents
4. **Plan Templates**: Reusable plan patterns
5. **Plan Visualization**: Visual plan trees and progress

## Engineering Notes

- Plan state is updated atomically
- Plan history is append-only
- Plan metrics are collected via `tracing` crate
- Plan priorities are configurable at runtime
- Planner supports graceful shutdown
- Plan state can be serialized for persistence
- Planner is testable with mock plans
- Planner supports concurrent plan processing
