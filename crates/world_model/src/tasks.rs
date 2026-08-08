use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ActiveTask {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::InProgress => write!(f, "InProgress"),
            Self::Paused => write!(f, "Paused"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(msg) => write!(f, "Failed({})", msg),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_active_task_creation_and_progress_update() {
        let now = Utc::now();
        let mut task = ActiveTask {
            id: "task1".to_string(),
            description: "Test task".to_string(),
            status: TaskStatus::Pending,
            created_at: now,
            updated_at: now,
            progress: 0.0,
            metadata: [("key".to_string(), "value".to_string())].into(),
        };
        assert_eq!(task.id, "task1");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.progress, 0.0);

        task.status = TaskStatus::InProgress;
        task.progress = 50.0;
        assert_eq!(task.status, TaskStatus::InProgress);
        assert_eq!(task.progress, 50.0);
    }

    #[test]
    fn test_task_status_variants_and_display() {
        assert_eq!(format!("{}", TaskStatus::Pending), "Pending");
        assert_eq!(format!("{}", TaskStatus::InProgress), "InProgress");
        assert_eq!(format!("{}", TaskStatus::Paused), "Paused");
        assert_eq!(format!("{}", TaskStatus::Completed), "Completed");
        assert_eq!(
            format!("{}", TaskStatus::Failed("err".to_string())),
            "Failed(err)"
        );
        assert_eq!(format!("{}", TaskStatus::Cancelled), "Cancelled");
    }
}
