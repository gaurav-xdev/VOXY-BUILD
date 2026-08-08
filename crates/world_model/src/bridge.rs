use crate::activity::ActivityClassifier;
use crate::config::WorldModelConfig;
use crate::desktop::DesktopState;
use crate::emitter::ContextEmitter;
use crate::event::WorldModelEvent;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

pub struct DesktopEventBridge {
    watcher: Arc<DesktopWatcher>,
    emitter: Arc<ContextEmitter>,
    classifier: Arc<ActivityClassifier>,
    current_state: Arc<RwLock<BridgeState>>,
    loop_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    _event_rx: mpsc::Receiver<WorldModelEvent>,
    _context_rx: mpsc::Receiver<crate::emitter::DesktopContextUpdate>,
}

struct BridgeState {
    focused_app: Option<String>,
    activity_type: Option<String>,
    window_title: Option<String>,
    is_idle: bool,
}

use crate::watcher::DesktopWatcher;

impl DesktopEventBridge {
    pub fn new(config: WorldModelConfig) -> Self {
        let (emitter, event_rx, context_rx) = ContextEmitter::new();
        Self {
            watcher: Arc::new(DesktopWatcher::new(config)),
            emitter: Arc::new(emitter),
            classifier: Arc::new(ActivityClassifier::new()),
            current_state: Arc::new(RwLock::new(BridgeState {
                focused_app: None,
                activity_type: None,
                window_title: None,
                is_idle: false,
            })),
            loop_handle: Arc::new(RwLock::new(None)),
            _event_rx: event_rx,
            _context_rx: context_rx,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting desktop event bridge");

        self.watcher.start().await?;

        let watcher = self.watcher.clone();
        let emitter = self.emitter.clone();
        let _classifier = self.classifier.clone();
        let current_state = self.current_state.clone();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                let events = watcher.take_events().await;
                for event in events {
                    if let Err(e) = emitter.emit_raw(event.clone()).await {
                        warn!(error = %e, "Failed to emit event");
                    }

                    Self::update_state(&current_state, &event).await;

                    debug!(event = %event, "Event processed through bridge");
                }
            }
        });

        *self.loop_handle.write().await = Some(handle);

        info!("Desktop event bridge started");
        Ok(())
    }

    pub async fn stop(&self) {
        let mut handle = self.loop_handle.write().await;
        if let Some(h) = handle.take() {
            h.abort();
            info!("Desktop event bridge stopped");
        }
    }

    pub async fn get_current_context(&self) -> DesktopContext {
        let state = self.current_state.read().await;
        DesktopContext {
            focused_app: state.focused_app.clone(),
            activity_type: state.activity_type.clone(),
            window_title: state.window_title.clone(),
            is_idle: state.is_idle,
        }
    }

    pub async fn get_current_state(&self) -> DesktopState {
        self.watcher.current_snapshot().await
    }

    async fn update_state(state: &Arc<RwLock<BridgeState>>, event: &WorldModelEvent) {
        let mut guard = state.write().await;
        match event {
            WorldModelEvent::ApplicationFocused { app_id, .. } => {
                guard.focused_app = Some(app_id.clone());
                guard.is_idle = false;
            }
            WorldModelEvent::WindowChanged { window_title, .. } => {
                guard.window_title = Some(window_title.clone());
            }
            WorldModelEvent::ActivityChanged { activity_type, .. } => {
                guard.activity_type = Some(activity_type.clone());
            }
            WorldModelEvent::IdleStarted { .. } => {
                guard.is_idle = true;
            }
            WorldModelEvent::IdleEnded { new_app, .. } => {
                guard.focused_app = Some(new_app.clone());
                guard.is_idle = false;
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct DesktopContext {
    pub focused_app: Option<String>,
    pub activity_type: Option<String>,
    pub window_title: Option<String>,
    pub is_idle: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorldModelConfig;

    #[tokio::test]
    async fn test_bridge_creation() {
        let config = WorldModelConfig::default();
        let bridge = DesktopEventBridge::new(config);
        let ctx = bridge.get_current_context().await;
        assert!(ctx.focused_app.is_none());
        assert!(ctx.activity_type.is_none());
    }

    #[tokio::test]
    async fn test_bridge_state_update() {
        let state = Arc::new(RwLock::new(BridgeState {
            focused_app: None,
            activity_type: None,
            window_title: None,
            is_idle: false,
        }));

        let event = WorldModelEvent::ApplicationFocused {
            app_id: "code.exe".to_string(),
            app_name: "VS Code".to_string(),
            timestamp: chrono::Utc::now(),
        };

        DesktopEventBridge::update_state(&state, &event).await;
        let guard = state.read().await;
        assert_eq!(guard.focused_app.as_deref(), Some("code.exe"));
        assert!(!guard.is_idle);
    }
}
