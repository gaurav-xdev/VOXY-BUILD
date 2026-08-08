use crate::error::Result;
use crate::graph::KnowledgeGraph;
use crate::types::ConsolidationPolicy;
use crate::types::HermesDecision;
use crate::types::MemoryId;
use crate::types::MemoryItem;
use chrono::{DateTime, Utc};
use voxy_world_model::context::WorldContext;

#[derive(Debug, Clone)]
pub struct ExperienceAnalysis {
    pub experience_id: String,
    pub summary: String,
    pub emotional_tone: String,
    pub key_elements: Vec<String>,
    pub importance: f64,
    pub novelty: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PreferenceExtraction {
    pub preference_id: String,
    pub category: String,
    pub value: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub is_stable: bool,
}

#[derive(Debug, Clone)]
pub struct PatternDetection {
    pub pattern_id: String,
    pub pattern_type: String,
    pub description: String,
    pub frequency: f64,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HabitDetection {
    pub habit_id: String,
    pub trigger: String,
    pub action: String,
    pub frequency: f64,
    pub time_pattern: String,
    pub strength: f64,
    pub is_automatic: bool,
}

#[derive(Debug, Clone)]
pub struct RelationshipModel {
    pub relationship_id: String,
    pub entity_a: String,
    pub entity_b: String,
    pub relationship_type: String,
    pub strength: f64,
    pub interaction_count: u64,
    pub last_interaction: DateTime<Utc>,
    pub sentiment: f64,
}

#[derive(Debug, Clone)]
pub struct BehaviorAnalysis {
    pub behavior_id: String,
    pub behavior_type: String,
    pub description: String,
    pub frequency: f64,
    pub context: Vec<String>,
    pub impact: f64,
    pub is_positive: bool,
}

#[derive(Debug, Clone)]
pub struct SkillExtraction {
    pub skill_id: String,
    pub skill_name: String,
    pub proficiency: f64,
    pub practice_count: u64,
    pub last_practiced: DateTime<Utc>,
    pub steps: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ReflectionLearning {
    pub reflection_id: String,
    pub task_id: String,
    pub goal: String,
    pub outcome: String,
    pub success_factors: Vec<String>,
    pub failure_factors: Vec<String>,
    pub lessons_learned: Vec<String>,
    pub pattern_updates: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct HermesClassification {
    pub item_id: MemoryId,
    pub decision: HermesDecision,
    pub confidence: f64,
    pub reasons: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait HermesEngine: Send + Sync {
    async fn analyze_experience(
        &self,
        item: &MemoryItem,
        context: Option<&WorldContext>,
    ) -> Result<ExperienceAnalysis>;
    async fn extract_preferences(
        &self,
        items: &[MemoryItem],
        limit: usize,
    ) -> Result<Vec<PreferenceExtraction>>;
    async fn detect_patterns(
        &self,
        items: &[MemoryItem],
        window_days: u64,
    ) -> Result<Vec<PatternDetection>>;
    async fn detect_habits(
        &self,
        items: &[MemoryItem],
        window_days: u64,
    ) -> Result<Vec<HabitDetection>>;
    async fn model_relationship(
        &self,
        entity_a: &str,
        entity_b: &str,
        interactions: &[MemoryItem],
    ) -> Result<RelationshipModel>;
    async fn analyze_behavior(
        &self,
        items: &[MemoryItem],
        window_days: u64,
    ) -> Result<Vec<BehaviorAnalysis>>;
    async fn extract_skills(&self, items: &[MemoryItem]) -> Result<Vec<SkillExtraction>>;
    async fn build_knowledge(
        &self,
        items: &[MemoryItem],
        graph: &dyn KnowledgeGraph,
    ) -> Result<usize>;
    async fn reflect_on_task(
        &self,
        goal: &str,
        outcome: &str,
        steps: &[String],
        success: bool,
    ) -> Result<ReflectionLearning>;
    async fn classify(
        &self,
        item: &MemoryItem,
        analysis: &ExperienceAnalysis,
    ) -> Result<HermesClassification>;
    async fn consolidate_long_term(
        &self,
        items: &[MemoryItem],
        policy: &ConsolidationPolicy,
    ) -> Result<Vec<HermesClassification>>;
    async fn evolve_memory(&self, item: &MemoryItem, feedback: &str) -> Result<MemoryItem>;
}
