use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub cognitive: CognitiveConfig,
    pub reflection: ReflectionConfig,
    pub experience_replay: ExperienceReplayConfig,
    pub knowledge_validation: KnowledgeValidationConfig,
    pub skill_discovery: SkillDiscoveryConfig,
    pub workflow_learning: WorkflowLearningConfig,
    pub goal_manager: GoalManagerConfig,
    pub curiosity: CuriosityConfig,
    pub planning: PlanningConfig,
    pub integration: IntegrationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveConfig {
    pub max_concurrent_tasks: usize,
    pub decision_timeout_ms: u64,
    pub memory_retrieval_limit: usize,
    pub context_window_size: usize,
    pub confidence_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionConfig {
    pub enabled: bool,
    pub min_conversation_length: usize,
    pub analysis_depth: usize,
    pub lesson_retention_days: u32,
    pub max_lessons_per_day: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceReplayConfig {
    pub enabled: bool,
    pub buffer_size: usize,
    pub min_replay_score: f32,
    pub replay_interval_ms: u64,
    pub learning_rate: f32,
    pub discount_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeValidationConfig {
    pub enabled: bool,
    pub min_trust_score: f32,
    pub required_cross_references: usize,
    pub max_risk_level: RiskLevel,
    pub hallucination_threshold: f32,
    pub quarantine_retention_days: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDiscoveryConfig {
    pub enabled: bool,
    pub min_repetition_count: usize,
    pub observation_window_hours: u32,
    pub confidence_threshold: f32,
    pub max_suggestions_per_day: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLearningConfig {
    pub enabled: bool,
    pub max_steps_per_workflow: usize,
    pub min_occurrence_count: usize,
    pub pattern_similarity_threshold: f32,
    pub max_workflows_stored: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalManagerConfig {
    pub enabled: bool,
    pub max_active_goals: usize,
    pub check_interval_ms: u64,
    pub progress_update_threshold: f32,
    pub milestone_reminder_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityConfig {
    pub enabled: bool,
    pub pattern_detection_interval_ms: u64,
    pub min_pattern_occurrences: usize,
    pub suggestion_cooldown_ms: u64,
    pub max_suggestions_per_hour: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningConfig {
    pub enabled: bool,
    pub max_depth: usize,
    pub max_parallel_branches: usize,
    pub timeout_per_step_ms: u64,
    pub max_total_steps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub experience_layer: bool,
    pub memory: bool,
    pub brain: bool,
    pub personality: bool,
    pub automation: bool,
    pub event_bus: bool,
    pub world_model: bool,
    pub visual_presence: bool,
}

impl Default for CognitiveConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 8,
            decision_timeout_ms: 5000,
            memory_retrieval_limit: 10,
            context_window_size: 4096,
            confidence_threshold: 0.7,
        }
    }
}

impl Default for ReflectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_conversation_length: 3,
            analysis_depth: 5,
            lesson_retention_days: 365,
            max_lessons_per_day: 50,
        }
    }
}

impl Default for ExperienceReplayConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size: 10000,
            min_replay_score: 0.3,
            replay_interval_ms: 60000,
            learning_rate: 0.01,
            discount_factor: 0.95,
        }
    }
}

impl Default for KnowledgeValidationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_trust_score: 0.6,
            required_cross_references: 2,
            max_risk_level: RiskLevel::Medium,
            hallucination_threshold: 0.4,
            quarantine_retention_days: 30,
        }
    }
}

impl Default for SkillDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_repetition_count: 15,
            observation_window_hours: 168,
            confidence_threshold: 0.8,
            max_suggestions_per_day: 5,
        }
    }
}

impl Default for WorkflowLearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_steps_per_workflow: 50,
            min_occurrence_count: 3,
            pattern_similarity_threshold: 0.85,
            max_workflows_stored: 100,
        }
    }
}

impl Default for GoalManagerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_active_goals: 20,
            check_interval_ms: 300000,
            progress_update_threshold: 0.1,
            milestone_reminder_hours: 24,
        }
    }
}

impl Default for CuriosityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pattern_detection_interval_ms: 300000,
            min_pattern_occurrences: 3,
            suggestion_cooldown_ms: 3600000,
            max_suggestions_per_hour: 3,
        }
    }
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_depth: 10,
            max_parallel_branches: 4,
            timeout_per_step_ms: 10000,
            max_total_steps: 100,
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            experience_layer: true,
            memory: true,
            brain: true,
            personality: true,
            automation: true,
            event_bus: true,
            world_model: true,
            visual_presence: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.cognitive.max_concurrent_tasks, 8);
        assert!(config.reflection.enabled);
        assert!(config.knowledge_validation.enabled);
        assert_eq!(config.skill_discovery.min_repetition_count, 15);
        assert_eq!(config.planning.max_depth, 10);
    }

    #[test]
    fn test_config_serialization() {
        let config = OrchestratorConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OrchestratorConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.cognitive.max_concurrent_tasks,
            deserialized.cognitive.max_concurrent_tasks
        );
    }

    #[test]
    fn test_risk_level() {
        assert_eq!(RiskLevel::Low, RiskLevel::Low);
        assert_ne!(RiskLevel::Low, RiskLevel::Critical);
    }
}
