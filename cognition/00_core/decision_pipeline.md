# Decision Pipeline

## Purpose

The Decision Pipeline is the cognitive component responsible for determining what action VOXY should take in response to a stimulus. It transforms understanding into intent, evaluates options, assesses risks, and produces a concrete decision. The pipeline is designed to be:
- **Transparent**: Every decision can be explained
- **Consistent**: Similar inputs produce similar decisions
- **Controllable**: Users can influence decision-making
- **Auditable**: All decisions are logged for review
- **Improvable**: Decisions improve over time via learning

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                      DECISION PIPELINE                               │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    INTENT RECOGNITION                        │    │
│  │  Parse → Classify → Extract Entities → Resolve Ambiguity    │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    OPTION GENERATION                         │    │
│  │  Skill Matching → Tool Selection → Action Generation         │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    RISK ASSESSMENT                           │    │
│  │  Threat Analysis → Permission Check → Impact Assessment      │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    OPTION SCORING                            │    │
│  │  Utility → Cost → Risk → Confidence → Composite Score       │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    DECISION SELECTION                        │    │
│  │  Rank → Filter → Select → Validate → Commit                 │    │
│  └───────────────────────────┬─────────────────────────────────┘    │
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    DECISION OUTPUT                           │    │
│  │  Action │ Response │ Defer │ Skip │ Explain                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Decision Context

```rust
pub struct DecisionContext {
    /// The stimulus being processed
    pub stimulus: Stimulus,
    
    /// Current understanding of the stimulus
    pub understanding: Understanding,
    
    /// Relevant memories
    pub memories: Vec<Memory>,
    
    /// Active goals
    pub goals: GoalStack,
    
    /// Current context
    pub context: CognitiveContext,
    
    /// Available skills
    pub skills: Vec<Skill>,
    
    /// Available tools
    pub tools: Vec<Tool>,
    
    /// Permission state
    pub permissions: PermissionState,
    
    /// Risk history
    pub risk_history: RiskHistory,
}

pub struct Understanding {
    /// Parsed intent
    pub intent: Intent,
    
    /// Extracted entities
    pub entities: Vec<Entity>,
    
    /// Confidence in understanding
    pub confidence: f32,
    
    /// Alternative interpretations
    pub alternatives: Vec<AlternativeInterpretation>,
    
    /// Required clarifications
    pub clarifications: Vec<Clarification>,
}
```

## Outputs

### Decision

```rust
pub enum Decision {
    /// Execute an action
    Execute(Action),
    
    /// Respond to user
    Respond(Response),
    
    /// Ask for clarification
    Clarify(ClarificationRequest),
    
    /// Defer to later
    Defer(DeferConfig),
    
    /// Skip this stimulus
    Skip(SkipReason),
    
    /// Cancel pending action
    Cancel(CancelReason),
    
    /// Escalate to human
    Escalate(EscalationReason),
}

pub struct Action {
    pub action_type: ActionType,
    pub target: ActionTarget,
    pub parameters: ActionParameters,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub rollback_strategy: RollbackStrategy,
    pub verification: VerificationStrategy,
}

pub enum ActionType {
    /// Execute a skill
    Skill(SkillId),
    
    /// Call a tool
    Tool(ToolId),
    
    /// Make an API call
    Api(ApiCall),
    
    /// Modify system state
    SystemState(SystemStateChange),
    
    /// Communicate with user
    Communicate(Communication),
    
    /// Coordinate with agents
    AgentCoordination(AgentMessage),
    
    /// No action needed
    None,
}
```

## Internal State

### Decision State

```rust
pub struct DecisionState {
    /// Current decision stage
    pub stage: DecisionStage,
    
    /// Generated options
    pub options: Vec<DecisionOption>,
    
    /// Risk assessments
    pub risk_assessments: Vec<RiskAssessment>,
    
    /// Scored options
    pub scored_options: Vec<ScoredOption>,
    
    /// Selected decision
    pub selected: Option<Decision>,
    
    /// Decision trace
    pub trace: DecisionTrace,
    
    /// Decision history for this stimulus
    pub history: Vec<DecisionAttempt>,
}

pub enum DecisionStage {
    IntentRecognition,
    OptionGeneration,
    RiskAssessment,
    OptionScoring,
    DecisionSelection,
    Validation,
    Commitment,
}
```

