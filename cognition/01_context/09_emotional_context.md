# Emotional Context

## Purpose

The Emotional Context module estimates the user's emotional state from observable conversational cues. It explicitly does NOT infer emotional states as facts — it produces confidence-based signals that the context fusion system can use alongside other context sources. The system is designed to handle uncertainty gracefully and never makes sensitive assumptions about user emotions.

## Responsibilities

1. **Tone estimation**: Estimate conversational tone from wording and pacing
2. **Emotional signal detection**: Detect emotional signals from explicit user statements
3. **Confidence scoring**: Score confidence in emotional estimates
4. **Uncertainty handling**: Gracefully handle unknown or uncertain emotional states
5. **State tracking**: Track emotional state changes over time
6. **Sensitivity management**: Avoid making sensitive emotional assumptions

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     EMOTIONAL CONTEXT                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │Conversation│ │  Voice  │  │  User   │  │  Context     │   │   │
│  │  │ Text     │  │ Patterns│  │Explicit │  │  Fusion      │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              EmotionalEstimator                               │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Text Analysis                                          │  │   │
│  │  │  - Sentiment scoring                                    │  │   │
│  │  │  - Punctuation analysis                                 │  │   │
│  │  │  - Word choice analysis                                 │  │   │
│  │  │  - Emoji/emoticon detection                             │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Voice Analysis                                         │  │   │
│  │  │  - Speech rate                                          │  │   │
│  │  │  - Pause patterns                                       │  │   │
│  │  │  - Emphasis detection                                   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Explicit Statement Parsing                             │  │   │
│  │  │  - Direct emotional statements                          │  │   │
│  │  │  - Mood declarations                                    │  │   │
│  │  │  - State descriptions                                   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Confidence Scoring                                     │  │   │
│  │  │  - Signal reliability assessment                        │  │   │
│  │  │  - Uncertainty quantification                           │  │   │
│  │  │  - Confidence calibration                               │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              EmotionalSnapshot                                │   │
│  │  Point-in-time emotional state estimate with confidence      │   │
│  │  Consumed by: Context Fusion, Cognition, Personality         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Emotional Signals

```rust
pub struct EmotionalSignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: EmotionalSignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: serde_json::Value,
}

pub enum EmotionalSignalType {
    /// Text-based emotional cue
    TextCue {
        text: String,
        cues: Vec<TextEmotionalCue>,
    },
    
    /// Voice-based emotional cue
    VoiceCue {
        cues: Vec<VoiceEmotionalCue>,
    },
    
    /// Explicit user statement about emotional state
    ExplicitStatement {
        statement: String,
        parsed_emotion: Option<EmotionalState>,
    },
    
    /// User mood declaration
    MoodDeclaration {
        mood: String,
        intensity: Option<f64>,
    },
    
    /// Contextual emotional cue
    ContextualCue {
        cue_type: ContextualCueType,
        value: String,
    },
}

pub struct TextEmotionalCue {
    /// Cue type
    pub cue_type: TextCueType,
    
    /// Cue value
    pub value: String,
    
    /// Cue confidence
    pub confidence: f64,
}

pub enum TextCueType {
    /// Exclamation marks (excitement, frustration)
    Exclamation,
    
    /// Question marks (confusion, curiosity)
    Question,
    
    /// All caps (excitement, frustration)
    AllCaps,
    
    /// Repeated characters (emphasis)
    RepeatedChars,
    
    /// Emoji/emoticon
    Emoji { emoji: String },
    
    /// Sentiment word
    SentimentWord { word: String, sentiment: f64 },
    
    /// Filler words (hesitation, uncertainty)
    FillerWord { word: String },
    
    /// Positive markers
    PositiveMarker { word: String },
    
    /// Negative markers
    NegativeMarker { word: String },
    
    /// Urgency markers
    UrgencyMarker { word: String },
}

pub struct VoiceEmotionalCue {
    /// Cue type
    pub cue_type: VoiceCueType,
    
    /// Cue value
    pub value: f64,
    
    /// Cue confidence
    pub confidence: f64,
}

pub enum VoiceCueType {
    /// Speech rate (words per minute)
    SpeechRate,
    
    /// Pause duration (ms)
    PauseDuration,
    
    /// Emphasis strength
    EmphasisStrength,
    
    /// Pitch variation
    PitchVariation,
    
    /// Volume level
    VolumeLevel,
    
    /// Speech fluency
    SpeechFluency,
}

pub enum ContextualCueType {
    /// Time of day
    TimeOfDay,
    
    /// Activity type
    ActivityType,
    
    /// Recent events
    RecentEvents,
    
    /// Session duration
    SessionDuration,
    
    /// Fatigue level
    FatigueLevel,
}
```

