#[derive(Clone)]
pub struct VoiceRuntimeConfig {
    pub voice: voxy_voice::VoiceConfig,
    pub brain: voxy_brain::BrainConfig,
    pub echo_cancellation_enabled: bool,
    pub echo_cancellation_tail_ms: u32,
    pub turn_detection: TurnDetectionConfig,
    pub barge_in: BargeInConfig,
    pub streaming: StreamingConfig,
    pub latency_tracking_enabled: bool,
}

impl std::fmt::Debug for VoiceRuntimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VoiceRuntimeConfig")
            .field("echo_cancellation_enabled", &self.echo_cancellation_enabled)
            .field("echo_cancellation_tail_ms", &self.echo_cancellation_tail_ms)
            .field("turn_detection", &self.turn_detection)
            .field("barge_in", &self.barge_in)
            .field("streaming", &self.streaming)
            .field("latency_tracking_enabled", &self.latency_tracking_enabled)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct TurnDetectionConfig {
    pub min_speech_duration_ms: u64,
    pub max_speech_duration_ms: u64,
    pub end_of_utterance_silence_ms: u64,
    pub long_pause_threshold_ms: u64,
    pub min_silence_after_speech_ms: u64,
    pub turn_timeout_ms: u64,
}

#[derive(Debug, Clone)]
pub struct BargeInConfig {
    pub enabled: bool,
    pub min_tts_playback_ms: u64,
    pub interrupt_cooldown_ms: u64,
    pub propagate_to_brain: bool,
}

#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub enabled: bool,
    pub event_buffer_size: usize,
    pub partial_transcription_interval_ms: u64,
    pub tts_streaming_chunk_ms: u64,
}

impl Default for VoiceRuntimeConfig {
    fn default() -> Self {
        Self {
            voice: voxy_voice::VoiceConfig::default(),
            brain: voxy_brain::BrainConfig::default(),
            echo_cancellation_enabled: true,
            echo_cancellation_tail_ms: 128,
            turn_detection: TurnDetectionConfig::default(),
            barge_in: BargeInConfig::default(),
            streaming: StreamingConfig::default(),
            latency_tracking_enabled: true,
        }
    }
}

impl Default for TurnDetectionConfig {
    fn default() -> Self {
        Self {
            min_speech_duration_ms: 200,
            max_speech_duration_ms: 30000,
            end_of_utterance_silence_ms: 800,
            long_pause_threshold_ms: 2000,
            min_silence_after_speech_ms: 300,
            turn_timeout_ms: 15000,
        }
    }
}

impl Default for BargeInConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_tts_playback_ms: 500,
            interrupt_cooldown_ms: 200,
            propagate_to_brain: true,
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            event_buffer_size: 256,
            partial_transcription_interval_ms: 100,
            tts_streaming_chunk_ms: 20,
        }
    }
}
