# VOXY Recovery Flow

## Overview

This document describes the end-to-end recovery flow when a VOXY subsystem fails.

## Recovery Sequence

```
1. Subsystem fails
   ↓
2. Heartbeat stops (no new beat within 30s)
   ↓
3. HeartbeatTracker marks subsystem as dead
   ↓
4. RuntimeGuard detects dead subsystem via is_alive()
   ↓
5. SelfHealer.heal() called with exponential backoff
   ↓
6. Restart function executes (re-initialize subsystem)
   ↓
7. If restart fails: backoff doubles, retry
   ↓
8. If max_attempts exceeded: enter cooldown (5 min)
   ↓
9. After cooldown: reset failure count, allow retry
```

## Subsystem Recovery Matrix

| Subsystem | Restart Action | Max Attempts | Cooldown |
|-----------|---------------|--------------|----------|
| voice_pipeline | Re-init audio capture + STT + TTS | 3 | 300s |
| cognitive_bridge | Re-create orchestrator | 3 | 300s |
| experience_bridge | Re-start companion intelligence | 3 | 300s |
| desktop_bridge | Re-start world model + automation | 3 | 300s |

## Failure Scenarios

### Audio Device Lost
- WASAPI device disconnected
- Detection: `CpalInputStream` returns error on next read
- Recovery: Re-enumerate devices, create new stream with negotiated config

### Whisper Model Crash
- Whisper inference thread panics
- Detection: `spawn_blocking` returns `JoinError`
- Recovery: Re-create `WhisperState` (reloads ~225MB KV cache)

### Ollama Server Down
- Ollama process killed or port unreachable
- Detection: HTTP request fails in `health()` or `complete()`
- Recovery: No restart needed; health check marks as degraded, retries on next request

### Desktop Bridge Timeout
- UIA automation not responding
- Detection: `DesktopEventBridge` fails to get context
- Recovery: Re-create bridge, re-subscribe to desktop events

## Monitoring

- `RuntimeGuard::snapshot()` provides point-in-time status of all subsystems
- `RuntimeGuard::dashboard()` generates HTML view with subsystem health
- Heartbeats sent every 5 seconds in daemon's main timer loop
- Health checks run on-demand via `HealthMonitor::check_all()`
