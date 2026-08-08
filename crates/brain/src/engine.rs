use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tracing::{error, info, warn};

use voxy_cognition::config::CognitionConfig;
use voxy_cognition::context::AssembledContext as CognitionAssembledContext;
use voxy_cognition::intent::IntentInput;
use voxy_cognition::orchestration::{CognitiveEngine, CognitiveResult};
use voxy_companion::types::{
    CompanionInput, CompanionOutput, MissionState, SessionId as CompanionSessionId, WeatherContext,
};
use voxy_companion::CompanionEngine;
use voxy_context::fusion::ContextFusionEngine;
use voxy_context::{ContextManager, ContextSnapshotSet};
use voxy_human_dynamics::types::{BehaviorState, HdrInput, HdrOutput, UserId};
use voxy_human_dynamics::HumanDynamicsEngine;

use crate::config::BrainConfig;
use crate::error::{BrainError, Result};
use crate::latency::{LatencySnapshot, LatencyTracker, TurnTimer};
use crate::session::{SessionManager, SessionManagerConfig};
use crate::types::*;

pub struct UnifiedBrainEngine {
    #[allow(dead_code)]
    config: BrainConfig,
    state: RwLock<BrainState>,
    context_manager: ContextManager,
    fusion_engine: ContextFusionEngine,
    companion: Mutex<CompanionEngine>,
    human_dynamics: Mutex<HumanDynamicsEngine>,
    cognition: Arc<dyn CognitiveEngine>,
    event_tx: broadcast::Sender<BrainEvent>,
    latency: RwLock<LatencyTracker>,
    shutdown_flag: Arc<AtomicBool>,
    session_manager: SessionManager,
    #[allow(dead_code)]
    session_counter: RwLock<u64>,
}

