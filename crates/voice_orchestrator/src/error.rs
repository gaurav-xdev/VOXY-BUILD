#[derive(Debug, thiserror::Error)]
pub enum VoiceOrchestratorError {
    #[error("No wake word detector available")]
    NoWakeWordDetector,
    #[error("No VAD available")]
    NoVadAvailable,
    #[error("No STT engine available")]
    NoSttEngine,
    #[error("No TTS engine available")]
    NoTtsEngine,
    #[error("Pipeline not initialized")]
    PipelineNotInitialized,
    #[error("Pipeline already running")]
    PipelineAlreadyRunning,
    #[error("Pipeline error: {0}")]
    PipelineError(String),
    #[error("Transcription failed: {0}")]
    TranscriptionFailed(String),
    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),
    #[error("Wake word detection failed: {0}")]
    WakeWordFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error(transparent)]
    Hardware(#[from] voxy_hardware::HardwareError),
    #[error(transparent)]
    Provider(#[from] voxy_provider_core::ProviderError),
}

pub type Result<T> = std::result::Result<T, VoiceOrchestratorError>;
