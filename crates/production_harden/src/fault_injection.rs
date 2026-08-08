//! Fault injection — simulated failures for resilience testing.

#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use voxy_event_bus::EventBus;
#[cfg(test)]
use voxy_integration::recovery::{
    RecoveryAction, RecoveryConfig, SubsystemRecovery, SubsystemState,
};
#[cfg(test)]
use voxy_integration::telemetry::{CentralTelemetry, SubsystemMetrics};

#[tokio::test]
async fn fault_subsystem_crash_recovery() {
    let recovery = SubsystemRecovery::new(RecoveryConfig {
        max_restarts: 3,
        restart_delay_ms: 10,
        backoff_multiplier: 1.0,
        max_backoff_ms: 50,
        circuit_breaker_threshold: 5,
        circuit_breaker_reset_ms: 100,
    });

    recovery.register("llm_provider");
    recovery.register("stt_provider");
    recovery.register("tts_provider");

    recovery.mark_running("llm_provider");
    let action = recovery.report_failure("llm_provider", "connection timeout");
    assert!(matches!(action, RecoveryAction::Restart { .. }));

    let stt_health = recovery.health("stt_provider").unwrap();
    assert!(matches!(stt_health.state, SubsystemState::Stopped));

    let llm_health = recovery.health("llm_provider").unwrap();
    assert_eq!(llm_health.restart_count, 1);
}

#[tokio::test]
async fn fault_cascading_failure_isolation() {
    let recovery = SubsystemRecovery::new(RecoveryConfig {
        max_restarts: 1,
        restart_delay_ms: 10,
        backoff_multiplier: 1.0,
        max_backoff_ms: 50,
        circuit_breaker_threshold: 3,
        circuit_breaker_reset_ms: 100,
    });

    recovery.register("memory");
    recovery.register("planner");
    recovery.register("agents");

    recovery.mark_running("memory");
    recovery.report_failure("memory", "disk full");
    recovery.report_failure("memory", "disk full");
    let action = recovery.report_failure("memory", "disk full");

    assert!(matches!(action, RecoveryAction::CircuitOpen { .. }));

    let planner_health = recovery.health("planner").unwrap();
    let agents_health = recovery.health("agents").unwrap();
    assert!(matches!(planner_health.state, SubsystemState::Stopped));
    assert!(matches!(agents_health.state, SubsystemState::Stopped));
}

#[tokio::test]
async fn fault_event_bus_dead_letters() {
    let bus = EventBus::with_dead_letter(16, 10);

    let mut rx = bus.subscribe("test.topic").await.unwrap();

    for i in 0..5 {
        let event = voxy_shared::Event::new("test.topic", "test", vec![i]);
        let _ = bus.publish_with_dead_letter("test.topic", event).await;
    }

    let mut received = 0;
    while rx.try_recv().is_ok() {
        received += 1;
    }
    assert!(received > 0, "Should receive messages");
}

#[tokio::test]
async fn fault_telemetry_extreme_load() {
    let telemetry = CentralTelemetry::with_thresholds(100.0, 5, 1024, 80.0);

    for i in 0..200 {
        telemetry.report(SubsystemMetrics {
            name: format!("svc_{}", i % 5),
            latency_ms: if i % 3 == 0 { 500.0 } else { 50.0 },
            error_count: if i % 4 == 0 { 20 } else { 0 },
            warning_count: 0,
            cpu_percent: if i % 5 == 0 { 95.0 } else { 20.0 },
            memory_mb: if i % 6 == 0 { 2048 } else { 100 },
            queue_size: 0,
            events_per_sec: 0.0,
            uptime_seconds: 0,
            last_error: None,
            timestamp: chrono::Utc::now(),
        });
    }

    let alerts = telemetry.alerts();
    assert!(alerts.len() <= 100);

    let snap = telemetry.snapshot();
    assert_eq!(snap.subsystems.len(), 5);
}