## State Machine

```
┌─────────────────────────────────────────────────────────────────────┐
│                   DECISION PIPELINE STATE MACHINE                    │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │ INTENT_RECOGNITION│◀────────────────────────────────────┐        │
│  └────────┬─────────┘                                       │        │
│           │ (intent recognized)                             │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │ OPTION_GENERATION │◀─────────────────────┐              │        │
│  └────────┬─────────┘                       │              │        │
│           │ (options generated)             │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                       │              │        │
│  │ RISK_ASSESSMENT  │                       │              │        │
│  └────────┬─────────┘                       │              │        │
│           │ (risk assessed)                 │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                       │              │        │
│  │ OPTION_SCORING   │                       │              │        │
│  └────────┬─────────┘                       │              │        │
│           │ (options scored)                │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                       │              │        │
│  │ DECISION_SELECTION│──────────────────────┤              │        │
│  └────────┬─────────┘ (no viable option)    │              │        │
│           │ (decision selected)             │              │        │
│           ▼                                  │              │        │
│  ┌──────────────────┐                       │              │        │
│  │    VALIDATION    │───────────────────────┘              │        │
│  └────────┬─────────┘ (validation failed: regenerate)      │        │
│           │ (validation passed)                            │        │
│           ▼                                                  │        │
│  ┌──────────────────┐                                       │        │
│  │   COMMITMENT     │──────────────────────────────────────┘        │
│  └──────────────────┘ (commitment failed: retry)                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Intent Recognition

```rust
fn recognize_intent(understanding: &Understanding) -> Intent {
    let intent = match &understanding.intent {
        ParsedIntent::Command(cmd) => Intent::ExecuteCommand(cmd.clone()),
        ParsedIntent::Question(q) => Intent::AnswerQuestion(q.clone()),
        ParsedIntent::Conversation(c) => Intent::ContinueConversation(c.clone()),
        ParsedIntent::Request(r) => Intent::FulfillRequest(r.clone()),
        ParsedIntent::Complaint(c) => Intent::AddressComplaint(c.clone()),
        ParsedIntent::Unknown => Intent::ClarifyWithUser,
    };
    
    // Apply confidence threshold
    if understanding.confidence < CONFIDENCE_THRESHOLD {
        Intent::ClarifyWithUser
    } else {
        intent
    }
}
```

### Option Generation

```rust
fn generate_options(
    intent: &Intent,
    context: &DecisionContext,
) -> Vec<DecisionOption> {
    let mut options = Vec::new();
    
    // Match intent to skills
    for skill in &context.skills {
        if skill.matches_intent(intent) {
            options.push(DecisionOption::Skill(skill.clone()));
        }
    }
    
    // Match intent to tools
    for tool in &context.tools {
        if tool.matches_intent(intent) {
            options.push(DecisionOption::Tool(tool.clone()));
        }
    }
    
    // Generate fallback options
    if options.is_empty() {
        options.push(DecisionOption::ClarifyWithUser);
        options.push(DecisionOption::UseHeuristic(intent.clone()));
    }
    
    // Apply user preferences
    apply_user_preferences(&mut options, context);
    
    options
}
```

### Risk Assessment

```rust
fn assess_risk(
    option: &DecisionOption,
    context: &DecisionContext,
) -> RiskAssessment {
    let mut risks = Vec::new();
    
    // Check permission requirements
    if let Some(required_permission) = option.required_permission() {
        if !context.permissions.has(required_permission) {
            risks.push(Risk::PermissionRequired(required_permission));
        }
    }
    
    // Check resource requirements
    if let Some(resource) = option.required_resource() {
        if !context.resources.available(resource) {
            risks.push(Risk::ResourceUnavailable(resource));
        }
    }
    
    // Check potential side effects
    for side_effect in option.potential_side_effects() {
        risks.push(Risk::SideEffect(side_effect));
    }
    
    // Check historical failure rate
    let failure_rate = context.risk_history.failure_rate(option);
    if failure_rate > FAILURE_THRESHOLD {
        risks.push(Risk::HighFailureRate(failure_rate));
    }
    
    RiskAssessment {
        risks,
        severity: calculate_severity(&risks),
        mitigation: generate_mitigation(&risks),
    }
}
```

### Option Scoring

```rust
fn score_option(
    option: &DecisionOption,
    risk: &RiskAssessment,
    context: &DecisionContext,
) -> ScoredOption {
    let utility = calculate_utility(option, context);
    let cost = calculate_cost(option, context);
    let risk_score = risk.severity as f64;
    let confidence = calculate_confidence(option, context);
    let user_preference = calculate_user_preference(option, context);
    
    // Weighted composite score
    let score = utility * UTILITY_WEIGHT
        - cost * COST_WEIGHT
        - risk_score * RISK_WEIGHT
        + confidence * CONFIDENCE_WEIGHT
        + user_preference * PREFERENCE_WEIGHT;
    
    ScoredOption {
        option: option.clone(),
        score,
        breakdown: ScoreBreakdown {
            utility,
            cost,
            risk_score,
            confidence,
            user_preference,
        },
    }
}
```

### Decision Selection

```rust
fn select_decision(
    scored_options: &[ScoredOption],
    context: &DecisionContext,
) -> Option<Decision> {
    // Filter out options below threshold
    let viable: Vec<_> = scored_options
        .iter()
        .filter(|o| o.score >= MINIMUM_SCORE)
        .collect();
    
    if viable.is_empty() {
        return Some(Decision::Clarify(ClarificationRequest {
            reason: ClarificationReason::NoViableOption,
            options: scored_options.iter().map(|o| o.option.clone()).collect(),
        }));
    }
    
    // Sort by score
    let mut sorted = viable;
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    
    // Select top option
    let selected = sorted[0];
    
    // Apply safety checks
    if selected.option.requires_human_approval() {
        return Some(Decision::Escalate(EscalationReason::HumanApprovalRequired));
    }
    
    // Convert to decision
    Some(match &selected.option {
        DecisionOption::Skill(skill) => Decision::Execute(Action {
            action_type: ActionType::Skill(skill.id.clone()),
            target: ActionTarget::System,
            parameters: skill.default_parameters(),
            timeout: skill.timeout(),
            retry_policy: skill.retry_policy(),
            rollback_strategy: skill.rollback_strategy(),
            verification: skill.verification_strategy(),
        }),
        DecisionOption::Tool(tool) => Decision::Execute(Action {
            action_type: ActionType::Tool(tool.id.clone()),
            target: ActionTarget::System,
            parameters: tool.default_parameters(),
            timeout: tool.timeout(),
            retry_policy: tool.retry_policy(),
            rollback_strategy: tool.rollback_strategy(),
            verification: tool.verification_strategy(),
        }),
        DecisionOption::ClarifyWithUser => Decision::Clarify(ClarificationRequest {
            reason: ClarificationReason::AmbiguousIntent,
            options: Vec::new(),
        }),
        DecisionOption::UseHeuristic(intent) => {
            Decision::Respond(generate_heuristic_response(intent))
        }
    })
}
```

## Decision Logic

### When to Decide vs. Defer

```rust
fn should_decide_now(
    intent: &Intent,
    context: &DecisionContext,
) -> bool {
    // Always decide immediately for urgent requests
    if intent.is_urgent() {
        return true;
    }
    
    // Always decide immediately for user-initiated interactions
    if context.stimulus.is_user_initiated() {
        return true;
    }
    
    // Defer if cognitive load is high
    if context.cognitive_load > CognitiveLoad::Medium {
        return false;
    }
    
    // Defer if required information is missing
    if intent.requires_information() && !context.has_required_information(intent) {
        return false;
    }
    
    // Defer if user is busy
    if context.user_is_busy() {
        return false;
    }
    
    true
}
```

### When to Skip

```rust
fn should_skip(
    stimulus: &Stimulus,
    context: &DecisionContext,
) -> Option<SkipReason> {
    // Skip if stimulus is below attention threshold
    if stimulus.priority() < context.attention_threshold {
        return Some(SkipReason::BelowThreshold);
    }
    
    // Skip if already processing similar stimulus
    if context.is_duplicate(stimulus) {
        return Some(SkipReason::Duplicate);
    }
    
    // Skip if user is in "do not disturb" mode
    if context.user_do_not_disturb && !stimulus.is_urgent() {
        return Some(SkipReason::DoNotDisturb);
    }
    
    // Skip if cognitive resources are exhausted
    if context.energy_level == EnergyLevel::Critical {
        return Some(SkipReason::ResourceExhausted);
    }
    
    None
}
```

## Failure Modes

### 1. Intent Recognition Failure

**Symptom**: Unable to parse user intent
**Detection**: Low confidence score
**Recovery**: Ask for clarification
**Prevention**: Improve parsing models, maintain context

### 2. No Viable Options

**Symptom**: No options match the intent
**Detection**: Empty option list
**Recovery**: Generate fallback options, ask for clarification
**Prevention**: Maintain comprehensive skill/tool registry

### 3. Risk Assessment Overload

**Symptom**: Too many risks identified
**Detection**: Risk count exceeds threshold
**Recovery**: Prioritize risks, defer to human
**Prevention**: Clear risk policies, risk categorization

### 4. Scoring Conflict

**Symptom**: Multiple options have similar scores
**Detection**: Score difference below threshold
**Recovery**: Use tiebreaker rules, ask user preference
**Prevention**: Clear scoring weights, user preference learning

### 5. Decision Timeout

**Symptom**: Decision takes too long
**Detection**: Timeout exceeded
**Recovery**: Use default/heuristic decision
**Prevention**: Optimize pipeline, cache decisions

## Recovery Strategy

```rust
impl DecisionPipeline {
    async fn recover_from_failure(
        &self,
        failure: &DecisionFailure,
        context: &DecisionContext,
    ) -> Decision {
        match failure {
            DecisionFailure::IntentRecognitionFailed => {
                // Ask for clarification
                Decision::Clarify(ClarificationRequest {
                    reason: ClarificationReason::IntentUnclear,
                    options: Vec::new(),
                })
            }
            DecisionFailure::NoViableOptions => {
                // Use heuristic or ask for clarification
                if let Some(heuristic) = self.get_heuristic(&context.understanding) {
                    Decision::Respond(heuristic)
                } else {
                    Decision::Clarify(ClarificationRequest {
                        reason: ClarificationReason::NoMatchingSkill,
                        options: self.list_available_skills(),
                    })
                }
            }
            DecisionFailure::RiskAssessmentFailed => {
                // Defer to human
                Decision::Escalate(EscalationReason::RiskAssessmentFailed)
            }
            DecisionFailure::Timeout => {
                // Use default decision
                self.get_default_decision(&context.understanding)
            }
        }
    }
}
```

## Performance Considerations

### Latency Budget

| Stage | Target | Maximum | Measurement |
|-------|--------|---------|-------------|
| Intent Recognition | 5ms | 10ms | Per intent |
| Option Generation | 3ms | 5ms | Per option set |
| Risk Assessment | 2ms | 3ms | Per option |
| Option Scoring | 1ms | 2ms | Per option |
| Decision Selection | 1ms | 2ms | Per decision |
| Validation | 1ms | 2ms | Per decision |
| **Total** | **13ms** | **24ms** | **Per decision** |

### Optimization Strategies

1. **Intent Caching**: Cache parsed intents for similar inputs
2. **Option Pre-generation**: Pre-generate options for common intents
3. **Risk Pre-assessment**: Pre-assess risks for known actions
4. **Score Memoization**: Memoize scoring calculations
5. **Parallel Assessment**: Assess multiple options in parallel

## Security Considerations

### Permission Enforcement

```rust
fn enforce_permissions(
    decision: &Decision,
    context: &DecisionContext,
) -> Result<(), SecurityError> {
    match decision {
        Decision::Execute(action) => {
            let required = action.required_permissions();
            for permission in required {
                if !context.permissions.has(permission) {
                    return Err(SecurityError::PermissionDenied(permission));
                }
            }
            Ok(())
        }
        Decision::Respond(_) => Ok(()), // Responses don't need permissions
        _ => Ok(()),
    }
}
```

### Audit Logging

```rust
fn log_decision(
    decision: &Decision,
    context: &DecisionContext,
    trace: &DecisionTrace,
) {
    tracing::info!(
        decision = ?decision,
        intent = ?context.understanding.intent,
        confidence = context.understanding.confidence,
        risks = ?trace.risks,
        reasoning = %trace.reasoning,
        "Decision made"
    );
}
```

## Privacy Rules

1. **Intent Privacy**: User intents are never logged in plaintext
2. **Decision Privacy**: Decision details are anonymized for debugging
3. **Risk Privacy**: Risk assessments are aggregated, not individualized
4. **User Control**: Users can review and delete decision logs
5. **Data Minimization**: Only necessary decision data is retained

## Examples

### Example 1: Simple Command

```
Input: "Turn on the lights"
Intent: Command(lights, on)
Options:
  1. Skill: smart_home (score: 0.95)
  2. Tool: light_api (score: 0.85)
  3. Clarify (score: 0.3)
