use std::time::{Duration, Instant};

use chrono::Utc;
use voxy_companion::attention::ActivityKind;
use voxy_companion::config::CompanionConfig;
use voxy_companion::engine::CompanionEngine;
use voxy_companion::personality::CompanionPersonality;
use voxy_companion::types::*;

fn test_config() -> CompanionConfig {
    CompanionConfig::default()
}

fn test_personality() -> CompanionPersonality {
    CompanionPersonality::default_desktop()
}

fn make_input(presence: UserPresence, activity: Option<ActivityKind>) -> CompanionInput {
    CompanionInput {
        now: Utc::now(),
        session_id: SessionId::new(),
        user_presence: presence,
        current_activity: activity,
        time_since_last_interaction: Duration::from_secs(600),
        conversation_count_this_session: 2,
        total_session_duration: Duration::from_secs(3600),
        active_goals: vec!["Build context engine".to_string()],
        recent_milestones: vec!["Context Fusion complete".to_string()],
        weather: WeatherContext::Clear,
        stress_estimate: 0.2,
        idle_duration: Duration::from_secs(10),
        pending_tasks: 3,
        completed_tasks_today: 5,
        last_greeting: None,
        last_micro_interaction: None,
        last_memory_reference: None,
        mission_state: MissionState::Idle,
        focus_level: 0.7,
    }
}

// ============================================================================
// Integration Tests — Full Engine Lifecycle
// ============================================================================

#[test]
fn test_engine_creation() {
    let engine = CompanionEngine::new(test_config(), test_personality());
    assert_eq!(engine.update_count(), 0);
}

#[test]
fn test_engine_default() {
    let engine = CompanionEngine::default();
    assert_eq!(engine.update_count(), 0);
    assert!(engine.presence_state().to_breathing_speed() > 0.0);
}

#[test]
fn test_engine_multiple_updates() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));

    for _ in 0..100 {
        let output = engine.update(&input);
        assert!(output.update_latency_us < 2000);
    }
    assert_eq!(engine.update_count(), 100);
}

#[test]
fn test_engine_presence_transitions() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());

    // Active
    let output = engine.update(&make_input(
        UserPresence::Active,
        Some(ActivityKind::Coding),
    ));
    assert!(!output.silence || output.display.is_none());

    // Idle
    let input = make_input(
        UserPresence::Idle { since: Utc::now() },
        Some(ActivityKind::Browsing),
    );
    let output = engine.update(&input);
    assert!(output.presence_score < 0.8);

    // Away
    let input = make_input(UserPresence::Away { since: Utc::now() }, None);
    let output = engine.update(&input);
    assert!(output.presence_score < 0.5);
}

#[test]
fn test_engine_greeting_on_return() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());
    let input = CompanionInput {
        time_since_last_interaction: Duration::from_secs(600),
        ..make_input(UserPresence::Active, Some(ActivityKind::Coding))
    };
    let output = engine.update(&input);
    // Should get a greeting or be silent
    assert!(output.greeting.is_some() || output.silence);
}

#[test]
fn test_engine_milestone_recording() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());
    engine.record_milestone("Context Fusion Engine");
    engine.record_milestone("Cognition Integration");
    assert_eq!(engine.journey().entry_count(), 2);
}

#[test]
fn test_engine_mission_lifecycle() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());

    // Start mission
    engine
        .mission_mut()
        .start_mission(ActivityKind::Coding, "Implement feature", Utc::now());
    assert!(engine.mission().is_active());

    // Complete mission
    engine
        .mission_mut()
        .complete_mission("Feature implemented", Utc::now());
    assert!(!engine.mission().is_active());
    assert_eq!(engine.mission().completed_count(), 1);
}

#[test]
fn test_engine_latency_under_load() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));

    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        engine.update(&input);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;

    assert!(
        avg_us < 200.0,
        "Average update latency {:.0}us exceeds 200us target",
        avg_us
    );
}

#[test]
fn test_engine_personality_expression() {
    let engine = CompanionEngine::new(test_config(), test_personality());
    let meta = engine.personality().express(0.8, "Coding", 0.6);
    assert!(meta.confidence > 0.5);
    assert!(meta.energy > 0.0);
}

#[test]
fn test_engine_silence_intelligence() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());

    // Deep focus should produce silent output
    let input = CompanionInput {
        focus_level: 0.95,
        ..make_input(UserPresence::Focused, Some(ActivityKind::Debugging))
    };
    let output = engine.update(&input);
    // Should either be silent or have a very high-confidence reason to speak
    assert!(output.silence || output.greeting.is_some());
}

#[test]
fn test_engine_session_reset() {
    let mut engine = CompanionEngine::new(test_config(), test_personality());
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));

    // Run some updates
    for _ in 0..10 {
        engine.update(&input);
    }
    assert!(engine.update_count() > 0);
}

#[test]
fn test_engine_config_access() {
    let config = test_config();
    let engine = CompanionEngine::new(config.clone(), test_personality());
    assert_eq!(
        engine.config().update_interval,
        CompanionConfig::default().update_interval
    );
}
