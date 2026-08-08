use crate::config::CuriosityConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: Uuid,
    pub description: String,
    pub category: PatternCategory,
    pub occurrences: usize,
    pub confidence: f32,
    pub first_seen: chrono::DateTime<chrono::Utc>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PatternCategory {
    TimeUsage,
    AppUsage,
    Workflow,
    Behavior,
    Preference,
    Anomaly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub id: Uuid,
    pub pattern_id: Uuid,
    pub title: String,
    pub description: String,
    pub suggestion_type: SuggestionType,
    pub confidence: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Optimization,
    Automation,
    Learning,
    Health,
    Productivity,
}

pub struct CuriosityEngine {
    config: CuriosityConfig,
    patterns: Vec<Pattern>,
    suggestions: Vec<Suggestion>,
    observation_log: Vec<(String, chrono::DateTime<chrono::Utc>)>,
    last_suggestion_time: Option<chrono::DateTime<chrono::Utc>>,
}

const MAX_OBSERVATION_LOG: usize = 500;
const MAX_PATTERNS: usize = 200;
const MAX_SUGGESTIONS: usize = 100;

impl CuriosityEngine {
    pub fn new(config: CuriosityConfig) -> Self {
        Self {
            config,
            patterns: Vec::new(),
            suggestions: Vec::new(),
            observation_log: Vec::new(),
            last_suggestion_time: None,
        }
    }

    pub fn observe(&mut self, event: String) {
        self.observation_log.push((event, chrono::Utc::now()));
        if self.observation_log.len() > MAX_OBSERVATION_LOG {
            self.observation_log
                .drain(..self.observation_log.len() - MAX_OBSERVATION_LOG);
        }
    }

