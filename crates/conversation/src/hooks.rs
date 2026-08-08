use std::sync::Arc;

use async_trait::async_trait;
use parking_lot::RwLock;

use crate::error::Result;
use crate::session::{SessionId, SessionMetadata};
use crate::turn::Turn;
use voxy_personality::MoodState;

type SharedHook = Arc<dyn PersonalityHook>;

#[async_trait]
pub trait PersonalityHook: Send + Sync {
    fn name(&self) -> &str;
    async fn on_session_start(&self, metadata: &SessionMetadata) -> Result<()>;
    async fn on_session_end(&self, metadata: &SessionMetadata) -> Result<()>;
    async fn on_turn_start(&self, session_id: &SessionId, turn: &Turn) -> Result<()>;
    async fn on_turn_end(&self, session_id: &SessionId, turn: &Turn) -> Result<()>;
    async fn on_mood_change(
        &self,
        session_id: &SessionId,
        old_mood: &MoodState,
        new_mood: &MoodState,
    ) -> Result<()>;
    async fn on_input_received(&self, session_id: &SessionId, text: &str) -> Result<String>;
    async fn on_output_generated(&self, session_id: &SessionId, text: &str) -> Result<String>;
}

#[async_trait]
pub trait PersonalityHookRegistry: Send + Sync {
    async fn register_hook(&self, hook: Box<dyn PersonalityHook>) -> Result<()>;
    async fn unregister_hook(&self, name: &str) -> Result<()>;
    async fn execute_hooks<'a>(&self, event: HookEvent<'a>) -> Result<()>;
    fn hook_count(&self) -> usize;
}

pub enum HookEvent<'a> {
    SessionStart(&'a SessionMetadata),
    SessionEnd(&'a SessionMetadata),
    TurnStart {
        session_id: &'a SessionId,
        turn: &'a Turn,
    },
    TurnEnd {
        session_id: &'a SessionId,
        turn: &'a Turn,
    },
    MoodChange {
        session_id: &'a SessionId,
        old: &'a MoodState,
        new: &'a MoodState,
    },
    InputReceived {
        session_id: &'a SessionId,
        text: &'a str,
    },
    OutputGenerated {
        session_id: &'a SessionId,
        text: &'a str,
    },
}

pub struct InMemoryHookRegistry {
    hooks: RwLock<Vec<SharedHook>>,
}

impl Default for InMemoryHookRegistry {
    fn default() -> Self {
        Self {
            hooks: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl PersonalityHookRegistry for InMemoryHookRegistry {
    async fn register_hook(&self, hook: Box<dyn PersonalityHook>) -> Result<()> {
        let mut hooks = self.hooks.write();
        hooks.push(Arc::from(hook));
        Ok(())
    }

    async fn unregister_hook(&self, name: &str) -> Result<()> {
        let mut hooks = self.hooks.write();
        hooks.retain(|h| h.name() != name);
        Ok(())
    }

    async fn execute_hooks<'a>(&self, event: HookEvent<'a>) -> Result<()> {
        let snapshot = self.hooks.read().clone();
        for hook in &snapshot {
            match &event {
                HookEvent::SessionStart(metadata) => {
                    hook.on_session_start(metadata).await?;
                }
                HookEvent::SessionEnd(metadata) => {
                    hook.on_session_end(metadata).await?;
                }
                HookEvent::TurnStart { session_id, turn } => {
                    hook.on_turn_start(session_id, turn).await?;
                }
                HookEvent::TurnEnd { session_id, turn } => {
                    hook.on_turn_end(session_id, turn).await?;
                }
                HookEvent::MoodChange {
                    session_id,
                    old,
                    new,
                } => {
                    hook.on_mood_change(session_id, old, new).await?;
                }
                HookEvent::InputReceived { session_id, text } => {
                    let _ = hook.on_input_received(session_id, text).await?;
                }
                HookEvent::OutputGenerated { session_id, text } => {
                    let _ = hook.on_output_generated(session_id, text).await?;
                }
            }
        }
        Ok(())
    }

    fn hook_count(&self) -> usize {
        self.hooks.read().len()
    }
}