Risk Assessment:
  - Smart home: Low risk, permission granted
  - Light API: Medium risk, requires network
Selection: Skill: smart_home
Decision: Execute(Skill(smart_home, {action: turn_on, device: lights}))
```

### Example 2: Complex Request

```
Input: "Book me a flight to NYC"
Intent: Request(flight_booking, destination: NYC)
Options:
  1. Skill: flight_booking (score: 0.9)
  2. Tool: airline_api (score: 0.8)
  3. Clarify (score: 0.7)
Risk Assessment:
  - Flight booking: High risk, financial transaction
  - Airline API: High risk, external service
Selection: Clarify (missing dates, preferences)
Decision: Clarify("When exactly? Any airline preference?")
```

### Example 3: Ambiguous Input

```
Input: "Make it better"
Intent: Unknown (ambiguous)
Options:
  1. Clarify (score: 0.9)
  2. Use heuristic (score: 0.4)
Risk Assessment:
  - Clarify: No risk
  - Heuristic: Medium risk, wrong assumption
Selection: Clarify
Decision: Clarify("What would you like me to improve?")
```

## Edge Cases

### 1. Contradictory Instructions
**Scenario**: User says "do X" then immediately "do Y"
**Handling**: Cancel pending X, execute Y, explain cancellation

### 2. Impossible Request
**Scenario**: User asks for something physically impossible
**Handling**: Explain limitation, suggest alternatives

### 3. Partially Impossible Request
**Scenario**: User asks for multiple things, some possible
**Handling**: Do what's possible, explain what's not

### 4. Circular Request
**Scenario**: User asks for something that leads to the same request
**Handling**: Detect cycle, break with explanation

### 5. Overloaded Request
**Scenario**: User asks for too many things at once
**Handling**: Prioritize, do most important, defer rest

## Future Extensions

1. **Predictive Decisions**: Anticipate user needs before they ask
2. **Collaborative Decisions**: Multiple users influence decisions
3. **Adaptive Decisions**: Learn from decision outcomes
4. **Creative Decisions**: Generate novel solutions
5. **Ethical Decisions**: Apply ethical frameworks to decisions

## Engineering Notes

- Decisions are immutable once committed
- All decision state transitions are logged
- Decision traces are signed for integrity
- Decisions support rollback via `RollbackStrategy`
- Decision timeout is configurable per action type
- Decision caching is supported for repeated intents
- Decision conflicts are resolved via priority rules
- Decision explanations are generated automatically
