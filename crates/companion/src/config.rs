use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::attention::ActivityKind;

/// Core configuration for the Companion Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionConfig {
    /// Update cycle target duration.
    pub update_interval: Duration,

    /// Presence system configuration.
    pub presence: PresenceConfig,

    /// Greeting system configuration.
    pub greeting: GreetingConfig,

    /// Silence intelligence configuration.
    pub silence: SilenceConfig,

    /// Mission companion configuration.
    pub mission: MissionConfig,

    /// Micro interaction configuration.
    pub micro: MicroConfig,

    /// Conversation timing configuration.
    pub conversation: ConversationConfig,

    /// Memory moments configuration.
    pub memory: MemoryConfig,

    /// Presence score weights.
    pub score_weights: ScoreWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceConfig {
    /// Breathing cycle duration (inhale + exhale).
    pub breathing_period: Duration,
    /// Blink interval range (min, max).
    pub blink_interval_min: Duration,
    pub blink_interval_max: Duration,
    /// Blink duration.
    pub blink_duration: Duration,
    /// Pulse intensity range (0.0 - 1.0).
    pub pulse_intensity_min: f64,
    pub pulse_intensity_max: f64,
    /// Idle movement speed (0.0 - 1.0).
    pub idle_movement_speed: f64,
    /// Look-around probability per cycle (0.0 - 1.0).
    pub look_around_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GreetingConfig {
    /// Minimum time between greetings to same user.
    pub min_greeting_interval: Duration,
    /// Maximum greetings per session.
    pub max_greetings_per_session: usize,
    /// Time window for greeting deduplication.
    pub dedup_window: Duration,
    /// Base greeting scores by context.
    pub context_scores: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SilenceConfig {
    /// Focus threshold below which we never interrupt.
    pub focus_interrupt_threshold: f64,
    /// Minimum silence duration before re-engaging.
    pub min_silence_duration: Duration,
    /// Maximum continuous silence before subtle check-in.
    pub max_silence_duration: Duration,
    /// Annoyance threshold (0.0 = no annoyance, 1.0 = max).
    pub annoyance_threshold: f64,
    /// Cooldown after interruption.
    pub interruption_cooldown: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionConfig {
    /// Activities eligible for mission mode.
    pub mission_activities: Vec<ActivityKind>,
    /// Maximum mission duration before check-in.
    pub max_mission_duration: Duration,
    /// Summary generation timeout.
    pub summary_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroConfig {
    /// Minimum time between micro interactions.
    pub min_interval: Duration,
    /// Maximum micro interactions per hour.
    pub max_per_hour: usize,
    /// Cooldown after each micro interaction.
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    /// Base thinking pause duration.
    pub thinking_pause: Duration,
    /// Pause multiplier for complex topics (0.0 - 2.0).
    pub complexity_multiplier: f64,
    /// Maximum pause before user might think disconnected.
    pub max_pause: Duration,
    /// Speaking rate (words per minute metadata).
    pub speaking_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Minimum time between memory references.
    pub min_referral_interval: Duration,
    /// Maximum memory references per session.
    pub max_references_per_session: usize,
    /// How often to check for memory moments.
    pub check_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreWeights {
    pub user_activity: f64,
    pub focus_level: f64,
    pub time_of_day: f64,
    pub conversation_frequency: f64,
    pub mission_state: f64,
    pub stress_estimate: f64,
    pub idle_time: f64,
}

impl Default for CompanionConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_millis(100),
            presence: PresenceConfig::default(),
            greeting: GreetingConfig::default(),
            silence: SilenceConfig::default(),
            mission: MissionConfig::default(),
            micro: MicroConfig::default(),
            conversation: ConversationConfig::default(),
            memory: MemoryConfig::default(),
            score_weights: ScoreWeights::default(),
        }
    }
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            breathing_period: Duration::from_secs(4),
            blink_interval_min: Duration::from_secs(2),
            blink_interval_max: Duration::from_secs(6),
            blink_duration: Duration::from_millis(200),
            pulse_intensity_min: 0.3,
            pulse_intensity_max: 0.8,
            idle_movement_speed: 0.3,
            look_around_probability: 0.05,
        }
    }
}

impl Default for GreetingConfig {
    fn default() -> Self {
        let mut context_scores = HashMap::new();
        context_scores.insert("morning".to_string(), 0.8);
        context_scores.insert("return".to_string(), 0.9);
        context_scores.insert("first_meeting".to_string(), 0.7);
        context_scores.insert("post_mission".to_string(), 0.6);
        Self {
            min_greeting_interval: Duration::from_secs(300),
            max_greetings_per_session: 3,
            dedup_window: Duration::from_secs(3600),
            context_scores,
        }
    }
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            focus_interrupt_threshold: 0.7,
            min_silence_duration: Duration::from_secs(30),
            max_silence_duration: Duration::from_secs(1800),
            annoyance_threshold: 0.6,
            interruption_cooldown: Duration::from_secs(60),
        }
    }
}

impl Default for MissionConfig {
    fn default() -> Self {
        Self {
            mission_activities: vec![
                ActivityKind::Coding,
                ActivityKind::Research,
                ActivityKind::Planning,
                ActivityKind::Reading,
            ],
            max_mission_duration: Duration::from_secs(3600),
            summary_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for MicroConfig {
    fn default() -> Self {
        Self {
            min_interval: Duration::from_secs(120),
            max_per_hour: 4,
            cooldown: Duration::from_secs(60),
        }
    }
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            thinking_pause: Duration::from_millis(500),
            complexity_multiplier: 1.5,
            max_pause: Duration::from_secs(5),
            speaking_rate: 150.0,
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            min_referral_interval: Duration::from_secs(600),
            max_references_per_session: 5,
            check_interval: Duration::from_secs(300),
        }
    }
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            user_activity: 0.25,
            focus_level: 0.20,
            time_of_day: 0.10,
            conversation_frequency: 0.15,
            mission_state: 0.15,
            stress_estimate: 0.05,
            idle_time: 0.10,
        }
    }
}
