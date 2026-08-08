use crate::config::ReflectionConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SECURITY: Validate that lesson content doesn't contain prompt injection patterns.
fn validate_lesson_content(content: &str) -> bool {
    let lower = content.to_lowercase();
    let dangerous_patterns = [
        "ignore previous",
        "ignore above",
        "disregard instructions",
        "you are now",
        "new instructions:",
        "system:",
        "<system>",
        "</system>",
        "act as",
        "pretend to be",
        "roleplay as",
        "admin:",
        "sudo",
        "exec(",
        "rm -rf",
        "format(",
    ];
    !dangerous_patterns.iter().any(|p| lower.contains(p))
}

/// SECURITY: Sanitize lesson text — strip injection attempts and truncate.
fn sanitize_lesson_text(text: &str) -> String {
    let sanitized = text
        .replace("<system>", "&lt;system&gt;")
        .replace("</system>", "&lt;/system&gt;")
        .replace("<user_message>", "&lt;user_message&gt;")
        .replace("</user_message>", "&lt;/user_message&gt;");

    const MAX_LESSON_LEN: usize = 2000;
    if sanitized.len() > MAX_LESSON_LEN {
        let mut truncated = sanitized[..MAX_LESSON_LEN].to_string();
        truncated.push_str("...[truncated]");
        truncated
    } else {
        sanitized
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRecord {
    pub id: Uuid,
    pub messages: Vec<(String, String)>,
    pub context: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionResult {
    pub conversation_id: Uuid,
    pub quality_score: f32,
    pub correctness_score: f32,
    pub completeness_score: f32,
    pub helpfulness_score: f32,
    pub lessons: Vec<Lesson>,
    pub suggestions: Vec<String>,
    pub missed_context: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: Uuid,
    pub category: LessonCategory,
    pub description: String,
    pub original_memory: String,
    pub improvement: String,
    pub confidence: f32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LessonCategory {
    ResponseQuality,
    ContextAwareness,
    ToneAdjustment,
    KnowledgeGap,
    UserPreference,
    WorkflowOptimization,
}

pub struct ReflectionEngine {
    config: ReflectionConfig,
    lessons: Vec<Lesson>,
    reflections: Vec<ReflectionResult>,
}

const MAX_LESSONS: usize = 500;
const MAX_REFLECTIONS: usize = 200;

impl ReflectionEngine {
    pub fn new(config: ReflectionConfig) -> Self {
        Self {
            config,
            lessons: Vec::new(),
            reflections: Vec::new(),
        }
    }

    pub fn analyze_conversation(&mut self, record: ConversationRecord) -> Result<ReflectionResult> {
        if record.messages.len() < self.config.min_conversation_length {
            return Ok(ReflectionResult {
                conversation_id: record.id,
                quality_score: 0.5,
                correctness_score: 0.5,
                completeness_score: 0.5,
                helpfulness_score: 0.5,
                lessons: Vec::new(),
                suggestions: Vec::new(),
                missed_context: Vec::new(),
            });
        }

        let msg_count = record.messages.len() as f32;
        let avg_response_len: f32 = record
            .messages
            .iter()
            .filter(|(role, _)| role == "assistant")
            .map(|(_, content)| content.len() as f32)
            .sum::<f32>()
            / msg_count.max(1.0);

        let quality = (avg_response_len / 200.0).min(1.0);
        let correctness = if record.messages.iter().any(|(r, _)| r == "user") {
            0.7
        } else {
            0.5
        };
        let completeness = (msg_count / 10.0).min(1.0);
        let helpfulness = (quality + correctness + completeness) / 3.0;

        let mut lessons = Vec::new();
        let mut suggestions = Vec::new();
        let mut missed_context = Vec::new();

        if avg_response_len < 50.0 {
            let description = sanitize_lesson_text("Responses were too brief");
            let improvement = sanitize_lesson_text("Provide more detailed explanations");
            if validate_lesson_content(&description) && validate_lesson_content(&improvement) {
                lessons.push(Lesson {
                    id: Uuid::new_v4(),
                    category: LessonCategory::ResponseQuality,
                    description,
                    original_memory: sanitize_lesson_text("Short response pattern detected"),
                    improvement,
                    confidence: 0.8,
                    created_at: chrono::Utc::now(),
                });
            }
            suggestions.push("Consider providing more detailed responses".to_string());
        }

        if record.messages.len() > 5 {
            let user_msgs: Vec<_> = record
                .messages
                .iter()
                .filter(|(r, _)| r == "user")
                .collect();
            if user_msgs.len() > 3 {
                missed_context
                    .push("User asked many questions - check if all were addressed".to_string());
            }
        }

        let result = ReflectionResult {
            conversation_id: record.id,
            quality_score: quality,
            correctness_score: correctness,
            completeness_score: completeness,
            helpfulness_score: helpfulness,
            lessons: lessons.clone(),
            suggestions,
            missed_context,
        };

        self.lessons.extend(lessons);
        self.reflections.push(result.clone());

        // Evict oldest entries to prevent unbounded growth
        if self.lessons.len() > MAX_LESSONS {
            self.lessons.drain(0..self.lessons.len() - MAX_LESSONS);
        }
        if self.reflections.len() > MAX_REFLECTIONS {
            self.reflections
                .drain(0..self.reflections.len() - MAX_REFLECTIONS);
        }

        Ok(result)
    }

    pub fn get_lessons(&self) -> &[Lesson] {
        &self.lessons
    }

    pub fn get_lessons_by_category(&self, category: &LessonCategory) -> Vec<&Lesson> {
        self.lessons
            .iter()
            .filter(|l| l.category == *category)
            .collect()
    }

    pub fn get_reflections(&self) -> &[ReflectionResult] {
        &self.reflections
    }

    pub fn average_quality(&self) -> f32 {
        if self.reflections.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.reflections.iter().map(|r| r.quality_score).sum();
        sum / self.reflections.len() as f32
    }

    pub fn config(&self) -> &ReflectionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(msg_count: usize) -> ConversationRecord {
        let mut messages = Vec::new();
        for i in 0..msg_count {
            if i % 2 == 0 {
                messages.push(("user".to_string(), format!("Question {}", i)));
            } else {
                messages.push(("assistant".to_string(), format!("This is a detailed response to question {} with enough content to be meaningful and helpful to the user.", i)));
            }
        }
        ConversationRecord {
            id: Uuid::new_v4(),
            messages,
            context: "test".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_reflection_engine_creation() {
        let config = ReflectionConfig::default();
        let engine = ReflectionEngine::new(config);
        assert_eq!(engine.get_lessons().len(), 0);
        assert_eq!(engine.get_reflections().len(), 0);
    }

    #[test]
    fn test_analyze_short_conversation() {
        let config = ReflectionConfig::default();
        let mut engine = ReflectionEngine::new(config);
        let record = make_record(1);
        let result = engine.analyze_conversation(record).unwrap();
        assert_eq!(result.quality_score, 0.5);
        assert!(result.lessons.is_empty());
    }

    #[test]
    fn test_analyze_long_conversation() {
        let config = ReflectionConfig::default();
        let mut engine = ReflectionEngine::new(config);
        let record = make_record(10);
        let result = engine.analyze_conversation(record).unwrap();
        assert!(result.quality_score > 0.0);
    }

    #[test]
    fn test_lessons_by_category() {
        let config = ReflectionConfig::default();
        let mut engine = ReflectionEngine::new(config);
        let record = make_record(2);
        engine.analyze_conversation(record).unwrap();
        let quality_lessons = engine.get_lessons_by_category(&LessonCategory::ResponseQuality);
        assert!(quality_lessons.len() <= 1);
    }

    #[test]
    fn test_average_quality() {
        let config = ReflectionConfig::default();
        let mut engine = ReflectionEngine::new(config);
        assert_eq!(engine.average_quality(), 0.0);
        engine.analyze_conversation(make_record(10)).unwrap();
        assert!(engine.average_quality() > 0.0);
    }
}
