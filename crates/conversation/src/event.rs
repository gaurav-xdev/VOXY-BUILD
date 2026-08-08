use std::fmt;

use uuid::Uuid;

use crate::interrupt::InterruptionSource;
use crate::session::SessionId;
use crate::turn::TurnSource;
use crate::wake::WakeState;
use voxy_personality::MoodState;

pub enum ConversationEvent {
    SessionCreated {
        id: SessionId,
    },
    SessionStarted {
        id: SessionId,
    },
    SessionEnded {
        id: SessionId,
        turn_count: u64,
        duration_ms: f64,
    },
    SessionPaused {
        id: SessionId,
    },
    SessionResumed {
        id: SessionId,
    },
    TurnBegan {
        session_id: SessionId,
        turn_id: Uuid,
        source: TurnSource,
    },
    TurnEnded {
        session_id: SessionId,
        turn_id: Uuid,
        was_interrupted: bool,
    },
    InterruptionDetected {
        session_id: SessionId,
        source: InterruptionSource,
    },
    WakeStateChanged {
        session_id: SessionId,
        old: WakeState,
        new: WakeState,
    },
    InputReceived {
        session_id: SessionId,
        text: String,
        is_final: bool,
    },
    OutputGenerated {
        session_id: SessionId,
        text: String,
    },
    MoodChanged {
        session_id: SessionId,
        old: MoodState,
        new: MoodState,
    },
    PersonalityApplied {
        session_id: SessionId,
        profile_id: String,
    },
    Error {
        session_id: SessionId,
        error: String,
    },
}

impl fmt::Display for ConversationEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionCreated { id } => write!(f, "SessionCreated({})", id.0),
            Self::SessionStarted { id } => write!(f, "SessionStarted({})", id.0),
            Self::SessionEnded {
                id,
                turn_count,
                duration_ms,
            } => {
                write!(
                    f,
                    "SessionEnded({}, turns={}, duration={}ms)",
                    id.0, turn_count, duration_ms
                )
            }
            Self::SessionPaused { id } => write!(f, "SessionPaused({})", id.0),
            Self::SessionResumed { id } => write!(f, "SessionResumed({})", id.0),
            Self::TurnBegan {
                session_id,
                turn_id,
                source,
            } => {
                write!(
                    f,
                    "TurnBegan(session={}, turn={}, source={})",
                    session_id.0, turn_id, source
                )
            }
            Self::TurnEnded {
                session_id,
                turn_id,
                was_interrupted,
            } => {
                write!(
                    f,
                    "TurnEnded(session={}, turn={}, interrupted={})",
                    session_id.0, turn_id, was_interrupted
                )
            }
            Self::InterruptionDetected {
                session_id,
                source: _,
            } => {
                write!(f, "InterruptionDetected({})", session_id.0)
            }
            Self::WakeStateChanged {
                session_id,
                old,
                new,
            } => {
                write!(f, "WakeStateChanged({}, {} -> {})", session_id.0, old, new)
            }
            Self::InputReceived {
                session_id,
                text,
                is_final,
            } => {
                write!(
                    f,
                    "InputReceived({}, text_len={}, final={})",
                    session_id.0,
                    text.len(),
                    is_final
                )
            }
            Self::OutputGenerated { session_id, text } => {
                write!(
                    f,
                    "OutputGenerated({}, text_len={})",
                    session_id.0,
                    text.len()
                )
            }
            Self::MoodChanged {
                session_id,
                old,
                new,
            } => {
                write!(f, "MoodChanged({}, {:?} -> {:?})", session_id.0, old, new)
            }
            Self::PersonalityApplied {
                session_id,
                profile_id,
            } => {
                write!(
                    f,
                    "PersonalityApplied({}, profile={})",
                    session_id.0, profile_id
                )
            }
            Self::Error { session_id, error } => {
                write!(f, "Error({}, {})", session_id.0, error)
            }
        }
    }
}
