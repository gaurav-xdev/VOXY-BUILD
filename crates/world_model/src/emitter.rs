use crate::desktop::DesktopState;
use crate::event::WorldModelEvent;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopContextUpdate {
    pub window_count: usize,
    pub focused_app: Option<String>,
    pub active_window_title: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationLifecycleEvent {
    pub app_id: String,
    pub app_name: String,
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct ContextEmitter {
    event_tx: mpsc::Sender<WorldModelEvent>,
    context_tx: mpsc::Sender<DesktopContextUpdate>,
}

const EMITTER_CHANNEL_CAPACITY: usize = 256;

impl ContextEmitter {
    pub fn new() -> (
        Self,
        mpsc::Receiver<WorldModelEvent>,
        mpsc::Receiver<DesktopContextUpdate>,
    ) {
        let (event_tx, event_rx) = mpsc::channel(EMITTER_CHANNEL_CAPACITY);
        let (context_tx, context_rx) = mpsc::channel(EMITTER_CHANNEL_CAPACITY);

        (
            Self {
                event_tx,
                context_tx,
            },
            event_rx,
            context_rx,
        )
    }

    pub fn emit_desktop_updated(
        &self,
        state: &DesktopState,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::DesktopUpdated {
            window_count: state.windows.len(),
            focused_app: state.focused_app.clone(),
        };

        let _ = self.context_tx.try_send(DesktopContextUpdate {
            window_count: state.windows.len(),
            focused_app: state.focused_app.clone(),
            active_window_title: state
                .windows
                .iter()
                .find(|w| w.is_focused)
                .map(|w| w.title.clone()),
            timestamp: chrono::Utc::now(),
        });

        debug!(
            window_count = state.windows.len(),
            focused_app = ?state.focused_app,
            "Emitted desktop updated event"
        );

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_application_launched(
        &self,
        app_id: &str,
        app_name: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::ApplicationLaunched {
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
        };

        let _ = self.context_tx.try_send(DesktopContextUpdate {
            window_count: 0,
            focused_app: Some(app_id.to_string()),
            active_window_title: None,
            timestamp: chrono::Utc::now(),
        });

        info!(app_id, app_name, "Emitted application launched event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_application_closed(
        &self,
        app_id: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::ApplicationClosed {
            app_id: app_id.to_string(),
        };

        let _ = self.context_tx.try_send(DesktopContextUpdate {
            window_count: 0,
            focused_app: None,
            active_window_title: None,
            timestamp: chrono::Utc::now(),
        });

        info!(app_id, "Emitted application closed event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_device_connected(
        &self,
        device_id: &str,
        device_type: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::DeviceConnected {
            device_id: device_id.to_string(),
            device_type: device_type.to_string(),
        };

        info!(device_id, device_type, "Emitted device connected event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_device_disconnected(
        &self,
        device_id: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::DeviceDisconnected {
            device_id: device_id.to_string(),
        };

        info!(device_id, "Emitted device disconnected event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_task_created(
        &self,
        task_id: &str,
        description: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::TaskCreated {
            task_id: task_id.to_string(),
            description: description.to_string(),
        };

        info!(task_id, description, "Emitted task created event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_task_updated(
        &self,
        task_id: &str,
        status: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::TaskUpdated {
            task_id: task_id.to_string(),
            status: status.to_string(),
        };

        debug!(task_id, status, "Emitted task updated event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_task_completed(
        &self,
        task_id: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::TaskCompleted {
            task_id: task_id.to_string(),
        };

        info!(task_id, "Emitted task completed event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub fn emit_environment_changed(
        &self,
        description: &str,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        let event = WorldModelEvent::EnvironmentChanged {
            description: description.to_string(),
        };

        info!(description, "Emitted environment changed event");

        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }

    pub async fn emit_raw(
        &self,
        event: WorldModelEvent,
    ) -> Result<(), mpsc::error::SendError<WorldModelEvent>> {
        self.event_tx
            .try_send(event)
            .map_err(|e| mpsc::error::SendError(e.into_inner()))
    }
}

impl Default for ContextEmitter {
    fn default() -> Self {
        let (emitter, _, _) = Self::new();
        emitter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::DesktopState;

    #[test]
    fn test_emitter_creation() {
        let (_emitter, _event_rx, _context_rx) = ContextEmitter::new();
    }

    #[test]
    fn test_emit_desktop_updated() {
        let (emitter, mut event_rx, _) = ContextEmitter::new();
        let state = DesktopState {
            windows: vec![],
            active_window_id: None,
            workspaces: vec![],
            focused_app: None,
        };

        emitter.emit_desktop_updated(&state).unwrap();
        let event = event_rx.try_recv().unwrap();
        let display = format!("{}", event);
        assert!(display.contains("Desktop updated"));
    }

    #[test]
    fn test_emit_application_launched() {
        let (emitter, mut event_rx, _) = ContextEmitter::new();
        emitter
            .emit_application_launched("code.exe", "VS Code")
            .unwrap();
        let event = event_rx.try_recv().unwrap();
        let display = format!("{}", event);
        assert!(display.contains("Application launched"));
    }
}
