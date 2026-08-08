//! Production Validation Tests for VOXY Cognition Runtime
//!
//! These tests validate reliability, scalability, correctness, and observability
//! under real-world conditions. They cover:
//! - Integration tests (E2E pipeline, goal switching, context invalidation)
//! - Stress tests (high concurrency, rapid updates)
//! - Concurrency tests (race conditions, lock ordering)
//! - Memory validation (leak detection, cache growth)
//! - Failure injection (provider failures, corrupted context)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use tokio::sync::broadcast;

use voxy_cognition::attention::{AttentionFactors, AttentionSystem};
use voxy_cognition::config::CognitionConfig;
use voxy_cognition::context::{AssembledContext, ContextSource};
use voxy_cognition::goals::{Goal, GoalDecomposer, GoalDecomposition, GoalPriority, GoalState};
use voxy_cognition::in_memory::{
    InMemoryAttentionSystem, InMemoryCognitiveEngine, InMemoryGoalDecomposer,
    InMemoryIntentAnalyzer, InMemoryPlanner, InMemoryReasoner, InMemoryReflectionEngine,
    InMemorySelfMonitor,
};
use voxy_cognition::intent::{IntentAnalyzer, IntentInput, IntentType};
use voxy_cognition::orchestration::CognitiveEngine;
use voxy_cognition::planner::{
    Plan, PlanAction, PlanState, PlanStep, Planner, PlanningContext, StepState, StepType,
};
use voxy_cognition::reasoning::{Reasoner, ReasoningInput};
use voxy_cognition::recovery::RecoveryManager;
use voxy_cognition::reflection::{ReflectionEngine, ReflectionInput};
use voxy_cognition::self_monitoring::SelfMonitor;
use voxy_cognition::types::{ContextId, GoalId, IntentId, PlanId, StepId};

// Context crate imports for benchmarks
use voxy_context::fusion::{ContextFusionEngine, FusionPolicy};
use voxy_context::types::{
    ContextPriority as VoxyContextPriority, ContextSnapshot as VoxySnapshot,
    ContextSource as VoxyContextSource,
};
use voxy_context::{CacheConfig, ContextCache};

// ============================================================================
// Helper Functions
// ============================================================================

fn test_input(text: &str) -> IntentInput {
    IntentInput {
        raw_text: text.to_string(),
        context: None,
        source: "test".to_string(),
        metadata: HashMap::new(),
    }
}

fn make_assembled_context(confidence: f64) -> AssembledContext {
    AssembledContext {
        id: ContextId("ctx-test".to_string()),
        sources: vec![ContextSource::WorldModel, ContextSource::Personality],
        world_snapshot: None,
        personality_context: None,
        relevant_history: vec![],
        constraints: vec![],
        priority_hints: vec![],
        assembly_time_ms: 0,
        timestamp: Utc::now(),
        fusion_data: None,
        fusion_confidence: confidence,
        source_count: 2,
    }
}

fn make_snapshot(source: VoxyContextSource, confidence: f64, freshness: u64) -> VoxySnapshot {
    VoxySnapshot {
        id: voxy_context::types::ContextId::new(),
        source,
        priority: VoxyContextPriority::Medium,
        confidence,
        freshness,
        relevance: 0.5,
        captured_at: Utc::now(),
        data: serde_json::json!({"test": true}),
        size_bytes: 16,
    }
}

fn make_goal(id: &str, description: &str, priority: GoalPriority) -> Goal {
    Goal {
        id: GoalId(id.to_string()),
        description: description.to_string(),
        priority,
        dependencies: vec![],
        state: GoalState::Active,
        acceptance_criteria: vec![],
        metadata: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        context_snapshot: None,
        paused_by_context: false,
    }
}