## Outputs

### Emotional Snapshot

```rust
pub struct EmotionalSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Current emotional state estimate
    pub state: EmotionalState,
    
    /// Confidence in estimate
    pub confidence: f64,
    
    /// Uncertainty level
    pub uncertainty: f64,
    
    /// Contributing factors
    pub factors: Vec<EmotionalFactor>,
    
    /// Previous emotional state
    pub previous: Option<EmotionalState>,
    
    /// Emotional trajectory (improving, worsening, stable)
    pub trajectory: EmotionalTrajectory,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Recommendations for interaction
    pub recommendations: Vec<InteractionRecommendation>,
}

pub struct EmotionalState {
    /// Primary emotion
    pub primary: EmotionType,
    
    /// Secondary emotion (if any)
    pub secondary: Option<EmotionType>,
    
    /// Emotional intensity (0.0-1.0)
    pub intensity: f64,
    
    /// Emotional valence (negative: -1.0 to positive: 1.0)
    pub valence: f64,
    
    /// Emotional arousal (calm: 0.0 to excited: 1.0)
    pub arousal: f64,
    
    /// Is this estimate based on explicit user statement
    pub from_explicit: bool,
    
    /// Is this estimate based on inference
    pub from_inference: bool,
    
    /// Uncertainty in this state
    pub uncertainty: f64,
}

pub enum EmotionType {
    /// Calm - user is relaxed, at ease
    Calm,
    
    /// Focused - user is concentrating
    Focused,
    
    /// Frustrated - user is experiencing difficulty
    Frustrated,
    
    /// Excited - user is enthusiastic
    Excited,
    
    /// Confused - user is uncertain or lost
    Confused,
    
    /// Curious - user is exploring or asking questions
    Curious,
    
    /// Tired - user is fatigued
    Tired,
    
    /// Celebrating - user is celebrating success
    Celebrating,
    
    /// Neutral - no clear emotional signal
    Neutral,
    
    /// Unknown - unable to determine
    Unknown,
}

pub enum EmotionalTrajectory {
    /// Emotional state is improving
    Improving,
    
    /// Emotional state is worsening
    Worsening,
    
    /// Emotional state is stable
    Stable,
    
    /// Emotional state is volatile
    Volatile,
    
    /// Unable to determine trajectory
    Unknown,
}

pub struct EmotionalFactor {
    /// Factor type
    pub factor_type: EmotionalFactorType,
    
    /// Factor contribution (0.0-1.0)
    pub contribution: f64,
    
    /// Factor confidence
    pub confidence: f64,
    
    /// Factor description
    pub description: String,
}

pub enum EmotionalFactorType {
    TextSentiment,
    VoicePattern,
    ExplicitStatement,
    ContextualCue,
    ActivityContext,
    FatigueLevel,
    SessionDuration,
}

pub struct InteractionRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    
    /// Recommendation description
    pub description: String,
    
    /// Recommendation confidence
    pub confidence: f64,
    
    /// Recommendation priority
    pub priority: f64,
}

pub enum RecommendationType {
    /// Adjust response style
    AdjustStyle,
    
    /// Be more supportive
    BeSupportive,
    
    /// Be more concise
    BeConcise,
    
    /// Be more detailed
    BeDetailed,
    
    /// Suggest a break
    SuggestBreak,
    
    /// Acknowledge emotion
    AcknowledgeEmotion,
    
    /// Maintain current approach
    MaintainApproach,
    
    /// Proceed with caution
    ProceedWithCaution,
}
```

## Algorithms

### Emotional State Estimation

```rust
fn estimate_emotional_state(
    signals: &[EmotionalSignal],
    previous_state: &Option<EmotionalState>,
) -> EmotionalEstimate {
    let mut factors = Vec::new();
    let mut total_confidence = 0.0;
    
    // Analyze text cues
    for signal in signals {
        if let EmotionalSignalType::TextCue { text, cues } = &signal.signal_type {
            let text_factors = analyze_text_cues(text, cues);
            factors.extend(text_factors);
            total_confidence += signal.confidence;
        }
    }
    
    // Analyze voice cues
    for signal in signals {
        if let EmotionalSignalType::VoiceCue { cues } = &signal.signal_type {
            let voice_factors = analyze_voice_cues(cues);
            factors.extend(voice_factors);
            total_confidence += signal.confidence;
        }
    }
    
    // Check for explicit statements
    for signal in signals {
        if let EmotionalSignalType::ExplicitStatement { parsed_emotion, .. } = &signal.signal_type {
            if let Some(emotion) = parsed_emotion {
                factors.push(EmotionalFactor {
                    factor_type: EmotionalFactorType::ExplicitStatement,
                    contribution: 1.0,
                    confidence: signal.confidence,
                    description: format!("User explicitly stated: {:?}", emotion.primary),
                });
                total_confidence += signal.confidence;
            }
        }
    }
    
    // Calculate emotional state from factors
    let state = calculate_state_from_factors(&factors);
    
    // Calculate confidence
    let confidence = if total_confidence > 0.0 {
        (total_confidence / factors.len() as f64).min(1.0)
    } else {
        0.0
    };
    
    // Calculate uncertainty
    let uncertainty = calculate_uncertainty(&factors, previous_state);
    
    // Calculate trajectory
    let trajectory = calculate_trajectory(&state, previous_state);
    
    // Generate recommendations
    let recommendations = generate_recommendations(&state, &factors);
    
    EmotionalEstimate {
        state,
        confidence,
        uncertainty,
        factors,
        trajectory,
        recommendations,
    }
}
```

