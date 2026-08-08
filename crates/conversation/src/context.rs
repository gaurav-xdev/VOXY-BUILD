use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::turn::Turn;

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub entries: HashMap<String, String>,
    pub turn_history: Vec<Turn>,
    pub current_topic: Option<String>,
    pub session_start: DateTime<Utc>,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            turn_history: Vec::new(),
            current_topic: None,
            session_start: Utc::now(),
            user_id: None,
            device_id: None,
        }
    }
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
pub trait ContextTracker: Send + Sync {
    fn context(&self) -> &ConversationContext;
    async fn set(&mut self, key: &str, value: &str) -> Result<()>;
    fn get(&self, key: &str) -> Option<&str>;
    async fn remove(&mut self, key: &str) -> Result<()>;
    async fn clear(&mut self) -> Result<()>;
    fn turn_history(&self, n: usize) -> Vec<Turn>;
    fn current_topic(&self) -> Option<&str>;
    async fn set_current_topic(&mut self, topic: &str);
    fn has_key(&self, key: &str) -> bool;
    fn entry_count(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct InMemoryContextTracker {
    context: ConversationContext,
    max_turns: usize,
}

impl InMemoryContextTracker {
    pub fn new(max_turns: usize) -> Self {
        Self {
            context: ConversationContext::new(),
            max_turns,
        }
    }
}

#[async_trait]
impl ContextTracker for InMemoryContextTracker {
    fn context(&self) -> &ConversationContext {
        &self.context
    }

    async fn set(&mut self, key: &str, value: &str) -> Result<()> {
        self.context
            .entries
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.context.entries.get(key).map(|s| s.as_str())
    }

    async fn remove(&mut self, key: &str) -> Result<()> {
        self.context.entries.remove(key);
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.context.entries.clear();
        self.context.current_topic = None;
        Ok(())
    }

    fn turn_history(&self, n: usize) -> Vec<Turn> {
        let n = n.min(self.max_turns);
        self.context
            .turn_history
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    }

    fn current_topic(&self) -> Option<&str> {
        self.context.current_topic.as_deref()
    }

    async fn set_current_topic(&mut self, topic: &str) {
        self.context.current_topic = Some(topic.to_string());
    }

    fn has_key(&self, key: &str) -> bool {
        self.context.entries.contains_key(key)
    }

    fn entry_count(&self) -> usize {
        self.context.entries.len()
    }
}
