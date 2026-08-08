use crate::config::WorkflowLearningConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub action: String,
    pub target: String,
    pub parameters: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub steps: Vec<WorkflowStep>,
    pub app: String,
    pub occurrence_count: usize,
    pub avg_duration_ms: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMatch {
    pub workflow_id: Uuid,
    pub confidence: f32,
    pub next_predicted_step: Option<String>,
    pub remaining_steps: usize,
}

pub struct WorkflowLearner {
    config: WorkflowLearningConfig,
    workflows: Vec<Workflow>,
    current_session: Vec<WorkflowStep>,
    current_app: String,
}

impl WorkflowLearner {
    pub fn new(config: WorkflowLearningConfig) -> Self {
        Self {
            config,
            workflows: Vec::new(),
            current_session: Vec::new(),
            current_app: String::new(),
        }
    }

    pub fn record_step(&mut self, step: WorkflowStep, app: String) {
        self.current_app = app;
        self.current_session.push(step);
    }

    pub fn end_session(&mut self) -> Result<Option<Workflow>> {
        if self.current_session.len() < 2 {
            self.current_session.clear();
            return Ok(None);
        }

        let steps = std::mem::take(&mut self.current_session);
        let app = self.current_app.clone();

        let total_duration: u64 = steps.iter().map(|s| s.duration_ms).sum();

        let existing_idx = self.workflows.iter().position(|w| {
            w.app == app && w.steps.len() == steps.len() && self.steps_similar(&w.steps, &steps)
        });

        if let Some(idx) = existing_idx {
            let workflow = &mut self.workflows[idx];
            let old_total = workflow.avg_duration_ms * workflow.occurrence_count as u64;
            workflow.occurrence_count += 1;
            workflow.avg_duration_ms =
                (old_total + total_duration) / workflow.occurrence_count as u64;
            workflow.last_used = chrono::Utc::now();
            Ok(None)
        } else if self.workflows.len() < self.config.max_workflows_stored {
            let workflow = Workflow {
                id: Uuid::new_v4(),
                name: format!("{} workflow #{}", app, self.workflows.len() + 1),
                steps,
                app,
                occurrence_count: 1,
                avg_duration_ms: total_duration,
                created_at: chrono::Utc::now(),
                last_used: chrono::Utc::now(),
            };
            let w = workflow.clone();
            self.workflows.push(workflow);
            Ok(Some(w))
        } else {
            Ok(None)
        }
    }

    fn steps_similar(&self, a: &[WorkflowStep], b: &[WorkflowStep]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let match_count = a
            .iter()
            .zip(b.iter())
            .filter(|(sa, sb)| sa.action == sb.action && sa.target == sb.target)
            .count();
        (match_count as f32 / a.len() as f32) >= self.config.pattern_similarity_threshold
    }

    pub fn match_session(&self, steps: &[WorkflowStep], app: &str) -> Option<WorkflowMatch> {
        let mut best_match = None;
        let mut best_score = 0.0f32;

        for workflow in &self.workflows {
            if workflow.app != app {
                continue;
            }

            let prefix_len = steps.len().min(workflow.steps.len());
            if prefix_len == 0 {
                continue;
            }

            let match_count = steps
                .iter()
                .zip(workflow.steps.iter())
                .take(prefix_len)
                .filter(|(s1, s2)| s1.action == s2.action && s1.target == s2.target)
                .count();

            let score = match_count as f32 / workflow.steps.len() as f32;

            if score > best_score && score >= self.config.pattern_similarity_threshold {
                best_score = score;
                let next_step = if prefix_len < workflow.steps.len() {
                    Some(workflow.steps[prefix_len].action.clone())
                } else {
                    None
                };
                best_match = Some(WorkflowMatch {
                    workflow_id: workflow.id,
                    confidence: score,
                    next_predicted_step: next_step,
                    remaining_steps: workflow.steps.len().saturating_sub(prefix_len),
                });
            }
        }

        best_match
    }

    pub fn get_workflows(&self) -> &[Workflow] {
        &self.workflows
    }

    pub fn get_workflow(&self, id: Uuid) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.id == id)
    }

    pub fn delete_workflow(&mut self, id: Uuid) -> bool {
        let len_before = self.workflows.len();
        self.workflows.retain(|w| w.id != id);
        self.workflows.len() < len_before
    }

    pub fn config(&self) -> &WorkflowLearningConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_step(action: &str, target: &str) -> WorkflowStep {
        WorkflowStep {
            id: Uuid::new_v4(),
            action: action.to_string(),
            target: target.to_string(),
            parameters: Vec::new(),
            duration_ms: 100,
        }
    }

    #[test]
    fn test_workflow_learner_creation() {
        let config = WorkflowLearningConfig::default();
        let learner = WorkflowLearner::new(config);
        assert_eq!(learner.get_workflows().len(), 0);
    }

    #[test]
    fn test_record_and_end_session() {
        let config = WorkflowLearningConfig::default();
        let mut learner = WorkflowLearner::new(config);
        learner.record_step(make_step("open", "file.rs"), "VSCode".to_string());
        learner.record_step(make_step("edit", "file.rs"), "VSCode".to_string());
        let result = learner.end_session().unwrap();
        assert!(result.is_some());
        assert_eq!(learner.get_workflows().len(), 1);
    }

    #[test]
    fn test_duplicate_session_increments() {
        let config = WorkflowLearningConfig::default();
        let mut learner = WorkflowLearner::new(config);

        for _ in 0..3 {
            learner.record_step(make_step("open", "file.rs"), "VSCode".to_string());
            learner.record_step(make_step("edit", "file.rs"), "VSCode".to_string());
            learner.end_session().unwrap();
        }

        assert_eq!(learner.get_workflows().len(), 1);
        assert_eq!(learner.get_workflows()[0].occurrence_count, 3);
    }

    #[test]
    fn test_match_session() {
        let config = WorkflowLearningConfig {
            pattern_similarity_threshold: 0.6,
            ..Default::default()
        };
        let mut learner = WorkflowLearner::new(config);

        for _ in 0..5 {
            learner.record_step(make_step("open", "file.rs"), "VSCode".to_string());
            learner.record_step(make_step("edit", "file.rs"), "VSCode".to_string());
            learner.record_step(make_step("run", "cargo test"), "VSCode".to_string());
            learner.end_session().unwrap();
        }

        let session = vec![make_step("open", "file.rs"), make_step("edit", "file.rs")];
        let m = learner.match_session(&session, "VSCode");
        assert!(m.is_some());
        let m = m.unwrap();
        assert!(m.confidence > 0.5);
        assert!(m.next_predicted_step.is_some());
    }

    #[test]
    fn test_delete_workflow() {
        let config = WorkflowLearningConfig::default();
        let mut learner = WorkflowLearner::new(config);
        learner.record_step(make_step("open", "file.rs"), "VSCode".to_string());
        learner.record_step(make_step("edit", "file.rs"), "VSCode".to_string());
        learner.end_session().unwrap();
        let id = learner.get_workflows()[0].id;
        assert!(learner.delete_workflow(id));
        assert_eq!(learner.get_workflows().len(), 0);
    }

    #[test]
    fn test_short_session_ignored() {
        let config = WorkflowLearningConfig::default();
        let mut learner = WorkflowLearner::new(config);
        learner.record_step(make_step("open", "file.rs"), "VSCode".to_string());
        let result = learner.end_session().unwrap();
        assert!(result.is_none());
    }
}