    pub fn detect_patterns(&mut self) -> Result<Vec<Pattern>> {
        let mut event_counts: HashMap<String, usize> = HashMap::new();
        let mut event_first: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();
        let mut event_last: HashMap<String, chrono::DateTime<chrono::Utc>> = HashMap::new();

        for (event, timestamp) in &self.observation_log {
            *event_counts.entry(event.clone()).or_insert(0) += 1;
            event_first.entry(event.clone()).or_insert(*timestamp);
            event_last.insert(event.clone(), *timestamp);
        }

        let mut new_patterns = Vec::new();

        for (event, count) in &event_counts {
            if *count >= self.config.min_pattern_occurrences {
                let already_exists = self.patterns.iter().any(|p| p.description == *event);
                if !already_exists {
                    let category = self.categorize_event(event);
                    let pattern = Pattern {
                        id: Uuid::new_v4(),
                        description: event.clone(),
                        category,
                        occurrences: *count,
                        confidence: (*count as f32
                            / (self.config.min_pattern_occurrences as f32 * 3.0))
                            .min(1.0),
                        first_seen: *event_first.get(event).unwrap_or(&chrono::Utc::now()),
                        last_seen: *event_last.get(event).unwrap_or(&chrono::Utc::now()),
                        data: HashMap::new(),
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

    fn categorize_event(&self, event: &str) -> PatternCategory {
        let lower = event.to_lowercase();
        if lower.contains("debug") || lower.contains("error") {
            PatternCategory::Anomaly
        } else if lower.contains("hour") || lower.contains("time") {
            PatternCategory::TimeUsage
        } else if lower.contains("open") || lower.contains("launch") {
            PatternCategory::AppUsage
        } else if lower.contains("workflow") || lower.contains("automate") {
            PatternCategory::Workflow
        } else {
            PatternCategory::Behavior
        }
    }

    pub fn generate_suggestions(&mut self) -> Result<Vec<Suggestion>> {
        let now = chrono::Utc::now();
        if let Some(last) = self.last_suggestion_time {
            if (now - last).num_milliseconds() < self.config.suggestion_cooldown_ms as i64 {
                return Ok(Vec::new());
            }
        }

        let mut new_suggestions = Vec::new();

        for pattern in &self.patterns {
            let already_suggested = self.suggestions.iter().any(|s| s.pattern_id == pattern.id);
            if already_suggested {
                continue;
            }

            if pattern.confidence < 0.5 {
                continue;
            }

            let (title, description, suggestion_type) = match pattern.category {
                PatternCategory::TimeUsage => (
                    "Time Usage Pattern".to_string(),
                    format!(
                        "You tend to {} frequently. Consider time-blocking.",
                        pattern.description
                    ),
                    SuggestionType::Productivity,
                ),
                PatternCategory::AppUsage => (
                    "App Usage Pattern".to_string(),
                    format!(
                        "I noticed you use {} often. Want me to optimize your setup?",
                        pattern.description
                    ),
                    SuggestionType::Optimization,
                ),
                PatternCategory::Workflow => (
                    "Workflow Pattern".to_string(),
                    format!(
                        "I see a recurring pattern: {}. Shall I automate this?",
                        pattern.description
                    ),
                    SuggestionType::Automation,
                ),
                PatternCategory::Anomaly => (
                    "Anomaly Detected".to_string(),
                    format!(
                        "I noticed something unusual: {}. Is everything okay?",
                        pattern.description
                    ),
                    SuggestionType::Health,
                ),
                _ => (
                    "Behavior Pattern".to_string(),
                    format!("I observed: {}. Want to discuss this?", pattern.description),
                    SuggestionType::Learning,
                ),
            };

            let suggestion = Suggestion {
                id: Uuid::new_v4(),
                pattern_id: pattern.id,
                title,
                description,
                suggestion_type,
                confidence: pattern.confidence,
                created_at: now,
            };

            self.suggestions.push(suggestion.clone());
            new_suggestions.push(suggestion);

            // Evict oldest suggestions if at capacity
            if self.suggestions.len() > MAX_SUGGESTIONS {
                self.suggestions
                    .drain(0..self.suggestions.len() - MAX_SUGGESTIONS);
            }
        }

        if !new_suggestions.is_empty() {
            self.last_suggestion_time = Some(now);
        }

        Ok(new_suggestions)
    }

    pub fn dismiss_suggestion(&mut self, suggestion_id: Uuid) -> bool {
        let len_before = self.suggestions.len();
        self.suggestions.retain(|s| s.id != suggestion_id);
        self.suggestions.len() < len_before
    }

    pub fn get_patterns(&self) -> &[Pattern] {
        &self.patterns
    }

    pub fn get_suggestions(&self) -> &[Suggestion] {
        &self.suggestions
    }

    pub fn observation_log(&self) -> &[(String, chrono::DateTime<chrono::Utc>)] {
        &self.observation_log
    }

    pub fn config(&self) -> &CuriosityConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curiosity_engine_creation() {
        let config = CuriosityConfig::default();
        let engine = CuriosityEngine::new(config);
        assert_eq!(engine.get_patterns().len(), 0);
        assert_eq!(engine.get_suggestions().len(), 0);
    }

    #[test]
    fn test_observe() {
        let config = CuriosityConfig::default();
        let mut engine = CuriosityEngine::new(config);
        engine.observe("debugging Rust".to_string());
        assert_eq!(engine.observation_log().len(), 1);
    }

    #[test]
    fn test_detect_patterns_below_threshold() {
        let config = CuriosityConfig::default();
        let mut engine = CuriosityEngine::new(config);
        for _ in 0..2 {
            engine.observe("debugging Rust".to_string());
        }
        let patterns = engine.detect_patterns().unwrap();
        assert!(patterns.is_empty());
    }

    #[test]
    fn test_detect_patterns_above_threshold() {
        let config = CuriosityConfig::default();
        let mut engine = CuriosityEngine::new(config);
        for _ in 0..5 {
            engine.observe("debugging Rust".to_string());
        }
        let patterns = engine.detect_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_generate_suggestions() {
        let config = CuriosityConfig::default();
        let mut engine = CuriosityEngine::new(config);
        for _ in 0..10 {
            engine.observe("debugging Rust".to_string());
        }
        engine.detect_patterns().unwrap();
        let suggestions = engine.generate_suggestions().unwrap();
        assert!(suggestions.len() <= 1);
    }

    #[test]
    fn test_categorize_event() {
        let config = CuriosityConfig::default();
        let engine = CuriosityEngine::new(config);
        assert_eq!(
            engine.categorize_event("debug error"),
            PatternCategory::Anomaly
        );
        assert_eq!(
            engine.categorize_event("spent 2 hours"),
            PatternCategory::TimeUsage
        );
        assert_eq!(
            engine.categorize_event("open VSCode"),
            PatternCategory::AppUsage
        );
    }

    #[test]
    fn test_dismiss_suggestion() {
        let config = CuriosityConfig::default();
        let mut engine = CuriosityEngine::new(config);
        for _ in 0..10 {
            engine.observe("test event".to_string());
        }
        engine.detect_patterns().unwrap();
        engine.generate_suggestions().unwrap();
        if !engine.get_suggestions().is_empty() {
            let id = engine.get_suggestions()[0].id;
            assert!(engine.dismiss_suggestion(id));
            assert_eq!(engine.get_suggestions().len(), 0);
        }
    }
}
