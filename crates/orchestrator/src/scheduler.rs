use std::collections::{BTreeMap, HashMap, VecDeque};

use async_trait::async_trait;
use parking_lot::RwLock;

use crate::error::{OrchestratorError, Result};
use crate::types::{OrchestratorTask, TaskId, TaskPriority};

#[async_trait]
pub trait TaskSchedulerInternal: Send + Sync {
    async fn enqueue(&self, task: OrchestratorTask) -> Result<TaskId>;
    async fn dequeue(&self, max_priority: Option<TaskPriority>)
        -> Result<Option<OrchestratorTask>>;
    async fn peek(&self) -> Result<Option<OrchestratorTask>>;
    async fn remove(&self, task_id: &TaskId) -> Result<()>;
    async fn reorder(&self, task_id: &TaskId, priority: TaskPriority) -> Result<()>;
    async fn queue_length(&self) -> usize;
    async fn is_empty(&self) -> bool;
    async fn get_queue(&self, min_priority: Option<TaskPriority>) -> Result<Vec<OrchestratorTask>>;
    async fn clear(&self) -> Result<()>;
}

pub struct PriorityTaskScheduler {
    queues: RwLock<BTreeMap<u8, VecDeque<OrchestratorTask>>>,
    by_id: RwLock<HashMap<String, TaskId>>,
}

impl PriorityTaskScheduler {
    pub fn new() -> Self {
        Self {
            queues: RwLock::new(BTreeMap::new()),
            by_id: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for PriorityTaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskSchedulerInternal for PriorityTaskScheduler {
    async fn enqueue(&self, task: OrchestratorTask) -> Result<TaskId> {
        let priority_value = task.priority.value();
        let task_id = task.id.clone();
        self.by_id
            .write()
            .insert(task_id.0.clone(), task_id.clone());
        self.queues
            .write()
            .entry(priority_value)
            .or_default()
            .push_back(task);
        Ok(task_id)
    }

    async fn dequeue(
        &self,
        max_priority: Option<TaskPriority>,
    ) -> Result<Option<OrchestratorTask>> {
        let max_val = max_priority.map_or(u8::MAX, |p| p.value());
        let mut queues = self.queues.write();
        let mut highest_key: Option<u8> = None;

        for (&key, queue) in queues.iter() {
            if key <= max_val && !queue.is_empty() {
                match highest_key {
                    None => highest_key = Some(key),
                    Some(hk) if key > hk => highest_key = Some(key),
                    _ => {}
                }
            }
        }

        match highest_key {
            Some(key) => {
                if let Some(queue) = queues.get_mut(&key) {
                    let task = queue.pop_front().unwrap();
                    if queue.is_empty() {
                        queues.remove(&key);
                    }
                    self.by_id.write().remove(&task.id.0);
                    Ok(Some(task))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    async fn peek(&self) -> Result<Option<OrchestratorTask>> {
        let queues = self.queues.read();
        if let Some((_key, queue)) = queues.iter().next_back() {
            Ok(queue.front().cloned())
        } else {
            Ok(None)
        }
    }

    async fn remove(&self, task_id: &TaskId) -> Result<()> {
        let mut queues = self.queues.write();
        self.by_id.write().remove(&task_id.0);
        for queue in queues.values_mut() {
            if let Some(pos) = queue.iter().position(|t| t.id == *task_id) {
                queue.remove(pos);
                return Ok(());
            }
        }
        Err(OrchestratorError::TaskError(format!(
            "task {} not found in scheduler",
            task_id.0
        )))
    }

    async fn reorder(&self, task_id: &TaskId, priority: TaskPriority) -> Result<()> {
        let mut queues = self.queues.write();
        for queue in queues.values_mut() {
            if let Some(pos) = queue.iter().position(|t| t.id == *task_id) {
                let mut task = queue.remove(pos).unwrap();
                task.priority = priority;
                let new_key = priority.value();
                queues.entry(new_key).or_default().push_back(task);
                return Ok(());
            }
        }
        Err(OrchestratorError::TaskError(format!(
            "task {} not found for reorder",
            task_id.0
        )))
    }

    async fn queue_length(&self) -> usize {
        let queues = self.queues.read();
        queues.values().map(|q| q.len()).sum()
    }

    async fn is_empty(&self) -> bool {
        self.queue_length().await == 0
    }

    async fn get_queue(&self, min_priority: Option<TaskPriority>) -> Result<Vec<OrchestratorTask>> {
        let min_val = min_priority.map_or(0, |p| p.value());
        let queues = self.queues.read();
        let mut result = Vec::new();
        for (&key, queue) in queues.iter() {
            if key >= min_val {
                result.extend(queue.iter().cloned());
            }
        }
        Ok(result)
    }

    async fn clear(&self) -> Result<()> {
        self.queues.write().clear();
        self.by_id.write().clear();
        Ok(())
    }
}

pub struct NoopTaskScheduler;

#[async_trait]
impl TaskSchedulerInternal for NoopTaskScheduler {
    async fn enqueue(&self, task: OrchestratorTask) -> Result<TaskId> {
        Ok(task.id)
    }
    async fn dequeue(
        &self,
        _max_priority: Option<TaskPriority>,
    ) -> Result<Option<OrchestratorTask>> {
        Ok(None)
    }
    async fn peek(&self) -> Result<Option<OrchestratorTask>> {
        Ok(None)
    }
    async fn remove(&self, _task_id: &TaskId) -> Result<()> {
        Ok(())
    }
    async fn reorder(&self, _task_id: &TaskId, _priority: TaskPriority) -> Result<()> {
        Ok(())
    }
    async fn queue_length(&self) -> usize {
        0
    }
    async fn is_empty(&self) -> bool {
        true
    }
    async fn get_queue(
        &self,
        _min_priority: Option<TaskPriority>,
    ) -> Result<Vec<OrchestratorTask>> {
        Ok(Vec::new())
    }
    async fn clear(&self) -> Result<()> {
        Ok(())
    }
}
