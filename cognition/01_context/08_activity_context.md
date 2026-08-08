# Activity Context

## Purpose

The Activity Context module interprets what the user is currently doing by synthesizing signals from multiple context sources. It classifies the user's current activity (coding, gaming, watching videos, reading, writing, browsing, meetings, editing, design, research) and provides this classification to the context fusion system. It answers: *What is the user doing right now?*

## Responsibilities

1. **Activity classification**: Classify the user's current activity
2. **Activity transitions**: Track changes in user activity
3. **Activity duration**: Track how long the user has been in an activity
4. **Activity confidence**: Provide confidence in activity classification
5. **Activity history**: Maintain history of activity patterns
6. **Activity prediction**: Predict upcoming activity changes

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                      ACTIVITY CONTEXT                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │ Desktop │  │ Visual  │  │  Audio  │  │ Conversation │   │   │
│  │  │ State   │  │ Context │  │ Context │  │ Context      │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              ActivityClassifier                               │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Signal Aggregation                                     │  │   │
│  │  │  - Collect signals from all sources                     │  │   │
│  │  │  - Normalize signal weights                             │  │   │
│  │  │  - Apply temporal smoothing                             │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Activity Classification                                │  │   │
│  │  │  - Rule-based classification                            │  │   │
│  │  │  - ML-based classification                              │  │   │
│  │  │  - Confidence scoring                                   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Transition Detection                                   │  │   │
│  │  │  - Detect activity changes                              │  │   │
│  │  │  - Apply hysteresis                                     │  │   │
│  │  │  - Track transition patterns                            │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              ActivitySnapshot                                 │   │
│  │  Point-in-time view of user activity                         │   │
│  │  Consumed by: Context Fusion, Cognition, User Context        │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Activity Signals

```rust
pub struct ActivitySignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: ActivitySignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal weight (importance for activity classification)
    pub weight: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum ActivitySignalType {
    /// Application in foreground
    ForegroundApp {
        app_name: String,
        app_category: AppCategory,
        window_title: String,
    },
    
    /// File opened
    FileOpened {
        path: String,
        file_type: String,
        app: String,
    },
    
    /// User input pattern
    InputPattern {
        keystrokes_per_minute: f64,
        mouse_clicks_per_minute: f64,
        input_type: InputType,
    },
    
    /// Audio pattern
    AudioPattern {
        has_speech: bool,
        has_music: bool,
        has_typing: bool,
        noise_level: f64,
    },
    
    /// Visual pattern
    VisualPattern {
        has_code: bool,
        has_video: bool,
        has_images: bool,
        has_text: bool,
        ui_elements: Vec<String>,
    },
    
    /// Conversation pattern
    ConversationPattern {
        turn_rate: f64,
        avg_message_length: f64,
        has_questions: bool,
    },
    
    /// Device pattern
    DevicePattern {
        has_gamepad: bool,
        has_external_display: bool,
        has_headphones: bool,
    },
    
    /// Time pattern
    TimePattern {
        hour_of_day: u8,
        day_of_week: u8,
        is_working_hours: bool,
    },
    
    /// User explicit statement
    UserStatement {
        text: String,
    },
}

pub enum AppCategory {
    CodeEditor,
    IDE,
    Browser,
    Email,
    Chat,
    VideoConferencing,
    MediaPlayer,
    Game,
    Design,
    Office,
    Terminal,
    FileExplorer,
    Settings,
    Other(String),
}

pub enum InputType {
    Typing,
    Mouse,
    Gamepad,
    Voice,
    Touch,
}
```

## Outputs

### Activity Snapshot

