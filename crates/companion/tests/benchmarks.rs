use std::time::{Duration, Instant};

use chrono::Utc;
use voxy_companion::attention::ActivityKind;
use voxy_companion::config::CompanionConfig;
use voxy_companion::engine::CompanionEngine;
use voxy_companion::personality::CompanionPersonality;
use voxy_companion::types::*;

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

#[test]
fn bench_engine_update_latency() {
    let mut engine =
        CompanionEngine::new(CompanionConfig::default(), CompanionPersonality::default());
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));

    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        engine.update(&input);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;

    eprintln!(
        "Engine update avg: {:.1}us ({} iterations)",
        avg_us, iterations
    );
    assert!(
        avg_us < 200.0,
        "Average update latency {:.0}us exceeds 200us target",
        avg_us
    );
}

#[test]
fn bench_attention_model_latency() {
    let mut model = voxy_companion::AttentionModel::new();
    let now = Instant::now();
    let iterations = 10000;
    let start = Instant::now();
    for i in 0..iterations {
        let activity = if i % 2 == 0 {
            Some(ActivityKind::Coding)
        } else {
            Some(ActivityKind::Browsing)
        };
        let _ = model.update(
            activity,
            Duration::from_secs(10),
            0.2,
            None,
            now + Duration::from_millis(i as u64),
        );
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Attention model avg: {:.1}us", avg_us);
    assert!(
        avg_us < 50.0,
        "Attention model avg {:.0}us exceeds 50us",
        avg_us
    );
}

#[test]
fn bench_presence_system_latency() {
    let mut system = voxy_companion::PresenceSystem::default();
    let dt = Duration::from_millis(100);
    let presence = UserPresence::Active;
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = system.tick(dt, &presence);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Presence system avg: {:.1}us", avg_us);
    assert!(
        avg_us < 20.0,
        "Presence system avg {:.0}us exceeds 20us",
        avg_us
    );
}

#[test]
fn bench_greeting_engine_latency() {
    let mut engine = voxy_companion::GreetingEngine::default();
    let iterations = 1000;
    let start = Instant::now();
    for i in 0..iterations {
        let time = match i % 4 {
            0 => TimeContext::Morning,
            1 => TimeContext::Afternoon,
            2 => TimeContext::Evening,
            _ => TimeContext::Night,
        };
        let _ = engine.generate(
            time,
            WeatherContext::Clear,
            Duration::from_secs(600),
            i % 5 == 0,
            false,
            &[],
            Instant::now(),
        );
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Greeting engine avg: {:.1}us", avg_us);
    assert!(
        avg_us < 100.0,
        "Greeting engine avg {:.0}us exceeds 100us",
        avg_us
    );
}

#[test]
fn bench_silence_intelligence_latency() {
    let mut si = voxy_companion::SilenceIntelligence::default();
    let attention = voxy_companion::AttentionState {
        activity: ActivityKind::Coding,
        focus_level: 0.7,
        deep_focus: false,
        can_interrupt: true,
        stress_estimate: 0.2,
        state_duration: Duration::from_secs(60),
        detection_confidence: 0.8,
    };
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = si.decide(&attention, true, Instant::now());
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Silence intelligence avg: {:.1}us", avg_us);
    assert!(
        avg_us < 10.0,
        "Silence intelligence avg {:.0}us exceeds 10us",
        avg_us
    );
}

#[test]
fn bench_micro_engine_latency() {
    let mut engine = voxy_companion::MicroEngine::default();
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.generate(0.5, 3, 2, false, Instant::now());
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Micro engine avg: {:.1}us", avg_us);
    assert!(
        avg_us < 50.0,
        "Micro engine avg {:.0}us exceeds 50us",
        avg_us
    );
}

#[test]
fn bench_presence_score_latency() {
    let mut engine = voxy_companion::PresenceScoreEngine::default();
    let attention = voxy_companion::AttentionState {
        activity: ActivityKind::Coding,
        focus_level: 0.8,
        deep_focus: true,
        can_interrupt: false,
        stress_estimate: 0.2,
        state_duration: Duration::from_secs(60),
        detection_confidence: 0.8,
    };
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = engine.compute(&input, &attention);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!("Presence score avg: {:.1}us", avg_us);
    assert!(
        avg_us < 20.0,
        "Presence score avg {:.0}us exceeds 20us",
        avg_us
    );
}

#[test]
fn bench_full_pipeline_latency() {
    let mut engine =
        CompanionEngine::new(CompanionConfig::default(), CompanionPersonality::default());
    let input = make_input(UserPresence::Active, Some(ActivityKind::Coding));
    let iterations = 5000;
    let start = Instant::now();
    for i in 0..iterations {
        let presence = if i % 10 == 0 {
            UserPresence::Idle { since: Utc::now() }
        } else {
            UserPresence::Active
        };
        let activity = if i % 3 == 0 {
            Some(ActivityKind::Coding)
        } else if i % 3 == 1 {
            Some(ActivityKind::Browsing)
        } else {
            Some(ActivityKind::Reading)
        };
        let inp = CompanionInput {
            user_presence: presence,
            current_activity: activity,
            focus_level: 0.5 + (i as f64 % 5.0) * 0.1,
            ..input.clone()
        };
        let _ = engine.update(&inp);
    }
    let elapsed = start.elapsed();
    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    eprintln!(
        "Full pipeline avg: {:.1}us ({} iterations)",
        avg_us, iterations
    );
    assert!(
        avg_us < 200.0,
        "Full pipeline avg {:.0}us exceeds 200us target",
        avg_us
    );
}
