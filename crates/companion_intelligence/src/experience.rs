use crate::config::IntelligenceConfig;
use crate::conversation::{ConversationIntelligence, ConversationTurn};
use crate::decision::{Decision, DecisionContext, DecisionEngine};
use crate::emotional::{EmotionalSignal, EmotionalSnapshot, EmotionalStateMachine};
use crate::memory_importance::{MemoryImportanceEngine, MemoryItem};
use crate::personality_dynamics::{BasePersonality, Mood, PersonalityDynamics};
use crate::presence_engine::{PresenceEngine, PresenceEvent, PresenceState};
use crate::proactive::{ProactiveEngine, ProactiveSuggestion, SuggestionContext};

pub struct ExperienceLayer {
    emotional: EmotionalStateMachine,
    presence: PresenceEngine,
    conversation: ConversationIntelligence,
    memory_importance: MemoryImportanceEngine,
    proactive: ProactiveEngine,
    decision: DecisionEngine,
    personality: PersonalityDynamics,
    config: IntelligenceConfig,
}

#[derive(Debug, Clone)]
pub struct ExperienceSnapshot {
    pub emotional: EmotionalSnapshot,
    pub presence: crate::presence_engine::PresenceSnapshot,
    pub current_mood: Mood,
    pub mood_intensity: f64,
    pub active_suggestions: usize,
    pub memory_count: usize,
    pub conversation_depth: usize,
}

impl ExperienceLayer {
    pub fn new(config: IntelligenceConfig) -> Self {
        let personality = PersonalityDynamics::new(BasePersonality {
            name: "Voxy".to_string(),
            traits: vec![
                "helpful".to_string(),
                "attentive".to_string(),
                "friendly".to_string(),
                "intelligent".to_string(),
            ],
            default_mood: Mood::Calm,
            mood_volatility: config.personality.mood_decay_rate,
            enthusiasm_base: config.personality.confidence_update_rate,
            formality_base: config.personality.curiosity_trigger_threshold,
            humor_base: config.personality.humor_probability,
        });

        Self {
            emotional: EmotionalStateMachine::new(),
            presence: PresenceEngine::new(),
            conversation: ConversationIntelligence::new(
                config.conversation.max_memory_turns,
                config.conversation.topic_window_size,
            ),
            memory_importance: MemoryImportanceEngine::default(),
            proactive: ProactiveEngine::default(),
            decision: DecisionEngine::default(),
            personality,
            config,
        }
    }

    pub fn process_emotional_signal(&mut self, signal: EmotionalSignal) -> EmotionalSnapshot {
        let emotion_state = self.emotional.process_signal(signal);
        self.personality.adapt_to_user(emotion_state.valence);
        EmotionalSnapshot {
            primary: emotion_state.clone(),
            secondary: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn process_presence_event(&mut self, event: PresenceEvent) -> PresenceState {
        self.presence.process_event(event);
        self.presence.current_state()
    }

    pub fn process_conversation_turn(&mut self, turn: ConversationTurn) {
        self.conversation.process_turn(turn);
    }

    pub fn add_memory(&mut self, memory: MemoryItem) {
        self.memory_importance.add_memory(memory);
    }

    pub fn make_decision(&mut self, context: &DecisionContext) -> Decision {
        self.decision.make_decision(context)
    }

    pub fn get_suggestions(&mut self, context: &SuggestionContext) -> Vec<ProactiveSuggestion> {
        self.proactive.generate_suggestions(context)
    }

    pub fn get_snapshot(&self) -> ExperienceSnapshot {
        ExperienceSnapshot {
            emotional: self.emotional.get_snapshot(),
            presence: self.presence.get_snapshot(),
            current_mood: self.personality.current_mood(),
            mood_intensity: self.personality.mood_intensity(),
            active_suggestions: self.proactive.suggestions().len(),
            memory_count: self.memory_importance.memory_count(),
            conversation_depth: self.conversation.context().conversation_depth,
        }
    }

    pub fn emotional(&self) -> &EmotionalStateMachine {
        &self.emotional
    }

    pub fn presence(&self) -> &PresenceEngine {
        &self.presence
    }

    pub fn conversation(&self) -> &ConversationIntelligence {
        &self.conversation
    }

    pub fn memory_importance(&self) -> &MemoryImportanceEngine {
        &self.memory_importance
    }

    pub fn proactive(&self) -> &ProactiveEngine {
        &self.proactive
    }

    pub fn decision(&self) -> &DecisionEngine {
        &self.decision
    }

    pub fn personality(&self) -> &PersonalityDynamics {
        &self.personality
    }

    pub fn config(&self) -> &IntelligenceConfig {
        &self.config
    }

    pub fn voice_parameters(&self) -> VoiceParameters {
        VoiceParameters {
            speed: self.personality.voice_speed(),
            pause_duration: self.personality.pause_duration(),
            thinking_delay: self.personality.thinking_delay(),
            response_length: self.personality.response_length(),
            word_choice: self
                .personality
                .word_choice()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VoiceParameters {
    pub speed: f64,
    pub pause_duration: f64,
    pub thinking_delay: f64,
    pub response_length: f64,
    pub word_choice: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::IntelligenceConfig;

    #[test]
    fn test_experience_layer_creation() {
        let config = IntelligenceConfig::default();
        let layer = ExperienceLayer::new(config);
        let snapshot = layer.get_snapshot();
        assert_eq!(snapshot.memory_count, 0);
    }

    #[test]
    fn test_process_emotional_signal() {
        let config = IntelligenceConfig::default();
        let mut layer = ExperienceLayer::new(config);
        let signal = EmotionalSignal {
            signal_type: crate::emotional::SignalType::TaskCompletion,
            intensity: 0.8,
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
        };
        let snapshot = layer.process_emotional_signal(signal);
        assert!(snapshot.primary.confidence > 0.0);
    }

    #[test]
    fn test_voice_parameters() {
        let config = IntelligenceConfig::default();
        let layer = ExperienceLayer::new(config);
        let params = layer.voice_parameters();
        assert!(params.speed > 0.0);
        assert!(params.pause_duration > 0.0);
    }

    #[test]
    fn test_process_presence_event() {
        let config = IntelligenceConfig::default();
        let mut layer = ExperienceLayer::new(config);
        let event = PresenceEvent {
            event_type: crate::presence_engine::PresenceEventType::VoiceActivated,
            timestamp: chrono::Utc::now(),
            source: "test".to_string(),
            data: None,
        };
        let state = layer.process_presence_event(event);
        assert_eq!(state, PresenceState::Listening);
    }
}
