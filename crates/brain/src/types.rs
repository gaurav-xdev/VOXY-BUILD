use std::collections::HashMap;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use voxy_companion::types::UserPresence;

use crate::latency::LatencySnapshot;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnId(pub String);

impl TurnId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for TurnId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrainState {
    Idle,
    Processing,
    Interrupted,
    ShuttingDown,
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct BrainInput {
    pub session_id: SessionId,
    pub raw_text: String,
    pub user_presence: UserPresence,
    pub focus_level: f64,
    pub stress_level: f64,
    pub is_meeting: bool,
    pub time_since_last_interaction: Duration,
    pub session_duration: Duration,
    pub errors_this_session: usize,
    pub missions_completed: usize,
    pub missions_failed: usize,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainOutput {
    pub turn_id: TurnId,
    pub session_id: SessionId,
    pub response_text: Option<String>,
    pub cognitive_result: Option<CognitiveSummary>,
    pub companion: Option<CompanionSummary>,
    pub human_dynamics: Option<HdrSummary>,
    pub context_summary: Option<ContextSummary>,
    pub pipeline_duration_ms: u64,
    pub stage_latencies: LatencySnapshot,
    pub interrupted: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveSummary {
    pub intent_type: String,
    pub confidence: f64,
    pub success: bool,
    pub duration_ms: u64,
    pub plan_steps: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionSummary {
    pub display: Option<String>,
    pub silence: bool,
    pub presence_score: f64,
    pub greeting: bool,
    pub micro_interaction: bool,
    pub latency_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdrSummary {
    pub trust_score: f64,
    pub relationship_level: String,
    pub behavior_state: String,
    pub autonomy_level: f64,
    pub protection_allowed: bool,
    pub policy_violations: usize,
    pub latency_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub source_count: usize,
    pub confidence: f64,
    pub collection_time_ms: u64,
}

#[derive(Debug, Clone)]
pub enum BrainEvent {
    TurnStarted {
        turn_id: String,
        session_id: String,
    },
    ContextCollecting,
    ContextCollected {
        source_count: usize,
        duration_ms: u64,
    },
    CompanionUpdating,
    CompanionUpdated {
        display: Option<String>,
        silence: bool,
        duration_ms: u64,
    },
    HdrUpdating,
    HdrUpdated {
        trust_score: f64,
        protection_allowed: bool,
        duration_ms: u64,
    },
    CognitionProcessing,
    CognitionProcessed {
        intent_type: String,
        confidence: f64,
        duration_ms: u64,
    },
    TurnCompleted {
        turn_id: String,
        total_duration_ms: u64,
    },
    TurnInterrupted {
        turn_id: String,
        reason: String,
    },
    TurnFailed {
        turn_id: String,
        error: String,
    },
    HealthCheck {
        healthy: bool,
        details: HashMap<String, String>,
    },
    PipelineLatency {
        total_us: u64,
        breakdown: HashMap<String, u64>,
    },
}