#[tokio::test]
async fn fault_memory_capacity_enforcement() {
    let mut mem = voxy_memory::LongTermMemoryV2::new(100, 50, 10, 0.1);

    for i in 0..200 {
        let item = voxy_memory::MemoryItemV2 {
            id: voxy_memory::ltm_v2::MemoryId::new(),
            category: voxy_memory::MemoryCategory::Semantic,
            content: format!("Item {}", i),
            summary: None,
            tags: Vec::new(),
            importance: voxy_memory::ltm_v2::ImportanceFactors::default(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
            access_count: 0,
            version: 1,
            compressed: false,
            archived: false,
            project_id: None,
            related_memory_ids: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };
        mem.store(item);
    }

    assert!(
        mem.count() <= 110,
        "Memory should enforce capacity: {}",
        mem.count()
    );
}

#[tokio::test]
async fn fault_plugin_crash_isolation() {
    use voxy_cognitive_orchestrator::sdk::{MockPlugin, PluginRegistry};

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(MockPlugin::new("good_plugin")))
        .unwrap();
    registry
        .register(Box::new(MockPlugin::new("crash_plugin")))
        .unwrap();

    let result = registry.invoke("good_plugin", "ping", &std::collections::HashMap::new());
    assert!(result.is_ok());

    let result = registry.invoke("nonexistent", "cmd", &std::collections::HashMap::new());
    assert!(result.is_err());

    let result = registry.invoke(
        "good_plugin",
        "nonexistent_cmd",
        &std::collections::HashMap::new(),
    );
    assert!(result.is_err());

    assert_eq!(registry.count(), 2);
}

#[tokio::test]
async fn fault_boot_sequence_failure_recovery() {
    use voxy_integration::boot::{BootPhase, BootSequence, BootStatus};

    let boot = BootSequence::default_config();
    boot.begin();

    for phase in &[BootPhase::Kernel, BootPhase::Config, BootPhase::Database] {
        boot.start_phase(phase);
        boot.complete_phase(phase, None);
    }

    boot.start_phase(&BootPhase::Security);
    let decision = boot.fail_phase(&BootPhase::Security, "auth service unreachable");

    assert!(matches!(
        decision,
        voxy_integration::boot::RecoveryDecision::Retry { .. }
    ));

    let reports = boot.all_reports();
    let kernel_report = reports
        .iter()
        .find(|r| r.phase == BootPhase::Kernel)
        .unwrap();
    assert!(matches!(kernel_report.status, BootStatus::Completed));

    let security_report = reports
        .iter()
        .find(|r| r.phase == BootPhase::Security)
        .unwrap();
    assert!(matches!(security_report.status, BootStatus::Failed { .. }));

    assert!(!boot.is_ready());
}

#[tokio::test]
async fn fault_concurrent_subsystem_failures() {
    let recovery = Arc::new(SubsystemRecovery::new(RecoveryConfig {
        max_restarts: 2,
        restart_delay_ms: 10,
        backoff_multiplier: 1.0,
        max_backoff_ms: 50,
        circuit_breaker_threshold: 3,
        circuit_breaker_reset_ms: 100,
    }));

    for i in 0..10 {
        recovery.register(format!("svc_{}", i));
        recovery.mark_running(&format!("svc_{}", i));
    }

    let mut handles = Vec::new();
    for i in 0..10 {
        let recovery = recovery.clone();
        handles.push(tokio::spawn(async move {
            for _ in 0..5 {
                recovery.report_failure(&format!("svc_{}", i), "simulated crash");
                recovery.mark_running(&format!("svc_{}", i));
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let all_health = recovery.all_health();
    let total_restarts: u32 = all_health.values().map(|h| h.restart_count).sum();
    assert!(total_restarts > 0, "System should have attempted recovery");
}

#[tokio::test]
async fn fault_event_bus_overflow() {
    let bus = EventBus::new(2);

    let mut rx = bus.subscribe("overflow.test").await.unwrap();

    for i in 0..20 {
        let event =
            voxy_shared::Event::new("overflow.test", "test", format!("msg_{}", i).into_bytes());
        let _ = bus.publish("overflow.test", event).await;
    }

    let mut received = 0u64;
    let mut lagged = 0u64;
    loop {
        match rx.try_recv() {
            Ok(_event) => received += 1,
            Err(tokio::sync::broadcast::error::TryRecvError::Lagged(n)) => {
                lagged += n;
                break;
            }
            Err(tokio::sync::broadcast::error::TryRecvError::Empty) => break,
            Err(tokio::sync::broadcast::error::TryRecvError::Closed) => break,
        }
    }
    assert!(
        received + lagged <= 20,
        "Total should not exceed published: received={} lagged={}",
        received,
        lagged
    );
    assert!(
        lagged > 0 || received < 20,
        "Some messages should be lost to overflow: received={}",
        received
    );
}
