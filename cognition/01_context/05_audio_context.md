# Audio Context

## Purpose

The Audio Context module provides the COS with awareness of the acoustic environment. It detects speech, identifies speakers, recognizes background sounds, estimates noise levels, and manages microphone state. This module wraps the `audio` crate's DSP capabilities and `voice` crate's VAD/wake-word detection, extending them with a managed audio analysis pipeline. It answers: *Is someone speaking? Who is speaking? What background sounds are present? How noisy is it? Is the wake word detected with confidence?*

## Responsibilities

1. **Speech detection**: Detect when someone is speaking (VAD)
2. **Speaker identification**: Distinguish between multiple speakers
3. **Background sound detection**: Identify background sounds (music, TV, phone, etc.)
4. **Noise estimation**: Estimate ambient noise level
5. **Wake word detection**: Detect the wake word with confidence scoring
6. **Microphone state management**: Track and control microphone state
7. **Audio event classification**: Classify audio events for context
8. **Audio quality assessment**: Assess audio quality for STT accuracy

## Internal Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AUDIO CONTEXT                                  │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    INPUT SOURCES                              │   │
│  │                                                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐   │   │
│  │  │  Audio  │  │   VAD   │  │  Wake   │  │  Background  │   │   │
│  │  │ Capture │  │ Detector│  │  Word   │  │  Classifier  │   │   │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬───────┘   │   │
│  │       │            │            │               │            │   │
│  └───────┼────────────┼────────────┼───────────────┼────────────┘   │
│          │            │            │               │                 │
│          ▼            ▼            ▼               ▼                 │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              AudioContextManager                              │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Speech Pipeline                                        │  │   │
│  │  │  Capture → VAD → Speaker ID → Session                  │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Background Pipeline                                    │  │   │
│  │  │  Capture → Feature Extract → Classification → Events    │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Noise Pipeline                                        │  │   │
│  │  │  Capture → Feature Extract → Noise Estimation → Level   │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐  │   │
│  │  │  Wake Word Pipeline                                    │  │   │
│  │  │  Capture → Feature Extract → Wake Word Detection → Confidence │  │   │
│  │  └────────────────────────────────────────────────────────┘  │   │
│  │                                                               │   │
│  └──────────────────────────┬───────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │              AudioSnapshot                                    │   │
│  │  Point-in-time view of all audio context                     │   │
│  │  Consumed by: VoicePipeline, Conversation, Cognition         │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Inputs

### Audio Signals

