use crate::config::IntelligenceConfig;
use crate::conversation::{ConversationTurn, Speaker};
use crate::decision::Decision;
use crate::emotional::{EmotionalSignal, SignalType};
use crate::experience::{ExperienceLayer, ExperienceSnapshot, VoiceParameters};
use crate::memory_importance::{MemoryItem as CiMemoryItem, MemoryType as CiMemoryType};
use crate::personality_dynamics::Mood;
use crate::presence_engine::{PresenceEvent, PresenceEventType, PresenceState};
use crate::proactive::ProactiveSuggestion;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

#[derive(Debug, Clone)]
pub enum ExperienceInput {
    VoiceTranscript {
        text: String,
        is_final: bool,
    },
    VoiceActivity {
        active: bool,
    },
    DesktopFocusChanged {
        app: String,
        window_title: Option<String>,
    },
    DesktopActivityChanged {
        activity: String,
    },
    DesktopIdle {
        is_idle: bool,
    },
    MemoryStored {
        memory_id: String,
        importance: f64,
    },
    CognitionResult {
        intent: String,
        confidence: f64,
    },
    SystemEvent {
        event_type: String,
        data: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub struct ExperienceOutput {
    pub voice_params: VoiceParameters,
    pub presence_state: PresenceState,
    pub current_mood: Mood,
    pub mood_intensity: f64,
    pub active_suggestions: Vec<ProactiveSuggestion>,
    pub conversation_context: String,
    pub decision: Option<Decision>,
    pub emotional_snapshot: crate::emotional::EmotionalSnapshot,
}

pub struct ExperienceBridge {
    layer: Arc<RwLock<ExperienceLayer>>,
    input_tx: broadcast::Sender<ExperienceInput>,
    output_tx: broadcast::Sender<ExperienceOutput>,
    running: Arc<std::sync::atomic::AtomicBool>,
    task_handle: std::sync::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl ExperienceBridge {
    pub fn new(
        config: IntelligenceConfig,
    ) -> (
        Self,
        broadcast::Sender<ExperienceInput>,
        broadcast::Receiver<ExperienceOutput>,
    ) {
        let (input_tx, _) = broadcast::channel(256);
        let (output_tx, output_rx) = broadcast::channel(64);

        let layer = Arc::new(RwLock::new(ExperienceLayer::new(config)));

        let bridge = Self {
            layer,
            input_tx: input_tx.clone(),
            output_tx: output_tx.clone(),
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            task_handle: std::sync::Mutex::new(None),
        };

        (bridge, input_tx, output_rx)
    }

    pub fn input_sender(&self) -> broadcast::Sender<ExperienceInput> {
        self.input_tx.clone()
    }

    pub async fn start(&self) {
        let layer = self.layer.clone();
        let mut input_rx = self.input_tx.subscribe();
        let output_tx = self.output_tx.clone();
        let running = self.running.clone();

        info!("Starting ExperienceBridge");

        let handle = tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(500));
            let mut last_snapshot = ExperienceSnapshot {
                emotional: crate::emotional::EmotionalSnapshot {
                    primary: crate::emotional::EmotionState {
                        emotion: crate::emotional::EmotionType::Calm,
                        confidence: 0.5,
                        valence: 0.5,
                        arousal: 0.3,
                    },
                    secondary: None,
                    timestamp: chrono::Utc::now(),
                },
                presence: crate::presence_engine::PresenceSnapshot {
                    state: PresenceState::Sleeping,
                    duration_ms: 0,
                    power_level: 0.1,
                    last_transition: chrono::Utc::now(),
                    event_count: 0,
                },
                current_mood: Mood::Calm,
                mood_intensity: 0.5,
                active_suggestions: 0,
                memory_count: 0,
                conversation_depth: 0,
            };

            loop {
                tokio::select! {
                    input = input_rx.recv() => {
                        if !running.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }

                        match input {
                            Ok(input) => {
                                let mut layer_guard = layer.write().await;
                                Self::process_input(&mut layer_guard, input).await;
                                let snapshot = layer_guard.get_snapshot();

                                if Self::snapshot_changed(&last_snapshot, &snapshot) {
                                    last_snapshot = snapshot.clone();
                                    let output = ExperienceOutput {
                                        voice_params: layer_guard.voice_parameters(),
                                        presence_state: snapshot.presence.state,
                                        current_mood: snapshot.current_mood,
                                        mood_intensity: snapshot.mood_intensity,
                                        active_suggestions: Vec::new(),
                                        conversation_context: String::new(),
                                        decision: None,
                                        emotional_snapshot: snapshot.emotional,
                                    };

                                    let _ = output_tx.send(output);
                                }
                            }
                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                warn!(missed = n, "ExperienceBridge input lagged");
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                break;
                            }
                        }
                    }
                    _ = tick_interval.tick() => {
                        if !running.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }

                        let layer_guard = layer.read().await;
                        let snapshot = layer_guard.get_snapshot();

                        if Self::snapshot_changed(&last_snapshot, &snapshot) {
                            last_snapshot = snapshot.clone();
                            let output = ExperienceOutput {
                                voice_params: layer_guard.voice_parameters(),
                                presence_state: snapshot.presence.state,
                                current_mood: snapshot.current_mood,
                                mood_intensity: snapshot.mood_intensity,
                                active_suggestions: Vec::new(),
                                conversation_context: String::new(),
                                decision: None,
                                emotional_snapshot: snapshot.emotional,
                            };

                            let _ = output_tx.send(output);
                        }
                    }
                }
            }

