use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::config::ConversationConfig;
use crate::context::{ContextTracker, InMemoryContextTracker};
use crate::error::{ConversationError, Result};
use crate::event::ConversationEvent;
use crate::turn::{InMemoryTurnManager, TurnManager, TurnSource};
use crate::wake::{InMemoryWakeStateManager, WakeStateManager};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(pub Uuid);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self> {
        let uuid = Uuid::parse_str(s).map_err(|e| {
            ConversationError::SessionNotFound(format!("Invalid session ID: {}", e))
        })?;
        Ok(Self(uuid))
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    Created,
    Initializing,
    Active,
    Paused,
    Ending,
    Ended,
    Failed(String),
}

impl fmt::Display for SessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Active => write!(f, "Active"),
            Self::Paused => write!(f, "Paused"),
            Self::Ending => write!(f, "Ending"),
            Self::Ended => write!(f, "Ended"),
            Self::Failed(reason) => write!(f, "Failed({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub id: SessionId,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub turn_count: u64,
    pub personality_id: Option<String>,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
}

fn validate_transition(from: &SessionState, to: &SessionState) -> Result<()> {
    let valid = matches!(
        (from, to),
        (SessionState::Created, SessionState::Initializing)
            | (SessionState::Initializing, SessionState::Active)
            | (SessionState::Active, SessionState::Paused)
            | (SessionState::Active, SessionState::Ending)
            | (SessionState::Paused, SessionState::Active)
            | (SessionState::Paused, SessionState::Ending)
            | (SessionState::Ending, SessionState::Ended)
    );
    if valid {
        Ok(())
    } else {
        Err(ConversationError::InvalidStateTransition {
            from: from.to_string(),
            to: to.to_string(),
        })
    }
}

#[async_trait]
pub trait ConversationSession: Send + Sync {
    fn id(&self) -> &SessionId;
    fn state(&self) -> SessionState;
    fn metadata(&self) -> &SessionMetadata;
    async fn start(&mut self, user_id: Option<&str>, device_id: Option<&str>) -> Result<()>;
    async fn end(&mut self) -> Result<()>;
    async fn pause(&mut self) -> Result<()>;
    async fn resume(&mut self) -> Result<()>;
    async fn process_input(&mut self, text: &str, is_final: bool) -> Result<()>;
    async fn generate_output(&mut self, text: &str) -> Result<()>;
    fn turn_manager(&self) -> &dyn TurnManager;
    fn context_tracker(&self) -> &dyn ContextTracker;
    fn wake_manager(&self) -> &dyn WakeStateManager;
    async fn set_personality(&mut self, profile_id: &str) -> Result<()>;
    async fn on_event(
        &mut self,
        handler: Box<dyn Fn(ConversationEvent) + Send + Sync>,
    ) -> Result<()>;
}

#[async_trait]
pub trait SessionManager: Send + Sync {
    async fn create_session(&self) -> Result<Box<dyn ConversationSession>>;
    async fn get_session(&self, id: &SessionId) -> Result<Box<dyn ConversationSession>>;
    async fn end_session(&self, id: &SessionId) -> Result<()>;
    async fn list_sessions(&self) -> Result<Vec<SessionMetadata>>;
    async fn active_session_count(&self) -> usize;
    async fn cleanup_stale_sessions(&self, max_age_seconds: u64) -> Result<usize>;
}

pub struct InMemorySession {
    id: SessionId,
    state: SessionState,
    metadata: SessionMetadata,
    turn_mgr: InMemoryTurnManager,
    context: InMemoryContextTracker,
    wake: InMemoryWakeStateManager,
    event_handler: Option<Box<dyn Fn(ConversationEvent) + Send + Sync>>,
}

impl Clone for InMemorySession {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            state: self.state.clone(),
            metadata: self.metadata.clone(),
            turn_mgr: self.turn_mgr.clone(),
            context: self.context.clone(),
            wake: self.wake.clone(),
            event_handler: None,
        }
    }
}

