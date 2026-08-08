#[derive(Clone)]
pub struct VoiceConfig {
    pub audio: voxy_audio::AudioRuntimeConfig,
    pub conversation: voxy_conversation::ConversationConfig,
    pub orchestrator: voxy_voice_orchestrator::VoiceOrchestratorConfig,
    pub wake_word: String,
    pub wake_word_enabled: bool,
    pub vad_enabled: bool,
    pub vad_threshold: f32,
    pub auto_start_capture: bool,
    pub enable_diagnostics: bool,
    pub personality_id: Option<String>,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            audio: voxy_audio::AudioRuntimeConfig::default(),
            conversation: voxy_conversation::ConversationConfig::default(),
            orchestrator: voxy_voice_orchestrator::VoiceOrchestratorConfig::default(),
            wake_word: "hey voxy".into(),
            wake_word_enabled: true,
            vad_enabled: true,
            vad_threshold: 0.5,
            auto_start_capture: false,
            enable_diagnostics: true,
            personality_id: None,
        }
    }
}
