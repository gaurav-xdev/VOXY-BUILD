# User Context

## Purpose

The User Context module tracks what the user is doing, what they are trying to accomplish, how they are feeling, and what mode of operation they are in. Unlike environment context (which describes the machine), user context describes the *human*. It answers: *What is the user working on? What are their goals? How fatigued are they? What mode are they in?* This context is consumed by the `ContextAssembler` as a `ContextSource::UserHistory` source, and directly influences attention allocation, goal prioritization, response style, and interaction modality.

## Responsibilities

1. **Current task tracking**: What the user is currently working on
2. **Project awareness**: What project the user is contributing to
3. **Goal tracking**: Short-term and long-term user goals
4. **Attention estimation**: Where the user's attention is focused
5. **Fatigue estimation**: How tired the user is (not inferred as fact)
6. **Work session tracking**: Session duration, breaks, productivity patterns
7. **Mode detection**: Focus mode, gaming mode, meeting mode, travel mode, study mode, creative mode
8. **Sleep schedule**: User's typical sleep/wake times
9. **Preferences**: Interaction preferences, response style, modality preferences
10. **Interaction history**: How the user has interacted with VOXY over time

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       USER CONTEXT                                   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │ Activity │  │  Voice  │  │ Desktop │  │ Conversation │   │   │
│  │  │ Detector │  │ Patterns│  │ State   │  │  History     │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              UserContextTracker                               │   │
│  │  Maintains: current_task, current_project, goals, modes,     │   │
│  │             fatigue, session, preferences, history            │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              UserContextSnapshot                              │   │
│  │  Point-in-time view of all user context                      │   │
│  │  Consumed by ContextAssembler as ContextSource::UserHistory  │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### User Signals

```rust
pub struct UserSignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: UserSignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum UserSignalType {
    /// Text input detected
    TextInput { app: String, chars_per_minute: f64 },
    
    /// Voice input detected
    VoiceInput { duration: Duration, words_per_minute: f64 },
    
    /// Mouse activity detected
    MouseActivity { clicks_per_minute: f64, movement_speed: f64 },
    
    /// Keyboard activity detected
    KeyboardActivity { keystrokes_per_minute: f64 },
    
    /// Application switched
    AppSwitch { from: String, to: String },
    
    /// Window focused
    WindowFocus { window_id: String, app: String },
    
    /// File opened
    FileOpened { path: String, app: String },
    
    /// Project directory detected
    ProjectDirectory { path: String, project_type: String },
    
    /// Meeting joined
    MeetingJoined { platform: String, participants: u32 },
    
    /// Call received
    CallReceived { caller: Option<String>, duration: Option<Duration> },
    
    /// Break detected
    BreakDetected { duration: Duration, reason: BreakReason },
    
    /// Sleep detected
    SleepDetected { duration: Duration },
    
    /// Wake detected
    WakeDetected,
    
    /// User explicit statement
    UserStatement { text: String, intent: Option<String> },
}

pub enum BreakReason {
    /// No input for extended period
    IdleTimeout,
    
    /// System sleep/wake cycle
    SystemSleep,
    
    /// User explicitly paused
    ExplicitPause,
    
    /// Lunch/break time detected
    TimeOfDayBreak,
}
```

## Outputs

### User Context Snapshot

