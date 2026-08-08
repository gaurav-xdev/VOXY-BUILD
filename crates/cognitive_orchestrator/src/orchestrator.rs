use crate::config::{CognitiveConfig, OrchestratorConfig};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

const MAX_DECISION_HISTORY: usize = 1000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SubsystemKind {
    Memory,
    Skills,
    Automation,
    Personality,
    Planning,
    Reflection,
    Curiosity,
    GoalManager,
    Workflow,
    Knowledge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    AnswerNow { confidence: f32 },
    ThinkLonger { reason: String },
    RetrieveMemory { query: String },
    InvokeSkill { skill_id: String },
    ExecuteAutomation { automation_id: String },
    UsePersonality { style: String },
    Delegate { target: SubsystemKind, task: String },
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveTask {
    pub id: Uuid,
    pub input: String,
    pub context: CognitiveContext,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveContext {
    pub active_goals: Vec<String>,
    pub recent_memories: Vec<String>,
    pub current_mood: String,
    pub conversation_history: Vec<(String, String)>,
    pub active_workflows: Vec<String>,
    pub user_preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveDecision {
    pub task_id: Uuid,
    pub decision: DecisionType,
    pub confidence: f32,
    pub reasoning: String,
    pub subsystem_scores: HashMap<SubsystemKind, f32>,
}

pub struct CognitiveOrchestrator {
    config: CognitiveConfig,
    pending_tasks: Vec<CognitiveTask>,
    decision_history: Vec<CognitiveDecision>,
    subsystem_weights: HashMap<SubsystemKind, f32>,
}

impl CognitiveOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        let mut subsystem_weights = HashMap::new();
        subsystem_weights.insert(SubsystemKind::Memory, 0.8);
        subsystem_weights.insert(SubsystemKind::Skills, 0.7);
        subsystem_weights.insert(SubsystemKind::Automation, 0.6);
        subsystem_weights.insert(SubsystemKind::Personality, 0.5);
        subsystem_weights.insert(SubsystemKind::Planning, 0.9);
        subsystem_weights.insert(SubsystemKind::Reflection, 0.4);
        subsystem_weights.insert(SubsystemKind::Curiosity, 0.3);
        subsystem_weights.insert(SubsystemKind::GoalManager, 0.7);
        subsystem_weights.insert(SubsystemKind::Workflow, 0.6);
        subsystem_weights.insert(SubsystemKind::Knowledge, 0.8);

        Self {
            config: config.cognitive,
            pending_tasks: Vec::new(),
            decision_history: Vec::new(),
            subsystem_weights,
        }
    }

    pub fn create_task(&self, input: String, context: CognitiveContext) -> CognitiveTask {
        CognitiveTask {
            id: Uuid::new_v4(),
            input,
            context,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn decide(&mut self, task: &CognitiveTask) -> Result<CognitiveDecision> {
        let mut scores = HashMap::new();

        let input_lower = task.input.to_lowercase();

        let memory_score = if input_lower.contains("remember") || input_lower.contains("what was") {
            0.95
        } else if task.context.recent_memories.len() > 3 {
            0.7
        } else {
            0.3
        };
        scores.insert(SubsystemKind::Memory, memory_score);

        let skill_score = if input_lower.contains("automate") || input_lower.contains("workflow") {
            0.9
        } else {
            0.2
        };
        scores.insert(SubsystemKind::Skills, skill_score);

        let planning_score = if task.input.len() > 100 || input_lower.contains("step by step") {
            0.85
        } else {
            0.3
        };
        scores.insert(SubsystemKind::Planning, planning_score);

        let goal_score = if !task.context.active_goals.is_empty() {
            0.8
        } else {
            0.1
        };
        scores.insert(SubsystemKind::GoalManager, goal_score);

        let curiosity_score = if task.context.conversation_history.len() > 5 {
            0.6
        } else {
            0.2
        };
        scores.insert(SubsystemKind::Curiosity, curiosity_score);

        for kind in [
            SubsystemKind::Automation,
            SubsystemKind::Personality,
            SubsystemKind::Reflection,
            SubsystemKind::Workflow,
            SubsystemKind::Knowledge,
        ] {
            scores.insert(kind, 0.5);
        }

        let best = scores
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v));

        let (decision, confidence, reasoning) = if let Some((kind, score)) = best {
            if score > self.config.confidence_threshold {
                match kind {
                    SubsystemKind::Memory => (
                        DecisionType::RetrieveMemory {
                            query: task.input.clone(),
                        },
                        score,
                        "High memory relevance detected".to_string(),
                    ),
                    SubsystemKind::Skills => (
                        DecisionType::InvokeSkill {
                            skill_id: "auto".to_string(),
                        },
                        score,
                        "Skill invocation recommended".to_string(),
                    ),
                    SubsystemKind::Planning => (
                        DecisionType::ThinkLonger {
                            reason: "Complex task requires planning".to_string(),
                        },
                        score,
                        "Multi-step reasoning needed".to_string(),
                    ),
                    SubsystemKind::GoalManager => (
                        DecisionType::Delegate {
                            target: SubsystemKind::GoalManager,
                            task: task.input.clone(),
                        },
                        score,
                        "Active goals detected".to_string(),
                    ),
                    _ => (
                        DecisionType::AnswerNow { confidence: score },
                        score,
                        format!("Subsystem {:?} scored highest", kind),
                    ),
                }
            } else {
                (
                    DecisionType::ThinkLonger {
                        reason: "Confidence below threshold".to_string(),
                    },
                    score,
                    "Low confidence across all subsystems".to_string(),
                )
            }
        } else {
            (DecisionType::Skip, 0.0, "No subsystem scored".to_string())
        };

        let result = CognitiveDecision {
            task_id: task.id,
            decision,
            confidence,
            reasoning,
            subsystem_scores: scores,
        };

        self.decision_history.push(result.clone());
        if self.decision_history.len() > MAX_DECISION_HISTORY {
            self.decision_history
                .drain(..self.decision_history.len() - MAX_DECISION_HISTORY);
        }
        Ok(result)
    }

    pub fn get_decision(&self, task_id: Uuid) -> Option<&CognitiveDecision> {
        self.decision_history.iter().find(|d| d.task_id == task_id)
    }

    pub fn decision_history(&self) -> &[CognitiveDecision] {
        &self.decision_history
    }

    pub fn subsystem_weight(&self, kind: &SubsystemKind) -> f32 {
        self.subsystem_weights.get(kind).copied().unwrap_or(0.5)
    }

    pub fn set_subsystem_weight(&mut self, kind: SubsystemKind, weight: f32) {
        self.subsystem_weights.insert(kind, weight.clamp(0.0, 1.0));
    }

    pub fn pending_tasks(&self) -> &[CognitiveTask] {
        &self.pending_tasks
    }

    pub fn add_task(&mut self, task: CognitiveTask) {
        if self.pending_tasks.len() < self.config.max_concurrent_tasks {
            self.pending_tasks.push(task);
        }
    }

    pub fn complete_task(&mut self, task_id: Uuid) -> Option<CognitiveTask> {
        self.pending_tasks.retain(|t| t.id != task_id);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_context() -> CognitiveContext {
        CognitiveContext {
            active_goals: vec!["Learn Rust".to_string()],
            recent_memories: vec!["memory1".to_string()],
            current_mood: "neutral".to_string(),
            conversation_history: vec![],
            active_workflows: vec![],
            user_preferences: HashMap::new(),
        }
    }

    #[test]
    fn test_orchestrator_creation() {
        let config = OrchestratorConfig::default();
        let orch = CognitiveOrchestrator::new(config);
        assert_eq!(orch.pending_tasks().len(), 0);
        assert_eq!(orch.decision_history().len(), 0);
    }

    #[test]
    fn test_create_task() {
        let config = OrchestratorConfig::default();
        let orch = CognitiveOrchestrator::new(config);
        let task = orch.create_task("test input".to_string(), default_context());
        assert_eq!(task.input, "test input");
        assert!(!task.context.active_goals.is_empty());
    }

    #[test]
    fn test_decide_memory() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        let task = orch.create_task(
            "remember what we talked about".to_string(),
            default_context(),
        );
        let decision = orch.decide(&task).unwrap();
        assert!(decision.confidence > 0.5);
    }

    #[test]
    fn test_decide_planning() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        let task = orch.create_task("help me plan step by step how to build a website with many pages and complex navigation".to_string(), default_context());
        let decision = orch.decide(&task).unwrap();
        assert!(decision.confidence > 0.3);
    }

    #[test]
    fn test_decide_goals() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        let ctx = CognitiveContext {
            active_goals: vec!["goal1".to_string(), "goal2".to_string()],
            ..default_context()
        };
        let task = orch.create_task("how is my progress".to_string(), ctx);
        let decision = orch.decide(&task).unwrap();
        assert!(decision.confidence > 0.0);
    }

    #[test]
    fn test_subsystem_weights() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        assert_eq!(orch.subsystem_weight(&SubsystemKind::Memory), 0.8);
        orch.set_subsystem_weight(SubsystemKind::Memory, 0.5);
        assert_eq!(orch.subsystem_weight(&SubsystemKind::Memory), 0.5);
    }

    #[test]
    fn test_subsystem_weight_clamp() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        orch.set_subsystem_weight(SubsystemKind::Memory, 2.0);
        assert_eq!(orch.subsystem_weight(&SubsystemKind::Memory), 1.0);
        orch.set_subsystem_weight(SubsystemKind::Memory, -1.0);
        assert_eq!(orch.subsystem_weight(&SubsystemKind::Memory), 0.0);
    }

    #[test]
    fn test_add_and_complete_task() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        let task = orch.create_task("test".to_string(), default_context());
        let id = task.id;
        orch.add_task(task);
        assert_eq!(orch.pending_tasks().len(), 1);
        orch.complete_task(id);
        assert_eq!(orch.pending_tasks().len(), 0);
    }

    #[test]
    fn test_get_decision() {
        let config = OrchestratorConfig::default();
        let mut orch = CognitiveOrchestrator::new(config);
        let task = orch.create_task("test".to_string(), default_context());
        let id = task.id;
        orch.decide(&task).unwrap();
        assert!(orch.get_decision(id).is_some());
        assert!(orch.get_decision(Uuid::new_v4()).is_none());
    }
}
