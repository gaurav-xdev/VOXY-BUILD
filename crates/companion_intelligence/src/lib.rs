pub mod bridge;
pub mod companion_moments;
pub mod config;
pub mod conversation;
pub mod decision;
pub mod emotional;
pub mod error;
pub mod experience;
pub mod memory_importance;
pub mod personality_dynamics;
pub mod presence_engine;
pub mod proactive;

pub use bridge::{ExperienceBridge, ExperienceInput, ExperienceOutput};
pub use companion_moments::{CompanionMoment, MomentContext, MomentEngine, MomentType};
pub use config::IntelligenceConfig;
pub use conversation::{
    ConversationIntelligence, ConversationMemory, ReferenceResolver, TopicTracker,
};
pub use decision::{Decision, DecisionContext, DecisionEngine, DecisionType};
pub use emotional::{EmotionState, EmotionType, EmotionalSignal, EmotionalStateMachine};
pub use error::{IntelligenceError, Result};
pub use experience::{ExperienceLayer, ExperienceSnapshot, VoiceParameters};
pub use memory_importance::{
    ImportanceLevel, MemoryImportanceEngine, MemoryItem, MemoryScore, MemoryType,
};
pub use personality_dynamics::{BasePersonality, Mood, MoodEntry, PersonalityDynamics};
pub use presence_engine::{
    PresenceEngine, PresenceEvent, PresenceEventType, PresenceSnapshot, PresenceState,
};
pub use proactive::{
    CooldownEntry, ProactiveConfig, ProactiveEngine, ProactiveSuggestion, SuggestionContext,
    SuggestionType,
};

pub mod prelude {
    pub use crate::bridge::{ExperienceBridge, ExperienceInput, ExperienceOutput};
    pub use crate::companion_moments::{CompanionMoment, MomentContext, MomentEngine, MomentType};
    pub use crate::config::IntelligenceConfig;
    pub use crate::conversation::{
        ConversationIntelligence, ConversationMemory, ReferenceResolver, TopicTracker,
    };
    pub use crate::decision::{Decision, DecisionContext, DecisionEngine, DecisionType};
    pub use crate::emotional::{EmotionState, EmotionType, EmotionalSignal, EmotionalStateMachine};
    pub use crate::error::{IntelligenceError, Result};
    pub use crate::experience::{ExperienceLayer, ExperienceSnapshot, VoiceParameters};
    pub use crate::memory_importance::{
        ImportanceLevel, MemoryImportanceEngine, MemoryItem, MemoryScore, MemoryType,
    };
    pub use crate::personality_dynamics::{BasePersonality, Mood, MoodEntry, PersonalityDynamics};
    pub use crate::presence_engine::{
        PresenceEngine, PresenceEvent, PresenceEventType, PresenceSnapshot, PresenceState,
    };
    pub use crate::proactive::{
        CooldownEntry, ProactiveConfig, ProactiveEngine, ProactiveSuggestion, SuggestionContext,
        SuggestionType,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_compiles() {
        let _config = IntelligenceConfig::default();
        let _emotional = EmotionalStateMachine::new();
        let _presence = PresenceEngine::new();
        let _conversation = ConversationIntelligence::new(20, 10);
        let _memory = MemoryImportanceEngine::default();
        let _proactive = ProactiveEngine::default();
        let _decision = DecisionEngine::default();
        let _personality = PersonalityDynamics::default();
        let _moments = MomentEngine::new();
    }

    #[test]
    fn test_experience_layer_creation() {
        let config = IntelligenceConfig::default();
        let layer = ExperienceLayer::new(config);
        let snapshot = layer.get_snapshot();
        assert_eq!(snapshot.memory_count, 0);
    }
}
