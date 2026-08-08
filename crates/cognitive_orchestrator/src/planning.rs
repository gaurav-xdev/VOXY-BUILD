use crate::config::PlanningConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub status: PlanStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: Uuid,
    pub description: String,
    pub action: String,
    pub dependencies: Vec<Uuid>,
    pub status: StepStatus,
    pub result: Option<String>,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Planning,
    Executing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub struct PlanningEngine {
    config: PlanningConfig,
    plans: Vec<Plan>,
    execution_log: Vec<(Uuid, Uuid, StepStatus, String)>,
}

const MAX_PLANS: usize = 100;
#[allow(dead_code)]
const MAX_EXECUTION_LOG: usize = 500;

impl PlanningEngine {
    pub fn new(config: PlanningConfig) -> Self {
        Self {
            config,
            plans: Vec::new(),
            execution_log: Vec::new(),
        }
    }

    pub fn create_plan(&mut self, goal: String, steps: Vec<(String, String)>) -> Result<Plan> {
        if steps.len() > self.config.max_total_steps {
            return Err(crate::error::CognitiveError::Planning(format!(
                "Too many steps: {} > {}",
                steps.len(),
                self.config.max_total_steps
            )));
        }

        let plan_steps: Vec<PlanStep> = steps
            .into_iter()
            .map(|(desc, action)| PlanStep {
                id: Uuid::new_v4(),
                description: desc,
                action,
                dependencies: Vec::new(),
                status: StepStatus::Pending,
                result: None,
                branch: None,
            })
            .collect();

        let plan = Plan {
            id: Uuid::new_v4(),
            goal,
            steps: plan_steps,
            status: PlanStatus::Planning,
            created_at: chrono::Utc::now(),
            completed_at: None,
        };

        self.plans.push(plan.clone());

        // Evict oldest plans if at capacity
        if self.plans.len() > MAX_PLANS {
            self.plans.drain(0..self.plans.len() - MAX_PLANS);
        }

        Ok(plan)
    }

