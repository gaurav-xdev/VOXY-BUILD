//! Generic lifecycle state machine for runtimes.

pub mod error;

pub use error::{Result, StateMachineError};

/// Lifecycle states — standard for all VOXY runtimes (Voice, Memory, Automation, Vision, Plugins).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LifecycleState {
    Created,
    Initializing,
    Ready,
    Running,
    Busy,
    Paused,
    Recovering,
    Stopping,
    Stopped,
    Failed,
}

impl LifecycleState {
    /// Valid transitions from this state.
    pub fn valid_transitions(&self) -> Vec<LifecycleState> {
        match self {
            Self::Created => vec![Self::Initializing, Self::Failed],
            Self::Initializing => vec![Self::Ready, Self::Failed],
            Self::Ready => vec![Self::Running, Self::Stopping, Self::Failed],
            Self::Running => vec![Self::Busy, Self::Paused, Self::Stopping, Self::Failed],
            Self::Busy => vec![
                Self::Running,
                Self::Paused,
                Self::Recovering,
                Self::Stopping,
                Self::Failed,
            ],
            Self::Paused => vec![Self::Running, Self::Stopping, Self::Failed],
            Self::Recovering => vec![Self::Initializing, Self::Ready, Self::Failed, Self::Stopped],
            Self::Stopping => vec![Self::Stopped, Self::Failed],
            Self::Stopped => vec![Self::Initializing],
            Self::Failed => vec![Self::Recovering, Self::Stopped],
        }
    }

    /// Check if a transition is valid.
    pub fn can_transition_to(&self, target: &LifecycleState) -> bool {
        self.valid_transitions().contains(target)
    }

    /// Check if the state machine is in an operational state.
    pub fn is_operational(&self) -> bool {
        matches!(self, Self::Ready | Self::Running | Self::Busy)
    }

    /// Check if the state machine is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Failed)
    }
}

/// State machine for lifecycle management.
pub struct LifecycleStateMachine {
    state: LifecycleState,
    name: String,
}

impl LifecycleStateMachine {
    /// Create a new state machine.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            state: LifecycleState::Created,
            name: name.into(),
        }
    }

    /// Get the current state.
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// Get the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Attempt a transition.
    pub fn transition(&mut self, target: LifecycleState) -> Result<()> {
        if self.state.can_transition_to(&target) {
            tracing::debug!("[{}] {:?} -> {:?}", self.name, self.state, target);
            self.state = target;
            Ok(())
        } else {
            Err(StateMachineError::InvalidTransition {
                from: self.state,
                to: target,
            })
        }
    }

    /// Force a transition (bypasses validation).
    pub fn force_transition(&mut self, target: LifecycleState) {
        tracing::warn!("[{}] Force {:?} -> {:?}", self.name, self.state, target);
        self.state = target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_transitions() {
        let mut sm = LifecycleStateMachine::new("test");
        assert_eq!(sm.state(), LifecycleState::Created);

        sm.transition(LifecycleState::Initializing).unwrap();
        assert_eq!(sm.state(), LifecycleState::Initializing);

        sm.transition(LifecycleState::Ready).unwrap();
        assert_eq!(sm.state(), LifecycleState::Ready);

        sm.transition(LifecycleState::Running).unwrap();
        assert_eq!(sm.state(), LifecycleState::Running);

        sm.transition(LifecycleState::Busy).unwrap();
        assert_eq!(sm.state(), LifecycleState::Busy);

        sm.transition(LifecycleState::Running).unwrap();
        assert_eq!(sm.state(), LifecycleState::Running);
    }

    #[test]
    fn invalid_transition() {
        let mut sm = LifecycleStateMachine::new("test");
        let result = sm.transition(LifecycleState::Running);
        assert!(result.is_err());
    }

    #[test]
    fn valid_transitions() {
        assert!(LifecycleState::Created.can_transition_to(&LifecycleState::Initializing));
        assert!(LifecycleState::Created.can_transition_to(&LifecycleState::Failed));
        assert!(!LifecycleState::Created.can_transition_to(&LifecycleState::Running));
    }

    #[test]
    fn busy_transitions() {
        assert!(LifecycleState::Running.can_transition_to(&LifecycleState::Busy));
        assert!(LifecycleState::Busy.can_transition_to(&LifecycleState::Running));
        assert!(LifecycleState::Busy.can_transition_to(&LifecycleState::Paused));
        assert!(LifecycleState::Busy.can_transition_to(&LifecycleState::Recovering));
    }

    #[test]
    fn operational_states() {
        assert!(!LifecycleState::Created.is_operational());
        assert!(LifecycleState::Ready.is_operational());
        assert!(LifecycleState::Running.is_operational());
        assert!(LifecycleState::Busy.is_operational());
        assert!(!LifecycleState::Stopped.is_operational());
        assert!(!LifecycleState::Failed.is_operational());
    }

    #[test]
    fn terminal_states() {
        assert!(LifecycleState::Stopped.is_terminal());
        assert!(LifecycleState::Failed.is_terminal());
        assert!(!LifecycleState::Running.is_terminal());
    }
}
