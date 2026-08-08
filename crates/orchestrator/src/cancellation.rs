use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::error::{OrchestratorError, Result};
use crate::types::{SystemComponent, TaskId, TaskPriority};

use super::execution::CancellationFlag;

#[async_trait]
pub trait CancellationManager: Send + Sync {
    async fn register_cancellation_token(&self, task_id: &TaskId) -> Result<String>;
    async fn cancel_task(&self, task_id: &TaskId, reason: &str) -> Result<()>;
    async fn is_cancelled(&self, task_id: &TaskId) -> bool;
    async fn cancel_all(&self, reason: &str) -> Result<usize>;
    async fn cancel_by_component(&self, component: &SystemComponent, reason: &str)
        -> Result<usize>;
    async fn cancel_by_priority(&self, below_priority: TaskPriority, reason: &str)
        -> Result<usize>;
}

pub struct DefaultCancellationManager {
    tokens: RwLock<HashMap<String, Arc<CancellationFlag>>>,
}

impl DefaultCancellationManager {
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    pub async fn clear(&self) {
        self.tokens.write().clear();
    }
}

impl Default for DefaultCancellationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CancellationManager for DefaultCancellationManager {
    async fn register_cancellation_token(&self, task_id: &TaskId) -> Result<String> {
        let token_id = Uuid::new_v4().to_string();
        let flag = Arc::new(CancellationFlag::new());
        self.tokens.write().insert(token_id.clone(), flag);
        self.tokens
            .write()
            .insert(task_id.0.clone(), Arc::new(CancellationFlag::new()));
        Ok(token_id)
    }

    async fn cancel_task(&self, task_id: &TaskId, reason: &str) -> Result<()> {
        let tokens = self.tokens.read();
        if let Some(flag) = tokens.get(&task_id.0) {
            flag.cancel(reason);
            Ok(())
        } else {
            Err(OrchestratorError::CancellationError(format!(
                "no cancellation token for task {}",
                task_id.0
            )))
        }
    }

    async fn is_cancelled(&self, task_id: &TaskId) -> bool {
        let tokens = self.tokens.read();
        tokens
            .get(&task_id.0)
            .map(|f| f.is_cancelled())
            .unwrap_or(false)
    }

    async fn cancel_all(&self, reason: &str) -> Result<usize> {
        let tokens = self.tokens.read();
        let count = tokens.len();
        for flag in tokens.values() {
            flag.cancel(reason);
        }
        Ok(count)
    }

    async fn cancel_by_component(
        &self,
        _component: &SystemComponent,
        reason: &str,
    ) -> Result<usize> {
        let tokens = self.tokens.read();
        let count = tokens.len();
        for flag in tokens.values() {
            flag.cancel(reason);
        }
        Ok(count)
    }

    async fn cancel_by_priority(
        &self,
        _below_priority: TaskPriority,
        reason: &str,
    ) -> Result<usize> {
        let tokens = self.tokens.read();
        let count = tokens.len();
        for flag in tokens.values() {
            flag.cancel(reason);
        }
        Ok(count)
    }
}
