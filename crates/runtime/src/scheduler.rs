use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::error::Result;
use crate::traits::{
    ScheduleSpec, ScheduledTask, TaskHandle, TaskPriority, TaskScheduler, TaskStatus,
};

#[derive(Clone)]
pub struct InMemoryTaskHandle {
    task_id: Uuid,
    name: String,
    status: TaskStatus,
    created_at: chrono::DateTime<chrono::Utc>,
    priority: TaskPriority,
}

impl InMemoryTaskHandle {
    pub fn new(name: String, priority: TaskPriority) -> Self {
        Self {
            task_id: Uuid::new_v4(),
            name,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            priority,
        }
    }
}

#[async_trait]
impl TaskHandle for InMemoryTaskHandle {
    fn task_id(&self) -> &Uuid {
        &self.task_id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn status(&self) -> TaskStatus {
        self.status.clone()
    }

    async fn cancel(&self) -> Result<()> {
        Ok(())
    }

    async fn wait_for_completion(&self) -> Result<TaskStatus> {
        Ok(self.status.clone())
    }

    fn progress(&self) -> Option<f64> {
        None
    }

    fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }

    fn priority(&self) -> TaskPriority {
        self.priority
    }
}

struct TaskEntry {
    handle: InMemoryTaskHandle,
    #[allow(dead_code)]
    spec: Option<ScheduleSpec>,
}

pub struct InMemoryScheduler {
    tasks: RwLock<HashMap<Uuid, TaskEntry>>,
    paused: RwLock<bool>,
}

impl Default for InMemoryScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryScheduler {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            paused: RwLock::new(false),
        }
    }
}

#[async_trait]
impl TaskScheduler for InMemoryScheduler {
    async fn submit(
        &self,
        task: Box<dyn ScheduledTask>,
        priority: TaskPriority,
    ) -> Result<Box<dyn TaskHandle>> {
        let handle = InMemoryTaskHandle::new(task.name().to_string(), priority);
        let task_id = handle.task_id;
        let entry = TaskEntry {
            handle: handle.clone(),
            spec: None,
        };
        self.tasks.write().insert(task_id, entry);
        Ok(Box::new(handle))
    }

    async fn schedule(
        &self,
        task: Box<dyn ScheduledTask>,
        spec: ScheduleSpec,
    ) -> Result<Box<dyn TaskHandle>> {
        let handle = InMemoryTaskHandle::new(task.name().to_string(), TaskPriority::Normal);
        let task_id = handle.task_id;
        let entry = TaskEntry {
            handle: handle.clone(),
            spec: Some(spec),
        };
        self.tasks.write().insert(task_id, entry);
        Ok(Box::new(handle))
    }

    async fn cancel(&self, task_id: &Uuid) -> Result<()> {
        let tasks = self.tasks.read();
        if tasks.contains_key(task_id) {
            Ok(())
        } else {
            Err(crate::error::RuntimeError::TaskNotFound(
                task_id.to_string(),
            ))
        }
    }

    async fn status(&self, task_id: &Uuid) -> Option<TaskStatus> {
        let tasks = self.tasks.read();
        tasks.get(task_id).map(|entry| entry.handle.status())
    }

    async fn list_tasks(&self) -> Result<Vec<Box<dyn TaskHandle>>> {
        let tasks = self.tasks.read();
        Ok(tasks
            .values()
            .map(|entry| Box::new(entry.handle.clone()) as Box<dyn TaskHandle>)
            .collect())
    }

    async fn pause(&self) -> Result<()> {
        *self.paused.write() = true;
        Ok(())
    }

    async fn resume(&self) -> Result<()> {
        *self.paused.write() = false;
        Ok(())
    }

    fn is_paused(&self) -> bool {
        *self.paused.read()
    }

    async fn active_task_count(&self) -> usize {
        let tasks = self.tasks.read();
        tasks
            .values()
            .filter(|entry| matches!(entry.handle.status(), TaskStatus::Running))
            .count()
    }

    async fn queued_task_count(&self) -> usize {
        let tasks = self.tasks.read();
        tasks
            .values()
            .filter(|entry| matches!(entry.handle.status(), TaskStatus::Pending))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct NoopTask {
        name: String,
    }

    impl NoopTask {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl ScheduledTask for NoopTask {
        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(&self, _handle: Box<dyn TaskHandle>) -> Result<()> {
            Ok(())
        }

        fn timeout_seconds(&self) -> Option<u64> {
            None
        }
    }

    #[tokio::test]
    async fn test_scheduler_submit_and_list() {
        let scheduler = InMemoryScheduler::new();
        let task = Box::new(NoopTask::new("test-task"));
        let handle = scheduler.submit(task, TaskPriority::Normal).await.unwrap();
        assert_eq!(handle.name(), "test-task");
        assert_eq!(handle.status(), TaskStatus::Pending);

        let tasks = scheduler.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_scheduler_status() {
        let scheduler = InMemoryScheduler::new();
        let task = Box::new(NoopTask::new("status-test"));
        let handle = scheduler.submit(task, TaskPriority::High).await.unwrap();
        let task_id = *handle.task_id();
        let status = scheduler.status(&task_id).await;
        assert!(status.is_some());
        assert_eq!(status.unwrap(), TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_scheduler_cancel() {
        let scheduler = InMemoryScheduler::new();
        let task = Box::new(NoopTask::new("cancel-test"));
        let handle = scheduler
            .submit(task, TaskPriority::Critical)
            .await
            .unwrap();
        let task_id = *handle.task_id();
        let result = scheduler.cancel(&task_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_scheduler_cancel_not_found() {
        let scheduler = InMemoryScheduler::new();
        let task_id = Uuid::new_v4();
        let result = scheduler.cancel(&task_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scheduler_pause_resume() {
        let scheduler = InMemoryScheduler::new();
        assert!(!scheduler.is_paused());
        scheduler.pause().await.unwrap();
        assert!(scheduler.is_paused());
        scheduler.resume().await.unwrap();
        assert!(!scheduler.is_paused());
    }

    #[tokio::test]
    async fn test_scheduler_schedule() {
        let scheduler = InMemoryScheduler::new();
        let task = Box::new(NoopTask::new("scheduled-task"));
        let handle = scheduler
            .schedule(task, ScheduleSpec::Delay { seconds: 10 })
            .await
            .unwrap();
        assert_eq!(handle.name(), "scheduled-task");
        assert_eq!(handle.status(), TaskStatus::Pending);
    }

    #[tokio::test]
    async fn test_scheduler_task_counts() {
        let scheduler = InMemoryScheduler::new();
        let task1 = Box::new(NoopTask::new("task1"));
        let task2 = Box::new(NoopTask::new("task2"));
        scheduler.submit(task1, TaskPriority::Low).await.unwrap();
        scheduler.submit(task2, TaskPriority::Normal).await.unwrap();
        assert_eq!(scheduler.active_task_count().await, 0);
        assert_eq!(scheduler.queued_task_count().await, 2);
    }
}