impl InMemorySession {
    pub fn new(config: &ConversationConfig) -> Self {
        let id = SessionId::new();
        let now = Utc::now();
        Self {
            id: id.clone(),
            state: SessionState::Created,
            metadata: SessionMetadata {
                id,
                state: SessionState::Created,
                created_at: now,
                last_activity_at: now,
                turn_count: 0,
                personality_id: config.default_personality_id.clone(),
                user_id: None,
                device_id: None,
            },
            turn_mgr: InMemoryTurnManager::new(config.context_retention_turns),
            context: InMemoryContextTracker::new(config.context_retention_turns),
            wake: InMemoryWakeStateManager::new(config.auto_sleep_after_ms),
            event_handler: None,
        }
    }

    fn emit_event(&self, event: ConversationEvent) {
        if let Some(ref handler) = self.event_handler {
            handler(event);
        }
    }
}

#[async_trait]
impl ConversationSession for InMemorySession {
    fn id(&self) -> &SessionId {
        &self.id
    }

    fn state(&self) -> SessionState {
        self.state.clone()
    }

    fn metadata(&self) -> &SessionMetadata {
        &self.metadata
    }

    async fn start(&mut self, user_id: Option<&str>, device_id: Option<&str>) -> Result<()> {
        validate_transition(&self.state, &SessionState::Initializing)?;
        self.state = SessionState::Initializing;
        self.metadata.last_activity_at = Utc::now();
        if let Some(uid) = user_id {
            self.metadata.user_id = Some(uid.to_string());
        }
        if let Some(did) = device_id {
            self.metadata.device_id = Some(did.to_string());
        }
        validate_transition(&self.state, &SessionState::Active)?;
        self.state = SessionState::Active;
        self.metadata.state = SessionState::Active;
        self.metadata.last_activity_at = Utc::now();
        let _ = self.wake.wake().await;
        self.emit_event(ConversationEvent::SessionStarted {
            id: self.id.clone(),
        });
        Ok(())
    }

    async fn end(&mut self) -> Result<()> {
        validate_transition(&self.state, &SessionState::Ending)?;
        let old_state = self.state.clone();
        self.state = SessionState::Ending;
        self.metadata.last_activity_at = Utc::now();
        validate_transition(&self.state, &SessionState::Ended)?;
        self.state = SessionState::Ended;
        self.metadata.state = SessionState::Ended;
        self.metadata.last_activity_at = Utc::now();
        let _ = self.wake.sleep().await;
        let turn_count = self.turn_mgr.turn_count();
        let duration = (Utc::now() - self.metadata.created_at).num_milliseconds() as f64;
        self.emit_event(ConversationEvent::SessionEnded {
            id: self.id.clone(),
            turn_count,
            duration_ms: duration,
        });
        let _ = old_state;
        Ok(())
    }

    async fn pause(&mut self) -> Result<()> {
        validate_transition(&self.state, &SessionState::Paused)?;
        self.state = SessionState::Paused;
        self.metadata.state = SessionState::Paused;
        self.metadata.last_activity_at = Utc::now();
        self.emit_event(ConversationEvent::SessionPaused {
            id: self.id.clone(),
        });
        Ok(())
    }

    async fn resume(&mut self) -> Result<()> {
        validate_transition(&self.state, &SessionState::Active)?;
        self.state = SessionState::Active;
        self.metadata.state = SessionState::Active;
        self.metadata.last_activity_at = Utc::now();
        let _ = self.wake.wake().await;
        self.emit_event(ConversationEvent::SessionResumed {
            id: self.id.clone(),
        });
        Ok(())
    }

