# VOXY Self-Healing System

## Overview

The `SelfHealer` component provides automatic recovery for failed subsystems using exponential backoff and configurable restart strategies.

## Configuration

```rust
voxy_runtime_guard::HealingConfig {
    base_backoff_ms: 1000,       // Initial backoff: 1 second
    max_backoff_ms: 30000,       // Max backoff: 30 seconds
    max_restart_attempts: 3,     // Before cooldown
    cooldown_secs: 300,          // 5 minute cooldown
}
```

## Behavior

1. **First failure**: waits `base_backoff_ms`, then attempts restart
2. **Subsequent failures**: backoff doubles (1s → 2s → 4s → 8s → ...)
3. **Max attempts reached**: enters cooldown for `cooldown_secs`
4. **After cooldown**: resets failure count, allows restart again

## Registering a Subsystem

```rust
let healer = SelfHealer::new(config);
healer.register("my_subsystem", || async {
    // Restart logic: re-initialize the subsystem
    Ok(())
}).await;
```

## Triggering Healing

```rust
match healer.heal("my_subsystem").await {
    Ok(()) => tracing::info!("Subsystem recovered"),
    Err(e) => tracing::error!("Healing failed: {e}"),
}
```

## Manual Reset

```rust
healer.reset("my_subsystem").await;
// Failure count cleared, can heal again
```

## Checking State

```rust
let can_heal = healer.can_heal("my_subsystem").await;
let state = healer.get_state("my_subsystem").await;
let cooled_down = healer.is_cooled_down("my_subsystem").await;
```

## Integration

In the daemon, subsystems are registered via `RuntimeGuard::register_healable()` which automatically:
1. Registers the health check with `HealthMonitor`
2. Registers the heartbeat
3. Registers the restart function with `SelfHealer`

When a heartbeat goes stale, the guard can trigger `heal()` on the affected subsystem.
