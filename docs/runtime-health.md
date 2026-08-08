# VOXY Runtime Health Monitoring

## Overview

The `voxy-runtime-guard` crate provides production-grade health monitoring for all VOXY subsystems. It tracks subsystem health, detects dead heartbeats, attempts self-healing, and exposes a dashboard.

## Architecture

```
RuntimeGuard
├── HealthMonitor    (voxy-health)   — async health check runner
├── HeartbeatTracker (runtime_guard) — heartbeat-based liveness
├── SelfHealer       (runtime_guard) — exponential backoff restarts
├── DashboardData    (runtime_guard) — HTML/JSON status view
└── RuntimeSnapshot  (runtime_guard) — point-in-time status
```

## Usage

```rust
use voxy_runtime_guard::{RuntimeGuard, GuardConfig};

let guard = RuntimeGuard::new(GuardConfig::default());

// Register a subsystem with health check
guard.register_subsystem("audio", || async {
    voxy_health::HealthReport::new("audio", voxy_shared::HealthStatus::Healthy)
}).await;

// Register with self-healing
guard.register_healable(
    "whisper",
    || async { voxy_health::HealthReport::new("whisper", voxy_shared::HealthStatus::Healthy) },
    || async { Ok(()) }, // restart function
).await;

// Send heartbeats
guard.heartbeat("audio");

// Check health
let alive = guard.is_alive("audio");

// Take snapshot
let snap = guard.snapshot().await;

// Generate dashboard
let dash = guard.dashboard().await;
```

## Subsystem Registration

Each subsystem should:
1. Be registered via `register_subsystem()` or `register_healable()`
2. Have heartbeats sent periodically (recommended: every 5 seconds)
3. Return `HealthReport` from its health check function

## Heartbeat Tracker

- Dead detection: subsystem is marked dead if no heartbeat within `max_heartbeat_age` (default: 30s)
- `is_alive()` returns false if no heartbeat received or if dead
- Heartbeats are timestamped with `chrono::Utc`

## Self-Healing

- Exponential backoff: starts at `base_backoff_ms`, doubles each attempt
- Max attempts: `max_restart_attempts` (default: 3)
- Cooldown: after max attempts, wait `cooldown_secs` (default: 300s) before allowing restart
- `reset()` clears failure state for a subsystem

## Dashboard

- HTML dashboard with CSS styling, auto-refresh, subsystem table
- JSON snapshot for programmatic access
- System metrics (CPU, RAM) included in snapshot

## Integration with Daemon

The daemon creates a `RuntimeGuard` and registers all core subsystems:
- `voice_pipeline` — audio capture, wake word, VAD, STT, TTS
- `cognitive_bridge` — orchestrator, reflection, experience
- `experience_bridge` — companion intelligence, learning
- `desktop_bridge` — world model, automation

Heartbeats are sent in the main timer loop (every 5 seconds).