    async fn process_input(&mut self, text: &str, is_final: bool) -> Result<()> {
        if self.state != SessionState::Active {
            return Err(ConversationError::SessionNotActive(self.id.to_string()));
        }
        self.metadata.turn_count += 1;
        self.metadata.last_activity_at = Utc::now();
        let (turn_id, source) = {
            let turn = self.turn_mgr.begin_turn(TurnSource::UserInput).await?;
            (turn.id, TurnSource::UserInput)
        };
        let result = async {
            self.emit_event(ConversationEvent::TurnBegan {
                session_id: self.id.clone(),
                turn_id,
                source,
            });
            self.context.set("last_input", text).await?;
            self.emit_event(ConversationEvent::InputReceived {
                session_id: self.id.clone(),
                text: text.to_string(),
                is_final,
            });
            Ok::<_, crate::error::ConversationError>(())
        }
        .await;
        if result.is_err() {
            let _ = self.turn_mgr.end_turn(None).await;
        }
        result
    }

    async fn generate_output(&mut self, text: &str) -> Result<()> {
        if self.state != SessionState::Active {
            return Err(ConversationError::SessionNotActive(self.id.to_string()));
        }
        self.metadata.last_activity_at = Utc::now();
        self.turn_mgr.end_turn(Some(text)).await?;
        self.context.set("last_output", text).await?;
        self.emit_event(ConversationEvent::OutputGenerated {
            session_id: self.id.clone(),
            text: text.to_string(),
        });
        Ok(())
    }

    fn turn_manager(&self) -> &dyn TurnManager {
        &self.turn_mgr
    }

    fn context_tracker(&self) -> &dyn ContextTracker {
        &self.context
    }

    fn wake_manager(&self) -> &dyn WakeStateManager {
        &self.wake
    }

    async fn set_personality(&mut self, profile_id: &str) -> Result<()> {
        self.metadata.personality_id = Some(profile_id.to_string());
        self.emit_event(ConversationEvent::PersonalityApplied {
            session_id: self.id.clone(),
            profile_id: profile_id.to_string(),
        });
        Ok(())
    }

    async fn on_event(
        &mut self,
        handler: Box<dyn Fn(ConversationEvent) + Send + Sync>,
    ) -> Result<()> {
        self.event_handler = Some(handler);
        Ok(())
    }
}

pub struct InMemorySessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, InMemorySession>>>,
    config: ConversationConfig,
}

impl InMemorySessionManager {
    pub fn new(config: ConversationConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
}

#[async_trait]
impl SessionManager for InMemorySessionManager {
    async fn create_session(&self) -> Result<Box<dyn ConversationSession>> {
        let session = InMemorySession::new(&self.config);
        let id = session.id.clone();
        let mut map = self.sessions.write().await;
        map.insert(id.clone(), session);
        Ok(Box::new(map.get(&id).unwrap().clone()))
    }

    async fn get_session(&self, id: &SessionId) -> Result<Box<dyn ConversationSession>> {
        let map = self.sessions.read().await;
        let session = map
            .get(id)
            .ok_or_else(|| ConversationError::SessionNotFound(id.to_string()))?;
        Ok(Box::new(session.clone()))
    }

    async fn end_session(&self, id: &SessionId) -> Result<()> {
        let mut map = self.sessions.write().await;
        let mut session = map
            .remove(id)
            .ok_or_else(|| ConversationError::SessionNotFound(id.to_string()))?;
        if session.state != SessionState::Created && session.state != SessionState::Ended {
            session.end().await?;
        }
        Ok(())
    }

    async fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let map = self.sessions.read().await;
        Ok(map.values().map(|s| s.metadata.clone()).collect())
    }

    async fn active_session_count(&self) -> usize {
        let map = self.sessions.read().await;
        map.values()
            .filter(|s| s.state == SessionState::Active)
            .count()
    }

    async fn cleanup_stale_sessions(&self, max_age_seconds: u64) -> Result<usize> {
        let mut map = self.sessions.write().await;
        let now = Utc::now();
        let stale_ids: Vec<SessionId> = map
            .iter()
            .filter(|(_, s)| {
                let age = (now - s.metadata.last_activity_at).num_seconds() as u64;
                age >= max_age_seconds
            })
            .map(|(id, _)| id.clone())
            .collect();
        let count = stale_ids.len();
        for id in stale_ids {
            map.remove(&id);
        }
        Ok(count)
    }
}
