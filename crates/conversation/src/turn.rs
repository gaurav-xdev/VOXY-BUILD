use std::collections::VecDeque;
use std::fmt;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum TurnSource {
    UserInput,
    SystemResponse,
    WakeWord,
    Interruption,
    Internal,
}

impl fmt::Display for TurnSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserInput => write!(f, "UserInput"),
            Self::SystemResponse => write!(f, "SystemResponse"),
            Self::WakeWord => write!(f, "WakeWord"),
            Self::Interruption => write!(f, "Interruption"),
            Self::Internal => write!(f, "Internal"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TurnState {
    Created,
    ProcessingInput,
    AwaitingResponse,
    GeneratingResponse,
    Interrupted,
    Completed,
    Failed(String),
}

impl fmt::Display for TurnState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::ProcessingInput => write!(f, "ProcessingInput"),
            Self::AwaitingResponse => write!(f, "AwaitingResponse"),
            Self::GeneratingResponse => write!(f, "GeneratingResponse"),
            Self::Interrupted => write!(f, "Interrupted"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed(reason) => write!(f, "Failed({})", reason),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Turn {
    pub id: Uuid,
    pub source: TurnSource,
    pub state: TurnState,
    pub input_text: Option<String>,
    pub output_text: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<f64>,
    pub was_interrupted: bool,
    pub confidence: f32,
}

impl Turn {
    pub fn new(source: TurnSource) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            state: TurnState::Created,
            input_text: None,
            output_text: None,
            started_at: Utc::now(),
            ended_at: None,
            duration_ms: None,
            was_interrupted: false,
            confidence: 1.0,
        }
    }
}

#[async_trait]
pub trait TurnManager: Send + Sync {
    fn current_turn(&self) -> Option<&Turn>;
    fn turn_count(&self) -> u64;
    fn turn_history(&self, n: usize) -> Vec<Turn>;
    async fn begin_turn(&mut self, source: TurnSource) -> Result<&Turn>;
    async fn end_turn(&mut self, output: Option<&str>) -> Result<&Turn>;
    async fn interrupt_current(&mut self) -> Result<()>;
    fn last_turn(&self) -> Option<&Turn>;
    fn is_barge_in(&self) -> bool;
    async fn set_barge_in(&mut self, enabled: bool);
}

#[derive(Debug, Clone)]
pub struct InMemoryTurnManager {
    turns: VecDeque<Turn>,
    current: Option<Turn>,
    barge_in: bool,
    max_history: usize,
}

impl InMemoryTurnManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            turns: VecDeque::new(),
            current: None,
            barge_in: false,
            max_history,
        }
    }
}

#[async_trait]
impl TurnManager for InMemoryTurnManager {
    fn current_turn(&self) -> Option<&Turn> {
        self.current.as_ref()
    }

    fn turn_count(&self) -> u64 {
        self.turns.len() as u64 + if self.current.is_some() { 1 } else { 0 }
    }

    fn turn_history(&self, n: usize) -> Vec<Turn> {
        let mut history: Vec<Turn> = self.turns.iter().cloned().collect();
        if let Some(ref current) = self.current {
            history.push(current.clone());
        }
        history.into_iter().rev().take(n).collect()
    }

    async fn begin_turn(&mut self, source: TurnSource) -> Result<&Turn> {
        if self.current.is_some() {
            return Err(crate::error::ConversationError::TurnError(
                "A turn is already in progress".to_string(),
            ));
        }
        let turn = Turn::new(source);
        self.current = Some(turn);
        Ok(self.current.as_ref().unwrap())
    }

    async fn end_turn(&mut self, output: Option<&str>) -> Result<&Turn> {
        let mut turn = self.current.take().ok_or_else(|| {
            crate::error::ConversationError::TurnError("No active turn to end".to_string())
        })?;
        turn.state = TurnState::Completed;
        turn.output_text = output.map(|s| s.to_string());
        turn.ended_at = Some(Utc::now());
        let start = turn.started_at;
        let end = turn.ended_at.unwrap();
        turn.duration_ms = Some((end - start).num_milliseconds() as f64);
        self.turns.push_back(turn);
        if self.turns.len() > self.max_history {
            self.turns.pop_front();
        }
        Ok(self.turns.back().unwrap())
    }

    async fn interrupt_current(&mut self) -> Result<()> {
        if self.current.is_some() {
            let mut turn = self.current.take().unwrap();
            turn.state = TurnState::Interrupted;
            turn.was_interrupted = true;
            turn.ended_at = Some(Utc::now());
            let start = turn.started_at;
            let end = turn.ended_at.unwrap();
            turn.duration_ms = Some((end - start).num_milliseconds() as f64);
            self.turns.push_back(turn);
            if self.turns.len() > self.max_history {
                self.turns.pop_front();
            }
            Ok(())
        } else {
            Err(crate::error::ConversationError::TurnError(
                "No active turn to interrupt".to_string(),
            ))
        }
    }

    fn last_turn(&self) -> Option<&Turn> {
        self.turns.back()
    }

    fn is_barge_in(&self) -> bool {
        self.barge_in
    }

    async fn set_barge_in(&mut self, enabled: bool) {
        self.barge_in = enabled;
    }
}