    pub fn start_execution(&mut self, plan_id: Uuid) -> Result<bool> {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            if plan.status == PlanStatus::Planning {
                plan.status = PlanStatus::Executing;
                for step in &mut plan.steps {
                    if step.dependencies.is_empty() {
                        step.status = StepStatus::Ready;
                    }
                }
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn execute_step(&mut self, plan_id: Uuid, step_id: Uuid, result: String) -> Result<bool> {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                if step.status == StepStatus::Ready || step.status == StepStatus::Running {
                    step.status = StepStatus::Completed;
                    step.result = Some(result.clone());
                    self.execution_log
                        .push((plan_id, step_id, StepStatus::Completed, result));
                } else {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            let completed_ids: Vec<Uuid> = plan
                .steps
                .iter()
                .filter(|s| s.status == StepStatus::Completed)
                .map(|s| s.id)
                .collect();

            for step in &mut plan.steps {
                if step.status == StepStatus::Pending {
                    let deps_met = step
                        .dependencies
                        .iter()
                        .all(|dep_id| completed_ids.contains(dep_id));
                    if deps_met {
                        step.status = StepStatus::Ready;
                    }
                }
            }

            let all_done = plan
                .steps
                .iter()
                .all(|s| s.status == StepStatus::Completed || s.status == StepStatus::Skipped);
            if all_done {
                plan.status = PlanStatus::Completed;
                plan.completed_at = Some(chrono::Utc::now());
            }

            return Ok(true);
        }
        Ok(false)
    }

    pub fn fail_step(&mut self, plan_id: Uuid, step_id: Uuid, reason: String) -> Result<bool> {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            if let Some(step) = plan.steps.iter_mut().find(|s| s.id == step_id) {
                step.status = StepStatus::Failed;
                step.result = Some(reason.clone());
                plan.status = PlanStatus::Failed;
                self.execution_log
                    .push((plan_id, step_id, StepStatus::Failed, reason));
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn cancel_plan(&mut self, plan_id: Uuid) -> Result<bool> {
        if let Some(plan) = self.plans.iter_mut().find(|p| p.id == plan_id) {
            plan.status = PlanStatus::Cancelled;
            for step in &mut plan.steps {
                if step.status == StepStatus::Pending || step.status == StepStatus::Ready {
                    step.status = StepStatus::Skipped;
                }
            }
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_ready_steps(&self, plan_id: Uuid) -> Vec<&PlanStep> {
        if let Some(plan) = self.plans.iter().find(|p| p.id == plan_id) {
            plan.steps
                .iter()
                .filter(|s| s.status == StepStatus::Ready)
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_plans(&self) -> &[Plan] {
        &self.plans
    }

    pub fn get_plan(&self, id: Uuid) -> Option<&Plan> {
        self.plans.iter().find(|p| p.id == id)
    }

    pub fn execution_log(&self) -> &[(Uuid, Uuid, StepStatus, String)] {
        &self.execution_log
    }

    pub fn config(&self) -> &PlanningConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_steps() -> Vec<(String, String)> {
        vec![
            ("Step 1: Research".to_string(), "research topic".to_string()),
            ("Step 2: Write".to_string(), "write draft".to_string()),
            ("Step 3: Review".to_string(), "review draft".to_string()),
        ]
    }

    #[test]
    fn test_planning_engine_creation() {
        let config = PlanningConfig::default();
        let engine = PlanningEngine::new(config);
        assert_eq!(engine.get_plans().len(), 0);
    }

    #[test]
    fn test_create_plan() {
        let config = PlanningConfig::default();
        let mut engine = PlanningEngine::new(config);
        let plan = engine
            .create_plan("Learn Rust".to_string(), sample_steps())
            .unwrap();
        assert_eq!(plan.steps.len(), 3);
        assert_eq!(plan.status, PlanStatus::Planning);
    }

    #[test]
    fn test_start_execution() {
        let config = PlanningConfig::default();
        let mut engine = PlanningEngine::new(config);
        let plan = engine
            .create_plan("Learn Rust".to_string(), sample_steps())
            .unwrap();
        assert!(engine.start_execution(plan.id).unwrap());
        assert_eq!(
            engine.get_plan(plan.id).unwrap().status,
            PlanStatus::Executing
        );
    }

    #[test]
    fn test_execute_steps_sequentially() {
        let config = PlanningConfig::default();
        let mut engine = PlanningEngine::new(config);
        let plan = engine
            .create_plan("Learn Rust".to_string(), sample_steps())
            .unwrap();
        engine.start_execution(plan.id).unwrap();

        let ready = engine.get_ready_steps(plan.id);
        assert_eq!(ready.len(), 3);

        engine
            .execute_step(plan.id, ready[0].id, "done".to_string())
            .unwrap();
        let ready = engine.get_ready_steps(plan.id);
        assert_eq!(ready.len(), 2);

        engine
            .execute_step(plan.id, ready[0].id, "done".to_string())
            .unwrap();
        let ready = engine.get_ready_steps(plan.id);
        assert_eq!(ready.len(), 1);

        engine
            .execute_step(plan.id, ready[0].id, "done".to_string())
            .unwrap();
        assert_eq!(
            engine.get_plan(plan.id).unwrap().status,
            PlanStatus::Completed
        );
    }

    #[test]
    fn test_fail_step() {
        let config = PlanningConfig::default();
        let mut engine = PlanningEngine::new(config);
        let plan = engine
            .create_plan("Learn Rust".to_string(), sample_steps())
            .unwrap();
        engine.start_execution(plan.id).unwrap();
        let ready = engine.get_ready_steps(plan.id);
        engine
            .fail_step(plan.id, ready[0].id, "error".to_string())
            .unwrap();
        assert_eq!(engine.get_plan(plan.id).unwrap().status, PlanStatus::Failed);
    }

    #[test]
    fn test_cancel_plan() {
        let config = PlanningConfig::default();
        let mut engine = PlanningEngine::new(config);
        let plan = engine
            .create_plan("Learn Rust".to_string(), sample_steps())
            .unwrap();
        engine.start_execution(plan.id).unwrap();
        engine.cancel_plan(plan.id).unwrap();
        assert_eq!(
            engine.get_plan(plan.id).unwrap().status,
            PlanStatus::Cancelled
        );
    }

    #[test]
    fn test_max_steps_exceeded() {
        let config = PlanningConfig {
            max_total_steps: 2,
            ..Default::default()
        };
        let mut engine = PlanningEngine::new(config);
        let result = engine.create_plan("Big goal".to_string(), sample_steps());
        assert!(result.is_err());
    }
}
