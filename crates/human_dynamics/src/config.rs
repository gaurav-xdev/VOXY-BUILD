use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::types::{BehaviorState, ProtectionLevel, RelationshipLevel};

/// Core configuration for the Human Dynamics Runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdrConfig {
    pub update_interval: Duration,
    pub relationship: RelationshipConfig,
    pub trust: TrustConfig,
    pub behavior: BehaviorConfig,
    pub protection: ProtectionConfig,
    pub initiative: InitiativeConfig,
    pub humor: HumorConfig,
    pub confidence: ConfidenceConfig,
    pub policy: PolicyConfig,
    pub style: StyleConfig,
    pub recovery: RecoveryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipConfig {
    pub decay_rate: f64,
    pub growth_rate: f64,
    pub min_events_for_upgrade: usize,
    pub max_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustConfig {
    pub initial_score: f64,
    pub decay_per_absence: f64,
    pub growth_per_success: f64,
    pub penalty_per_failure: f64,
    pub penalty_per_correction: f64,
    pub penalty_per_false_alarm: f64,
    pub bonus_per_permission: f64,
    pub max_score: f64,
    pub min_score: f64,
    pub event_history_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub default_state: BehaviorState,
    pub transition_cooldown: Duration,
    pub max_time_in_state: Duration,
    pub deep_focus_threshold: f64,
    pub sleeping_after: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionConfig {
    pub default_level: ProtectionLevel,
    pub auto_protect_delete: bool,
    pub auto_protect_send: bool,
    pub auto_protect_modify: bool,
    pub max_reversible_impact: f64,
    pub confirmation_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeConfig {
    pub enabled: bool,
    pub min_trust_for_initiative: f64,
    pub min_relationship_for_initiative: RelationshipLevel,
    pub cooldown: Duration,
    pub max_per_hour: usize,
    pub deep_focus_respect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumorConfig {
    pub enabled: bool,
    pub max_per_hour: usize,
    pub min_relationship: RelationshipLevel,
    pub min_confidence: f64,
    pub cooldown: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceConfig {
    pub high_threshold: f64,
    pub explain_below: f64,
    pub max_explanation_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub never_interrupt_meetings: bool,
    pub never_joke_on_failure: bool,
    pub never_celebrate_partial: bool,
    pub protect_before_obey: bool,
    pub always_explain_refusal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub adapt_to_relationship: bool,
    pub min_formality: f64,
    pub max_formality: f64,
    pub min_verbosity: f64,
    pub max_verbosity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub max_recovery_attempts: usize,
    pub acknowledgment_required: bool,
    pub auto_correct: bool,
    pub cooldown: Duration,
}

impl Default for HdrConfig {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_millis(50),
            relationship: RelationshipConfig::default(),
            trust: TrustConfig::default(),
            behavior: BehaviorConfig::default(),
            protection: ProtectionConfig::default(),
            initiative: InitiativeConfig::default(),
            humor: HumorConfig::default(),
            confidence: ConfidenceConfig::default(),
            policy: PolicyConfig::default(),
            style: StyleConfig::default(),
            recovery: RecoveryConfig::default(),
        }
    }
}

impl Default for RelationshipConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.01,
            growth_rate: 0.05,
            min_events_for_upgrade: 20,
            max_score: 1.0,
        }
    }
}

impl Default for TrustConfig {
    fn default() -> Self {
        Self {
            initial_score: 0.5,
            decay_per_absence: 0.02,
            growth_per_success: 0.05,
            penalty_per_failure: 0.1,
            penalty_per_correction: 0.08,
            penalty_per_false_alarm: 0.12,
            bonus_per_permission: 0.03,
            max_score: 1.0,
            min_score: 0.0,
            event_history_limit: 100,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            default_state: BehaviorState::Observing,
            transition_cooldown: Duration::from_secs(2),
            max_time_in_state: Duration::from_secs(3600),
            deep_focus_threshold: 0.8,
            sleeping_after: Duration::from_secs(1800),
        }
    }
}

impl Default for ProtectionConfig {
    fn default() -> Self {
        Self {
            default_level: ProtectionLevel::Medium,
            auto_protect_delete: true,
            auto_protect_send: false,
            auto_protect_modify: false,
            max_reversible_impact: 0.5,
            confirmation_timeout: Duration::from_secs(30),
        }
    }
}

impl Default for InitiativeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_trust_for_initiative: 0.6,
            min_relationship_for_initiative: RelationshipLevel::Familiar,
            cooldown: Duration::from_secs(120),
            max_per_hour: 4,
            deep_focus_respect: true,
        }
    }
}

impl Default for HumorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_per_hour: 2,
            min_relationship: RelationshipLevel::Familiar,
            min_confidence: 0.7,
            cooldown: Duration::from_secs(300),
        }
    }
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            high_threshold: 0.75,
            explain_below: 0.5,
            max_explanation_depth: 0.8,
        }
    }
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            never_interrupt_meetings: true,
            never_joke_on_failure: true,
            never_celebrate_partial: true,
            protect_before_obey: true,
            always_explain_refusal: true,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            adapt_to_relationship: true,
            min_formality: 0.2,
            max_formality: 0.9,
            min_verbosity: 0.2,
            max_verbosity: 0.8,
        }
    }
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_recovery_attempts: 3,
            acknowledgment_required: true,
            auto_correct: true,
            cooldown: Duration::from_secs(5),
        }
    }
}
