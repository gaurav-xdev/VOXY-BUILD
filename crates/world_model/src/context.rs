use crate::desktop::DesktopState;
use crate::devices::ConnectedDevice;
use crate::environment::UserEnvironment;
use crate::event::WorldModelEvent;
use crate::tasks::ActiveTask;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub desktop: DesktopState,
    pub environment: UserEnvironment,
    pub devices: Vec<ConnectedDevice>,
    pub tasks: Vec<ActiveTask>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct WorldContext {
    pub snapshot: WorldSnapshot,
    pub recent_events: Vec<WorldModelEvent>,
    pub relevance_score: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopState;
    use crate::environment::UserEnvironment;
    use chrono::Utc;

    #[test]
    fn test_world_snapshot_creation() {
        let snapshot = WorldSnapshot {
            desktop: DesktopState {
                windows: vec![],
                active_window_id: None,
                workspaces: vec![],
                focused_app: None,
            },
            environment: UserEnvironment::default(),
            devices: vec![],
            tasks: vec![],
            timestamp: Utc::now(),
        };
        assert!(snapshot.desktop.windows.is_empty());
        assert!(snapshot.devices.is_empty());
        assert!(snapshot.tasks.is_empty());
    }

    #[test]
    fn test_world_context_creation() {
        let snapshot = WorldSnapshot {
            desktop: DesktopState {
                windows: vec![],
                active_window_id: None,
                workspaces: vec![],
                focused_app: None,
            },
            environment: UserEnvironment::default(),
            devices: vec![],
            tasks: vec![],
            timestamp: Utc::now(),
        };
        let context = WorldContext {
            snapshot,
            recent_events: vec![],
            relevance_score: Some(0.95),
        };
        assert!(context.recent_events.is_empty());
        assert_eq!(context.relevance_score.unwrap(), 0.95);
    }
}
