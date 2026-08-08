use voxy_brain::BrainError;

#[derive(Debug, thiserror::Error)]
pub enum VoiceRuntimeError {
    #[error("Voice runtime not initialized")]
    NotInitialized,
    #[error("Voice runtime already initialized")]
    AlreadyInitialized,
    #[error("Voice runtime not running")]
    NotRunning,
    #[error("Voice runtime already running")]
    AlreadyRunning,
    #[error("Wake word detector not configured")]
    NoWakeWordDetector,
    #[error("VAD detector not configured")]
    NoVadDetector,
    #[error("STT engine not configured")]
    NoSttEngine,
    #[error("TTS engine not configured")]
    NoTtsEngine,
    #[error("Audio capture error: {0}")]
    CaptureError(String),
    #[error("Audio playback error: {0}")]
    PlaybackError(String),
    #[error("Transcription error: {0}")]
    TranscriptionError(String),
    #[error("Synthesis error: {0}")]
    SynthesisError(String),
    #[error("Turn processing error: {0}")]
    TurnError(String),
    #[error("Brain integration error: {0}")]
    BrainError(String),
    #[error("Echo cancellation error: {0}")]
    EchoCancellationError(String),
    #[error("VAD detector error: {0}")]
    VadDetectorError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Interrupted")]
    Interrupted,
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    #[error(transparent)]
    Audio(#[from] voxy_audio::AudioError),
    #[error(transparent)]
    Brain(#[from] BrainError),
}

impl From<VoiceRuntimeError> for String {
    fn from(e: VoiceRuntimeError) -> Self {
        e.to_string()
    }
}

pub type Result<T> = std::result::Result<T, VoiceRuntimeError>;