            info!("ExperienceBridge stopped");
        });

        *self.task_handle.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);
    }

    pub async fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let handle = self
            .task_handle
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take();
        if let Some(handle) = handle {
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
                Ok(Ok(())) => info!("ExperienceBridge task stopped cleanly"),
                Ok(Err(e)) => warn!("ExperienceBridge task panicked: {e}"),
                Err(_) => warn!("ExperienceBridge task did not stop in 5s"),
            }
        }
    }

    pub async fn get_snapshot(&self) -> ExperienceSnapshot {
        let layer = self.layer.read().await;
        layer.get_snapshot()
    }

    pub async fn get_voice_parameters(&self) -> VoiceParameters {
        let layer = self.layer.read().await;
        layer.voice_parameters()
    }

    async fn process_input(layer: &mut ExperienceLayer, input: ExperienceInput) {
        match input {
            ExperienceInput::VoiceTranscript { text, is_final } => {
                let _span = tracing::debug_span!("experience_voice_transcript").entered();
                debug!(text = %text, is_final, "Processing voice transcript");

                if is_final && !text.is_empty() {
                    let turn = ConversationTurn {
                        id: uuid::Uuid::new_v4().to_string(),
                        speaker: Speaker::User,
                        text: text.clone(),
                        timestamp: chrono::Utc::now(),
                        topics: Vec::new(),
                        entities: Vec::new(),
                        sentiment: 0.5,
                    };
                    layer.process_conversation_turn(turn);

                    let signal = EmotionalSignal {
                        signal_type: SignalType::VoiceTone,
                        intensity: 0.6,
                        timestamp: chrono::Utc::now(),
                        source: "voice".to_string(),
                    };
                    layer.process_emotional_signal(signal);

                    let event = PresenceEvent {
                        event_type: PresenceEventType::UserSpoke,
                        timestamp: chrono::Utc::now(),
                        source: "voice".to_string(),
                        data: Some(text),
                    };
                    layer.process_presence_event(event);
                }
            }
            ExperienceInput::VoiceActivity { active } => {
                let _span = tracing::debug_span!("experience_voice_activity").entered();
                debug!(active, "Processing voice activity");

                if active {
                    let event = PresenceEvent {
                        event_type: PresenceEventType::VoiceActivated,
                        timestamp: chrono::Utc::now(),
                        source: "voice".to_string(),
                        data: None,
                    };
                    layer.process_presence_event(event);
                }
            }
            ExperienceInput::DesktopFocusChanged {
                app,
                window_title: _,
            } => {
                let _span = tracing::debug_span!("experience_desktop_focus").entered();
                debug!(app = %app, "Processing desktop focus change");

                let signal = EmotionalSignal {
                    signal_type: SignalType::WindowSwitching,
                    intensity: 0.5,
                    timestamp: chrono::Utc::now(),
                    source: "desktop".to_string(),
                };
                layer.process_emotional_signal(signal);

                let event = PresenceEvent {
                    event_type: PresenceEventType::ActivityDetected,
                    timestamp: chrono::Utc::now(),
                    source: "desktop".to_string(),
                    data: Some(app),
                };
                layer.process_presence_event(event);
            }
            ExperienceInput::DesktopActivityChanged { activity } => {
                let _span = tracing::debug_span!("experience_desktop_activity").entered();
                debug!(activity = %activity, "Processing desktop activity change");

                let signal = EmotionalSignal {
                    signal_type: match activity.as_str() {
                        "coding" => SignalType::RapidTyping,
                        "browsing" => SignalType::ScrollSpeed,
                        _ => SignalType::WindowSwitching,
                    },
                    intensity: 0.6,
                    timestamp: chrono::Utc::now(),
                    source: "desktop".to_string(),
                };
                layer.process_emotional_signal(signal);
            }
            ExperienceInput::DesktopIdle { is_idle } => {
                let _span = tracing::debug_span!("experience_desktop_idle").entered();
                debug!(is_idle, "Processing desktop idle state");

                let event = PresenceEvent {
                    event_type: if is_idle {
                        PresenceEventType::UserIdle
                    } else {
                        PresenceEventType::UserReturned
                    },
                    timestamp: chrono::Utc::now(),
                    source: "desktop".to_string(),
                    data: None,
                };
                layer.process_presence_event(event);
            }
            ExperienceInput::MemoryStored {
                memory_id,
                importance,
            } => {
                let _span = tracing::debug_span!("experience_memory_stored").entered();
                debug!(memory_id = %memory_id, importance, "Processing memory stored");

                let ci_memory = CiMemoryItem {
                    id: memory_id.clone(),
                    memory_type: CiMemoryType::Episodic,
                    content: String::new(),
                    created_at: chrono::Utc::now(),
                    last_accessed: chrono::Utc::now(),
                    access_count: 0,
                    decay_rate: 0.0,
                    semantic_value: importance,
                    project_value: 0.5,
                    emotional_weight: 0.3,
                    tags: Vec::new(),
                    context: std::collections::HashMap::new(),
                };
                layer.add_memory(ci_memory);
            }
            ExperienceInput::CognitionResult { intent, confidence } => {
                let _span = tracing::debug_span!("experience_cognition_result").entered();
                debug!(intent = %intent, confidence, "Processing cognition result");

                let signal = EmotionalSignal {
                    signal_type: SignalType::TaskCompletion,
                    intensity: confidence,
                    timestamp: chrono::Utc::now(),
                    source: "cognition".to_string(),
                };
                layer.process_emotional_signal(signal);
            }
            ExperienceInput::SystemEvent { event_type, data } => {
                let _span = tracing::debug_span!("experience_system_event").entered();
                debug!(event_type = %event_type, "Processing system event");

                let event = PresenceEvent {
                    event_type: match event_type.as_str() {
                        "emergency" => PresenceEventType::EmergencyAlert,
                        "task_complete" => PresenceEventType::TaskCompleted,
                        "achievement" => PresenceEventType::AchievementUnlocked,
                        _ => PresenceEventType::SystemEvent,
                    },
                    timestamp: chrono::Utc::now(),
                    source: "system".to_string(),
                    data,
                };
                layer.process_presence_event(event);
            }
        }
    }

    fn snapshot_changed(last: &ExperienceSnapshot, current: &ExperienceSnapshot) -> bool {
        last.current_mood != current.current_mood
            || (last.mood_intensity - current.mood_intensity).abs() > 0.1
            || last.presence.state != current.presence.state
            || last.conversation_depth != current.conversation_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_experience_bridge_creation() {
        let config = IntelligenceConfig::default();
        let (bridge, _tx, _rx) = ExperienceBridge::new(config);
        let snapshot = bridge.get_snapshot().await;
        assert_eq!(snapshot.memory_count, 0);
    }

    #[tokio::test]
    async fn test_experience_bridge_start_stop() {
        let config = IntelligenceConfig::default();
        let (bridge, _tx, _rx) = ExperienceBridge::new(config);
        bridge.start().await;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        bridge.stop().await;
    }

    #[tokio::test]
    async fn test_experience_bridge_voice_input() {
        let config = IntelligenceConfig::default();
        let (bridge, tx, mut rx) = ExperienceBridge::new(config);
        bridge.start().await;

        let _ = tx.send(ExperienceInput::VoiceTranscript {
            text: "hello".to_string(),
            is_final: true,
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        if let Ok(output) = rx.try_recv() {
            assert!(output.voice_params.speed > 0.0);
        }

        bridge.stop().await;
    }

    #[tokio::test]
    async fn test_experience_bridge_desktop_input() {
        let config = IntelligenceConfig::default();
        let (bridge, tx, _rx) = ExperienceBridge::new(config);
        bridge.start().await;

        let _ = tx.send(ExperienceInput::DesktopFocusChanged {
            app: "code.exe".to_string(),
            window_title: Some("main.rs".to_string()),
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let snapshot = bridge.get_snapshot().await;
        assert!(snapshot.mood_intensity >= 0.0);

        bridge.stop().await;
    }

    #[test]
    fn test_snapshot_change_detection() {
        let snapshot1 = ExperienceSnapshot {
            emotional: crate::emotional::EmotionalSnapshot {
                primary: crate::emotional::EmotionState {
                    emotion: crate::emotional::EmotionType::Calm,
                    confidence: 0.5,
                    valence: 0.5,
                    arousal: 0.3,
                },
                secondary: None,
                timestamp: chrono::Utc::now(),
            },
            presence: crate::presence_engine::PresenceSnapshot {
                state: PresenceState::Sleeping,
                duration_ms: 0,
                power_level: 0.1,
                last_transition: chrono::Utc::now(),
                event_count: 0,
            },
            current_mood: Mood::Calm,
            mood_intensity: 0.5,
            active_suggestions: 0,
            memory_count: 0,
            conversation_depth: 0,
        };

        let mut snapshot2 = snapshot1.clone();
        assert!(!ExperienceBridge::snapshot_changed(&snapshot1, &snapshot2));

        snapshot2.current_mood = Mood::Cheerful;
        assert!(ExperienceBridge::snapshot_changed(&snapshot1, &snapshot2));
    }
}
