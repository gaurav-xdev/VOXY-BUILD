use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use parking_lot::RwLock;

use crate::error::Result;
use crate::types::TaskId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum InterruptionSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

#[derive(Debug, Clone)]
pub struct InterruptionEvent {
    pub id: String,
    pub source: String,
    pub task_id: Option<TaskId>,
    pub reason: String,
    pub severity: InterruptionSeverity,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[async_trait]
pub trait InterruptionManager: Send + Sync {
    async fn register_handler(
        &self,
        source: &str,
        handler: Box<dyn Fn(InterruptionEvent) -> Result<()> + Send + Sync>,
    ) -> Result<()>;
    async fn unregister_handler(&self, source: &str) -> Result<()>;
    async fn emit_interruption(&self, event: InterruptionEvent) -> Result<()>;
    async fn get_active_interruptions(&self) -> Result<Vec<InterruptionEvent>>;
    async fn is_interrupted(&self, task_id: &TaskId) -> bool;
    async fn clear_interruptions(&self, source: &str) -> Result<()>;
}

pub struct DefaultInterruptionManager {
    #[allow(clippy::type_complexity)]
    handlers: RwLock<HashMap<String, Box<dyn Fn(InterruptionEvent) -> Result<()> + Send + Sync>>>,
    active: RwLock<Vec<InterruptionEvent>>,
    interrupted_tasks: RwLock<HashMap<String, bool>>,
}

impl DefaultInterruptionManager {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            active: RwLock::new(Vec::new()),
            interrupted_tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn clear(&self) {
        self.handlers.write().clear();
        self.active.write().clear();
        self.interrupted_tasks.write().clear();
    }
}

impl Default for DefaultInterruptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl InterruptionManager for DefaultInterruptionManager {
    async fn register_handler(
        &self,
        source: &str,
        handler: Box<dyn Fn(InterruptionEvent) -> Result<()> + Send + Sync>,
    ) -> Result<()> {
        self.handlers.write().insert(source.to_string(), handler);
        Ok(())
    }

    async fn unregister_handler(&self, source: &str) -> Result<()> {
        self.handlers.write().remove(source);
        Ok(())
    }

    async fn emit_interruption(&self, event: InterruptionEvent) -> Result<()> {
        if let Some(task_id) = &event.task_id {
            self.interrupted_tasks
                .write()
                .insert(task_id.0.clone(), true);
        }
        self.active.write().push(event.clone());

        let handlers = self.handlers.read();
        if let Some(handler) = handlers.get(&event.source) {
            handler(event)?;
        }
        Ok(())
    }

    async fn get_active_interruptions(&self) -> Result<Vec<InterruptionEvent>> {
        Ok(self.active.read().clone())
    }

    async fn is_interrupted(&self, task_id: &TaskId) -> bool {
        self.interrupted_tasks
            .read()
            .get(&task_id.0)
            .copied()
            .unwrap_or(false)
    }

    async fn clear_interruptions(&self, source: &str) -> Result<()> {
        if source == "*" {
            self.active.write().clear();
            self.interrupted_tasks.write().clear();
        } else {
            self.active.write().retain(|e| e.source != source);
            self.interrupted_tasks.write().clear();
        }
        Ok(())
    }
}
