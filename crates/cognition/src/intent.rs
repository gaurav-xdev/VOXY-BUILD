use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::types::{ConfidenceScore, IntentId, Urgency};
use voxy_world_model::context::WorldContext;

#[derive(Debug, Clone)]
pub struct IntentInput {
    pub raw_text: String,
    pub context: Option<WorldContext>,
    pub source: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntentType {
    Query,
    Command,
    Creation,
    Modification,
    Deletion,
    Navigation,
    Communication,
    Entertainment,
    Productivity,
    Learning,
    Custom(String),
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query => write!(f, "Query"),
            Self::Command => write!(f, "Command"),
            Self::Creation => write!(f, "Creation"),
            Self::Modification => write!(f, "Modification"),
            Self::Deletion => write!(f, "Deletion"),
            Self::Navigation => write!(f, "Navigation"),
            Self::Communication => write!(f, "Communication"),
            Self::Entertainment => write!(f, "Entertainment"),
            Self::Productivity => write!(f, "Productivity"),
            Self::Learning => write!(f, "Learning"),
            Self::Custom(s) => write!(f, "Custom({})", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IntentAnalysis {
    pub intent_id: IntentId,
    pub intent_type: IntentType,
    pub confidence: ConfidenceScore,
    pub primary_action: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub requires_planning: bool,
    pub requires_reasoning: bool,
    pub urgency: Urgency,
    pub alternate_interpretations: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[async_trait]
pub trait IntentAnalyzer: Send + Sync {
    async fn analyze(&self, input: &IntentInput) -> Result<IntentAnalysis>;
    async fn analyze_streaming(&self, input: &IntentInput, partial: bool)
        -> Result<IntentAnalysis>;
}
