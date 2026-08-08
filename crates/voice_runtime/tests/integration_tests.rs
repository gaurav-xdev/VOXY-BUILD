use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use voxy_brain::types::*;
use voxy_brain::*;
use voxy_cognition::attention::{AttentionRecommendation, AttentionScore};
use voxy_cognition::config::CognitionConfig;
use voxy_cognition::context::AssembledContext;
use voxy_cognition::error::Result as CognitionResult;
use voxy_cognition::intent::{IntentAnalysis, IntentInput, IntentType};
use voxy_cognition::orchestration::{CognitiveEngine, CognitiveResult, DecisionOutput};
use voxy_cognition::planner::Plan;
use voxy_cognition::types::Urgency;
use voxy_cognition::types::{ConfidenceScore, IntentId, PlanId};
use voxy_voice_runtime::*;

struct MockCognitiveEngine;

#[async_trait]
impl CognitiveEngine for MockCognitiveEngine {
    async fn init(&self, _config: &CognitionConfig) -> CognitionResult<()> {
        Ok(())
    }
    async fn shutdown(&self) -> CognitionResult<()> {
        Ok(())
    }
    async fn process(&self, input: &IntentInput) -> CognitionResult<CognitiveResult> {
        Ok(CognitiveResult {
            intent: IntentAnalysis {
                intent_id: IntentId("mock-intent".into()),
                intent_type: IntentType::Query,
                confidence: ConfidenceScore::new(0.85).unwrap(),
                primary_action: "respond".into(),
                parameters: HashMap::new(),
                requires_planning: false,
                requires_reasoning: false,
                urgency: Urgency::Medium,
                alternate_interpretations: Vec::new(),
                timestamp: chrono::Utc::now(),
            },
            plan: Some(Plan {
                id: PlanId("mock-plan".into()),
                goals: Vec::new(),
                steps: Vec::new(),
                state: voxy_cognition::planner::PlanState::Draft,
                estimated_total_duration_ms: 100,
                parallelism_possible: false,
                fallback_plan_id: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }),
            context: None,
            result: serde_json::json!({"response": format!("Mock: {}", input.raw_text)}),
            confidence: ConfidenceScore::new(0.85).unwrap(),
            reflection: None,
            duration_ms: 5,
            success: true,
            errors: Vec::new(),
            decision: Some(DecisionOutput {
                decision: "respond".into(),
                confidence: ConfidenceScore::new(0.9).unwrap(),
                reason: "user asked".into(),
                context_summary: "mock".into(),
                priority: "normal".into(),
                timestamp: chrono::Utc::now(),
            }),
            attention_score: Some(AttentionScore {
                score: 0.7,
                breakdown: HashMap::new(),
                confidence: ConfidenceScore::new(0.8).unwrap(),
                recommendation: AttentionRecommendation::FocusPrimary,
            }),
        })
    }
    async fn process_streaming(&self, input: &IntentInput) -> CognitionResult<CognitiveResult> {
        self.process(input).await
    }
    async fn state(&self) -> voxy_cognition::types::CognitiveState {
        voxy_cognition::types::CognitiveState::Idle
    }
    async fn current_intent(&self) -> Option<IntentId> {
        None
    }
    async fn current_plan(&self) -> Option<PlanId> {
        None
    }
    async fn cancel(&self, _intent_id: &IntentId) -> CognitionResult<()> {
        Ok(())
    }
    async fn pause(&self) -> CognitionResult<()> {
        Ok(())
    }
    async fn resume(&self) -> CognitionResult<()> {
        Ok(())
    }
    async fn process_with_context(
        &self,
        input: &IntentInput,
        _context: &AssembledContext,
    ) -> CognitionResult<CognitiveResult> {
        self.process(input).await
    }
    async fn diagnostics(&self) -> CognitionResult<serde_json::Value> {
        Ok(serde_json::json!({"healthy": true}))
    }
}

fn make_runtime() -> VoiceRuntimeEngine {
    let config = VoiceRuntimeConfig::default();
    let brain = UnifiedBrainEngine::new(BrainConfig::default(), Arc::new(MockCognitiveEngine));
    VoiceRuntimeEngine::new(config, brain)
}

#[tokio::test]
async fn test_runtime_creation() {
    let runtime = make_runtime();
    assert_eq!(runtime.state().await, VoiceRuntimeState::Idle);
    assert!(!runtime.is_running());
    assert!(!runtime.is_speaking());
}

#[tokio::test]
async fn test_runtime_init_shutdown() {
    let runtime = make_runtime();
    assert!(runtime.init().await.is_ok());
    assert_eq!(runtime.state().await, VoiceRuntimeState::Idle);
    assert!(runtime.shutdown().await.is_ok());
    assert_eq!(runtime.state().await, VoiceRuntimeState::Shutdown);
}