```rust
pub struct AudioSignal {
    /// Signal identifier
    pub id: String,
    
    /// Signal type
    pub signal_type: AudioSignalType,
    
    /// Signal timestamp
    pub observed_at: DateTime<Utc>,
    
    /// Signal confidence
    pub confidence: f64,
    
    /// Signal data
    pub data: AudioSignalData,
}

pub enum AudioSignalType {
    /// Speech detected
    SpeechDetected {
        speaker_id: Option<String>,
        start_time: DateTime<Utc>,
        confidence: f64,
    },
    
    /// Speech ended
    SpeechEnded {
        speaker_id: Option<String>,
        end_time: DateTime<Utc>,
        duration_ms: u64,
    },
    
    /// Wake word detected
    WakeWordDetected {
        keyword: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    
    /// Background sound detected
    BackgroundSound {
        sound_type: BackgroundSoundType,
        confidence: f64,
        volume_db: f64,
    },
    
    /// Noise level changed
    NoiseLevelChanged {
        level_db: f64,
        classification: NoiseLevel,
    },
    
    /// Speaker identified
    SpeakerIdentified {
        speaker_id: String,
        confidence: f64,
    },
    
    /// Audio quality changed
    QualityChanged {
        quality: AudioQuality,
        snr_db: f64,
    },
    
    /// Microphone state changed
    MicStateChanged {
        state: MicrophoneState,
    },
}

pub enum BackgroundSoundType {
    Music,
    Television,
    Phone,
    Alarm,
    Traffic,
    Wind,
    Rain,
    Crowd,
    Typing,
    Cooking,
    Animal,
    Other(String),
}

pub enum NoiseLevel {
    Quiet,
    Moderate,
    Loud,
    VeryLoud,
}

pub enum AudioQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Unusable,
}

pub enum MicrophoneState {
    /// Microphone is on and capturing
    Active,
    
    /// Microphone is on but muted
    Muted,
    
    /// Microphone is off
    Off,
    
    /// Microphone permission not granted
    NoPermission,
    
    /// Microphone device not available
    NoDevice,
}

pub struct AudioSignalData {
    /// Raw audio features
    pub features: AudioFeatures,
    
    /// Processed results
    pub results: Option<AudioResults>,
}

pub struct AudioFeatures {
    /// RMS energy (dB)
    pub rms_db: f64,
    
    /// Peak amplitude
    pub peak_amplitude: f64,
    
    /// Zero crossing rate
    pub zero_crossing_rate: f64,
    
    /// Spectral centroid
    pub spectral_centroid: f64,
    
    /// Spectral rolloff
    pub spectral_rolloff: f64,
    
    /// MFCC features
    pub mfcc: Vec<f64>,
    
    /// Duration of audio chunk (ms)
    pub chunk_duration_ms: u64,
    
    /// Sample rate
    pub sample_rate: u32,
}

pub struct AudioResults {
    /// VAD result
    pub vad: Option<VadResult>,
    
    /// Speaker identification
    pub speaker: Option<SpeakerResult>,
    
    /// Background classification
    pub background: Option<BackgroundResult>,
    
    /// Noise estimation
    pub noise: Option<NoiseResult>,
    
    /// Wake word detection
    pub wake_word: Option<WakeWordResult>,
}

pub struct VadResult {
    /// Is speech present
    pub is_speech: bool,
    
    /// Speech probability
    pub speech_probability: f64,
    
    /// Speech start time
    pub start_time: Option<DateTime<Utc>>,
    
    /// Speech duration so far
    pub duration_ms: u64,
}

pub struct SpeakerResult {
    /// Speaker identifier
    pub speaker_id: String,
    
    /// Speaker label (if known)
    pub label: Option<String>,
    
    /// Identification confidence
    pub confidence: f64,
    
    /// Number of speakers detected
    pub speaker_count: u32,
}

pub struct BackgroundResult {
    /// Background sound type
    pub sound_type: BackgroundSoundType,
    
    /// Classification confidence
    pub confidence: f64,
    
    /// Sound volume (dB)
    pub volume_db: f64,
}

pub struct NoiseResult {
    /// Estimated noise level (dB)
    pub level_db: f64,
    
    /// Noise classification
    pub classification: NoiseLevel,
    
    /// Signal-to-noise ratio (dB)
    pub snr_db: f64,
    
    /// Noise is stationary (constant) or non-stationary (varying)
    pub stationary: bool,
}

pub struct WakeWordResult {
    /// Wake word detected
    pub detected: bool,
    
    /// Detection confidence
    pub confidence: f64,
    
    /// Wake word keyword
    pub keyword: String,
    
    /// False positive probability
    pub false_positive概率: f64,
}
```

## Outputs

### Audio Snapshot

