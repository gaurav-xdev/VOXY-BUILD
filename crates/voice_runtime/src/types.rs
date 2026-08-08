use serde::{Deserialize, Serialize};

use voxy_brain::BrainEvent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceSessionId(pub String);

impl VoiceSessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for VoiceSessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VoiceTurnId(pub String);

impl VoiceTurnId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for VoiceTurnId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceRuntimeState {
    Idle,
    Listening,
    ProcessingSpeech,
    Speaking,
    Interrupted,
    ShuttingDown,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoiceStreamEvent {
    WakeWordDetected {
        confidence: f32,
        latency_ms: u64,
    },
    VoiceActivityStarted {
        timestamp_ms: u64,
    },
    VoiceActivityEnded {
        duration_ms: u64,
    },
    PartialTranscription {
        text: String,
        confidence: f32,
        is_final: bool,
    },
    TurnStarted {
        turn_id: String,
        session_id: String,
    },
    TurnProcessing {
        stage: String,
    },
    TurnCompleted {
        turn_id: String,
        response_text: Option<String>,
        total_duration_ms: u64,
    },
    TurnFailed {
        turn_id: String,
        error: String,
    },
    SynthesisStarted {
        text: String,
    },
    SynthesisChunk {
        sample_rate: u32,
        duration_ms: u64,
    },
    SynthesisCompleted {
        duration_ms: u64,
    },
    BargeInDetected {
        tts_playback_ms: u64,
    },
    BrainEventForwarded {
        event: String,
    },
    LatencyReport {
        wake_word_us: u64,
        vad_us: u64,
        stt_us: u64,
        brain_us: u64,
        tts_us: u64,
        total_us: u64,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainOutputSummary {
    pub intent_type: String,
    pub confidence: f64,
    pub success: bool,
    pub response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LatencyBreakdown {
    pub wake_word_us: u64,
    pub vad_us: u64,
    pub echo_cancellation_us: u64,
    pub stt_us: u64,
    pub brain_us: u64,
    pub tts_us: u64,
    pub total_us: u64,
}

impl From<voxy_brain::BrainOutput> for BrainOutputSummary {
    fn from(bo: voxy_brain::BrainOutput) -> Self {
        Self {
            intent_type: bo
                .cognitive_result
                .as_ref()
                .map(|c| c.intent_type.clone())
                .unwrap_or_default(),
            confidence: bo
                .cognitive_result
                .as_ref()
                .map(|c| c.confidence)
                .unwrap_or(0.0),
            success: bo
                .cognitive_result
                .as_ref()
                .map(|c| c.success)
                .unwrap_or(false),
            response: bo.response_text.clone(),
        }
    }
}

pub fn brain_event_to_stream(event: &BrainEvent) -> Option<VoiceStreamEvent> {
    match event {
        BrainEvent::TurnStarted {
            turn_id,
            session_id,
        } => Some(VoiceStreamEvent::TurnStarted {
            turn_id: turn_id.clone(),
            session_id: session_id.clone(),
        }),
        BrainEvent::ContextCollecting => Some(VoiceStreamEvent::TurnProcessing {
            stage: "context".into(),
        }),
        BrainEvent::CompanionUpdating => Some(VoiceStreamEvent::TurnProcessing {
            stage: "companion".into(),
        }),
        BrainEvent::HdrUpdating => Some(VoiceStreamEvent::TurnProcessing {
            stage: "human_dynamics".into(),
        }),
        BrainEvent::CognitionProcessing => Some(VoiceStreamEvent::TurnProcessing {
            stage: "cognition".into(),
        }),
        BrainEvent::TurnCompleted {
            turn_id,
            total_duration_ms,
        } => Some(VoiceStreamEvent::TurnCompleted {
            turn_id: turn_id.clone(),
            response_text: None,
            total_duration_ms: *total_duration_ms,
        }),
        BrainEvent::TurnFailed { turn_id, error } => Some(VoiceStreamEvent::TurnFailed {
            turn_id: turn_id.clone(),
            error: error.clone(),
        }),
        BrainEvent::TurnInterrupted { turn_id, reason } => Some(VoiceStreamEvent::TurnFailed {
            turn_id: turn_id.clone(),
            error: format!("Interrupted: {reason}"),
        }),
        _ => None,
    }
}
