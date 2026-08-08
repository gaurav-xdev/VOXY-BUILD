use crate::config::SkillDiscoveryConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPattern {
    pub id: Uuid,
    pub description: String,
    pub actions: Vec<String>,
    pub frequency: usize,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSuggestion {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub name: String,
    pub description: String,
    pub estimated_time_saved: u32,
    pub confidence: f32,
    pub workflow_steps: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct SkillDiscovery {
    config: SkillDiscoveryConfig,
    patterns: Vec<TaskPattern>,
    suggestions: Vec<SkillSuggestion>,
    action_history: Vec<(String, String, chrono::DateTime<chrono::Utc>)>,
}

const MAX_ACTION_HISTORY: usize = 500;
const MAX_PATTERNS: usize = 200;
const MAX_SUGGESTIONS: usize = 100;

impl SkillDiscovery {
    pub fn new(config: SkillDiscoveryConfig) -> Self {
        Self {
            config,
            patterns: Vec::new(),
            suggestions: Vec::new(),
            action_history: Vec::new(),
        }
    }

    pub fn record_action(&mut self, action: String, app: String) {
        self.action_history.push((action, app, chrono::Utc::now()));
        if self.action_history.len() > MAX_ACTION_HISTORY {
            self.action_history
                .drain(..self.action_history.len() - MAX_ACTION_HISTORY);
        }
    }

    pub fn detect_patterns(&mut self) -> Result<Vec<TaskPattern>> {
        let mut action_counts: HashMap<String, usize> = HashMap::new();
        let mut action_apps: HashMap<String, Vec<String>> = HashMap::new();
        let mut action_first: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
        let mut action_last: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();

        let cutoff = chrono::Utc::now()
            - chrono::Duration::hours(self.config.observation_window_hours as i64);

        for (action, app, timestamp) in &self.action_history {
            if *timestamp < cutoff {
                continue;
            }
            *action_counts.entry(action.clone()).or_insert(0) += 1;
            action_apps
                .entry(action.clone())
                .or_default()
                .push(app.clone());
            action_first.entry(action.clone()).or_insert(*timestamp);
            action_last.insert(action.clone(), *timestamp);
        }

        let mut new_patterns = Vec::new();

        for (action, count) in &action_counts {
            if *count >= self.config.min_repetition_count {
                let already_exists = self.patterns.iter().any(|p| p.description == *action);
                if !already_exists {
                    let apps: Vec<String> = action_apps
                        .get(action)
                        .map(|a| a.to_vec())
                        .unwrap_or_default();

                    let pattern = TaskPattern {
                        id: Uuid::new_v4(),
                        description: action.clone(),
                        actions: vec![action.clone()],
                        frequency: *count,
                        first_seen: *action_first.get(action).unwrap_or(&chrono::Utc::now()),
                        last_seen: *action_last.get(action).unwrap_or(&chrono::Utc::now()),
                        apps,
                    };

                    self.patterns.push(pattern.clone());
                    new_patterns.push(pattern);

                    // Evict oldest patterns if at capacity
                    if self.patterns.len() > MAX_PATTERNS {
                        self.patterns.drain(0..self.patterns.len() - MAX_PATTERNS);
                    }
                }
            }
        }

        Ok(new_patterns)
    }

    pub fn generate_suggestions(&mut self) -> Result<Vec<SkillSuggestion>> {
        let mut new_suggestions = Vec::new();

        for pattern in &self.patterns {
            let already_suggested = self.suggestions.iter().any(|s| s.pattern_id == pattern.id);
            if already_suggested {
                continue;
            }

            let confidence = (pattern.frequency as f32
                / (self.config.min_repetition_count as f32 * 2.0))
                .min(1.0);

            if confidence >= self.config.confidence_threshold {
                let suggestion = SkillSuggestion {
                    id: Uuid::new_v4(),
                    pattern_id: pattern.id,
                    name: format!("Automate: {}", pattern.description),
                    description: format!(
                        "I noticed you perform '{}' frequently. Would you like me to automate this?",
                        pattern.description
                    ),
                    estimated_time_saved: (pattern.frequency * 30).min(600) as u32,
                    confidence,
                    workflow_steps: pattern.actions.clone(),
                    created_at: chrono::Utc::now(),
                };

                self.suggestions.push(suggestion.clone());
                new_suggestions.push(suggestion);

                // Evict oldest suggestions if at capacity
                if self.suggestions.len() > MAX_SUGGESTIONS {
                    self.suggestions
                        .drain(0..self.suggestions.len() - MAX_SUGGESTIONS);
                }
            }
        }

        Ok(new_suggestions)
    }

    pub fn get_patterns(&self) -> &[TaskPattern] {
        &self.patterns
    }

    pub fn get_suggestions(&self) -> &[SkillSuggestion] {
        &self.suggestions
    }

    pub fn dismiss_suggestion(&mut self, suggestion_id: Uuid) -> bool {
        let len_before = self.suggestions.len();
        self.suggestions.retain(|s| s.id != suggestion_id);
        self.suggestions.len() < len_before
    }

    pub fn action_history(&self) -> &[(String, String, chrono::DateTime<chrono::Utc>)] {
        &self.action_history
    }

    pub fn config(&self) -> &SkillDiscoveryConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_discovery_creation() {
        let config = SkillDiscoveryConfig::default();
        let discovery = SkillDiscovery::new(config);
        assert_eq!(discovery.get_patterns().len(), 0);
        assert_eq!(discovery.get_suggestions().len(), 0);
    }

    #[test]
    fn test_record_action() {
        let config = SkillDiscoveryConfig::default();
        let mut discovery = SkillDiscovery::new(config);
        discovery.record_action("open file".to_string(), "VSCode".to_string());
        assert_eq!(discovery.action_history().len(), 1);
    }

    #[test]
    fn test_detect_patterns_below_threshold() {
        let config = SkillDiscoveryConfig::default();
        let mut discovery = SkillDiscovery::new(config);
        for _ in 0..5 {
            discovery.record_action("open file".to_string(), "VSCode".to_string());
        }
        let patterns = discovery.detect_patterns().unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_patterns_above_threshold() {
        let config = SkillDiscoveryConfig::default();
        let mut discovery = SkillDiscovery::new(config);
        for _ in 0..20 {
            discovery.record_action("open file".to_string(), "VSCode".to_string());
        }
        let patterns = discovery.detect_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].frequency, 20);
    }

    #[test]
    fn test_generate_suggestions() {
        let config = SkillDiscoveryConfig {
            min_repetition_count: 5,
            confidence_threshold: 0.5,
            ..Default::default()
        };
        let mut discovery = SkillDiscovery::new(config);
        for _ in 0..20 {
            discovery.record_action("open file".to_string(), "VSCode".to_string());
        }
        discovery.detect_patterns().unwrap();
        let suggestions = discovery.generate_suggestions().unwrap();
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].confidence > 0.0);
    }

    #[test]
    fn test_dismiss_suggestion() {
        let config = SkillDiscoveryConfig {
            min_repetition_count: 5,
            confidence_threshold: 0.5,
            ..Default::default()
        };
        let mut discovery = SkillDiscovery::new(config);
        for _ in 0..20 {
            discovery.record_action("open file".to_string(), "VSCode".to_string());
        }
        discovery.detect_patterns().unwrap();
        discovery.generate_suggestions().unwrap();
        let id = discovery.get_suggestions()[0].id;
        assert!(discovery.dismiss_suggestion(id));
        assert_eq!(discovery.get_suggestions().len(), 0);
    }

    #[test]
    fn test_no_duplicate_patterns() {
        let config = SkillDiscoveryConfig::default();
        let mut discovery = SkillDiscovery::new(config);
        for _ in 0..20 {
            discovery.record_action("open file".to_string(), "VSCode".to_string());
        }
        discovery.detect_patterns().unwrap();
        discovery.detect_patterns().unwrap();
        assert_eq!(discovery.get_patterns().len(), 1);
    }
}
