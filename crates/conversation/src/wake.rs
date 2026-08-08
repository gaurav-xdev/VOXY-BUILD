use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum WakeState {
    Asleep,
    Dozing,
    Awake,
    Listening,
    Processing,
}

impl fmt::Display for WakeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asleep => write!(f, "Asleep"),
            Self::Dozing => write!(f, "Dozing"),
            Self::Awake => write!(f, "Awake"),
            Self::Listening => write!(f, "Listening"),
            Self::Processing => write!(f, "Processing"),
        }
    }
}

#[async_trait]
pub trait WakeStateManager: Send + Sync {
    fn state(&self) -> WakeState;
    async fn transition_to(&mut self, state: WakeState) -> Result<()>;
    async fn wake(&mut self) -> Result<()>;
    async fn sleep(&mut self) -> Result<()>;
    fn idle_duration_ms(&self) -> u64;
    async fn tick_idle(&mut self, elapsed_ms: u64) -> Result<Option<WakeState>>;
    fn last_transition(&self) -> DateTime<Utc>;
    fn wake_count(&self) -> u64;
}

fn is_valid_transition(from: &WakeState, to: &WakeState) -> bool {
    matches!(
        (from, to),
        (WakeState::Asleep, WakeState::Awake)
            | (WakeState::Awake, WakeState::Listening)
            | (WakeState::Listening, WakeState::Processing)
            | (WakeState::Listening, WakeState::Awake)
            | (WakeState::Processing, WakeState::Awake)
            | (WakeState::Awake, WakeState::Asleep)
            | (WakeState::Dozing, WakeState::Awake)
            | (WakeState::Awake, WakeState::Dozing)
            | (WakeState::Dozing, WakeState::Asleep)
    )
}

#[derive(Debug, Clone)]
pub struct InMemoryWakeStateManager {
    state: WakeState,
    last_transition: DateTime<Utc>,
    wake_count: u64,
    idle_accumulated_ms: u64,
    auto_sleep_ms: u64,
}

impl InMemoryWakeStateManager {
    pub fn new(auto_sleep_ms: u64) -> Self {
        Self {
            state: WakeState::Asleep,
            last_transition: Utc::now(),
            wake_count: 0,
            idle_accumulated_ms: 0,
            auto_sleep_ms,
        }
    }
}

#[async_trait]
impl WakeStateManager for InMemoryWakeStateManager {
    fn state(&self) -> WakeState {
        self.state.clone()
    }

    async fn transition_to(&mut self, state: WakeState) -> Result<()> {
        if !is_valid_transition(&self.state, &state) {
            return Err(crate::error::ConversationError::InvalidStateTransition {
                from: self.state.to_string(),
                to: state.to_string(),
            });
        }
        if self.state == WakeState::Awake && state == WakeState::Asleep {
            self.idle_accumulated_ms = 0;
        }
        if state == WakeState::Awake && self.state == WakeState::Asleep {
            self.wake_count += 1;
        }
        self.state = state;
        self.last_transition = Utc::now();
        self.idle_accumulated_ms = 0;
        Ok(())
    }

    async fn wake(&mut self) -> Result<()> {
        if self.state == WakeState::Asleep || self.state == WakeState::Dozing {
            self.transition_to(WakeState::Awake).await
        } else {
            Ok(())
        }
    }

    async fn sleep(&mut self) -> Result<()> {
        if self.state != WakeState::Asleep {
            self.transition_to(WakeState::Asleep).await
        } else {
            Ok(())
        }
    }

    fn idle_duration_ms(&self) -> u64 {
        self.idle_accumulated_ms
    }

    async fn tick_idle(&mut self, elapsed_ms: u64) -> Result<Option<WakeState>> {
        if self.state == WakeState::Awake || self.state == WakeState::Dozing {
            self.idle_accumulated_ms += elapsed_ms;
            if self.idle_accumulated_ms >= self.auto_sleep_ms {
                let old_state = self.state.clone();
                self.transition_to(WakeState::Asleep).await?;
                return Ok(Some(old_state));
            }
        } else {
            self.idle_accumulated_ms = 0;
        }
        Ok(None)
    }

    fn last_transition(&self) -> DateTime<Utc> {
        self.last_transition
    }

    fn wake_count(&self) -> u64 {
        self.wake_count
    }
}