```rust
pub struct AudioSnapshot {
    /// Snapshot identifier
    pub id: String,
    
    /// Capture timestamp
    pub captured_at: DateTime<Utc>,
    
    /// Microphone state
    pub mic_state: MicrophoneState,
    
    /// Speech state
    pub speech: SpeechState,
    
    /// Speaker information
    pub speakers: SpeakerInfo,
    
    /// Background sounds
    pub background: Vec<BackgroundSoundInfo>,
    
    /// Noise level
    pub noise: NoiseInfo,
    
    /// Wake word state
    pub wake_word: WakeWordState,
    
    /// Audio quality
    pub quality: AudioQualityInfo,
    
    /// Audio events (recent)
    pub recent_events: Vec<AudioEventInfo>,
    
    /// Data freshness
    pub freshness: u64,
    
    /// Confidence in snapshot
    pub confidence: f64,
}

pub struct SpeechState {
    /// Is speech currently detected
    pub is_speech: bool,
    
    /// Current speaker
    pub current_speaker: Option<String>,
    
    /// Speech duration (ms)
    pub duration_ms: u64,
    
    /// Speech started at
    pub started_at: Option<DateTime<Utc>>,
    
    /// Is user speaking
    pub is_user: bool,
    
    /// Is system speaking (TTS)
    pub is_system: bool,
    
    /// Speech confidence
    pub confidence: f64,
}

pub struct SpeakerInfo {
    /// Number of speakers detected
    pub count: u32,
    
    /// Current speaker
    pub current: Option<SpeakerDetail>,
    
    /// Known speakers
    pub known_speakers: Vec<SpeakerDetail>,
    
    /// Speaker change detected
    pub change_detected: bool,
}

pub struct SpeakerDetail {
    /// Speaker identifier
    pub id: String,
    
    /// Speaker label
    pub label: Option<String>,
    
    /// Speaker confidence
    pub confidence: f64,
    
    /// Is this the user
    pub is_user: bool,
    
    /// Last speaking time
    pub last_spoke_at: DateTime<Utc>,
}

pub struct BackgroundSoundInfo {
    /// Sound type
    pub sound_type: BackgroundSoundType,
    
    /// Confidence
    pub confidence: f64,
    
    /// Volume (dB)
    pub volume_db: f64,
    
    /// Duration detected
    pub duration: Duration,
    
    /// Is interfering with speech
    pub interfering: bool,
}

pub struct NoiseInfo {
    /// Noise level (dB)
    pub level_db: f64,
    
    /// Noise classification
    pub classification: NoiseLevel,
    
    /// SNR (dB)
    pub snr_db: f64,
    
    /// Noise is stationary
    pub stationary: bool,
    
    /// Noise trend (increasing, decreasing, stable)
    pub trend: NoiseTrend,
    
    /// Estimated impact on STT accuracy
    pub stt_impact: SttImpact,
}

pub enum NoiseTrend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

pub enum SttImpact {
    None,
    Minor,
    Moderate,
    Severe,
}

pub struct WakeWordState {
    /// Wake word active
    pub active: bool,
    
    /// Wake word keyword
    pub keyword: String,
    
    /// Last detection confidence
    pub last_confidence: f64,
    
    /// Last detection time
    pub last_detected_at: Option<DateTime<Utc>>,
    
    /// Total detections in session
    pub detection_count: u32,
    
    /// False positive rate
    pub false_positive_rate: f64,
}

pub struct AudioQualityInfo {
    /// Overall quality
    pub quality: AudioQuality,
    
    /// SNR (dB)
    pub snr_db: f64,
    
    /// Clarity score (0.0-1.0)
    pub clarity: f64,
    
    /// Audio artifacts detected
    pub artifacts: Vec<AudioArtifact>,
    
    /// Recommended action
    pub recommendation: Option<String>,
}

pub enum AudioArtifact {
    Clipping,
    Echo,
    Reverb,
    Static,
    Dropouts,
    BackgroundNoise,
    Distortion,
}

pub struct AudioEventInfo {
    /// Event type
    pub event_type: String,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Event confidence
    pub confidence: f64,
    
    /// Event description
    pub description: String,
}
```

## State Transitions

