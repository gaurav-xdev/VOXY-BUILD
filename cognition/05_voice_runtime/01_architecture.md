# VOXY Voice Runtime Architecture

## Overview

The Voice Runtime (`voxy-voice-runtime`) is the production integration layer that combines the existing voice pipeline with the Unified Brain Orchestrator, providing:

- Wake word detection (<150ms target)
- Streaming STT
- Interruptible speech
- Barge-in (user can interrupt VOXY while it is speaking)
- Low-latency TTS
- Voice Activity Detection
- Echo cancellation
- Conversation turn detection
- Full integration with voxy-brain
- Real-time streaming events for UI

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    VoiceRuntimeEngine                    │
│                                                         │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │   VoicePipeline   │  │ EchoCanceller │  │ TurnDetector  │  │
│  │  (voxy-voice)     │  │               │  │               │  │
│  │  - Wake word       │  │  - Capture    │  │  - EOU        │  │
│  │  - VAD             │  │  - Playback   │  │  - Long pause │  │
│  │  - STT             │  │  - Suppression│  │  - Timeout    │  │
│  │  - TTS             │  │               │  │  - Max dur    │  │
│  └─────────┬───────┘  └──────────────┘  └───────────────┘  │
│            │                                                 │
│  ┌─────────▼───────┐  ┌──────────────┐  ┌───────────────┐  │
│  │  UnifiedBrainEngine│  │StreamingManager│ │LatencyTracker │  │
│  │  (voxy-brain)     │  │               │  │               │  │
│  │  - Context         │  │  - Events     │  │  - Per-stage  │  │
│  │  - Companion       │  │  - Buffer     │  │  - μs timing  │  │
│  │  - HDR             │  │  - Subscribe  │  │               │  │
│  │  - Cognition       │  │               │  │               │  │
│  └───────────────────┘  └───────────────┘  └───────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Components

### VoiceRuntimeEngine

The main orchestrator that coordinates all voice subsystems:

```rust
pub struct VoiceRuntimeEngine {
    config: VoiceRuntimeConfig,
    state: RwLock<VoiceRuntimeState>,
    voice_pipeline: Arc<VoicePipeline>,
    brain: Arc<UnifiedBrainEngine>,
    turn_detector: Arc<TurnDetector>,
    echo_canceller: Arc<EchoCanceller>,
    streaming: Arc<StreamingManager>,
    // ...
}
```

**Lifecycle:**
1. `new()` - Create with config and brain
2. `init()` - Initialize pipeline and brain
3. `start()` - Start audio capture and listening
4. `process_audio_frame()` - Process incoming audio
5. `speak()` - Synthesize and play response
6. `interrupt()` - Barge-in with brain propagation
7. `stop()` - Stop capture and listening
8. `shutdown()` - Full teardown

### EchoCanceller

Real-time echo cancellation using adaptive filtering:

```rust
pub struct EchoCanceller {
    enabled: bool,
    tail_length: usize,
    reference_buffer: Arc<Mutex<VecDeque<f32>>>,
    suppression_factor: f32,
}
```

**Processing:**
- `process_capture()` - Record reference signal
- `process_playback()` - Record output signal
- `process_input()` - Apply echo cancellation to microphone input

### TurnDetector

Smart conversation turn detection:

```rust
pub struct TurnDetector {
    config: TurnDetectionConfig,
    is_in_turn: Arc<AtomicBool>,
    speech_start: Arc<RwLock<Option<Instant>>>,
    last_speech_time: Arc<RwLock<Option<Instant>>>,
    silence_duration_ms: Arc<AtomicU64>,
}
```

**Turn Boundaries:**
- `EndOfUtterance` - Silence after speech exceeds threshold
- `LongPause` - Extended silence during speech
- `MaxDurationReached` - Speech exceeds maximum duration
- `Timeout` - Turn timeout exceeded

### StreamingManager

Real-time event streaming for UI:

```rust
pub struct StreamingManager {
    event_tx: broadcast::Sender<VoiceStreamEvent>,
    event_buffer: Arc<Mutex<VecDeque<VoiceStreamEvent>>>,
}
```

**Events:**
- `WakeWordDetected` - Wake word detected with confidence
- `VoiceActivityStarted/Ended` - VAD state changes
- `PartialTranscription` - Streaming STT results
- `TurnStarted/Completed/Failed` - Turn lifecycle
- `SynthesisStarted/Completed` - TTS lifecycle
- `BargeInDetected` - User interruption
- `BrainEventForwarded` - Brain processing events
- `LatencyReport` - Per-stage timing

## Integration Flow

### Audio Processing Pipeline

```
Audio Input
    │
    ▼
┌─────────────┐
│ Echo Cancel  │  Capture reference signal
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    VAD      │  Detect voice activity
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Wake Word   │  Detect "Hey VOXY"
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Turn Detect │  Determine utterance boundaries
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    STT      │  Transcribe speech to text
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Brain     │  Process through cognition pipeline
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    TTS      │  Synthesize response
└──────┬──────┘
       │
       ▼
Audio Output
```

### Barge-in Flow

```
User speaks during TTS playback
    │
    ▼
┌─────────────┐
│ VAD detects │
│   voice     │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Check cooldn│  Prevent rapid re-trigger
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Interrupt TTS│  Stop current playback
└──────┬──────┘
       │
       ▼
┌─────────────┐
│Cancel Brain │  Cancel current turn
│   turn      │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Emit event  │  Notify UI of barge-in
└─────────────┘
```

## Configuration

```rust
pub struct VoiceRuntimeConfig {
    pub voice: VoiceConfig,           // Voice pipeline config
    pub brain: BrainConfig,           // Brain orchestrator config
    pub echo_cancellation_enabled: bool,
    pub echo_cancellation_tail_ms: u32,
    pub turn_detection: TurnDetectionConfig,
    pub barge_in: BargeInConfig,
    pub streaming: StreamingConfig,
    pub latency_tracking_enabled: bool,
}
```

### Turn Detection Config

```rust
pub struct TurnDetectionConfig {
    pub min_speech_duration_ms: u64,      // 200ms
    pub max_speech_duration_ms: u64,      // 30s
    pub end_of_utterance_silence_ms: u64, // 800ms
    pub long_pause_threshold_ms: u64,     // 2000ms
    pub min_silence_after_speech_ms: u64, // 300ms
    pub turn_timeout_ms: u64,             // 15s
}
```

### Barge-in Config

```rust
pub struct BargeInConfig {
    pub enabled: bool,                    // true
    pub min_tts_playback_ms: u64,         // 500ms
    pub interrupt_cooldown_ms: u64,       // 200ms
    pub propagate_to_brain: bool,         // true
}
```

## Latency Targets

| Stage | Target | Notes |
|-------|--------|-------|
| Wake word | <150ms | Energy-based detection |
| VAD | <1ms | RMS energy calculation |
| Echo cancellation | <10ms | Adaptive filter |
| STT | Varies | Depends on provider |
| Brain | <50ms | Full pipeline |
| TTS | <100ms | First chunk |
| Total | <500ms | End-to-end |

## Tests

26 integration tests covering:
- Runtime lifecycle (init, start, stop, shutdown)
- Turn detection (silence, timeout, max duration)
- Echo cancellation (enabled, disabled, suppression)
- Streaming events (emit, subscribe, buffering)
- Voice event conversion
- Brain event conversion
- Latency tracking
- Configuration defaults
- ID uniqueness
- State transitions
