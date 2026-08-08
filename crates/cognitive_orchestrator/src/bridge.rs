use crate::config::OrchestratorConfig;
use crate::curiosity::CuriosityEngine;
use crate::experience_replay::ExperienceReplay;
use crate::goal_manager::GoalManager;
use crate::knowledge_validation::KnowledgeValidator;
use crate::orchestrator::CognitiveOrchestrator;
use crate::planning::PlanningEngine;
use crate::reflection::ReflectionEngine;
use crate::skill_discovery::SkillDiscovery;
use crate::workflow_learning::WorkflowLearner;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum CognitiveEvent {
    DecisionMade(String),
    ReflectionComplete(String),
    PatternDetected(String),
    SkillSuggested(String),
    GoalUpdated(String),
    SuggestionReady(String),
    PlanCreated(String),
}

pub struct CognitiveBridge {
    pub orchestrator: CognitiveOrchestrator,
    pub reflection: ReflectionEngine,
    pub experience_replay: ExperienceReplay,
    pub knowledge_validator: KnowledgeValidator,
    pub skill_discovery: SkillDiscovery,
    pub workflow_learner: WorkflowLearner,
    pub goal_manager: GoalManager,
    pub curiosity: CuriosityEngine,
    pub planning: PlanningEngine,
    event_tx: broadcast::Sender<CognitiveEvent>,
    config: OrchestratorConfig,
}

impl CognitiveBridge {
    pub fn new(config: OrchestratorConfig) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        Self {
            orchestrator: CognitiveOrchestrator::new(config.clone()),
            reflection: ReflectionEngine::new(config.reflection.clone()),
            experience_replay: ExperienceReplay::new(config.experience_replay.clone()),
            knowledge_validator: KnowledgeValidator::new(config.knowledge_validation.clone()),
            skill_discovery: SkillDiscovery::new(config.skill_discovery.clone()),
            workflow_learner: WorkflowLearner::new(config.workflow_learning.clone()),
            goal_manager: GoalManager::new(config.goal_manager.clone()),
            curiosity: CuriosityEngine::new(config.curiosity.clone()),
            planning: PlanningEngine::new(config.planning.clone()),
            event_tx,
            config,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CognitiveEvent> {
        self.event_tx.subscribe()
    }

    pub fn emit(&self, event: CognitiveEvent) {
        let _ = self.event_tx.send(event);
    }

    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    pub fn shutdown(&self) {
        tracing::info!("Cognitive Bridge shutting down");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let config = OrchestratorConfig::default();
        let bridge = CognitiveBridge::new(config);
        assert_eq!(bridge.orchestrator.pending_tasks().len(), 0);
        assert_eq!(bridge.reflection.get_lessons().len(), 0);
        assert_eq!(bridge.experience_replay.buffer().len(), 0);
        assert_eq!(bridge.knowledge_validator.get_validated().len(), 0);
        assert_eq!(bridge.skill_discovery.get_patterns().len(), 0);
        assert_eq!(bridge.workflow_learner.get_workflows().len(), 0);
        assert_eq!(bridge.goal_manager.get_goals().len(), 0);
        assert_eq!(bridge.curiosity.get_patterns().len(), 0);
        assert_eq!(bridge.planning.get_plans().len(), 0);
    }

    #[test]
    fn test_bridge_subscribe() {
        let config = OrchestratorConfig::default();
        let bridge = CognitiveBridge::new(config);
        let mut rx = bridge.subscribe();
        bridge.emit(CognitiveEvent::DecisionMade("test".to_string()));
        assert!(rx.try_recv().is_ok());
    }
}
