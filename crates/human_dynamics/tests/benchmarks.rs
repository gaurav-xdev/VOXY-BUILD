use std::time::{Duration, Instant};

use voxy_human_dynamics::engine::HumanDynamicsEngine;
use voxy_human_dynamics::types::*;

fn bench_config() -> voxy_human_dynamics::config::HdrConfig {
    voxy_human_dynamics::config::HdrConfig::default()
}

fn bench_input(pending: bool) -> HdrInput {
    HdrInput {
        now: chrono::Utc::now(),
        instant_now: Instant::now(),
        user_id: UserId("bench-user".to_string()),
        user_present: true,
        current_behavior: BehaviorState::Observing,
        activity_description: "Benchmark run".to_string(),
        pending_action: if pending {
            Some(Action {
                id: "bench-action".to_string(),
                kind: ActionKind::Speak,
                description: "Status update".to_string(),
                protection_level: ProtectionLevel::Low,
                reversible: true,
                impact: 0.1,
            })
        } else {
            None
        },
        recent_trust_events: vec![TrustEvent {
            kind: TrustEventKind::TaskCompleted,
            impact: 0.0,
            timestamp: chrono::Utc::now(),
            context: "Done".to_string(),
        }],
        time_since_last_interaction: Duration::from_secs(30),
        session_duration: Duration::from_secs(600),
        errors_this_session: 0,
        corrections_this_session: 0,
        missions_completed: 5,
        missions_failed: 0,
        is_meeting: false,
        focus_level: 0.5,
        stress_level: 0.2,
    }
}

#[test]
fn bench_full_pipeline_with_action() {
    let mut engine = HumanDynamicsEngine::new(bench_config());
    let input = bench_input(true);
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _output = engine.update(&input);
    }
    let elapsed = start.elapsed();
    let per_iter_us = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Full pipeline (with action): {:.1}μs/iter ({:.0} iterations/sec)",
        per_iter_us,
        iterations as f64 / elapsed.as_secs_f64()
    );
    assert!(
        per_iter_us < 100.0,
        "Full pipeline latency {}μs exceeds 100μs target",
        per_iter_us
    );
}

#[test]
fn bench_full_pipeline_no_action() {
    let mut engine = HumanDynamicsEngine::new(bench_config());
    let input = bench_input(false);
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _output = engine.update(&input);
    }
    let elapsed = start.elapsed();
    let per_iter_us = elapsed.as_micros() as f64 / iterations as f64;
    println!(
        "Full pipeline (no action): {:.1}μs/iter ({:.0} iterations/sec)",
        per_iter_us,
        iterations as f64 / elapsed.as_secs_f64()
    );
    assert!(
        per_iter_us < 100.0,
        "Full pipeline latency {}μs exceeds 100μs target",
        per_iter_us
    );
}

#[test]
fn bench_pipeline_100_events() {
    let mut engine = HumanDynamicsEngine::new(bench_config());
    let events: Vec<TrustEvent> = (0..100)
        .map(|_| TrustEvent {
            kind: TrustEventKind::SuccessfulMission,
            impact: 0.0,
            timestamp: chrono::Utc::now(),
            context: "Done".to_string(),
        })
        .collect();
    let input = HdrInput {
        now: chrono::Utc::now(),
        instant_now: Instant::now(),
        user_id: UserId("bench-user".to_string()),
        user_present: true,
        current_behavior: BehaviorState::Observing,
        activity_description: "Benchmark".to_string(),
        pending_action: Some(Action {
            id: "bench-action".to_string(),
            kind: ActionKind::Speak,
            description: "Status".to_string(),
            protection_level: ProtectionLevel::Low,
            reversible: true,
            impact: 0.1,
        }),
        recent_trust_events: events,
        time_since_last_interaction: Duration::from_secs(30),
        session_duration: Duration::from_secs(600),
        errors_this_session: 0,
        corrections_this_session: 0,
        missions_completed: 5,
        missions_failed: 0,
        is_meeting: false,
        focus_level: 0.5,
        stress_level: 0.2,
    };
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _output = engine.update(&input);
    }
    let elapsed = start.elapsed();
    let per_iter_us = elapsed.as_micros() as f64 / iterations as f64;
    println!("Full pipeline (100 events): {:.1}μs/iter", per_iter_us);
    assert!(
        per_iter_us < 200.0,
        "Pipeline with 100 events latency {}μs exceeds 200μs target",
        per_iter_us
    );
}
