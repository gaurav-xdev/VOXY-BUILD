#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("Voice pipeline not initialized")]
    NotInitialized,
    #[error("Voice pipeline already initialized")]
    AlreadyInitialized,
    #[error("Wake word detector not set")]
    NoWakeWordDetector,
    #[error("VAD detector not set")]
    NoVadDetector,
    #[error("STT engine not set")]
    NoSttEngine,
    #[error("TTS engine not set")]
    NoTtsEngine,
    #[error("Audio device error: {0}")]
    AudioDeviceError(String),
    #[error("Capture error: {0}")]
    CaptureError(String),
    #[error("Playback error: {0}")]
    PlaybackError(String),
    #[error("Speech session error: {0}")]
    SpeechSessionError(String),
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Already running")]
    AlreadyRunning,
    #[error("Not running")]
    NotRunning,
    #[error(transparent)]
    Audio(#[from] voxy_audio::AudioError),
    #[error(transparent)]
    Conversation(#[from] voxy_conversation::ConversationError),
    #[error(transparent)]
    Orchestrator(#[from] voxy_voice_orchestrator::VoiceOrchestratorError),
}

pub type Result<T> = std::result::Result<T, VoiceError>;