```
┌─────────────────────────────────────────────────────────────────────┐
│                  AUDIO CONTEXT STATE MACHINE                         │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │   INITIALIZING   │                                               │
│  └────────┬─────────┘                                               │
│           │ (mic permission granted)                                │
│           ▼                                                          │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   LISTENING      │────▶│   MUTED          │                     │
│  └────────┬─────────┘     └────────┬─────────┘                     │
│           │ (speech detected)       │ (unmute)                       │
│           ▼                         ▼                                │
│  ┌──────────────────┐     ┌──────────────────┐                     │
│  │   SPEECH_ACTIVE  │◀────│   LISTENING      │                     │
│  └────────┬─────────┘     └──────────────────┘                     │
│           │ (speech ended)                                           │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   PROCESSING     │                                               │
│  └────────┬─────────┘                                               │
│           │ (processing complete)                                   │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   LISTENING      │                                               │
│  └──────────────────┘                                               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Decision Logic

### When to Activate Speech Detection

```rust
fn should_activate_speech_detection(
    audio_snapshot: &AudioSnapshot,
    config: &AudioConfig,
) -> bool {
    // Activate if wake word is active
    if audio_snapshot.wake_word.active {
        return true;
    }
    
    // Activate if user is in conversation mode
    if config.conversation_mode {
        return true;
    }
    
    // Activate if meeting mode is active
    if config.meeting_mode {
        return true;
    }
    
    // Activate if noise level changes significantly
    if matches!(audio_snapshot.noise.trend, NoiseTrend::Volatile) {
        return true;
    }
    
    false
}
```

### When to Identify Speakers

```rust
fn should_identify_speakers(
    audio_snapshot: &AudioSnapshot,
    config: &AudioConfig,
) -> bool {
    // Identify if multiple speakers detected
    if audio_snapshot.speakers.count > 1 {
        return true;
    }
    
    // Identify if meeting mode is active
    if config.meeting_mode {
        return true;
    }
    
    // Identify if speaker change detected
    if audio_snapshot.speakers.change_detected {
        return true;
    }
    
    false
}
```

## Failure Modes

| Mode | Detection | Recovery | Prevention |
|------|-----------|----------|------------|
| Mic permission denied | Permission error | Inform user, request permission | Request permissions at startup |
| Mic device unavailable | Device error | Fall back to text input | Multiple device fallback |
| VAD false positive | Low confidence | Increase VAD threshold | Adaptive VAD threshold |
| Wake word false positive | Low confidence | Increase wake word threshold | Personalized wake word model |
| Speaker ID failure | Low confidence | Use generic speaker label | Multiple speaker ID models |
| Audio quality degradation | SNR below threshold | Inform user, suggest environment change | Noise cancellation |
| Background noise overload | Noise level critical | Reduce processing, inform user | Noise reduction algorithms |

### Recovery Strategy

```rust
impl AudioContextManager {
    async fn recover_from_mic_failure(&self, error: &MicError) -> Option<AudioSnapshot> {
        match error {
            MicError::PermissionDenied => {
                tracing::warn!("Microphone permission denied");
                // Fall back to text-only mode
                self.fallback_to_text_mode();
                None
            }
            MicError::DeviceUnavailable => {
                tracing::warn!("Microphone device unavailable, trying alternative");
                self.try_alternative_device().await
            }
            MicError::AudioStreamError => {
                tracing::warn!("Audio stream error, restarting capture");
                self.restart_audio_capture().await
            }
        }
    }
}
```

## Privacy Considerations

1. **Audio processing**: All audio processing is local, never transmitted. Audio is processed in real-time and immediately discarded.
2. **Speaker identification**: Speaker labels are stored locally, never transmitted. Users control speaker labels.
3. **Wake word detection**: Wake word processing is local, never transmitted. Wake word models are stored locally.
4. **Background sounds**: Background sound classification is local, never logged or transmitted.
5. **Noise estimation**: Noise levels are local, never transmitted.
6. **No audio recording**: Audio is processed in real-time, never recorded or stored persistently.
7. **Microphone state**: Microphone state is visible to the user at all times.
8. **Permission control**: Users can revoke microphone permission at any time.

## Security Considerations

1. **Permission model**: Microphone access requires explicit OS-level permission.
2. **Secure processing**: Audio data is processed in secure memory, not written to disk.
3. **No remote transmission**: Audio data never leaves the device without explicit user consent.
4. **Integrity verification**: Audio streams are verified for integrity before processing.
5. **Access control**: Only authorized COS components can access audio context.
6. **Audit logging**: Audio context access is auditable.
7. **Tamper detection**: Microphone state tampering is detected and reported.

## Future Extensibility

1. **Multi-microphone support**: Simultaneous capture from multiple microphones
2. **Audio scene understanding**: Understand complex audio scenes
3. **Emotion detection from voice**: Detect emotion from voice patterns (not identity)
4. **Language detection**: Detect spoken language
5. **Audio search**: Search through conversation audio
6. **Audio transcription**: Real-time transcription of all speech
7. **Audio summaries**: Summarize audio content
8. **Accessibility features**: Enhanced audio for accessibility

## Examples

### Example 1: Speech Detection in Quiet Environment

```
Audio Snapshot:
  mic_state: Active
  speech: { is_speech: true, current_speaker: "user", confidence: 0.95 }
  noise: { level_db: 30, classification: Quiet, snr_db: 40 }
  quality: { quality: Excellent, clarity: 0.95 }
Impact: STT accuracy expected to be high
```

### Example 2: Speech in Noisy Environment

```
Audio Snapshot:
  mic_state: Active
  speech: { is_speech: true, current_speaker: "user", confidence: 0.7 }
  noise: { level_db: 70, classification: Loud, snr_db: 10 }
  background: [{ sound_type: Television, volume_db: 65, interfering: true }]
  quality: { quality: Fair, clarity: 0.6 }
Impact: STT accuracy may be reduced, inform user
```

### Example 3: Wake Word Detection

```
Audio Snapshot:
  wake_word: { active: true, keyword: "Hey VOXY", confidence: 0.92 }
  speech: { is_speech: false }
  noise: { level_db: 40, classification: Moderate }
Action: Activate conversation mode, begin listening for user input
```

## Engineering Notes

- Audio capture uses platform-specific APIs (Windows: WASAPI, Linux: ALSA/PulseAudio, macOS: Core Audio)
- VAD uses the `whisper` crate's built-in VAD with configurable sensitivity
- Wake word detection uses the `voice` crate's wake word detector
- Speaker identification uses voice embedding similarity (not identity verification)
- Background classification uses audio feature extraction + ML classification
- Noise estimation uses RMS energy and spectral analysis
- Audio quality assessment uses SNR and artifact detection
- All audio processing is real-time, no audio is stored
- Audio snapshots are produced on-demand, not cached
- Microphone state is monitored via OS callbacks
- All timestamps use `chrono::DateTime<Utc>` for consistency