#[tokio::test]
async fn test_runtime_double_init() {
    let runtime = make_runtime();
    runtime.init().await.unwrap();
    let err = runtime.init().await.unwrap_err();
    assert!(matches!(err, VoiceRuntimeError::AlreadyInitialized));
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_runtime_shutdown_without_init() {
    let runtime = make_runtime();
    let err = runtime.shutdown().await.unwrap_err();
    assert!(matches!(err, VoiceRuntimeError::CaptureError(_)));
}

#[tokio::test]
async fn test_runtime_session_id() {
    let runtime = make_runtime();
    let id = runtime.session_id().await;
    assert!(!id.0.is_empty());

    let new_id = VoiceSessionId("test-session".into());
    runtime.set_session_id(new_id.clone()).await;
    assert_eq!(runtime.session_id().await.0, "test-session");
}

#[tokio::test]
async fn test_runtime_subscribe_events() {
    let runtime = make_runtime();
    let _rx = runtime.subscribe_events();
}

#[tokio::test]
async fn test_runtime_latency() {
    let runtime = make_runtime();
    let latency = runtime.latency().await;
    assert_eq!(latency.total_us, 0);
    assert_eq!(latency.wake_word_us, 0);
    assert_eq!(latency.vad_us, 0);
    assert_eq!(latency.stt_us, 0);
    assert_eq!(latency.brain_us, 0);
    assert_eq!(latency.tts_us, 0);
}

#[tokio::test]
async fn test_runtime_config() {
    let runtime = make_runtime();
    let config = runtime.config();
    assert!(config.echo_cancellation_enabled);
    assert!(config.barge_in.enabled);
    assert!(config.streaming.enabled);
    assert!(config.latency_tracking_enabled);
    assert_eq!(config.echo_cancellation_tail_ms, 128);
}

#[tokio::test]
async fn test_runtime_event_count() {
    let runtime = make_runtime();
    assert_eq!(runtime.event_count(), 0);
}

#[tokio::test]
async fn test_runtime_is_in_turn() {
    let runtime = make_runtime();
    assert!(!runtime.is_in_turn());
}

#[tokio::test]
async fn test_turn_detector_basic() {
    let config = TurnDetectionConfig::default();
    let detector = TurnDetector::new(config);

    assert!(!detector.is_in_turn());

    let boundary = detector.process_frame(true).await;
    assert_eq!(boundary, TurnBoundary::None);
    assert!(detector.is_in_turn());

    let boundary = detector.process_frame(false).await;
    assert_eq!(boundary, TurnBoundary::None);

    detector.reset().await;
    assert!(!detector.is_in_turn());
}

#[tokio::test]
async fn test_turn_detector_end_of_utterance() {
    let config = TurnDetectionConfig {
        end_of_utterance_silence_ms: 100,
        ..Default::default()
    };
    let detector = TurnDetector::new(config);

    detector.process_frame(true).await;
    assert!(detector.is_in_turn());

    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

    let boundary = detector.process_frame(false).await;
    assert_eq!(boundary, TurnBoundary::EndOfUtterance);
    assert!(!detector.is_in_turn());
}

#[tokio::test]
async fn test_echo_canceller_basic() {
    let canceller = EchoCanceller::new(true, 128, 16000);
    assert!(canceller.is_enabled());

    let mut audio = vec![0.1, 0.2, 0.3, 0.4];
    canceller.process_capture(&mut audio);
    assert_eq!(canceller.frame_count(), 1);

    canceller.process_playback(&audio);
    canceller.process_input(&mut audio);
    assert!(audio.iter().all(|&s| s.abs() <= 1.0));
}

#[tokio::test]
async fn test_echo_canceller_disabled() {
    let canceller = EchoCanceller::new(false, 128, 16000);
    assert!(!canceller.is_enabled());

    let mut audio = vec![0.1, 0.2, 0.3, 0.4];
    let original = audio.clone();
    canceller.process_input(&mut audio);
    assert_eq!(audio, original);
}

#[tokio::test]
async fn test_echo_canceller_suppression() {
    let canceller = EchoCanceller::new(true, 128, 16000).with_suppression_factor(0.9);

    let reference = vec![0.5; 128];
    canceller.process_playback(&reference);

    let mut input = vec![0.5; 128];
    canceller.process_input(&mut input);

    for sample in &input {
        assert!(sample.abs() <= 1.0);
    }
}

#[tokio::test]
async fn test_streaming_manager() {
    let manager = StreamingManager::new(10, 100);

    manager.emit(VoiceStreamEvent::TurnStarted {
        turn_id: "t1".into(),
        session_id: "s1".into(),
    });

    assert_eq!(manager.event_count(), 1);
    assert!(!manager.buffered_events().is_empty());
}

#[tokio::test]
async fn test_streaming_manager_subscribe() {
    let manager = StreamingManager::new(10, 100);
    let mut rx = manager.subscribe();

    manager.emit(VoiceStreamEvent::VoiceActivityStarted { timestamp_ms: 0 });

    let event = tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await;
    assert!(event.is_ok());
}

#[tokio::test]
async fn test_streaming_partial_transcription_throttle() {
    let manager = StreamingManager::new(10, 100);

    assert!(manager.should_send_partial_transcription());
    assert!(!manager.should_send_partial_transcription());
}

#[tokio::test]
async fn test_voice_event_to_stream() {
    let events = vec![
        voxy_voice::VoiceEvent::WakeWordDetected { confidence: 0.9 },
        voxy_voice::VoiceEvent::VoiceActivityStarted,
        voxy_voice::VoiceEvent::VoiceActivityEnded { duration_ms: 1000 },
        voxy_voice::VoiceEvent::TranscriptionResult {
            text: "hello".into(),
            is_final: true,
            confidence: 0.85,
        },
        voxy_voice::VoiceEvent::SynthesisStarted { text: "hi".into() },
        voxy_voice::VoiceEvent::SynthesisCompleted { duration_ms: 500 },
        voxy_voice::VoiceEvent::TranscriptionError {
            error: "test".into(),
        },
        voxy_voice::VoiceEvent::SynthesisError {
            error: "test".into(),
        },
        voxy_voice::VoiceEvent::PipelineStateChanged {
            state: "test".into(),
        },
    ];

    for event in &events {
        let stream_events = voice_event_to_stream(event);
        if !matches!(event, voxy_voice::VoiceEvent::PipelineStateChanged { .. }) {
            assert!(!stream_events.is_empty());
        }
    }
}

#[tokio::test]
async fn test_brain_event_to_stream() {
    let events = vec![
        BrainEvent::TurnStarted {
            turn_id: "t1".into(),
            session_id: "s1".into(),
        },
        BrainEvent::ContextCollecting,
        BrainEvent::CompanionUpdating,
        BrainEvent::HdrUpdating,
        BrainEvent::CognitionProcessing,
        BrainEvent::TurnCompleted {
            turn_id: "t1".into(),
            total_duration_ms: 100,
        },
        BrainEvent::TurnFailed {
            turn_id: "t1".into(),
            error: "err".into(),
        },
        BrainEvent::TurnInterrupted {
            turn_id: "t1".into(),
            reason: "cancel".into(),
        },
    ];

    for event in &events {
        let stream_event = brain_event_to_stream(event);
        assert!(stream_event.is_some());
    }
}

#[tokio::test]
async fn test_latency_breakdown_default() {
    let latency = LatencyBreakdown::default();
    assert_eq!(latency.wake_word_us, 0);
    assert_eq!(latency.vad_us, 0);
    assert_eq!(latency.echo_cancellation_us, 0);
    assert_eq!(latency.stt_us, 0);
    assert_eq!(latency.brain_us, 0);
    assert_eq!(latency.tts_us, 0);
    assert_eq!(latency.total_us, 0);
}

#[test]
fn test_voice_runtime_state_debug() {
    let states = vec![
        VoiceRuntimeState::Idle,
        VoiceRuntimeState::Listening,
        VoiceRuntimeState::ProcessingSpeech,
        VoiceRuntimeState::Speaking,
        VoiceRuntimeState::Interrupted,
        VoiceRuntimeState::ShuttingDown,
        VoiceRuntimeState::Shutdown,
    ];
    for state in &states {
        let debug = format!("{:?}", state);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_voice_runtime_state_equality() {
    assert_eq!(VoiceRuntimeState::Idle, VoiceRuntimeState::Idle);
    assert_ne!(VoiceRuntimeState::Idle, VoiceRuntimeState::Listening);
}

#[test]
fn test_voice_session_id_uniqueness() {
    let id1 = VoiceSessionId::new();
    let id2 = VoiceSessionId::new();
    assert_ne!(id1.0, id2.0);
}

#[test]
fn test_voice_turn_id_uniqueness() {
    let id1 = VoiceTurnId::new();
    let id2 = VoiceTurnId::new();
    assert_ne!(id1.0, id2.0);
}

#[test]
fn test_turn_boundary_equality() {
    assert_eq!(TurnBoundary::None, TurnBoundary::None);
    assert_ne!(TurnBoundary::None, TurnBoundary::EndOfUtterance);
    assert_ne!(TurnBoundary::LongPause, TurnBoundary::Timeout);
}