```rust
pub struct UserContextSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Current task
    pub current_task: Option<CurrentTask>,
    
    /// Current project
    pub current_project: Option<CurrentProject>,
    
    /// Active goals
    pub active_goals: Vec<UserGoal>,
    
    /// Attention state
    pub attention: AttentionState,
    
    /// Fatigue estimate
    pub fatigue: FatigueEstimate,
    
    /// Work session
    pub session: WorkSession,
    
    /// Current mode
    pub mode: UserMode,
    
    /// Sleep schedule
    pub sleep_schedule: Option<SleepSchedule>,
    
    /// Preferences
    pub preferences: UserPreferences,
    
    /// Interaction history summary
    pub interaction_summary: InteractionSummary,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct CurrentTask {
    /// Task description
    pub description: String,
    
    /// Task type
    pub task_type: TaskType,
    
    /// Associated application
    pub application: String,
    
    /// Associated file/project
    pub context_path: Option<String>,
    
    /// Task start time
    pub started_at: DateTime<Utc>,
    
    /// Task duration so far
    pub duration: Duration,
    
    /// Task progress estimate (0.0-1.0)
    pub progress: Option<f64>,
    
    /// Task confidence
    pub confidence: f64,
}

pub enum TaskType {
    Coding,
    Writing,
    Research,
    Browsing,
    Communication,
    Design,
    Gaming,
    MediaConsumption,
    SystemAdministration,
    Meeting,
    Learning,
    Creative,
    Unknown,
}

pub struct CurrentProject {
    /// Project name
    pub name: String,
    
    /// Project path
    pub path: String,
    
    /// Project type
    pub project_type: String,
    
    /// Primary language/framework
    pub language: Option<String>,
    
    /// Project size estimate
    pub size_estimate: ProjectSize,
    
    /// Last activity in project
    pub last_activity: DateTime<Utc>,
}

pub enum ProjectSize {
    Small,
    Medium,
    Large,
    Enterprise,
}

pub struct UserGoal {
    /// Goal identifier
    pub id: String,
    
    /// Goal description
    pub description: String,
    
    /// Goal priority
    pub priority: GoalPriority,
    
    /// Goal status
    pub status: GoalStatus,
    
    /// Goal creation time
    pub created_at: DateTime<Utc>,
    
    /// Goal deadline
    pub deadline: Option<DateTime<Utc>>,
    
    /// Goal progress (0.0-1.0)
    pub progress: f64,
    
    /// Associated tasks
    pub tasks: Vec<String>,
}

pub enum GoalPriority {
    Critical,
    High,
    Medium,
    Low,
}

pub enum GoalStatus {
    Active,
    InProgress,
    Completed,
    Deferred,
    Cancelled,
}

pub struct AttentionState {
    /// Where attention is focused
    pub focus_target: FocusTarget,
    
    /// Attention confidence (0.0-1.0)
    pub confidence: f64,
    
    /// Attention duration at current target
    pub focus_duration: Duration,
    
    /// Number of context switches in current session
    pub context_switches: u32,
    
    /// Attention quality estimate
    pub quality: AttentionQuality,
    
    /// Distraction level
    pub distraction_level: DistractionLevel,
}

pub enum FocusTarget {
    /// Focused on a specific application
    Application(String),
    
    /// Focused on a specific file
    File(String),
    
    /// Focused on a conversation
    Conversation(String),
    
    /// Focused on a task
    Task(String),
    
    /// Scattered attention
    Scattered,
    
    /// No clear focus
    None,
}

pub enum AttentionQuality {
    Deep,
    Focused,
    Normal,
    Distracted,
    Fragmented,
}

pub enum DistractionLevel {
    None,
    Low,
    Moderate,
    High,
}

pub struct FatigueEstimate {
    /// Overall fatigue level (0.0 = alert, 1.0 = exhausted)
    pub level: f64,
    
    /// Confidence in estimate (0.0-1.0)
    pub confidence: f64,
    
    /// Factors contributing to fatigue
    pub factors: Vec<FatigueFactor>,
    
    /// Time awake since last sleep
    pub time_awake: Option<Duration>,
    
    /// Time in current session
    pub session_duration: Duration,
    
    /// Number of breaks taken
    pub breaks_taken: u32,
    
    /// Estimated cognitive capacity remaining
    pub cognitive_capacity: f64,
}

pub struct FatigueFactor {
    /// Factor type
    pub factor_type: FatigueFactorType,
    
    /// Factor contribution (0.0-1.0)
    pub contribution: f64,
    
    /// Factor description
    pub description: String,
}

pub enum FatigueFactorType {
    SessionDuration,
    TimeSinceBreak,
    TimeSinceSleep,
    CognitiveLoad,
    RepetitiveTask,
    TimeOfDay,
    UserStated,
}

pub struct WorkSession {
    /// Session identifier
    pub id: String,
    
    /// Session start time
    pub started_at: DateTime<Utc>,
    
    /// Current session duration
    pub duration: Duration,
    
    /// Number of breaks in session
    pub breaks: u32,
    
    /// Total break duration
    pub break_duration: Duration,
    
    /// Active work time
    pub active_time: Duration,
    
    /// Session productivity estimate
    pub productivity: Option<f64>,
    
    /// Session type
    pub session_type: SessionType,
}

pub enum SessionType {
    Work,
    Study,
    Gaming,
    Creative,
    Meeting,
   休闲,
}

pub enum UserMode {
    /// Normal operating mode
    Normal,
    
    /// Focus mode: minimize interruptions
    Focus,
    
    /// Gaming mode: low latency, minimal background
    Gaming,
    
    /// Meeting mode: reduce voice interaction
    Meeting,
    
    /// Travel mode: offline-capable, minimal resources
    Travel,
    
    /// Study mode: educational content prioritized
    Study,
    
    /// Creative mode: creative tools prioritized
    Creative,
    
    /// Sleep mode: minimal activity, do not disturb
    Sleep,
    
    /// Custom mode
    Custom { name: String },
}

pub struct SleepSchedule {
    /// Typical bedtime (hour:minute, 24h format)
    pub bedtime: (u8, u8),
    
    /// Typical wake time (hour:minute, 24h format)
    pub wake_time: (u8, u8),
    
    /// Confidence in schedule
    pub confidence: f64,
    
    /// Last known sleep time
    pub last_sleep: Option<DateTime<Utc>>,
    
    /// Last known wake time
    pub last_wake: Option<DateTime<Utc>>,
    
    /// Is user currently in sleep period
    pub is_sleeping: bool,
    
    /// Estimated time until next sleep
    pub time_until_sleep: Option<Duration>,
}

pub struct UserPreferences {
    /// Response verbosity preference
    pub verbosity: VerbosityPreference,
    
    /// Response formality preference
    pub formality: FormalityPreference,
    
    /// Preferred interaction modality
    pub modality: ModalityPreference,
    
    /// Notification preferences
    pub notifications: NotificationPreferences,
    
    /// Proactivity preference
    pub proactivity: ProactivityPreference,
    
    /// Timezone
    pub timezone: String,
    
    /// Language
    pub language: String,
    
    /// Accessibility needs
    pub accessibility: Vec<AccessibilityNeed>,
}

pub enum VerbosityPreference {
    Minimal,
    Concise,
    Normal,
    Detailed,
    Comprehensive,
}

pub enum FormalityPreference {
    Casual,
    Friendly,
    Professional,
    Formal,
}

pub enum ModalityPreference {
    Voice,
    Text,
    Mixed,
    Adaptive,
}

pub struct NotificationPreferences {
    /// Enable notifications
    pub enabled: bool,
    
    /// Quiet hours
    pub quiet_hours: Option<(u8, u8)>,
    
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    
    /// Priority threshold for notifications
    pub priority_threshold: GoalPriority,
}

pub enum NotificationChannel {
    Voice,
    Text,
    Visual,
    Haptic,
}

pub enum ProactivityPreference {
    /// Never proactively suggest
    Never,
    
    /// Suggest when explicitly enabled
    OnRequest,
    
    /// Suggest when confident
    WhenConfident,
    
    /// Always suggest
    Always,
}

pub enum AccessibilityNeed {
    ScreenReader,
    HighContrast,
    LargeText,
    KeyboardNavigation,
    VoiceControl,
    ReducedMotion,
}

pub struct InteractionSummary {
    /// Total interactions today
    pub today_count: u32,
    
    /// Total interactions this week
    pub week_count: u32,
    
    /// Average interactions per day
    pub daily_average: f64,
    
    /// Most common interaction type
    pub common_type: String,
    
    /// Most common time of interaction
    pub common_time: String,
    
    /// Satisfaction estimate (based on interaction patterns)
    pub satisfaction: Option<f64>,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                   USER CONTEXT STATE MACHINE                         │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (user signals received)                                  │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   TRACKING       │────▶│   MODE ACTIVE    │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (mode change)           │ (mode timeout/end)             │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   MODE TRANSITION│────▶│   TRACKING       │                     │
│  └──────────────────┘     └──────────────────┘                     │
│           │                                                          │
│           │ (extended inactivity)                                   │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   IDLE           │                                               │
│  └────────┬─────────┘                                               │
│           │ (user activity resumes)                                  │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   TRACKING       │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### Mode Detection

```rust
fn detect_user_mode(signals: &[UserSignal], environment: &EnvironmentSnapshot) -> UserMode {
    // Check explicit user statement first
    if let Some(UserSignalType::UserStatement { text, .. }) = signals.iter().find_map(|s| {
        if matches!(s.signal_type, UserSignalType::UserStatement { .. }) {
            Some(&s.signal_type)
        } else {
            None
        }
    }) {
        if let Some(mode) = parse_mode_from_statement(text) {
            return mode;
        }
    }
    
    // Check application signals for mode clues
    let app_signals: Vec<_> = signals.iter()
        .filter(|s| matches!(s.signal_type, UserSignalType::AppSwitch { .. } | UserSignalType::WindowFocus { .. }))
        .collect();
    
    // Gaming mode: game application in foreground
    if is_game_in_foreground(&app_signals) {
        return UserMode::Gaming;
    }
    
    // Meeting mode: meeting application in foreground
    if is_meeting_in_foreground(&app_signals) {
        return UserMode::Meeting;
    }
    
    // Study mode: educational application or study project
    if is_study_context(&app_signals, environment) {
        return UserMode::Study;
    }
    
    // Creative mode: creative application in foreground
    if is_creative_in_foreground(&app_signals) {
        return UserMode::Creative;
    }
    
    // Travel mode: limited network, battery-powered
    if is_travel_context(environment) {
        return UserMode::Travel;
    }
    
    // Focus mode: extended single-app focus
    if is_focus_context(&app_signals) {
        return UserMode::Focus;
    }
    
    UserMode::Normal
}
```

### Fatigue Estimation

```rust
fn estimate_fatigue(
    signals: &[UserSignal],
    session: &WorkSession,
    sleep_schedule: Option<&SleepSchedule>,
    environment: &EnvironmentSnapshot,
) -> FatigueEstimate {
    let mut factors = Vec::new();
    let mut total_contribution = 0.0;
    
    // Session duration factor
    let session_minutes = session.duration.num_minutes() as f64;
    let session_factor = (session_minutes / 120.0).min(1.0); // Max contribution at 2 hours
    factors.push(FatigueFactor {
        factor_type: FatigueFactorType::SessionDuration,
        contribution: session_factor,
        description: format!("Session duration: {} minutes", session_minutes),
    });
    total_contribution += session_factor * 0.3;
    
    // Time since break factor
    let break_factor = if session.breaks == 0 {
        (session_minutes / 60.0).min(1.0)
    } else {
        0.0
    };
    factors.push(FatigueFactor {
        factor_type: FatigueFactorType::TimeSinceBreak,
        contribution: break_factor,
        description: format!("Breaks taken: {}", session.breaks),
    });
    total_contribution += break_factor * 0.2;
    
    // Time of day factor
    let hour = environment.temporal.hour as f64;
    let time_of_day_factor = if hour >= 22.0 || hour < 6.0 {
        0.8 // Late night/early morning
    } else if hour >= 14.0 && hour <= 16.0 {
        0.4 // Post-lunch dip
    } else {
        0.1
    };
    factors.push(FatigueFactor {
        factor_type: FatigueFactorType::TimeOfDay,
        contribution: time_of_day_factor,
        description: format!("Time of day factor: {:.2}", time_of_day_factor),
    });
    total_contribution += time_of_day_factor * 0.2;
    
    // Time since sleep factor
    if let Some(sleep) = sleep_schedule {
        if let Some(last_wake) = sleep.last_wake {
            let time_awake = Utc::now() - last_wake;
            let awake_hours = time_awake.num_hours() as f64;
            let sleep_factor = ((awake_hours - 8.0) / 8.0).max(0.0).min(1.0);
            factors.push(FatigueFactor {
                factor_type: FatigueFactorType::TimeSinceSleep,
                contribution: sleep_factor,
                description: format!("Time awake: {:.1} hours", awake_hours),
            });
            total_contribution += sleep_factor * 0.3;
        }
    }
    
    let level = total_contribution.min(1.0);
    let confidence = calculate_fatigue_confidence(&factors);
    
    FatigueEstimate {
        level,
        confidence,
        factors,
        time_awake: None, // Would be calculated from sleep schedule
        session_duration: session.duration,
        breaks_taken: session.breaks,
        cognitive_capacity: (1.0 - level * 0.5).max(0.0), // Fatigue reduces capacity by up to 50%
    }
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Signal loss | No signals for > 5 minutes | Use last known state, reduce confidence | Event-driven updates, fallback polling |
| Stale user state | Freshness > threshold | Log warning, use cached state | Adaptive polling, event triggers |
| Incorrect mode detection | User correction | Update mode immediately | Explicit user override, confidence thresholds |
| Fatigue over-estimation | User correction | Reduce confidence, recalibrate | Conservative estimation, user feedback |
| Goal staleness | Goal not referenced for > 24h | Prompt user, suggest archival | Regular goal review prompts |
| Preference drift | Preferences change over time | Learn from interactions | Adaptive preference learning |

### Recovery Strategy

```rust
impl UserContextTracker {
    async fn recover_from_signal_loss(&self) {
        // Reduce confidence in current state
        self.confidence.store(
            (self.confidence.load(Ordering::Relaxed) * 0.5) as u64,
            Ordering::Relaxed,
        );
        
        // Log warning
        tracing::warn!("User signal loss detected, reducing confidence");
        
        // Schedule periodic check for signal recovery
        self.schedule_recovery_check();
    }
}
```

## Privacy Considerations

1. **Task tracking**: Task descriptions are stored locally, never transmitted. Users can disable task tracking.
2. **Goal tracking**: Goals are stored locally. Users have full control over goal data.
3. **Fatigue estimation**: Fatigue estimates are derived from observable patterns, not physiological data. Never claimed as medical fact.
4. **Mode detection**: Mode is inferred from application usage patterns. No content analysis.
5. **Sleep schedule**: Sleep schedule is estimated from system usage patterns. Never claimed as ground truth.
6. **Interaction history**: History is stored locally. Users can view, export, and delete history.
7. **Preferences**: Preferences are stored locally. Never transmitted without explicit consent.
8. **Attention tracking**: Attention is estimated from application focus, not eye tracking or biometrics.

## Security Considerations

1. **Data storage**: All user context data is stored in local encrypted database.
2. **Access control**: Only the user and authorized COS components can access user context.
3. **Data retention**: User context data is retained according to user-configured retention policy.
4. **Export control**: User data export requires explicit user action.
5. **Deletion**: User can delete all user context data at any time.
6. **No profiling**: User context is not used for advertising, selling, or third-party profiling.

## Future Extensibility

1. **Biometric integration**: Heart rate, eye tracking for attention estimation (with explicit user permission)
2. **Calendar integration**: Import calendar for richer goal/task context
3. **Project management integration**: Import from Jira, Trello, GitHub Issues
4. **Learning from feedback**: User corrects incorrect context → system learns
5. **Multi-user context**: Context for multiple users on shared devices
6. **Cross-device context**: Context syncs across user's devices
7. **Predictive context**: Predict user needs before explicit request

## Examples

### Example 1: Focused Coding Session

```
Signals: [TextInput { app: "VSCode", cpm: 120 }, WindowFocus { app: "VSCode" }]
Duration: 45 minutes
Breaks: 0
Time: 14:30 Wednesday
CurrentTask: { description: "Implementing auth module", task_type: Coding, confidence: 0.9 }
Mode: Focus
Fatigue: { level: 0.3, confidence: 0.7, cognitive_capacity: 0.85 }
Attention: { focus_target: File("auth.rs"), quality: Deep, distraction_level: None }
```

### Example 2: Late Night Research

```
Signals: [AppSwitch { from: "Chrome", to: "Chrome" }, TextInput { app: "Chrome", cpm: 45 }]
Duration: 3 hours
Breaks: 1
Time: 23:45 Tuesday
CurrentTask: { description: "Researching Rust async patterns", task_type: Research, confidence: 0.8 }
Mode: Normal
Fatigue: { level: 0.7, confidence: 0.6, cognitive_capacity: 0.65 }
SleepSchedule: { is_sleeping: false, time_until_sleep: 45 minutes }
```

### Example 3: Meeting Mode

```
Signals: [AppSwitch { to: "Zoom" }, MeetingJoined { platform: "Zoom", participants: 5 }]
Time: 10:00 Monday
CurrentTask: { description: "Sprint planning meeting", task_type: Meeting, confidence: 0.95 }
Mode: Meeting
Preferences: { modality: Text, verbosity: Concise }
Attention: { focus_target: Application("Zoom"), quality: Normal }
```

## Engineering Notes

- User context is tracked via application monitoring, not content analysis
- Mode detection uses application heuristics, not ML classification
- Fatigue estimation is a rough heuristic, not a medical measurement
- All timestamps use `chrono::DateTime<Utc>` for consistency
- User context snapshots are stored in a ring buffer (default: 100 snapshots)
- User preferences are loaded from encrypted local database at startup
- Interaction history is aggregated daily to reduce storage requirements
- Mode transitions have a configurable cooldown (default: 5 minutes) to prevent thrashing
