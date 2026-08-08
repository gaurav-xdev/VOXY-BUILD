use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentState {
    Unknown,
    Initializing,
    Running,
    Degraded(String),
    Failed(String),
    Stopped,
}

impl ComponentState {
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    pub fn is_degraded(&self) -> bool {
        matches!(self, Self::Degraded(_))
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped)
    }

    pub fn message(&self) -> Option<&str> {
        match self {
            Self::Degraded(msg) | Self::Failed(msg) => Some(msg),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateTracker {
    states: Arc<RwLock<HashMap<String, ComponentState>>>,
}

impl StateTracker {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, name: &str) {
        let mut states = self.states.write().await;
        states
            .entry(name.to_string())
            .or_insert(ComponentState::Unknown);
    }

    pub async fn set_state(&self, name: &str, state: ComponentState) {
        let mut states = self.states.write().await;
        states.insert(name.to_string(), state);
    }

    pub async fn get_state(&self, name: &str) -> Option<ComponentState> {
        let states = self.states.read().await;
        states.get(name).cloned()
    }

    pub async fn all_states(&self) -> HashMap<String, ComponentState> {
        self.states.read().await.clone()
    }

    pub async fn is_healthy(&self, name: &str) -> bool {
        let states = self.states.read().await;
        matches!(states.get(name), Some(ComponentState::Running))
    }

    pub async fn unhealthy_components(&self) -> Vec<(String, ComponentState)> {
        let states = self.states.read().await;
        states
            .iter()
            .filter(|(_, state)| state.is_failed() || state.is_degraded())
            .map(|(name, state)| (name.clone(), state.clone()))
            .collect()
    }
}

impl Default for StateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn state_transitions() {
        let tracker = StateTracker::new();
        tracker.register("comp1").await;
        assert_eq!(
            tracker.get_state("comp1").await,
            Some(ComponentState::Unknown)
        );

        tracker
            .set_state("comp1", ComponentState::Initializing)
            .await;
        assert_eq!(
            tracker.get_state("comp1").await,
            Some(ComponentState::Initializing)
        );

        tracker.set_state("comp1", ComponentState::Running).await;
        assert_eq!(
            tracker.get_state("comp1").await,
            Some(ComponentState::Running)
        );
    }

    #[tokio::test]
    async fn health_checks() {
        let tracker = StateTracker::new();
        tracker.register("svc1").await;
        assert!(!tracker.is_healthy("svc1").await);

        tracker.set_state("svc1", ComponentState::Running).await;
        assert!(tracker.is_healthy("svc1").await);

        tracker
            .set_state("svc1", ComponentState::Degraded("slow".into()))
            .await;
        assert!(!tracker.is_healthy("svc1").await);
    }

    #[tokio::test]
    async fn unhealthy_components() {
        let tracker = StateTracker::new();
        tracker.register("svc1").await;
        tracker.register("svc2").await;
        tracker.register("svc3").await;

        tracker.set_state("svc1", ComponentState::Running).await;
        tracker
            .set_state("svc2", ComponentState::Failed("crash".into()))
            .await;
        tracker
            .set_state("svc3", ComponentState::Degraded("oom".into()))
            .await;

        let unhealthy = tracker.unhealthy_components().await;
        assert_eq!(unhealthy.len(), 2);
        let names: Vec<&str> = unhealthy.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"svc2"));
        assert!(names.contains(&"svc3"));
    }

    #[tokio::test]
    async fn all_states() {
        let tracker = StateTracker::new();
        tracker.register("a").await;
        tracker.register("b").await;
        tracker.set_state("a", ComponentState::Running).await;
        tracker.set_state("b", ComponentState::Stopped).await;

        let states = tracker.all_states().await;
        assert_eq!(states.len(), 2);
        assert_eq!(states.get("a"), Some(&ComponentState::Running));
        assert_eq!(states.get("b"), Some(&ComponentState::Stopped));
    }

    #[test]
    fn component_state_predicates() {
        assert!(ComponentState::Running.is_running());
        assert!(!ComponentState::Running.is_failed());
        assert!(ComponentState::Failed("err".into()).is_failed());
        assert!(ComponentState::Degraded("warn".into()).is_degraded());
        assert!(ComponentState::Stopped.is_stopped());
        assert_eq!(ComponentState::Failed("err".into()).message(), Some("err"));
        assert_eq!(ComponentState::Running.message(), None);
    }
}
