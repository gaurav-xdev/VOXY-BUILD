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
use voxy_cognition::types::{CognitiveState, ConfidenceScore, IntentId, PlanId};

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
            result: serde_json::json!({"response": format!("Mock response for: {}", input.raw_text)}),
            confidence: ConfidenceScore::new(0.85).unwrap(),
            reflection: None,
            duration_ms: 5,
            success: true,
            errors: Vec::new(),
            decision: Some(DecisionOutput {
                decision: "respond".into(),
                confidence: ConfidenceScore::new(0.9).unwrap(),
                reason: "user asked a question".into(),
                context_summary: "mock context".into(),
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

    async fn state(&self) -> CognitiveState {
        CognitiveState::Idle
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
        Ok(serde_json::json!({"healthy": true, "engine": "mock"}))
    }
}

fn make_brain() -> UnifiedBrainEngine {
    let cognition = Arc::new(MockCognitiveEngine);
    UnifiedBrainEngine::new(BrainConfig::default(), cognition)
}

fn make_input(text: &str) -> BrainInput {
    BrainInput {
        session_id: SessionId::new(),
        raw_text: text.to_string(),
        user_presence: voxy_companion::types::UserPresence::Active,
        focus_level: 0.5,
        stress_level: 0.2,
        is_meeting: false,
        time_since_last_interaction: std::time::Duration::from_secs(30),
        session_duration: std::time::Duration::from_secs(600),
        errors_this_session: 0,
        missions_completed: 5,
        missions_failed: 0,
        metadata: HashMap::new(),
    }
}

#[tokio::test]
async fn test_brain_init_and_shutdown() {
    let brain = make_brain();
    assert!(brain.init().await.is_ok());
    assert_eq!(brain.state(), BrainState::Idle);
    assert!(brain.shutdown().await.is_ok());
    assert_eq!(brain.state(), BrainState::Shutdown);
}

#[tokio::test]
async fn test_brain_process_turn() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let input = make_input("Hello VOXY");
    let output = brain.process_turn(input).await.unwrap();

    assert!(output.response_text.is_some());
    assert!(output.cognitive_result.is_some());
    assert!(output.companion.is_some());
    assert!(output.human_dynamics.is_some());
    assert!(output.stage_latencies.total_us > 0);

    brain.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_brain_multiple_turns() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let session_id = SessionId::new();
    for i in 0..5 {
        let mut input = make_input(&format!("Message {}", i));
        input.session_id = session_id.clone();
        let _output = brain.process_turn(input).await.unwrap();
    }

    let health = brain.health_check().await.unwrap();
    assert_eq!(health.active_sessions, 1);
    assert_eq!(health.total_turns, 5);

    brain.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_brain_health_check() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let health = brain.health_check().await.unwrap();
    assert_eq!(health.components.len(), 4);
    assert!(health.components.contains_key("cognition"));
    assert!(health.components.contains_key("context"));
    assert!(health.components.contains_key("companion"));
    assert!(health.components.contains_key("human_dynamics"));

    brain.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_brain_latency_tracking() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let input = make_input("Latency test");
    let _output = brain.process_turn(input).await.unwrap();

    let latency = brain.latency();
    assert!(latency.total_us > 0);

    let avg = brain.latency_average();
    assert!(avg.total_us > 0);

    brain.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_brain_shutdown_blocks_processing() {
    let brain = make_brain();
    brain.init().await.unwrap();
    brain.shutdown().await.unwrap();

    let input = make_input("Should fail");
    let result = brain.process_turn(input).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_brain_cancel_turn() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let session_id = SessionId::new();
    let result = brain.cancel_turn(&session_id).await;
    assert!(result.is_ok());

    brain.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_brain_events() {
    let brain = make_brain();
    brain.init().await.unwrap();

    let mut rx = brain.subscribe();

    let input = make_input("Event test");
    let brain_arc = Arc::new(brain);

    let brain_for_task = brain_arc.clone();
    let handle = tokio::spawn(async move {
        let _output = brain_for_task.process_turn(input).await;
    });

    let mut events_received = 0;
    let _timeout = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while let Ok(_event) = rx.recv().await {
            events_received += 1;
        }
    })
    .await;

    handle.await.unwrap();
    assert!(events_received > 0);

    brain_arc.shutdown().await.unwrap();
}