```rust
pub struct ActivitySnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Current activity
    pub current: ActivityInfo,
    
    /// Previous activity
    pub previous: Option<ActivityInfo>,
    
    /// Activity history (last N activities)
    pub history: Vec<ActivityInfo>,
    
    /// Activity transitions
    pub transitions: Vec<ActivityTransition>,
    
    /// Activity metrics
    pub metrics: ActivityMetrics,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct ActivityInfo {
    /// Activity type
    pub activity_type: ActivityType,
    
    /// Activity description
    pub description: String,
    
    /// Activity confidence
    pub confidence: f64,
    
    /// Activity start time
    pub started_at: DateTime<Utc>,
    
    /// Activity duration
    pub duration: Duration,
    
    /// Activity context
    pub context: HashMap<String, String>,
    
    /// Activity signals used
    pub signals_used: Vec<String>,
}

pub enum ActivityType {
    /// Coding activity
    Coding {
        language: Option<String>,
        framework: Option<String>,
        task: CodingTask,
    },
    
    /// Gaming activity
    Gaming {
        game: Option<String>,
        genre: Option<String>,
        mode: GameMode,
    },
    
    /// Watching videos
    WatchingVideo {
        platform: Option<String>,
        content_type: VideoContentType,
    },
    
    /// Reading activity
    Reading {
        content_type: ReadingContentType,
        source: Option<String>,
    },
    
    /// Writing activity
    Writing {
        content_type: WritingContentType,
        app: String,
    },
    
    /// Browsing activity
    Browsing {
        browser: String,
        site_type: SiteType,
    },
    
    /// Meeting activity
    Meeting {
        platform: Option<String>,
        participants: Option<u32>,
        role: MeetingRole,
    },
    
    /// Editing activity
    Editing {
        content_type: EditingContentType,
        app: String,
    },
    
    /// Design activity
    Design {
        design_type: DesignType,
        app: String,
    },
    
    /// Research activity
    Research {
        topic: Option<String>,
        sources: Vec<String>,
    },
    
    /// Communication activity
    Communication {
        channel: CommunicationChannel,
        app: String,
    },
    
    /// System administration
    SystemAdmin {
        task: String,
        tools: Vec<String>,
    },
    
    /// Creative activity
    Creative {
        creative_type: CreativeType,
        app: String,
    },
    
    /// Idle
    Idle,
    
    /// Unknown
    Unknown,
}

pub enum CodingTask {
    Debugging,
    Refactoring,
    Implementing,
    Reviewing,
    Testing,
    Documenting,
    Planning,
    Learning,
}

pub enum GameMode {
    SinglePlayer,
    Multiplayer,
    CoOp,
    Competitive,
    Tutorial,
}

pub enum VideoContentType {
    Tutorial,
    Entertainment,
    News,
    Educational,
    Live,
}

pub enum ReadingContentType {
    Article,
    Documentation,
    Book,
    Code,
    Email,
    SocialMedia,
    News,
}

pub enum WritingContentType {
    Document,
    Email,
    Code,
    Chat,
    Notes,
    Blog,
    Report,
}

pub enum SiteType {
    Social,
    News,
    Shopping,
    Reference,
    Entertainment,
    Education,
    Other(String),
}

pub enum MeetingRole {
    Presenter,
    Attendee,
    Organizer,
    Observer,
}

pub enum EditingContentType {
    Image,
    Video,
    Audio,
    Document,
    Code,
    Spreadsheet,
    Presentation,
}

pub enum DesignType {
    UI,
    UX,
    Graphic,
    Architecture,
    Data,
}

pub enum CommunicationChannel {
    Email,
    Chat,
    Video,
    Voice,
    Forum,
}

pub enum CreativeType {
    Writing,
    Music,
    Art,
    Video,
    Code,
}

pub struct ActivityTransition {
    /// From activity
    pub from: ActivityType,
    
    /// To activity
    pub to: ActivityType,
    
    /// Transition time
    pub timestamp: DateTime<Utc>,
    
    /// Transition confidence
    pub confidence: f64,
    
    /// Transition reason
    pub reason: TransitionReason,
}

pub enum TransitionReason {
    UserInitiated,
    TimeBased,
    ContextChange,
    Explicit,
}

pub struct ActivityMetrics {
    /// Total activities today
    pub today_count: u32,
    
    /// Most common activity today
    pub most_common_today: ActivityType,
    
    /// Time spent per activity today
    pub time_per_activity: HashMap<String, Duration>,
    
    /// Average activity duration
    pub avg_duration: Duration,
    
    /// Activity switch rate (switches per hour)
    pub switch_rate: f64,
    
    /// Focus score (0.0-1.0, higher = more focused)
    pub focus_score: f64,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  ACTIVITY CONTEXT STATE MACHINE                      │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (signals received)                                       │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   CLASSIFYING    │────▶│   DEGRADED       │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (classification done)   │ (classification restored)      │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   TRACKING       │◀────│   CLASSIFYING    │                     │
│  └────────┬─────────┘     └──────────────────┘                     │
│           │ (activity change)                                        │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   TRANSITIONING  │                                               │
│  └────────┬─────────┘                                               │
│           │ (transition complete)                                   │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   TRACKING       │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Algorithms

### Activity Classification

```rust
fn classify_activity(
    signals: &[ActivitySignal],
    previous_activity: &Option<ActivityInfo>,
) -> ActivityClassification {
    let mut scores: HashMap<ActivityType, f64> = HashMap::new();
    
    // Rule-based classification
    for signal in signals {
        match &signal.signal_type {
            ActivitySignalType::ForegroundApp { app_category, .. } => {
                match app_category {
                    AppCategory::CodeEditor | AppCategory::IDE => {
                        *scores.entry(ActivityType::Coding { language: None, framework: None, task: CodingTask::Implementing }).or_insert(0.0) += signal.weight * signal.confidence;
                    }
                    AppCategory::Browser => {
                        *scores.entry(ActivityType::Browsing { browser: "unknown".to_string(), site_type: SiteType::Other("unknown".to_string()) }).or_insert(0.0) += signal.weight * signal.confidence;
                    }
                    AppCategory::VideoConferencing => {
                        *scores.entry(ActivityType::Meeting { platform: None, participants: None, role: MeetingRole::Attendee }).or_insert(0.0) += signal.weight * signal.confidence;
                    }
                    AppCategory::Game => {
                        *scores.entry(ActivityType::Gaming { game: None, genre: None, mode: GameMode::SinglePlayer }).or_insert(0.0) += signal.weight * signal.confidence;
                    }
                    AppCategory::MediaPlayer => {
                        *scores.entry(ActivityType::WatchingVideo { platform: None, content_type: VideoContentType::Entertainment }).or_insert(0.0) += signal.weight * signal.confidence;
                    }
                    _ => {}
                }
            }
            ActivitySignalType::VisualPattern { has_code, has_video, has_text, .. } => {
                if *has_code {
                    *scores.entry(ActivityType::Coding { language: None, framework: None, task: CodingTask::Implementing }).or_insert(0.0) += signal.weight * signal.confidence * 0.5;
                }
                if *has_video {
                    *scores.entry(ActivityType::WatchingVideo { platform: None, content_type: VideoContentType::Entertainment }).or_insert(0.0) += signal.weight * signal.confidence * 0.3;
                }
            }
            ActivitySignalType::InputPattern { input_type, .. } => {
                match input_type {
                    InputType::Gamepad => {
                        *scores.entry(ActivityType::Gaming { game: None, genre: None, mode: GameMode::SinglePlayer }).or_insert(0.0) += signal.weight * signal.confidence * 0.4;
                    }
                    InputType::Typing => {
                        // Typing could be coding, writing, or chatting - need more context
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    // Find highest scoring activity
    let best_activity = scores.into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(activity, score)| (activity, score));
    
    if let Some((activity, score)) = best_activity {
        ActivityClassification {
            activity_type: activity,
            confidence: score.min(1.0),
            signals_used: signals.iter().map(|s| s.id.clone()).collect(),
        }
    } else {
        ActivityClassification {
            activity_type: ActivityType::Unknown,
            confidence: 0.0,
            signals_used: vec![],
        }
    }
}
```

## Decision Logic

### When to Classify Activity

```rust
fn should_classify_activity(
    signals: &[ActivitySignal],
    current_activity: &Option<ActivityInfo>,
    config: &ActivityConfig,
) -> bool {
    // Classify if no current activity
    if current_activity.is_none() {
        return true;
    }
    
    // Classify if foreground app changed
    if signals.iter().any(|s| matches!(s.signal_type, ActivitySignalType::ForegroundApp { .. })) {
        return true;
    }
    
    // Classify if confidence is low
    if let Some(activity) = current_activity {
        if activity.confidence < config.reclassify_threshold {
            return true;
        }
    }
    
    // Classify periodically
    if should_periodic_classification(config) {
        return true;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Classification failure | Low confidence | Use previous activity | Multiple classification strategies |
| Signal loss | No signals for extended period | Use last known activity | Fallback polling |
| Transition thrashing | Rapid activity changes | Apply hysteresis | Cooldown period |
| Context confusion | Conflicting signals | Weight signals by reliability | Signal quality assessment |
| Activity drift | Classification accuracy degrades | Re-calibrate classifier | Continuous learning |

### Recovery Strategy

```rust
impl ActivityClassifier {
    async fn recover_from_classification_failure(
        &self,
        signals: &[ActivitySignal],
    ) -> ActivityClassification {
        // Try simpler classification
        if let Some(classification) = self.simple_classification(signals) {
            return classification;
        }
        
        // Use previous activity with reduced confidence
        if let Some(prev) = self.last_activity.read().await.as_ref() {
            return ActivityClassification {
                activity_type: prev.activity_type.clone(),
                confidence: prev.confidence * 0.5,
                signals_used: vec![],
            };
        }
        
        // Default to unknown
        ActivityClassification {
            activity_type: ActivityType::Unknown,
            confidence: 0.0,
            signals_used: vec![],
        }
    }
}
```

## Privacy Considerations

1. **Activity tracking**: Activity information is stored locally, never transmitted.
2. **Application monitoring**: Application names are stored locally, never transmitted.
3. **File tracking**: File paths are stored locally, never transmitted.
4. **User control**: Users can disable activity tracking entirely.
5. **No content analysis**: Activity classification uses application and pattern data, not content.
6. **No profiling**: Activity data is not used for advertising or profiling.
7. **Data retention**: Activity data is retained according to user-configured policy.

## Security Considerations

1. **Data storage**: Activity data is stored in encrypted local database.
2. **Access control**: Only authorized COS components can access activity data.
3. **Integrity**: Activity data is tamper-evident.
4. **Audit logging**: Activity data access is auditable.
5. **No remote transmission**: Activity data never leaves the device without explicit consent.

## Future Extensibility

1. **ML-based classification**: Machine learning models for more accurate classification
2. **Activity prediction**: Predict upcoming activity changes
3. **Activity analytics**: Analyze activity patterns over time
4. **Activity automation**: Automatically adjust COS behavior based on activity
5. **Activity sharing**: Share activity context across user's devices
6. **Activity goals**: Track activity towards user goals
7. **Activity wellness**: Monitor activity for wellness insights

## Examples

### Example 1: Coding Activity

```
Signals: [
  ForegroundApp { app_name: "VSCode", app_category: CodeEditor, window_title: "main.rs" },
  VisualPattern { has_code: true, has_text: true },
  InputPattern { keystrokes_per_minute: 120, input_type: Typing },
]
Classification: { activity_type: Coding { language: "Rust", task: Implementing }, confidence: 0.95 }
Duration: 45 minutes
Focus Score: 0.9
```

### Example 2: Meeting Activity

```
Signals: [
  ForegroundApp { app_name: "Zoom", app_category: VideoConferencing },
  AudioPattern { has_speech: true, has_typing: false },
  DevicePattern { has_headphones: true },
]
Classification: { activity_type: Meeting { platform: "Zoom", role: Attendee }, confidence: 0.9 }
Duration: 30 minutes
```

### Example 3: Activity Transition

```
Previous: { activity_type: Coding, confidence: 0.9, duration: 45m }
Signal: ForegroundApp { app_name: "Chrome", app_category: Browser }
New: { activity_type: Browsing { site_type: Reference }, confidence: 0.8 }
Transition: Coding → Browsing
Reason: UserInitiated (app switch)
```

## Engineering Notes

- Activity classification uses rule-based + lightweight ML approach
- Signal aggregation uses weighted scoring with temporal smoothing
- Transition detection uses hysteresis to prevent thrashing
- Activity history is stored in ring buffer (default: 100 activities)
- Focus score is calculated from activity duration, switch rate, and input patterns
- All timestamps use `chrono::DateTime<Utc>` for consistency
- Activity classification runs in background thread to avoid blocking
