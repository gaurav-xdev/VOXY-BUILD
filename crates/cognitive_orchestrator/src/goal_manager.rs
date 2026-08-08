use crate::config::GoalManagerConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: GoalStatus,
    pub progress: f32,
    pub milestones: Vec<Milestone>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    Active,
    Completed,
    Paused,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub goal_id: Uuid,
    pub old_progress: f32,
    pub new_progress: f32,
    pub milestone_completed: Option<String>,
    pub message: String,
}

pub struct GoalManager {
    config: GoalManagerConfig,
    goals: Vec<Goal>,
    progress_history: Vec<GoalProgress>,
}

const MAX_GOALS: usize = 100;
const MAX_PROGRESS_HISTORY: usize = 500;

impl GoalManager {
    pub fn new(config: GoalManagerConfig) -> Self {
        Self {
            config,
            goals: Vec::new(),
            progress_history: Vec::new(),
        }
    }

    pub fn add_goal(
        &mut self,
        title: String,
        description: String,
        deadline: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Goal> {
        let active_count = self
            .goals
            .iter()
            .filter(|g| g.status == GoalStatus::Active)
            .count();
        if active_count >= self.config.max_active_goals {
            return Err(crate::error::CognitiveError::GoalManager(
                "Maximum active goals reached".to_string(),
            ));
        }

        let goal = Goal {
            id: Uuid::new_v4(),
            title,
            description,
            status: GoalStatus::Active,
            progress: 0.0,
            milestones: Vec::new(),
            tags: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deadline,
        };

        self.goals.push(goal.clone());

        if self.goals.len() > MAX_GOALS {
            self.goals.drain(0..self.goals.len() - MAX_GOALS);
        }

        Ok(goal)
    }

    pub fn add_milestone(&mut self, goal_id: Uuid, title: String) -> Result<bool> {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.milestones.push(Milestone {
                id: Uuid::new_v4(),
                title,
                completed: false,
                completed_at: None,
            });
            goal.updated_at = chrono::Utc::now();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update_progress(
        &mut self,
        goal_id: Uuid,
        progress: f32,
    ) -> Result<Option<GoalProgress>> {
        let progress = progress.clamp(0.0, 1.0);

        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            let old_progress = goal.progress;
            goal.progress = progress;
            goal.updated_at = chrono::Utc::now();

            if progress >= 1.0 {
                goal.status = GoalStatus::Completed;
                for milestone in &mut goal.milestones {
                    if !milestone.completed {
                        milestone.completed = true;
                        milestone.completed_at = Some(chrono::Utc::now());
                    }
                }
            }

            let milestone_completed = if old_progress < 1.0 && progress >= 1.0 {
                Some("Goal completed!".to_string())
            } else {
                None
            };

            let result = GoalProgress {
                goal_id,
                old_progress,
                new_progress: progress,
                milestone_completed: milestone_completed.clone(),
                message: if progress >= 1.0 {
                    "Goal completed!".to_string()
                } else {
                    format!("Progress updated: {:.0}%", progress * 100.0)
                },
            };

            self.progress_history.push(result.clone());

            if self.progress_history.len() > MAX_PROGRESS_HISTORY {
                self.progress_history
                    .drain(0..self.progress_history.len() - MAX_PROGRESS_HISTORY);
            }

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    pub fn complete_milestone(&mut self, goal_id: Uuid, milestone_id: Uuid) -> Result<bool> {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            if let Some(milestone) = goal.milestones.iter_mut().find(|m| m.id == milestone_id) {
                milestone.completed = true;
                milestone.completed_at = Some(chrono::Utc::now());
                goal.updated_at = chrono::Utc::now();

                let completed_count = goal.milestones.iter().filter(|m| m.completed).count();
                let total = goal.milestones.len();
                if total > 0 {
                    goal.progress = completed_count as f32 / total as f32;
                }

                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn pause_goal(&mut self, goal_id: Uuid) -> Result<bool> {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            if goal.status == GoalStatus::Active {
                goal.status = GoalStatus::Paused;
                goal.updated_at = chrono::Utc::now();
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn resume_goal(&mut self, goal_id: Uuid) -> Result<bool> {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            if goal.status == GoalStatus::Paused {
                goal.status = GoalStatus::Active;
                goal.updated_at = chrono::Utc::now();
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn cancel_goal(&mut self, goal_id: Uuid) -> Result<bool> {
        if let Some(goal) = self.goals.iter_mut().find(|g| g.id == goal_id) {
            goal.status = GoalStatus::Cancelled;
            goal.updated_at = chrono::Utc::now();
            return Ok(true);
        }
        Ok(false)
    }

    pub fn get_goals(&self) -> &[Goal] {
        &self.goals
    }

    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.goals
            .iter()
            .filter(|g| g.status == GoalStatus::Active)
            .collect()
    }

    pub fn get_goal(&self, id: Uuid) -> Option<&Goal> {
        self.goals.iter().find(|g| g.id == id)
    }

    pub fn progress_history(&self) -> &[GoalProgress] {
        &self.progress_history
    }

    pub fn config(&self) -> &GoalManagerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_manager_creation() {
        let config = GoalManagerConfig::default();
        let manager = GoalManager::new(config);
        assert_eq!(manager.get_goals().len(), 0);
    }

    #[test]
    fn test_add_goal() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal(
                "Learn Rust".to_string(),
                "Master Rust programming".to_string(),
                None,
            )
            .unwrap();
        assert_eq!(goal.title, "Learn Rust");
        assert_eq!(goal.status, GoalStatus::Active);
        assert_eq!(goal.progress, 0.0);
    }

    #[test]
    fn test_max_goals_limit() {
        let config = GoalManagerConfig {
            max_active_goals: 2,
            ..Default::default()
        };
        let mut manager = GoalManager::new(config);
        manager
            .add_goal("Goal 1".to_string(), "".to_string(), None)
            .unwrap();
        manager
            .add_goal("Goal 2".to_string(), "".to_string(), None)
            .unwrap();
        let result = manager.add_goal("Goal 3".to_string(), "".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_progress() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal("Learn Rust".to_string(), "".to_string(), None)
            .unwrap();
        let result = manager.update_progress(goal.id, 0.5).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().new_progress, 0.5);
    }

    #[test]
    fn test_complete_goal() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal("Learn Rust".to_string(), "".to_string(), None)
            .unwrap();
        let result = manager.update_progress(goal.id, 1.0).unwrap();
        assert!(result.is_some());
        assert_eq!(
            manager.get_goal(goal.id).unwrap().status,
            GoalStatus::Completed
        );
    }

    #[test]
    fn test_milestones() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal("Learn Rust".to_string(), "".to_string(), None)
            .unwrap();
        manager
            .add_milestone(goal.id, "Complete book".to_string())
            .unwrap();
        manager
            .add_milestone(goal.id, "Build project".to_string())
            .unwrap();
        assert_eq!(manager.get_goal(goal.id).unwrap().milestones.len(), 2);
    }

    #[test]
    fn test_pause_resume() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal("Learn Rust".to_string(), "".to_string(), None)
            .unwrap();
        manager.pause_goal(goal.id).unwrap();
        assert_eq!(
            manager.get_goal(goal.id).unwrap().status,
            GoalStatus::Paused
        );
        manager.resume_goal(goal.id).unwrap();
        assert_eq!(
            manager.get_goal(goal.id).unwrap().status,
            GoalStatus::Active
        );
    }

    #[test]
    fn test_cancel_goal() {
        let config = GoalManagerConfig::default();
        let mut manager = GoalManager::new(config);
        let goal = manager
            .add_goal("Learn Rust".to_string(), "".to_string(), None)
            .unwrap();
        manager.cancel_goal(goal.id).unwrap();
        assert_eq!(
            manager.get_goal(goal.id).unwrap().status,
            GoalStatus::Cancelled
        );
    }
}