### Confidence Scoring

```rust
fn calculate_emotional_confidence(
    factors: &[EmotionalFactor],
    signal_count: usize,
) -> f64 {
    if factors.is_empty() {
        return 0.0;
    }
    
    // Factor confidence weights
    let explicit_weight = 1.0;
    let text_weight = 0.6;
    let voice_weight = 0.7;
    let contextual_weight = 0.4;
    
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    
    for factor in factors {
        let weight = match factor.factor_type {
            EmotionalFactorType::ExplicitStatement => explicit_weight,
            EmotionalFactorType::TextSentiment => text_weight,
            EmotionalFactorType::VoicePattern => voice_weight,
            EmotionalFactorType::ContextualCue => contextual_weight,
            _ => 0.5,
        };
        
        weighted_sum += factor.confidence * weight;
        total_weight += weight;
    }
    
    // Normalize by number of signals (more signals = more confidence)
    let signal_bonus = (signal_count as f64 / 10.0).min(0.2);
    
    let base_confidence = if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        0.0
    };
    
    (base_confidence + signal_bonus).min(1.0)
}

fn calculate_uncertainty(
    factors: &[EmotionalFactor],
    previous_state: &Option<EmotionalState>,
) -> f64 {
    let mut uncertainty = 0.0;
    
    // More factors = less uncertainty
    if factors.len() < 3 {
        uncertainty += 0.3;
    }
    
    // Conflicting factors increase uncertainty
    if has_conflicting_factors(factors) {
        uncertainty += 0.2;
    }
    
    // Low confidence factors increase uncertainty
    let avg_factor_confidence = factors.iter().map(|f| f.confidence).sum::<f64>() / factors.len() as f64;
    if avg_factor_confidence < 0.5 {
        uncertainty += 0.2;
    }
    
    // No previous state increases uncertainty
    if previous_state.is_none() {
        uncertainty += 0.1;
    }
    
    uncertainty.min(1.0)
}
```

## Decision Logic

### When to Estimate Emotional State

