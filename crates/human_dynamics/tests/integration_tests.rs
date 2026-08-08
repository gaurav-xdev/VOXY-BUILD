use std::time::{Duration, Instant};

use voxy_human_dynamics::engine::HumanDynamicsEngine;
use voxy_human_dynamics::types::*;

fn default_config() -> voxy_human_dynamics::config::HdrConfig {
    voxy_human_dynamics::config::HdrConfig::default()
}

fn make_input(pending: Option<Action>, trust_events: Vec<TrustEvent>, meeting: bool) -> HdrInput {
    HdrInput {
        now: chrono::Utc::now(),
        instant_now: Instant::now(),
        user_id: UserId("user-1".to_string()),
        user_present: true,
        current_behavior: BehaviorState::Observing,
        activity_description: "Working on project".to_string(),
        pending_action: pending,
        recent_trust_events: trust_events,
        time_since_last_interaction: Duration::from_secs(30),
        session_duration: Duration::from_secs(600),
        errors_this_session: 0,
        corrections_this_session: 0,
        missions_completed: 5,
        missions_failed: 0,
        is_meeting: meeting,
        focus_level: 0.5,
        stress_level: 0.2,
    }
}

#[test]
fn test_full_pipeline_high_trust() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let events: Vec<TrustEvent> = (0..30)
        .map(|_| TrustEvent {
            kind: TrustEventKind::SuccessfulMission,
            impact: 0.0,
            timestamp: chrono::Utc::now(),
            context: "Success".to_string(),
        })
        .collect();

    let input = make_input(
        Some(Action {
            id: "a1".to_string(),
            kind: ActionKind::Speak,
            description: "Report status".to_string(),
            protection_level: ProtectionLevel::Low,
            reversible: true,
            impact: 0.1,
        }),
        events,
        false,
    );

    let output = engine.update(&input);
    assert!(output.trust_score > 0.5);
    assert!(output.relationship_level as u8 >= RelationshipLevel::Familiar as u8);
    assert!(output.update_latency_us < 100);
}

#[test]
fn test_full_pipeline_meeting_blocked() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let input = make_input(
        Some(Action {
            id: "a1".to_string(),
            kind: ActionKind::Speak,
            description: "Interrupt".to_string(),
            protection_level: ProtectionLevel::Medium,
            reversible: true,
            impact: 0.2,
        }),
        vec![],
        true,
    );

    let output = engine.update(&input);
    assert!(output
        .policy_violations
        .iter()
        .any(|v| v.contains("meeting")));
}

#[test]
fn test_full_pipeline_critical_action_denied() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let input = make_input(
        Some(Action {
            id: "a1".to_string(),
            kind: ActionKind::Delete,
            description: "Delete all data".to_string(),
            protection_level: ProtectionLevel::Critical,
            reversible: false,
            impact: 0.9,
        }),
        vec![],
        false,
    );

    let output = engine.update(&input);
    assert!(!output.protection_decision.allowed);
}

#[test]
fn test_full_pipeline_recovery_on_errors() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let mut input = make_input(None, vec![], false);
    input.errors_this_session = 3;

    let output = engine.update(&input);
    assert!(output.recovery.is_some());
}

#[test]
fn test_full_pipeline_low_trust_no_initiative() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let mut input = make_input(None, vec![], false);
    input.user_present = false;
    input.time_since_last_interaction = Duration::from_secs(3600);

    let output = engine.update(&input);
    assert!(!output.initiative_decision.may_speak);
}

#[test]
fn test_full_pipeline_style_adapts() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let events: Vec<TrustEvent> = (0..30)
        .map(|_| TrustEvent {
            kind: TrustEventKind::SuccessfulMission,
            impact: 0.0,
            timestamp: chrono::Utc::now(),
            context: "Success".to_string(),
        })
        .collect();

    let input = make_input(None, events, false);
    let output = engine.update(&input);
    assert!(output.style.formality < 0.7);
}

#[test]
fn test_full_pipeline_latency_under_100us() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let input = make_input(
        Some(Action {
            id: "a1".to_string(),
            kind: ActionKind::Speak,
            description: "Quick check".to_string(),
            protection_level: ProtectionLevel::Low,
            reversible: true,
            impact: 0.1,
        }),
        vec![TrustEvent {
            kind: TrustEventKind::TaskCompleted,
            impact: 0.0,
            timestamp: chrono::Utc::now(),
            context: "Done".to_string(),
        }],
        false,
    );

    let output = engine.update(&input);
    assert!(
        output.update_latency_us < 100,
        "Latency was {}us",
        output.update_latency_us
    );
}

#[test]
fn test_full_pipeline_serialization_roundtrip() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    let input = make_input(
        Some(Action {
            id: "a1".to_string(),
            kind: ActionKind::Speak,
            description: "Hello".to_string(),
            protection_level: ProtectionLevel::Low,
            reversible: true,
            impact: 0.1,
        }),
        vec![],
        false,
    );

    let output = engine.update(&input);
    let json = serde_json::to_string(&output).unwrap();
    let deserialized: HdrOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(output.trust_score, deserialized.trust_score);
    assert_eq!(output.relationship_level, deserialized.relationship_level);
}

#[test]
fn test_multiple_updates_accumulate_state() {
    let mut engine = HumanDynamicsEngine::new(default_config());
    for i in 0..10 {
        let input = make_input(
            Some(Action {
                id: format!("a{}", i),
                kind: ActionKind::Speak,
                description: "Update".to_string(),
                protection_level: ProtectionLevel::Low,
                reversible: true,
                impact: 0.1,
            }),
            vec![TrustEvent {
                kind: TrustEventKind::SuccessfulMission,
                impact: 0.0,
                timestamp: chrono::Utc::now(),
                context: "Done".to_string(),
            }],
            false,
        );
        let output = engine.update(&input);
        assert!(output.trust_score >= 0.0);
    }
}
