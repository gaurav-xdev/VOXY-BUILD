use chrono::{DateTime, Utc};
use serde_json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PreferenceEntry {
    pub id: String,
    pub category: String,
    pub key: String,
    pub value: serde_json::Value,
    pub confidence: f64,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub evidence_count: u64,
    pub is_stable: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub id: String,
    pub pattern_type: String,
    pub description: String,
    pub trigger_context: Vec<String>,
    pub frequency: f64,
    pub effectiveness: f64,
    pub last_observed: DateTime<Utc>,
    pub adaptation_count: u64,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ReinforcementSignal {
    pub id: String,
    pub action_id: String,
    pub action_description: String,
    pub reward: f64,
    pub context: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct FeedbackEntry {
    pub id: String,
    pub user_id: String,
    pub target_id: String,
    pub target_type: String,
    pub sentiment: f64,
    pub comment: Option<String>,
    pub context: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CalibrationSample {
    pub id: String,
    pub estimator_id: String,
    pub predicted_confidence: f64,
    pub actual_correctness: bool,
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StrategyEffectiveness {
    pub strategy_id: String,
    pub strategy_name: String,
    pub effectiveness_score: f64,
    pub trial_count: u64,
    pub success_count: u64,
    pub average_duration_ms: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AdaptiveThreshold {
    pub id: String,
    pub name: String,
    pub current_value: f64,
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub adjustment_rate: f64,
    pub last_adjusted: DateTime<Utc>,
    pub adjustment_count: u64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserPreferenceProfile {
    pub preferences: Vec<PreferenceEntry>,
    pub behavior_patterns: Vec<BehaviorPattern>,
    pub adaptive_thresholds: Vec<AdaptiveThreshold>,
    pub learning_rate: f64,
    pub adaptation_level: f64,
    pub last_updated: DateTime<Utc>,
}