```rust
fn should_estimate_emotion(
    signals: &[EmotionalSignal],
    current_state: &Option<EmotionalState>,
    config: &EmotionalConfig,
) -> bool {
    // Always estimate if explicit statement received
    if signals.iter().any(|s| matches!(s.signal_type, EmotionalSignalType::ExplicitStatement { .. })) {
        return true;
    }
    
    // Estimate if enough text cues
    let text_cue_count = signals.iter()
        .filter(|s| matches!(s.signal_type, EmotionalSignalType::TextCue { .. }))
        .count();
    if text_cue_count >= config.min_text_cues_for_estimation {
        return true;
    }
    
    // Estimate if confidence is low
    if let Some(state) = current_state {
        if state.uncertainty > config.re_estimation_threshold {
            return true;
        }
    }
    
    // Estimate periodically
    if should_periodic_estimation(config) {
        return true;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Over-estimation | Confidence too high | Reduce confidence, acknowledge uncertainty | Conservative confidence scoring |
| Under-estimation | Confidence too low | Use more signals, improve analysis | Better signal extraction |
| Cultural bias | Consistent misclassification | Adjust for cultural context | Diverse training data |
| Sarcasm detection failure | Misclassified emotion | Fall back to neutral | Sarcasm detection models |
| Explicit statement misparse | Conflict with other signals | Prefer explicit statement | Better parsing |
| Emotional drift | Inconsistent state | Apply temporal smoothing | Smoothing algorithms |

### Recovery Strategy

```rust
impl EmotionalEstimator {
    async fn recover_from_estimation_failure(
        &self,
        signals: &[EmotionalSignal],
    ) -> EmotionalEstimate {
        // Try simpler estimation
        if let Some(estimate) = self.simple_estimation(signals) {
            return estimate;
        }
        
        // Use previous state with reduced confidence
        if let Some(prev) = self.previous_state.read().await.as_ref() {
            return EmotionalEstimate {
                state: EmotionalState {
                    primary: EmotionType::Unknown,
                    secondary: None,
                    intensity: 0.5,
                    valence: 0.0,
                    arousal: 0.5,
                    from_explicit: false,
                    from_inference: true,
                    uncertainty: 0.8,
                },
                confidence: 0.2,
                uncertainty: 0.8,
                factors: vec![],
                trajectory: EmotionalTrajectory::Unknown,
                recommendations: vec![],
            };
        }
        
        // Default to neutral with high uncertainty
        EmotionalEstimate::default()
    }
}
```

## Privacy Considerations

1. **Emotional data**: Emotional estimates are stored locally, never transmitted.
2. **No profiling**: Emotional data is not used for advertising or profiling.
3. **User control**: Users can disable emotional estimation entirely.
4. **Sensitivity**: Emotional data is treated as sensitive personal data.
5. **No assumptions**: The system never claims to know how the user feels.
6. **Uncertainty disclosure**: Uncertainty levels are always disclosed.
7. **Explicit preference**: Emotional estimation only uses explicit signals when available.
8. **Data retention**: Emotional data is retained according to user-configured policy.

## Security Considerations

1. **Data storage**: Emotional data is stored in encrypted local database.
2. **Access control**: Only authorized COS components can access emotional data.
3. **Integrity**: Emotional data is tamper-evident.
4. **Audit logging**: Emotional data access is auditable.
5. **No remote transmission**: Emotional data never leaves the device without explicit consent.
6. **Sensitivity classification**: Emotional data is classified as sensitive.

## Future Extensibility

1. **Voice emotion detection**: Enhanced voice-based emotion detection
2. **Facial expression analysis**: Facial expression-based emotion detection (with explicit permission)
3. **Biometric integration**: Heart rate, skin conductance for emotion detection (with explicit permission)
4. **Cultural adaptation**: Adapt emotion detection for cultural context
5. **Personalized models**: Learn individual emotional patterns
6. **Emotion tracking over time**: Track emotional patterns over extended periods
7. **Emotion-aware automation**: Automatically adjust behavior based on emotional state

## Examples

### Example 1: Frustration Detection

```
Signals: [
  TextCue { text: "This isn't working!!!", cues: [Exclamation, AllCaps, NegativeMarker("isn't")] },
  VoiceCue { cues: [SpeechRate(high), EmphasisStrength(high)] },
]
Estimate: {
  state: { primary: Frustrated, intensity: 0.8, valence: -0.7, arousal: 0.7, uncertainty: 0.3 },
  confidence: 0.85,
  recommendations: [
    AcknowledgeEmotion("Acknowledge the user's frustration"),
    BeSupportive("Offer help and support"),
    ProceedWithCaution("Be careful with next steps"),
  ]
}
```

### Example 2: Excitement Detection

```
Signals: [
  TextCue { text: "That worked! Awesome!!", cues: [Exclamation, PositiveMarker("awesome")] },
  ExplicitStatement { statement: "I'm so excited!", parsed_emotion: Some(Excited) },
]
Estimate: {
  state: { primary: Excited, intensity: 0.9, valence: 0.9, arousal: 0.8, from_explicit: true, uncertainty: 0.1 },
  confidence: 0.95,
  recommendations: [
    AcknowledgeEmotion("Acknowledge the user's excitement"),
    MaintainApproach("Continue with current approach"),
  ]
}
```

### Example 3: Uncertainty Handling

```
Signals: [
  TextCue { text: "Hmm, not sure about this", cues: [FillerWord("Hmm"), Question] },
]
Estimate: {
  state: { primary: Confused, intensity: 0.5, valence: -0.2, arousal: 0.3, uncertainty: 0.6 },
  confidence: 0.4,
  recommendations: [
    BeDetailed("Provide more detailed explanation"),
    AcknowledgeEmotion("Acknowledge the uncertainty"),
  ]
}
```

## Engineering Notes

- Emotional estimation is explicitly non-deterministic — results may vary
- Confidence scores are calibrated to be conservative (under-promise, over-deliver)
- Text analysis uses keyword matching and simple NLP, not deep learning
- Voice analysis uses basic audio features, not emotion recognition models
- Explicit user statements always override inferred states
- Emotional state is smoothed over time to prevent rapid oscillation
- All timestamps use `chrono::DateTime<Utc>` for consistency
- Emotional data is stored with user consent only
