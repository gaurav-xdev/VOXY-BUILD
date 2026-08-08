use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique identifier for a user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

/// Relationship level between VOXY and the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationshipLevel {
    /// First interaction, no history.
    Professional,
    /// Some interactions, building familiarity.
    Familiar,
    /// Established trust, reliable patterns.
    Trusted,
    /// Long-term companion, deep understanding.
    LongTermCompanion,
}

impl RelationshipLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.85 {
            Self::LongTermCompanion
        } else if score >= 0.65 {
            Self::Trusted
        } else if score >= 0.40 {
            Self::Familiar
        } else {
            Self::Professional
        }
    }

    pub fn trust_multiplier(&self) -> f64 {
        match self {
            Self::Professional => 0.5,
            Self::Familiar => 0.7,
            Self::Trusted => 0.85,
            Self::LongTermCompanion => 1.0,
        }
    }
}

/// Trust factor — a single data point about user trust.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvent {
    pub kind: TrustEventKind,
    pub impact: f64,
    pub timestamp: DateTime<Utc>,
    pub context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustEventKind {
    SuccessfulMission,
    Correction,
    FalseAlarm,
    PermissionGranted,
    PermissionDenied,
    ManualOverride,
    TaskCompleted,
    TaskFailed,
    UserReturned,
    UserAbsent,
}

/// Behavior state — what VOXY is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorState {
    Listening,
    Thinking,
    Working,
    Observing,
    Protecting,
    Teaching,
    Celebrating,
    Waiting,
    DeepFocus,
    MissionMode,
    Sleeping,
}

impl BehaviorState {
    pub fn can_transition_to(&self, target: &BehaviorState) -> bool {
        use BehaviorState::*;
        matches!(
            (self, target),
            (Listening, Thinking)
                | (Listening, Observing)
                | (Listening, Waiting)
                | (Thinking, Working)
                | (Thinking, Listening)
                | (Thinking, Protecting)
                | (Working, Celebrating)
                | (Working, Thinking)
                | (Working, Listening)
                | (Working, Protecting)
                | (Observing, Listening)
                | (Observing, Thinking)
                | (Observing, Protecting)
                | (Observing, Waiting)
                | (Protecting, Listening)
                | (Protecting, Working)
                | (Protecting, Thinking)
                | (Teaching, Listening)
                | (Teaching, Thinking)
                | (Teaching, Waiting)
                | (Celebrating, Listening)
                | (Celebrating, Waiting)
                | (Celebrating, Observing)
                | (Waiting, Listening)
                | (Waiting, Thinking)
                | (Waiting, Observing)
                | (Waiting, DeepFocus)
                | (Waiting, MissionMode)
                | (Waiting, Sleeping)
                | (DeepFocus, Listening)
                | (DeepFocus, Working)
                | (DeepFocus, Observing)
                | (MissionMode, Working)
                | (MissionMode, Thinking)
                | (MissionMode, Listening)
                | (MissionMode, Celebrating)
                | (Sleeping, Waiting)
                | (Sleeping, Observing)
        )
    }

    pub fn is_interruptible(&self) -> bool {
        matches!(
            self,
            Self::Waiting | Self::Observing | Self::Listening | Self::Celebrating
        )
    }

    pub fn requires_silence(&self) -> bool {
        matches!(self, Self::DeepFocus | Self::Sleeping)
    }
}

/// Protection policy for actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectionLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl ProtectionLevel {
    pub fn confirmation_required(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

/// Action that VOXY might take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub kind: ActionKind,
    pub description: String,
    pub protection_level: ProtectionLevel,
    pub reversible: bool,
    pub impact: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    Speak,
    Execute,
    Modify,
    Delete,
    Send,
    Navigate,
    Configure,
    Learn,
}

/// Interaction style parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionStyle {
    pub sentence_length: SentenceLength,
    pub formality: f64,
    pub pace: f64,
    pub initiative: f64,
    pub verbosity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SentenceLength {
    Terse,
    Short,
    Medium,
    Long,
    Detailed,
}

impl InteractionStyle {
    pub fn professional() -> Self {
        Self {
            sentence_length: SentenceLength::Short,
            formality: 0.8,
            pace: 0.5,
            initiative: 0.3,
            verbosity: 0.3,
        }
    }

    pub fn familiar() -> Self {
        Self {
            sentence_length: SentenceLength::Medium,
            formality: 0.5,
            pace: 0.6,
            initiative: 0.5,
            verbosity: 0.5,
        }
    }

    pub fn companion() -> Self {
        Self {
            sentence_length: SentenceLength::Medium,
            formality: 0.3,
            pace: 0.7,
            initiative: 0.6,
            verbosity: 0.6,
        }
    }
}

/// Recovery record for mistakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecord {
    pub error_description: String,
    pub recovery_action: String,
    pub acknowledged: bool,
    pub timestamp: DateTime<Utc>,
    pub resolution_time: Duration,
}

/// Confidence level for a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub enum ConfidenceLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl ConfidenceLevel {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            Self::VeryHigh
        } else if score >= 0.75 {
            Self::High
        } else if score >= 0.5 {
            Self::Medium
        } else if score >= 0.25 {
            Self::Low
        } else {
            Self::VeryLow
        }
    }

    pub fn should_explain(&self) -> bool {
        matches!(self, Self::VeryLow | Self::Low)
    }
}

/// Humor context for adaptive humor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumorContext {
    pub relationship_score: f64,
    pub context_appropriateness: f64,
    pub timing_score: f64,
    pub confidence: f64,
    pub recent_humor_count: usize,
}

/// Full input to one HDR update cycle.
#[derive(Debug, Clone)]
pub struct HdrInput {
    pub now: DateTime<Utc>,
    pub instant_now: Instant,
    pub user_id: UserId,
    pub user_present: bool,
    pub current_behavior: BehaviorState,
    pub activity_description: String,
    pub pending_action: Option<Action>,
    pub recent_trust_events: Vec<TrustEvent>,
    pub time_since_last_interaction: Duration,
    pub session_duration: Duration,
    pub errors_this_session: usize,
    pub corrections_this_session: usize,
    pub missions_completed: usize,
    pub missions_failed: usize,
    pub is_meeting: bool,
    pub focus_level: f64,
    pub stress_level: f64,
}

/// Full output of one HDR update cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdrOutput {
    pub behavior_state: BehaviorState,
    pub relationship_level: RelationshipLevel,
    pub trust_score: f64,
    pub autonomy_level: f64,
    pub confirmation_level: f64,
    pub initiative_level: f64,
    pub protection_decision: ProtectionDecision,
    pub initiative_decision: InitiativeDecision,
    pub confidence: ConfidenceOutput,
    pub humor_decision: HumorDecision,
    pub style: InteractionStyle,
    pub recovery: Option<RecoveryAction>,
    pub policy_violations: Vec<String>,
    pub update_latency_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionDecision {
    pub allowed: bool,
    pub reason: String,
    pub requires_confirmation: bool,
    pub alternative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitiativeDecision {
    pub may_speak: bool,
    pub reason: String,
    pub priority: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceOutput {
    pub score: f64,
    pub level: ConfidenceLevel,
    pub should_explain: bool,
    pub explanation_depth: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumorDecision {
    pub use_humor: bool,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAction {
    pub acknowledge: bool,
    pub correct: bool,
    pub description: String,
}
