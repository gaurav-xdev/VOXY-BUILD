#[derive(Debug, Clone)]
pub struct ConversationConfig {
    pub session_timeout_seconds: u64,
    pub max_turns_per_session: u64,
    pub enable_barge_in: bool,
    pub barge_in_sensitivity: f32,
    pub idle_timeout_ms: u64,
    pub wake_on_voice: bool,
    pub wake_on_wake_word: bool,
    pub auto_sleep_after_ms: u64,
    pub context_retention_turns: usize,
    pub enable_personality_hooks: bool,
    pub default_personality_id: Option<String>,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            session_timeout_seconds: 3600,
            max_turns_per_session: 1000,
            enable_barge_in: true,
            barge_in_sensitivity: 0.5,
            idle_timeout_ms: 30000,
            wake_on_voice: true,
            wake_on_wake_word: true,
            auto_sleep_after_ms: 60000,
            context_retention_turns: 50,
            enable_personality_hooks: true,
            default_personality_id: None,
        }
    }
}