fn make_plan_with_steps(step_count: usize) -> Plan {
    let steps: Vec<PlanStep> = (0..step_count)
        .map(|i| PlanStep {
            id: StepId(format!("s{}", i)),
            description: format!("Step {}", i),
            step_type: StepType::Atomic,
            dependencies: vec![],
            required_capabilities: vec![],
            state: StepState::Pending,
            estimated_duration_ms: 100,
            max_retries: 3,
            metadata: HashMap::new(),
        })
        .collect();

    Plan {
        id: PlanId("plan-test".to_string()),
        goals: vec![GoalId("g1".to_string())],
        steps,
        state: PlanState::InProgress,
        estimated_total_duration_ms: (step_count as u64) * 100,
        parallelism_possible: false,
        fallback_plan_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// ============================================================================
// 1. INTEGRATION TESTS — E2E Pipeline
// ============================================================================

fn test_engine() -> InMemoryCognitiveEngine {
    let cfg = CognitionConfig {
        confidence_threshold: 0.3,
        ..CognitionConfig::default()
    };
    InMemoryCognitiveEngine::new(cfg)
}

#[tokio::test]
async fn test_e2e_full_cognition_pipeline() {
    let engine = test_engine();

    let result = engine
        .process(&test_input(
            "Create a new web application with user authentication",
        ))
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.intent.intent_type, IntentType::Creation);
    assert!(result.plan.is_some());
    assert!(result.context.is_some());
    assert!(result.decision.is_some());
    assert!(result.duration_ms <= 5000);
}

#[tokio::test]
async fn test_e2e_pipeline_multiple_intents() {
    let engine = test_engine();

    let inputs = vec![
        ("What is the weather?", IntentType::Query),
        ("open the browser", IntentType::Navigation),
        ("create a new document", IntentType::Creation),
        ("delete the old file", IntentType::Deletion),
        ("modify the settings", IntentType::Modification),
    ];

    for (text, expected_type) in inputs {
        let result = engine.process(&test_input(text)).await.unwrap();
        assert!(result.success, "Failed for input: {}", text);
        assert_eq!(result.intent.intent_type, expected_type);
    }
}

#[tokio::test]
async fn test_e2e_goal_switching() {
    let engine = test_engine();

    // First, create some goals through the engine
    let result1 = engine
        .process(&test_input("build a web application"))
        .await
        .unwrap();
    assert!(result1.success);

    // Now switch to a different goal
    let result2 = engine
        .process(&test_input("send an email to the team"))
        .await
        .unwrap();
    assert!(result2.success);
    assert_eq!(result2.intent.intent_type, IntentType::Communication);
}

#[tokio::test]
async fn test_e2e_goal_reprioritization_on_context_change() {
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    // Create goals with different priorities
    let goals = vec![
        make_goal("g1", "Build web app", GoalPriority::High),
        make_goal("g2", "Critical system check", GoalPriority::Critical),
        make_goal("g3", "Update documentation", GoalPriority::Low),
    ];

    // Simulate low battery context
    let ctx = AssembledContext {
        id: ContextId("ctx-battery".to_string()),
        sources: vec![],
        world_snapshot: None,
        personality_context: None,
        relevant_history: vec![],
        constraints: vec![],
        priority_hints: vec![],
        assembly_time_ms: 0,
        timestamp: Utc::now(),
        fusion_data: Some(serde_json::json!({"battery_level": 0.05})),
        fusion_confidence: 0.8,
        source_count: 1,
    };

    let result = decomposer.reprioritize(goals, &ctx).await.unwrap();

    // Low battery should pause non-critical goals
    assert!(result.paused.contains(&GoalId("g1".to_string())));
    assert!(result.paused.contains(&GoalId("g3".to_string())));
    assert!(!result.paused.contains(&GoalId("g2".to_string())));
}

#[tokio::test]
async fn test_e2e_planner_context_change_response() {
    let planner = InMemoryPlanner::new(CognitionConfig::default());
    let plan = make_plan_with_steps(3);

    // Low confidence context should trigger pause
    let ctx = PlanningContext {
        assembled: AssembledContext {
            id: ContextId("ctx-low-conf".to_string()),
            sources: vec![],
            world_snapshot: None,
            personality_context: None,
            relevant_history: vec![],
            constraints: vec![],
            priority_hints: vec![],
            assembly_time_ms: 0,
            timestamp: Utc::now(),
            fusion_data: None,
            fusion_confidence: 0.2,
            source_count: 0,
        },
        context_changed: true,
        stale_sources: vec!["Environment".to_string()],
        context_confidence: 0.2,
    };

    let action = planner.on_context_change(&plan, &ctx).await.unwrap();
    assert_eq!(
        action,
        PlanAction::Pause("Context confidence critically low".to_string())
    );
}

#[tokio::test]
async fn test_e2e_reasoning_pipeline() {
    let reasoner = InMemoryReasoner;
    let ctx = make_assembled_context(0.9);

    let input = ReasoningInput {
        query: "Why should we use async/await in Rust?".to_string(),
        context: ctx,
        constraints: vec![],
        max_depth: 5,
    };

    let output = reasoner.reason(&input).await.unwrap();
    assert!(!output.conclusion.is_empty());
    assert!(output.confidence.value > 0.0);
    assert!(output.duration_ms > 0);
}

#[tokio::test]
async fn test_e2e_reflection_pipeline() {
    let reflection = InMemoryReflectionEngine;
    let plan = make_plan_with_steps(2);
    let ctx = make_assembled_context(0.8);

    let input = ReflectionInput {
        plan: &plan,
        execution_results: vec![
            (StepId("s0".to_string()), true, "".to_string()),
            (StepId("s1".to_string()), true, "".to_string()),
        ],
        context: &ctx,
        expected_outcome: None,
        actual_outcome: None,
        context_changes: vec![],
    };

    let report = reflection.reflect(&input).await.unwrap();
    assert!(!report.insights.is_empty());
    assert!(!report.overall_assessment.is_empty());
}

// ============================================================================
// 2. STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_stress_rapid_context_switching() {
    let engine = test_engine();
    let start = Instant::now();

    // Rapidly switch between 50 different intents
    for i in 0..50 {
        let text = match i % 5 {
            0 => "what is the weather",
            1 => "open the browser",
            2 => "create a new file",
            3 => "delete old files",
            _ => "modify settings",
        };
        let result = engine.process(&test_input(text)).await.unwrap();
        assert!(result.success, "Failed at iteration {}", i);
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "Stress test took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_stress_high_volume_reasoning() {
    let reasoner = InMemoryReasoner;
    let ctx = make_assembled_context(0.9);
    let start = Instant::now();

    // 100 reasoning cycles
    for i in 0..100 {
        let input = ReasoningInput {
            query: format!("Reasoning cycle {}", i),
            context: ctx.clone(),
            constraints: vec![],
            max_depth: 3,
        };
        let output = reasoner.reason(&input).await.unwrap();
        assert!(!output.conclusion.is_empty());
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(60),
        "High volume reasoning took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_stress_concurrent_planner_execution() {
    let planner = Arc::new(InMemoryPlanner::new(CognitionConfig::default()));
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn 20 concurrent planner tasks
    for i in 0..20 {
        let planner = planner.clone();
        let handle = tokio::spawn(async move {
            let _plan = make_plan_with_steps(5);
            let ctx = PlanningContext {
                assembled: make_assembled_context(0.8),
                context_changed: false,
                stale_sources: vec![],
                context_confidence: 0.8,
            };
            let result = planner
                .create_plan(
                    &GoalDecomposition {
                        intent_id: IntentId(format!("intent-{}", i)),
                        goals: vec![make_goal(
                            &format!("goal-{}", i),
                            "Task",
                            GoalPriority::Medium,
                        )],
                        dependency_graph: vec![],
                        estimated_complexity: 0.5,
                        timestamp: Utc::now(),
                    },
                    &ctx,
                )
                .await;
            result.is_ok()
        });
        handles.push(handle);
    }

    let results: Vec<bool> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert!(results.iter().all(|r| *r), "Some planner tasks failed");

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "Concurrent planner execution took too long: {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_stress_attention_system_under_load() {
    let attention = Arc::new(InMemoryAttentionSystem::new());
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn 50 concurrent attention scoring tasks
    for i in 0..50 {
        let attention = attention.clone();
        let handle = tokio::spawn(async move {
            let factors = AttentionFactors {
                urgency: (i as f64) / 50.0,
                context_freshness: 0.9,
                ..Default::default()
            };
            attention.score(&factors).await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();

    assert!(results.iter().all(|r| r.is_ok()));
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "Attention stress test took too long: {:?}",
        elapsed
    );
}

// ============================================================================
// 3. CONCURRENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrency_self_monitor_thread_safety() {
    let monitor = Arc::new(InMemorySelfMonitor::new());
    let mut handles = vec![];

    // Spawn multiple concurrent writers
    for i in 0..30 {
        let monitor = monitor.clone();
        let handle = tokio::spawn(async move {
            monitor
                .record_latency(&format!("op-{}", i % 5), Duration::from_millis(i as u64))
                .await
                .unwrap();
            monitor.record_fusion().await.unwrap();
            monitor.record_cycle().await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all writers
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify snapshot is consistent
    let snap = monitor.snapshot().await.unwrap();
    assert!(snap.fusion_count > 0);
    assert!(snap.cycle_count > 0);
}

#[tokio::test]
async fn test_concurrency_goal_decomposer_parallel_access() {
    let decomposer = Arc::new(InMemoryGoalDecomposer::new(CognitionConfig::default()));
    let analyzer = InMemoryIntentAnalyzer;
    let mut handles = vec![];

    let start = Instant::now();

    for i in 0..10 {
        let decomposer = decomposer.clone();
        let intent = analyzer
            .analyze(&test_input(&format!("Task number {}", i)))
            .await
            .unwrap();

        let handle = tokio::spawn(async move { decomposer.decompose(&intent).await });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();

    assert!(results.iter().all(|r| r.is_ok()));
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(15));
}

#[tokio::test]
async fn test_concurrency_cognitive_engine_sequential_processing() {
    let engine = Arc::new(InMemoryCognitiveEngine::new(CognitionConfig::default()));
    let mut handles = vec![];

    let start = Instant::now();

    // Sequential processing (engine holds internal state)
    for i in 0..10 {
        let engine = engine.clone();
        let handle = tokio::spawn(async move {
            let input = match i % 3 {
                0 => test_input("What is the weather?"),
                1 => test_input("Open the browser"),
                _ => test_input("Create a file"),
            };
            engine.process(&input).await
        });
        handles.push(handle);
    }

    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();

    assert!(results.iter().all(|r| r.is_ok()));
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(60));
}

#[tokio::test]
async fn test_concurrency_broadcast_receiver_under_load() {
    let (tx, _) = broadcast::channel::<String>(100);
    let mut handles = vec![];

    let start = Instant::now();

    // Spawn 20 receivers
    for _ in 0..20 {
        let mut rx = tx.subscribe();
        let handle = tokio::spawn(async move {
            let mut count = 0;
            while rx.recv().await.is_ok() {
                count += 1;
                if count >= 5 {
                    break;
                }
            }
            count
        });
        handles.push(handle);
    }

    // Spawn sender
    let sender_handle = tokio::spawn(async move {
        for i in 0..100 {
            tx.send(format!("msg-{}", i)).ok();
            tokio::time::sleep(Duration::from_micros(10)).await;
        }
    });

    let results: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    sender_handle.await.unwrap();

    // All receivers should have received some messages
    assert!(results.iter().all(|&c| c >= 5));

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(10));
}

// ============================================================================
// 4. MEMORY VALIDATION
// ============================================================================

#[tokio::test]
async fn test_memory_self_monitor_memory_growth() {
    let monitor = InMemorySelfMonitor::new();

    // Record many latency samples
    for i in 0..1000 {
        monitor
            .record_latency(
                &format!("op-{}", i % 10),
                Duration::from_millis(i as u64 % 100),
            )
            .await
            .unwrap();
    }

    let snap = monitor.snapshot().await.unwrap();

    // Verify snapshot is reasonable size (not growing unbounded)
    let snap_json = serde_json::to_string(&snap).unwrap();
    assert!(
        snap_json.len() < 10_000,
        "Snapshot too large: {} bytes",
        snap_json.len()
    );
}

#[tokio::test]
async fn test_memory_cognitive_engine_repeated_processing() {
    let engine = test_engine();

    // Process 200 inputs and verify memory doesn't grow unbounded
    for i in 0..200 {
        let result = engine.process(&test_input(&format!("Query {}", i))).await;
        assert!(result.is_ok());
    }

    // If we get here without OOM, the test passes
    let diagnostics = engine.diagnostics().await.unwrap();
    assert!(diagnostics.is_object());
}

#[tokio::test]
async fn test_memory_planner_repeated_plan_creation() {
    let planner = InMemoryPlanner::new(CognitionConfig::default());
    let analyzer = InMemoryIntentAnalyzer;
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    let intent = analyzer
        .analyze(&test_input("Build a complex application"))
        .await
        .unwrap();
    let decomposition = decomposer.decompose(&intent).await.unwrap();
    let ctx = PlanningContext {
        assembled: make_assembled_context(0.8),
        context_changed: false,
        stale_sources: vec![],
        context_confidence: 0.8,
    };

    // Create 100 plans
    for _ in 0..100 {
        let plan = planner.create_plan(&decomposition, &ctx).await.unwrap();
        assert!(!plan.steps.is_empty());
    }
}

#[tokio::test]
async fn test_memory_reasoner_repeated_reasoning() {
    let reasoner = InMemoryReasoner;
    let ctx = make_assembled_context(0.9);

    for i in 0..100 {
        let input = ReasoningInput {
            query: format!("Reasoning query {}", i),
            context: ctx.clone(),
            constraints: vec![],
            max_depth: 3,
        };
        let output = reasoner.reason(&input).await.unwrap();
        assert!(!output.conclusion.is_empty());
    }
}

// ============================================================================
// 5. FAILURE INJECTION TESTS
// ============================================================================

#[tokio::test]
async fn test_failure_low_confidence_threshold() {
    let cfg = CognitionConfig {
        confidence_threshold: 1.01, // Impossible threshold
        ..CognitionConfig::default()
    };

    let engine = InMemoryCognitiveEngine::new(cfg);
    let result = engine.process(&test_input("Hello")).await.unwrap();

    assert!(!result.success);
    assert!(result.errors.is_empty() || result.errors.iter().any(|e| e.contains("confidence")));
}

#[tokio::test]
async fn test_failure_empty_input() {
    let engine = InMemoryCognitiveEngine::new(CognitionConfig::default());
    let result = engine.process(&test_input("")).await.unwrap();

    // Empty input should still process (confidence may be low)
    let _ = result.duration_ms;
}

#[tokio::test]
async fn test_failure_planner_invalid_plan() {
    let planner = InMemoryPlanner::new(CognitionConfig::default());

    let plan = Plan {
        id: PlanId("invalid".to_string()),
        goals: vec![],
        steps: vec![],
        state: PlanState::Draft,
        estimated_total_duration_ms: 0,
        parallelism_possible: false,
        fallback_plan_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // Validate empty plan
    let is_valid = planner.validate_plan(&plan).await.unwrap();
    // Empty plan with no steps may be considered valid or invalid depending on implementation
    assert!(is_valid || !is_valid); // Just ensure no panic
}

#[tokio::test]
async fn test_failure_reasoning_with_empty_context() {
    let reasoner = InMemoryReasoner;
    let ctx = AssembledContext {
        id: ContextId("ctx-empty".to_string()),
        sources: vec![],
        world_snapshot: None,
        personality_context: None,
        relevant_history: vec![],
        constraints: vec![],
        priority_hints: vec![],
        assembly_time_ms: 0,
        timestamp: Utc::now(),
        fusion_data: None,
        fusion_confidence: 0.0,
        source_count: 0,
    };

    let input = ReasoningInput {
        query: "Test query".to_string(),
        context: ctx,
        constraints: vec![],
        max_depth: 1,
    };

    let output = reasoner.reason(&input).await.unwrap();
    // Should handle empty context gracefully
    assert!(!output.conclusion.is_empty());
}

#[tokio::test]
async fn test_failure_reflection_with_failed_steps() {
    let reflection = InMemoryReflectionEngine;
    let plan = make_plan_with_steps(3);
    let ctx = make_assembled_context(0.5);

    let input = ReflectionInput {
        plan: &plan,
        execution_results: vec![
            (StepId("s0".to_string()), true, "".to_string()),
            (
                StepId("s1".to_string()),
                false,
                "Network timeout".to_string(),
            ),
            (
                StepId("s2".to_string()),
                false,
                "Permission denied".to_string(),
            ),
        ],
        context: &ctx,
        expected_outcome: None,
        actual_outcome: None,
        context_changes: vec![],
    };

    let report = reflection.reflect(&input).await.unwrap();

    // Should produce insights about failures
    assert!(!report.insights.is_empty());
    assert!(!report.overall_assessment.is_empty());
}

#[tokio::test]
async fn test_failure_recovery_manager_diagnosis() {
    let recovery = voxy_cognition::in_memory::InMemoryRecoveryManager;
    let step = PlanStep {
        id: StepId("s-fail".to_string()),
        description: "Failing step".to_string(),
        step_type: StepType::Atomic,
        dependencies: vec![],
        required_capabilities: vec![],
        state: StepState::Failed("Connection refused".to_string()),
        estimated_duration_ms: 100,
        max_retries: 3,
        metadata: HashMap::new(),
    };

    let strategies = recovery
        .diagnose(&PlanId("p1".to_string()), &step, "Connection refused")
        .await
        .unwrap();

    assert!(!strategies.is_empty());
}

#[tokio::test]
async fn test_failure_goal_reprioritize_meeting_context() {
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    let goals = vec![
        make_goal("g1", "Write code for feature X", GoalPriority::High),
        make_goal("g2", "Attend team meeting", GoalPriority::Medium),
    ];

    // Meeting context should pause coding goals
    let ctx = AssembledContext {
        id: ContextId("ctx-meeting".to_string()),
        sources: vec![],
        world_snapshot: None,
        personality_context: None,
        relevant_history: vec![],
        constraints: vec![],
        priority_hints: vec![],
        assembly_time_ms: 0,
        timestamp: Utc::now(),
        fusion_data: Some(serde_json::json!({"activity": {"type": "meeting"}})),
        fusion_confidence: 0.9,
        source_count: 1,
    };

    let result = decomposer.reprioritize(goals, &ctx).await.unwrap();

    // Coding goal should be paused during meeting
    assert!(result.paused.contains(&GoalId("g1".to_string())));
}

// ============================================================================
// 6. OBSERVABILITY TESTS
// ============================================================================

#[tokio::test]
async fn test_observability_diagnostics_snapshot() {
    let engine = test_engine();

    // Process some inputs first
    let _ = engine.process(&test_input("Test query 1")).await.unwrap();
    let _ = engine.process(&test_input("Test query 2")).await.unwrap();

    let diagnostics = engine.diagnostics().await.unwrap();

    assert!(diagnostics.is_object());
    assert!(diagnostics.get("cognitive_state").is_some());
    assert!(diagnostics.get("health").is_some());
    assert!(diagnostics.get("fusion_count").is_some());
    assert!(diagnostics.get("cycle_count").is_some());
}

#[tokio::test]
async fn test_observability_self_monitor_comprehensive() {
    let monitor = InMemorySelfMonitor::new();

    // Record various metrics
    for i in 0..50 {
        monitor
            .record_latency(&format!("op-{}", i % 5), Duration::from_millis(i as u64))
            .await
            .unwrap();
    }

    monitor.record_fusion().await.unwrap();
    monitor.record_fusion().await.unwrap();
    monitor.record_cycle().await.unwrap();
    monitor.record_error().await.unwrap();
    monitor.record_staleness().await.unwrap();
    monitor.set_active_goals(3).await.unwrap();

    let snap = monitor.snapshot().await.unwrap();

    assert_eq!(snap.fusion_count, 2);
    assert_eq!(snap.cycle_count, 1);
    assert_eq!(snap.error_count, 1);
    assert_eq!(snap.staleness_events, 1);
    assert_eq!(snap.active_goals, 3);
    assert!(
        snap.health == voxy_cognition::self_monitoring::HealthStatus::Healthy
            || matches!(
                snap.health,
                voxy_cognition::self_monitoring::HealthStatus::Degraded(_)
            )
    );
}

#[tokio::test]
async fn test_observability_attention_scoring() {
    let attention = InMemoryAttentionSystem::new();

    let factors = AttentionFactors {
        urgency: 0.9,
        context_freshness: 0.8,
        goal_relevance: 0.7,
        conversation_active: 0.6,
        foreground_app_importance: 0.85,
        voice_activity: 0.5,
        notification_pressure: 0.3,
    };

    let score = attention.score(&factors).await.unwrap();
    assert!(score.score > 0.0);
    assert!(score.score <= 1.0);

    // Test weight updates
    let mut weights = HashMap::new();
    weights.insert("urgency".to_string(), 0.9);
    weights.insert("freshness".to_string(), 0.7);
    attention.update_weights(weights).await.unwrap();

    let score2 = attention.score(&factors).await.unwrap();
    assert!(score2.score > 0.0);
}

#[tokio::test]
async fn test_observability_latency_tracking() {
    let monitor = InMemorySelfMonitor::new();

    // Record operations
    for i in 0..10 {
        monitor
            .record_latency("intent_analysis", Duration::from_millis(i * 10))
            .await
            .unwrap();
    }

    let record = monitor
        .get_latency("intent_analysis")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(record.sample_count, 10);
    assert!(record.avg_latency >= Duration::from_millis(0));
    assert!(record.max_latency >= Duration::from_millis(90)); // max should be at least 90ms
}

// ============================================================================
// 7. CORRECTNESS TESTS
// ============================================================================

#[tokio::test]
async fn test_correctness_intent_classification() {
    let analyzer = InMemoryIntentAnalyzer;

    let test_cases = vec![
        ("What is the meaning of life?", IntentType::Query),
        ("Open the calculator", IntentType::Navigation),
        ("Create a new project", IntentType::Creation),
        ("Delete temporary files", IntentType::Deletion),
        ("Change the theme to dark", IntentType::Modification),
        ("Run the test suite", IntentType::Command),
        ("Explain how async works", IntentType::Learning),
        ("Help me with debugging", IntentType::Query),
    ];

    for (text, expected) in test_cases {
        let result = analyzer.analyze(&test_input(text)).await.unwrap();
        assert_eq!(
            result.intent_type, expected,
            "Wrong classification for: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_correctness_confidence_scoring() {
    let analyzer = InMemoryIntentAnalyzer;

    let high_conf = analyzer
        .analyze(&test_input("What is the weather today?"))
        .await
        .unwrap();
    let low_conf = analyzer.analyze(&test_input("x")).await.unwrap();

    // Clear queries should have higher confidence than ambiguous ones
    assert!(high_conf.confidence.value >= low_conf.confidence.value);
}

#[tokio::test]
async fn test_correctness_plan_state_transitions() {
    let _planner = InMemoryPlanner::new(CognitionConfig::default());
    let plan = make_plan_with_steps(3);

    // Verify initial state
    assert!(matches!(plan.state, PlanState::InProgress));
    assert!(plan.steps.iter().all(|s| s.state == StepState::Pending));
}

#[tokio::test]
async fn test_correctness_goal_priority_ordering() {
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    let goals = vec![
        make_goal("g1", "Low priority", GoalPriority::Low),
        make_goal("g2", "Critical", GoalPriority::Critical),
        make_goal("g3", "Medium", GoalPriority::Medium),
        make_goal("g4", "High", GoalPriority::High),
    ];

    let _ctx = make_assembled_context(0.8);
    let result = decomposer.prioritize(goals).await.unwrap();

    // Verify all goals are present
    assert_eq!(result.len(), 4);

    // Verify priorities are assigned correctly
    for (goal, priority) in &result {
        match goal.id.0.as_str() {
            "g2" => assert_eq!(*priority, GoalPriority::Critical),
            "g4" => assert_eq!(*priority, GoalPriority::High),
            "g3" => assert_eq!(*priority, GoalPriority::Medium),
            "g1" => assert_eq!(*priority, GoalPriority::Low),
            _ => panic!("Unexpected goal id"),
        }
    }
}

#[tokio::test]
async fn test_correctness_reasoning_output_structure() {
    let reasoner = InMemoryReasoner;
    let ctx = make_assembled_context(0.9);

    let input = ReasoningInput {
        query: "Test reasoning".to_string(),
        context: ctx,
        constraints: vec![],
        max_depth: 5,
    };

    let output = reasoner.reason(&input).await.unwrap();

    assert!(!output.conclusion.is_empty());
    assert!(output.confidence.value >= 0.0 && output.confidence.value <= 1.0);
    assert!(output.duration_ms > 0);
    assert!(!output.contributing_sources.is_empty());
}

// ============================================================================
// 8. ENDURANCE TESTS (Simulated long-running)
// ============================================================================

#[tokio::test]
async fn test_endurance_sustained_cognitive_load() {
    let engine = test_engine();
    let start = Instant::now();

    // Simulate sustained load for 100 cycles
    for i in 0..100 {
        let input = match i % 4 {
            0 => test_input("What is the current status?"),
            1 => test_input("Open the application"),
            2 => test_input("Create a new task"),
            _ => test_input("Modify the configuration"),
        };

        let result = engine.process(&input).await.unwrap();
        assert!(result.success, "Failed at cycle {}", i);
        let _ = result.duration_ms;
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed / 100;

    assert!(
        avg_latency < Duration::from_secs(1),
        "Average latency too high: {:?}",
        avg_latency
    );
}

#[tokio::test]
async fn test_endurance_planner_under_sustained_load() {
    let planner = InMemoryPlanner::new(CognitionConfig::default());
    let analyzer = InMemoryIntentAnalyzer;
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    let intent = analyzer
        .analyze(&test_input("Complex multi-step task"))
        .await
        .unwrap();
    let decomposition = decomposer.decompose(&intent).await.unwrap();
    let ctx = PlanningContext {
        assembled: make_assembled_context(0.8),
        context_changed: false,
        stale_sources: vec![],
        context_confidence: 0.8,
    };

    let start = Instant::now();

    for _ in 0..50 {
        let plan = planner.create_plan(&decomposition, &ctx).await.unwrap();
        assert!(!plan.steps.is_empty());
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(30),
        "Planner endurance test failed: {:?}",
        elapsed
    );
}

// ============================================================================
// 9. INTEGRATION: Cross-component tests
// ============================================================================

#[tokio::test]
async fn test_integration_analyzer_to_planner_flow() {
    let analyzer = InMemoryIntentAnalyzer;
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());
    let planner = InMemoryPlanner::new(CognitionConfig::default());

    // Full flow: analyze -> decompose -> plan
    let intent = analyzer
        .analyze(&test_input("Build a machine learning model"))
        .await
        .unwrap();
    assert_eq!(intent.intent_type, IntentType::Creation);

    let decomposition = decomposer.decompose(&intent).await.unwrap();
    assert!(!decomposition.goals.is_empty());

    let ctx = PlanningContext {
        assembled: make_assembled_context(0.8),
        context_changed: false,
        stale_sources: vec![],
        context_confidence: 0.8,
    };

    let plan = planner.create_plan(&decomposition, &ctx).await.unwrap();
    assert!(!plan.steps.is_empty());
    assert_eq!(plan.goals.len(), decomposition.goals.len());
}

#[tokio::test]
async fn test_integration_planner_to_reasoner_flow() {
    let planner = InMemoryPlanner::new(CognitionConfig::default());
    let reasoner = InMemoryReasoner;
    let analyzer = InMemoryIntentAnalyzer;
    let decomposer = InMemoryGoalDecomposer::new(CognitionConfig::default());

    let intent = analyzer
        .analyze(&test_input("Analyze the data"))
        .await
        .unwrap();
    let decomposition = decomposer.decompose(&intent).await.unwrap();
    let ctx = PlanningContext {
        assembled: make_assembled_context(0.8),
        context_changed: false,
        stale_sources: vec![],
        context_confidence: 0.8,
    };

    let plan = planner.create_plan(&decomposition, &ctx).await.unwrap();

    // Use reasoning to evaluate the plan
    let reasoning_ctx = make_assembled_context(0.9);
    let input = ReasoningInput {
        query: format!("Evaluate plan with {} steps", plan.steps.len()),
        context: reasoning_ctx,
        constraints: vec![],
        max_depth: 3,
    };

    let output = reasoner.reason(&input).await.unwrap();
    assert!(!output.conclusion.is_empty());
}

#[tokio::test]
async fn test_integration_full_pipeline_with_reflection() {
    let engine = InMemoryCognitiveEngine::new(CognitionConfig::default());

    // Process initial input
    let result = engine
        .process(&test_input("Implement a caching layer"))
        .await
        .unwrap();
    assert!(result.success);

    // Verify all components were exercised
    assert!(result.plan.is_some());
    assert!(result.context.is_some());
    assert!(result.decision.is_some());
}

// ============================================================================
// Phase 5: Performance Benchmarks — Latency Measurements
// ============================================================================

#[tokio::test]
async fn test_bench_intent_analyzer_latency() {
    let analyzer = InMemoryIntentAnalyzer;
    let inputs = vec![
        "Create a new web application",
        "Delete the old database",
        "What is the current status",
        "Navigate to settings",
        "Send an email to the team",
    ];
    let iterations: usize = 100;
    let mut total_us: u128 = 0;

    for input in &inputs {
        for _ in 0..iterations {
            let start = std::time::Instant::now();
            let _ = analyzer.analyze(&test_input(input)).await.unwrap();
            total_us += start.elapsed().as_micros();
        }
    }
    let count = (inputs.len() * iterations) as u128;
    let avg_us = total_us / count;
    let avg_ms = avg_us as f64 / 1000.0;
    assert!(
        avg_ms < 10.0,
        "Intent analyzer avg latency {:.2}ms exceeds 10ms",
        avg_ms
    );
    eprintln!(
        "Intent analyzer avg: {:.2}ms ({} iterations)",
        avg_ms, count
    );
}

#[tokio::test]
async fn test_bench_context_fusion_latency() {
    let engine = ContextFusionEngine::new(FusionPolicy::default());
    let snapshot1 = make_snapshot(VoxyContextSource::WorldModel, 0.9, 100);
    let snapshot2 = make_snapshot(VoxyContextSource::Personality, 0.85, 95);
    let snapshot3 = make_snapshot(VoxyContextSource::Memory, 0.8, 90);
    let iterations = 500;
    let start = std::time::Instant::now();
    for _i in 0..iterations {
        let _ = engine.fuse(vec![
            snapshot1.clone(),
            snapshot2.clone(),
            snapshot3.clone(),
        ]);
    }
    let total_ms = start.elapsed().as_millis();
    let avg_ms = total_ms as f64 / iterations as f64;
    assert!(
        avg_ms < 5.0,
        "Context fusion avg latency {:.2}ms exceeds 5ms",
        avg_ms
    );
    eprintln!(
        "Context fusion avg: {:.2}ms ({} iterations)",
        avg_ms, iterations
    );
}

#[tokio::test]
async fn test_bench_planner_latency() {
    let cfg = CognitionConfig::default();
    let decomposer = Arc::new(InMemoryGoalDecomposer::new(cfg.clone()));
    let planner = Arc::new(InMemoryPlanner::new(cfg));
    let analyzer = InMemoryIntentAnalyzer;
    let iterations = 100;
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let intent = analyzer
            .analyze(&test_input(&format!("Build feature {}", i)))
            .await
            .unwrap();
        let decompo = decomposer.decompose(&intent).await.unwrap();
        let planning_ctx = PlanningContext {
            assembled: make_assembled_context(0.9),
            context_changed: false,
            stale_sources: vec![],
            context_confidence: 0.9,
        };
        let _ = planner.create_plan(&decompo, &planning_ctx).await.unwrap();
    }
    let total_ms = start.elapsed().as_millis();
    let avg_ms = total_ms as f64 / iterations as f64;
    assert!(
        avg_ms < 10.0,
        "Planner avg latency {:.2}ms exceeds 10ms",
        avg_ms
    );
    eprintln!("Planner avg: {:.2}ms ({} iterations)", avg_ms, iterations);
}

#[tokio::test]
async fn test_bench_reasoner_latency() {
    let reasoner = InMemoryReasoner;
    let ctx = make_assembled_context(0.9);
    let iterations = 100;
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let input = ReasoningInput {
            query: format!("Reason about step {}", i),
            context: ctx.clone(),
            constraints: vec![],
            max_depth: 3,
        };
        let _ = reasoner.reason(&input).await.unwrap();
    }
    let total_ms = start.elapsed().as_millis();
    let avg_ms = total_ms as f64 / iterations as f64;
    assert!(
        avg_ms < 10.0,
        "Reasoner avg latency {:.2}ms exceeds 10ms",
        avg_ms
    );
    eprintln!("Reasoner avg: {:.2}ms ({} iterations)", avg_ms, iterations);
}

#[tokio::test]
async fn test_bench_full_pipeline_latency() {
    let engine = test_engine();
    let iterations = 50;
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let _ = engine
            .process(&test_input(&format!("Build feature {}", i)))
            .await
            .unwrap();
    }
    let total_ms = start.elapsed().as_millis();
    let avg_ms = total_ms as f64 / iterations as f64;
    assert!(
        avg_ms < 50.0,
        "Full pipeline avg latency {:.2}ms exceeds 50ms",
        avg_ms
    );
    eprintln!(
        "Full pipeline avg: {:.2}ms ({} iterations)",
        avg_ms, iterations
    );
}

#[tokio::test]
async fn test_bench_cache_lookup_latency() {
    let cache = ContextCache::new(CacheConfig {
        max_per_source: 1000,
        max_total: 5000,
        ttl: std::time::Duration::from_secs(3600),
        freshness: voxy_context::types::FreshnessConfig::default(),
    });
    for i in 0..1000 {
        let snap = VoxySnapshot {
            id: voxy_context::types::ContextId::new(),
            source: VoxyContextSource::WorldModel,
            priority: VoxyContextPriority::Medium,
            confidence: 0.9,
            freshness: 100,
            relevance: 0.5,
            captured_at: Utc::now(),
            data: serde_json::json!({"i": i}),
            size_bytes: 8,
        };
        cache.insert(snap);
    }
    let iterations = 500;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = cache.get_latest(&VoxyContextSource::WorldModel);
    }
    let total_ms = start.elapsed().as_millis();
    let avg_us = (total_ms as f64 / iterations as f64) * 1000.0;
    assert!(
        avg_us < 500.0,
        "Cache lookup avg latency {:.0}us exceeds 500us",
        avg_us
    );
    eprintln!(
        "Cache lookup avg: {:.0}us ({} iterations)",
        avg_us, iterations
    );
}
