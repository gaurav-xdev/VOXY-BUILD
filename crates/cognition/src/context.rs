use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::event::CognitionEvent;
use crate::intent::IntentAnalysis;
use crate::types::ContextId;
use voxy_personality::traits::PersonalityProfile;
use voxy_world_model::context::WorldSnapshot;

/// Local context source identifier (cognition-level).
#[derive(Debug, Clone)]
pub enum ContextSource {
    WorldModel,
    Personality,
    UserHistory,
    SystemState,
    ExternalService(String),
}

/// Cognition-level assembled context — wraps both local and fusion context.
#[derive(Debug, Clone)]
pub struct AssembledContext {
    pub id: ContextId,
    pub sources: Vec<ContextSource>,
    pub world_snapshot: Option<WorldSnapshot>,
    pub personality_context: Option<serde_json::Value>,
    pub relevant_history: Vec<String>,
    pub constraints: Vec<String>,
    pub priority_hints: Vec<String>,
    pub assembly_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    /// Reference to the fusion engine's assembled context data.
    pub fusion_data: Option<serde_json::Value>,
    /// Overall confidence from the fusion engine.
    pub fusion_confidence: f64,
    /// Number of sources that contributed.
    pub source_count: usize,
}

impl AssembledContext {
    /// Create from a voxy-context `AssembledContext`.
    pub fn from_fusion_context(
        fusion: &voxy_context::AssembledContext,
        intent: Option<&IntentAnalysis>,
    ) -> Self {
        let mut sources = Vec::new();
        for src in fusion.included_sources() {
            sources.push(ContextSource::ExternalService(src.to_string()));
        }

        let priority_hints = if let Some(intent) = intent {
            vec![format!("urgency: {:?}", intent.urgency)]
        } else {
            vec![]
        };

        Self {
            id: ContextId(fusion.id.clone()),
            sources,
            world_snapshot: None,
            personality_context: None,
            relevant_history: vec![],
            constraints: vec![],
            priority_hints,
            assembly_time_ms: 0,
            timestamp: fusion.assembled_at,
            fusion_data: Some(fusion.data.clone()),
            fusion_confidence: fusion.overall_confidence,
            source_count: fusion.source_count,
        }
    }

    /// Get a field from the fusion data.
    pub fn get_field(&self, path: &str) -> Option<&serde_json::Value> {
        self.fusion_data.as_ref().and_then(|d| {
            let parts: Vec<&str> = path.split('.').collect();
            let mut current = d;
            for part in parts {
                current = current.get(part)?;
            }
            Some(current)
        })
    }

    /// Check if context has expired data.
    pub fn is_stale(&self, max_age_secs: u64) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds() as u64;
        age > max_age_secs
    }
}

pub struct ContextAssemblyInput<'a> {
    pub intent: &'a IntentAnalysis,
    pub world_snapshot: &'a WorldSnapshot,
    pub personality: Option<Box<dyn PersonalityProfile>>,
    pub recent_events: Vec<CognitionEvent>,
}

#[async_trait]
pub trait ContextAssembler: Send + Sync {
    async fn assemble(&self, input: &ContextAssemblyInput<'_>) -> Result<AssembledContext>;
    async fn refresh(&self, context: &AssembledContext) -> Result<AssembledContext>;
    async fn merge(&self, contexts: &[AssembledContext]) -> Result<AssembledContext>;
}
