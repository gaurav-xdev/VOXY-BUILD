use crate::types::{MemoryId, MemoryItem, MemoryState, MemoryType};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct Preference {
    pub id: String,
    pub category: String,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub observations: u32,
    pub last_observed: chrono::DateTime<chrono::Utc>,
    pub is_stable: bool,
}

pub struct PreferenceTracker {
    state: Arc<RwLock<PreferenceTrackerState>>,
}

struct PreferenceTrackerState {
    preferences: HashMap<String, Preference>,
    observation_counts: HashMap<String, u32>,
}

impl PreferenceTracker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(PreferenceTrackerState {
                preferences: HashMap::new(),
                observation_counts: HashMap::new(),
            })),
        }
    }

    pub async fn track_app_usage(&self, app_id: &str, duration_secs: u64) {
        let mut state = self.state.write().await;
        let key = format!("app_usage:{}", app_id);

        let (observations, confidence) = {
            let pref = state
                .preferences
                .entry(key.clone())
                .or_insert_with(|| Preference {
                    id: format!("pref-{}", Uuid::new_v4()),
                    category: "app_usage".to_string(),
                    key: key.clone(),
                    value: app_id.to_string(),
                    confidence: 0.0,
                    observations: 0,
                    last_observed: Utc::now(),
                    is_stable: false,
                });

            pref.observations += 1;
            pref.last_observed = Utc::now();
            pref.confidence = (pref.observations as f64 / 100.0).min(1.0);
            pref.is_stable = pref.observations >= 10 && pref.confidence >= 0.5;

            (pref.observations, pref.confidence)
        };

        *state.observation_counts.entry(key).or_insert(0) += 1;

        debug!(
            app_id,
            duration_secs, observations, confidence, "Tracked app usage"
        );
    }

    pub async fn track_window_title(&self, app_id: &str, title: &str) {
        let mut state = self.state.write().await;
        let key = format!("window_title:{}", app_id);

        let pref = state
            .preferences
            .entry(key.clone())
            .or_insert_with(|| Preference {
                id: format!("pref-{}", Uuid::new_v4()),
                category: "window_title".to_string(),
                key: key.clone(),
                value: title.to_string(),
                confidence: 0.0,
                observations: 0,
                last_observed: Utc::now(),
                is_stable: false,
            });

        pref.observations += 1;
        pref.last_observed = Utc::now();
        pref.confidence = (pref.observations as f64 / 50.0).min(1.0);
        pref.is_stable = pref.observations >= 5 && pref.confidence >= 0.3;

        debug!(
            app_id,
            title,
            observations = pref.observations,
            "Tracked window title"
        );
    }

    pub async fn track_task_pattern(&self, task_type: &str, frequency: f64) {
        let mut state = self.state.write().await;
        let key = format!("task_pattern:{}", task_type);

        let pref = state
            .preferences
            .entry(key.clone())
            .or_insert_with(|| Preference {
                id: format!("pref-{}", Uuid::new_v4()),
                category: "task_pattern".to_string(),
                key: key.clone(),
                value: task_type.to_string(),
                confidence: 0.0,
                observations: 0,
                last_observed: Utc::now(),
                is_stable: false,
            });

        pref.observations += 1;
        pref.last_observed = Utc::now();
        pref.confidence = frequency;
        pref.is_stable = pref.observations >= 5 && pref.confidence >= 0.5;

        debug!(
            task_type,
            frequency,
            observations = pref.observations,
            "Tracked task pattern"
        );
    }

    pub async fn get_preferences(&self) -> Vec<Preference> {
        let state = self.state.read().await;
        state.preferences.values().cloned().collect()
    }

    pub async fn get_stable_preferences(&self) -> Vec<Preference> {
        let state = self.state.read().await;
        state
            .preferences
            .values()
            .filter(|p| p.is_stable)
            .cloned()
            .collect()
    }

    pub async fn get_preference(&self, key: &str) -> Option<Preference> {
        let state = self.state.read().await;
        state.preferences.get(key).cloned()
    }

    pub async fn update_preference_value(&self, key: &str, new_value: &str) -> bool {
        let mut state = self.state.write().await;
        if let Some(pref) = state.preferences.get_mut(key) {
            pref.value = new_value.to_string();
            pref.last_observed = Utc::now();
            debug!(key, new_value, "Updated preference value");
            true
        } else {
            warn!(key, "Preference not found for update");
            false
        }
    }

    pub async fn to_memory_items(&self) -> Vec<MemoryItem> {
        let state = self.state.read().await;
        state
            .preferences
            .values()
            .filter(|p| p.is_stable)
            .map(|p| MemoryItem {
                id: MemoryId(p.id.clone()),
                memory_type: MemoryType::Semantic,
                state: MemoryState::Active,
                content: serde_json::json!({
                    "category": p.category,
                    "key": p.key,
                    "value": p.value,
                    "confidence": p.confidence,
                    "observations": p.observations,
                    "last_observed": p.last_observed.to_rfc3339(),
                }),
                importance: p.confidence * 0.8,
                timestamp: p.last_observed,
                last_accessed: p.last_observed,
                access_count: p.observations as u64,
                context_tags: vec!["preference".to_string(), p.category.clone()],
                source: "preference_tracker".to_string(),
                version: 1,
                ttl: None,
                metadata: HashMap::new(),
                embedding: None,
                parent_id: None,
                related_ids: vec![],
            })
            .collect()
    }

    pub async fn stats(&self) -> PreferenceStats {
        let state = self.state.read().await;
        let total = state.preferences.len();
        let stable = state.preferences.values().filter(|p| p.is_stable).count();
        let categories: Vec<String> = state
            .preferences
            .values()
            .map(|p| p.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        PreferenceStats {
            total_preferences: total,
            stable_preferences: stable,
            categories,
        }
    }
}

impl Default for PreferenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct PreferenceStats {
    pub total_preferences: usize,
    pub stable_preferences: usize,
    pub categories: Vec<String>,
}

use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracker_creation() {
        let tracker = PreferenceTracker::new();
        let stats = tracker.stats().await;
        assert_eq!(stats.total_preferences, 0);
    }

    #[tokio::test]
    async fn test_track_app_usage() {
        let tracker = PreferenceTracker::new();
        tracker.track_app_usage("code.exe", 60).await;
        let prefs = tracker.get_preferences().await;
        assert_eq!(prefs.len(), 1);
        assert_eq!(prefs[0].value, "code.exe");
    }

    #[tokio::test]
    async fn test_stable_preference() {
        let tracker = PreferenceTracker::new();
        for _ in 0..55 {
            tracker.track_app_usage("code.exe", 60).await;
        }
        let stable = tracker.get_stable_preferences().await;
        assert!(!stable.is_empty());
    }

    #[tokio::test]
    async fn test_to_memory_items() {
        let tracker = PreferenceTracker::new();
        for _ in 0..55 {
            tracker.track_app_usage("code.exe", 60).await;
        }
        let items = tracker.to_memory_items().await;
        assert!(!items.is_empty());
        assert_eq!(items[0].memory_type, MemoryType::Semantic);
    }
}
