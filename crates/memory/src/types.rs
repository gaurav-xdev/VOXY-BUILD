use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemoryId(pub String);

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryType {
    Working,
    ShortTerm,
    Episodic,
    Semantic,
    Procedural,
    Vector,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Working => write!(f, "Working"),
            Self::ShortTerm => write!(f, "ShortTerm"),
            Self::Episodic => write!(f, "Episodic"),
            Self::Semantic => write!(f, "Semantic"),
            Self::Procedural => write!(f, "Procedural"),
            Self::Vector => write!(f, "Vector"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryState {
    Active,
    Dormant,
    Compressed,
    Archived,
}

impl fmt::Display for MemoryState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Dormant => write!(f, "Dormant"),
            Self::Compressed => write!(f, "Compressed"),
            Self::Archived => write!(f, "Archived"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryItem {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub state: MemoryState,
    pub content: serde_json::Value,
    pub importance: f64,
    pub timestamp: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub context_tags: Vec<String>,
    pub source: String,
    pub version: u64,
    pub ttl: Option<chrono::Duration>,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub parent_id: Option<MemoryId>,
    pub related_ids: Vec<MemoryId>,
}

#[derive(Debug, Clone)]
pub struct ImportanceFactors {
    pub recency: f64,
    pub frequency: f64,
    pub relevance: f64,
    pub emotional_salience: f64,
    pub goal_alignment: f64,
    pub user_priority: f64,
    pub novelty: f64,
}

impl Default for ImportanceFactors {
    fn default() -> Self {
        Self {
            recency: 0.0,
            frequency: 0.0,
            relevance: 0.0,
            emotional_salience: 0.0,
            goal_alignment: 0.0,
            user_priority: 0.0,
            novelty: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportanceScore {
    pub overall: f64,
    pub factors: ImportanceFactors,
    pub explanations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MemoryQuery {
    pub query_text: Option<String>,
    pub memory_types: Option<Vec<MemoryType>>,
    pub states: Option<Vec<MemoryState>>,
    pub tags: Option<Vec<String>>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub min_importance: Option<f64>,
    pub max_results: usize,
    pub include_embeddings: bool,
    pub source_filter: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ConsolidationPolicy {
    pub min_age_seconds: u64,
    pub min_access_count: u64,
    pub min_importance: f64,
    pub max_items_per_run: usize,
}

#[derive(Debug, Clone)]
pub struct CompressionPolicy {
    pub max_age_days: u64,
    pub min_importance_threshold: f64,
    pub compress_unaccessed_days: u64,
}

#[derive(Debug, Clone)]
pub struct ForgettingPolicy {
    pub active_to_dormant_days: u64,
    pub dormant_to_compressed_days: u64,
    pub compressed_to_archived_days: u64,
    pub archived_retention_days: u64,
    pub min_importance_to_keep: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HermesDecision {
    Ignore,
    TemporaryMemory,
    LongTermMemory,
    Preference,
    Habit,
    Knowledge,
    ProceduralSkill,
}

impl fmt::Display for HermesDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ignore => write!(f, "Ignore"),
            Self::TemporaryMemory => write!(f, "TemporaryMemory"),
            Self::LongTermMemory => write!(f, "LongTermMemory"),
            Self::Preference => write!(f, "Preference"),
            Self::Habit => write!(f, "Habit"),
            Self::Knowledge => write!(f, "Knowledge"),
            Self::ProceduralSkill => write!(f, "ProceduralSkill"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub changes: Vec<String>,
    pub previous_id: Option<MemoryId>,
}