impl UnifiedBrainEngine {
    pub fn new(config: BrainConfig, cognition: Arc<dyn CognitiveEngine>) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        Self {
            context_manager: ContextManager::new(config.context.clone()),
            fusion_engine: ContextFusionEngine::default(),
            companion: Mutex::new(CompanionEngine::new(
                config.companion.clone(),
                config.companion_personality.clone(),
            )),
            human_dynamics: Mutex::new(HumanDynamicsEngine::new(config.human_dynamics.clone())),
            cognition,
            state: RwLock::new(BrainState::Idle),
            event_tx,
            latency: RwLock::new(LatencyTracker::new()),
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            session_manager: SessionManager::new(SessionManagerConfig::default()),
            session_counter: RwLock::new(0),
            config,
        }
    }

    pub async fn init(&self) -> Result<()> {
        {
            let mut state = self.state.write();
            if *state != BrainState::Idle && *state != BrainState::Shutdown {
                return Err(BrainError::AlreadyInitialized);
            }
            *state = BrainState::Idle;
        }

        self.cognition
            .init(&CognitionConfig::default())
            .await
            .map_err(|e| BrainError::CognitionError(e.to_string()))?;

        info!("UnifiedBrainEngine initialized");
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        {
            let mut state = self.state.write();
            *state = BrainState::ShuttingDown;
        }

        self.shutdown_flag.store(true, Ordering::SeqCst);

        self.cognition
            .shutdown()
            .await
            .map_err(|e| BrainError::CognitionError(e.to_string()))?;

        {
            let mut state = self.state.write();
            *state = BrainState::Shutdown;
        }

        info!("UnifiedBrainEngine shutdown complete");
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<BrainEvent> {
        self.event_tx.subscribe()
    }

    pub fn emit_event(&self, event: BrainEvent) {
        let _ = self.event_tx.send(event);
    }

    pub async fn process_turn(&self, input: BrainInput) -> Result<BrainOutput> {
        {
            let state = self.state.read();
            match *state {
                BrainState::ShuttingDown | BrainState::Shutdown => {
                    return Err(BrainError::ShutdownInProgress);
                }
                _ => {}
            }
        }

        if self.shutdown_flag.load(Ordering::SeqCst) {
            return Err(BrainError::ShutdownInProgress);
        }

        let turn_id = TurnId::new();
        let start = Instant::now();

        {
            let mut state = self.state.write();
            *state = BrainState::Processing;
        }

        self.emit_event(BrainEvent::TurnStarted {
            turn_id: turn_id.0.clone(),
            session_id: input.session_id.0.clone(),
        });

        // Create or touch session via SessionManager (handles TTL + capacity)
        self.session_manager.get_or_create(&input.session_id);

        let mut timer = self.latency.write().start_turn();

        let context_result = self.collect_context(&mut timer).await;

        let companion_result = self
            .update_companion(&input, &context_result, &mut timer)
            .await;

        let hdr_result = self
            .update_human_dynamics(&input, &companion_result, &mut timer)
            .await;

        let cognition_result = self
            .process_cognition(&input, &context_result, &hdr_result, &mut timer)
            .await;

        let mut snapshot = timer.finish();
        snapshot.context_us = context_result.1;
        snapshot.companion_us = companion_result.1;
        snapshot.hdr_us = hdr_result.1;
        snapshot.cognition_us = cognition_result.as_ref().map(|r| r.1).unwrap_or(0);
        snapshot.overhead_us = snapshot.total_us.saturating_sub(
            snapshot.context_us + snapshot.companion_us + snapshot.hdr_us + snapshot.cognition_us,
        );

        self.latency.write().record(snapshot.clone());

        let response_text = cognition_result
            .as_ref()
            .and_then(|(r, _)| extract_response_text(r));

        let output = BrainOutput {
            turn_id: turn_id.clone(),
            session_id: input.session_id.clone(),
            response_text,
            cognitive_result: cognition_result.as_ref().map(|(r, _)| CognitiveSummary {
                intent_type: format!("{:?}", r.intent.intent_type),
                confidence: r.confidence.value,
                success: r.success,
                duration_ms: r.duration_ms,
                plan_steps: r.plan.as_ref().map(|p| p.steps.len()).unwrap_or(0),
                errors: r.errors.clone(),
            }),
            companion: companion_result.0.as_ref().map(|c| CompanionSummary {
                display: c.display.clone(),
                silence: c.silence,
                presence_score: c.presence_score,
                greeting: c.greeting.is_some(),
                micro_interaction: c.micro_interaction.is_some(),
                latency_us: c.update_latency_us,
            }),
            human_dynamics: hdr_result.0.as_ref().map(|h| HdrSummary {
                trust_score: h.trust_score,
                relationship_level: format!("{:?}", h.relationship_level),
                behavior_state: format!("{:?}", h.behavior_state),
                autonomy_level: h.autonomy_level,
                protection_allowed: h.protection_decision.allowed,
                policy_violations: h.policy_violations.len(),
                latency_us: h.update_latency_us,
            }),
            context_summary: context_result.0.as_ref().map(|c| ContextSummary {
                source_count: c.len(),
                confidence: 0.85,
                collection_time_ms: c.collection_time_ms,
            }),
            pipeline_duration_ms: start.elapsed().as_millis() as u64,
            stage_latencies: snapshot,
            interrupted: {
                let state = self.state.read();
                *state == BrainState::Interrupted
            },
            errors: Vec::new(),
        };

        {
            let mut state = self.state.write();
            *state = BrainState::Idle;
        }

        // Record turn in session manager
        self.session_manager.record_turn(&input.session_id);

        self.emit_event(BrainEvent::TurnCompleted {
            turn_id: turn_id.0,
            total_duration_ms: output.pipeline_duration_ms,
        });

        Ok(output)
    }

    async fn collect_context(&self, timer: &mut TurnTimer) -> (Option<ContextSnapshotSet>, u64) {
        self.emit_event(BrainEvent::ContextCollecting);
        timer.begin_context();

        let result = self.context_manager.collect().await;

        let duration = timer.end_context();

        match result {
            Ok(snapshots) => {
                self.emit_event(BrainEvent::ContextCollected {
                    source_count: snapshots.len(),
                    duration_ms: duration / 1000,
                });
                (Some(snapshots), duration)
            }
            Err(e) => {
                warn!("Context collection failed: {}", e);
                self.emit_event(BrainEvent::ContextCollected {
                    source_count: 0,
                    duration_ms: duration / 1000,
                });
                (None, duration)
            }
        }
    }

    async fn update_companion(
        &self,
        input: &BrainInput,
        _context: &(Option<ContextSnapshotSet>, u64),
        timer: &mut TurnTimer,
    ) -> (Option<CompanionOutput>, u64) {
        self.emit_event(BrainEvent::CompanionUpdating);
        timer.begin_companion();

        let companion_input = CompanionInput {
            now: chrono::Utc::now(),
            session_id: CompanionSessionId(input.session_id.0.clone()),
            user_presence: input.user_presence.clone(),
            current_activity: None,
            time_since_last_interaction: input.time_since_last_interaction,
            conversation_count_this_session: 0,
            total_session_duration: input.session_duration,
            active_goals: Vec::new(),
            recent_milestones: Vec::new(),
            weather: WeatherContext::Unknown,
            stress_estimate: input.stress_level,
            idle_duration: input.time_since_last_interaction,
            pending_tasks: 0,
            completed_tasks_today: 0,
            last_greeting: None,
            last_micro_interaction: None,
            last_memory_reference: None,
            mission_state: MissionState::Idle,
            focus_level: input.focus_level,
        };

        let result = {
            let mut companion = self.companion.lock().await;
            companion.update(&companion_input)
        };

        let duration = timer.end_companion();

        self.emit_event(BrainEvent::CompanionUpdated {
            display: result.display.clone(),
            silence: result.silence,
            duration_ms: duration / 1000,
        });

        (Some(result), duration)
    }

    async fn update_human_dynamics(
        &self,
        input: &BrainInput,
        _companion: &(Option<CompanionOutput>, u64),
        timer: &mut TurnTimer,
    ) -> (Option<HdrOutput>, u64) {
        self.emit_event(BrainEvent::HdrUpdating);
        timer.begin_hdr();

        let hdr_input = HdrInput {
            now: chrono::Utc::now(),
            instant_now: Instant::now(),
            user_id: UserId(input.session_id.0.clone()),
            user_present: true,
            current_behavior: BehaviorState::Observing,
            activity_description: String::new(),
            pending_action: None,
            recent_trust_events: Vec::new(),
            time_since_last_interaction: input.time_since_last_interaction,
            session_duration: input.session_duration,
            errors_this_session: input.errors_this_session,
            corrections_this_session: 0,
            missions_completed: input.missions_completed,
            missions_failed: input.missions_failed,
            is_meeting: input.is_meeting,
            focus_level: input.focus_level,
            stress_level: input.stress_level,
        };

        let result = {
            let mut hdr = self.human_dynamics.lock().await;
            hdr.update(&hdr_input)
        };

        let duration = timer.end_hdr();

        self.emit_event(BrainEvent::HdrUpdated {
            trust_score: result.trust_score,
            protection_allowed: result.protection_decision.allowed,
            duration_ms: duration / 1000,
        });

        (Some(result), duration)
    }

    async fn process_cognition(
        &self,
        input: &BrainInput,
        context: &(Option<ContextSnapshotSet>, u64),
        _hdr: &(Option<HdrOutput>, u64),
        timer: &mut TurnTimer,
    ) -> Option<(CognitiveResult, u64)> {
        self.emit_event(BrainEvent::CognitionProcessing);
        timer.begin_cognition();

        let intent_input = IntentInput {
            raw_text: input.raw_text.clone(),
            context: None,
            source: "brain".to_string(),
            metadata: input.metadata.clone(),
        };

        let result = if let Some(snapshots) = &context.0 {
            let snapshot_vec: Vec<_> = snapshots.snapshots.values().cloned().collect();
            let fusion_assembled = self.fusion_engine.fuse(snapshot_vec);
            let cognition_assembled =
                CognitionAssembledContext::from_fusion_context(&fusion_assembled, None);
            self.cognition
                .process_with_context(&intent_input, &cognition_assembled)
                .await
        } else {
            self.cognition.process(&intent_input).await
        };

        let duration = timer.end_cognition();

        match result {
            Ok(cognitive_result) => {
                self.emit_event(BrainEvent::CognitionProcessed {
                    intent_type: format!("{:?}", cognitive_result.intent.intent_type),
                    confidence: cognitive_result.confidence.value,
                    duration_ms: duration / 1000,
                });
                Some((cognitive_result, duration))
            }
            Err(e) => {
                error!("Cognition processing failed: {}", e);
                self.emit_event(BrainEvent::TurnFailed {
                    turn_id: String::new(),
                    error: e.to_string(),
                });
                None
            }
        }
    }

    pub async fn cancel_turn(&self, session_id: &SessionId) -> Result<()> {
        let state = self.state.read();
        if *state != BrainState::Processing {
            return Ok(());
        }

        {
            let mut state = self.state.write();
            *state = BrainState::Interrupted;
        }

        self.emit_event(BrainEvent::TurnInterrupted {
            turn_id: String::new(),
            reason: format!("Cancelled by session {}", session_id.0),
        });

        Ok(())
    }

    pub async fn health_check(&self) -> Result<HealthReport> {
        let state = self.state.read().clone();
        let latency = self.latency.read().average();
        let session_stats = self.session_manager.stats();

        let cognition_health = self
            .cognition
            .diagnostics()
            .await
            .map(|d| {
                let healthy = d.get("healthy").and_then(|v| v.as_bool()).unwrap_or(true);
                if healthy {
                    ComponentHealth::Healthy
                } else {
                    ComponentHealth::Degraded("Cognition degraded".into())
                }
            })
            .unwrap_or_else(|e| ComponentHealth::Unhealthy(e.to_string()));

        Ok(HealthReport {
            state,
            active_sessions: session_stats.active_sessions,
            total_turns: session_stats.total_turns,
            latency,
            components: {
                let mut m = HashMap::new();
                m.insert("cognition".into(), cognition_health);
                m.insert("context".into(), ComponentHealth::Healthy);
                m.insert("companion".into(), ComponentHealth::Healthy);
                m.insert("human_dynamics".into(), ComponentHealth::Healthy);
                m
            },
        })
    }

    pub fn state(&self) -> BrainState {
        self.state.read().clone()
    }

    pub fn latency(&self) -> LatencySnapshot {
        self.latency.read().current().clone()
    }

    pub fn latency_average(&self) -> LatencySnapshot {
        self.latency.read().average()
    }
}

fn extract_response_text(result: &CognitiveResult) -> Option<String> {
    result
        .result
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| {
            result
                .result
                .get("response")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            result
                .result
                .get("text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub state: BrainState,
    pub active_sessions: usize,
    pub total_turns: usize,
    pub latency: LatencySnapshot,
    pub components: HashMap<String, ComponentHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentHealth {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
