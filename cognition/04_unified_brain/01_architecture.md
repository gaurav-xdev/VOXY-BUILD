# Unified Brain Orchestrator

## Overview

The Unified Brain Orchestrator (`voxy-brain`) is the top-level runtime that integrates all four VOXY runtimes into a single asynchronous event-driven pipeline. It enables VOXY to sustain real-time conversations by coordinating context collection, companion presence, human dynamics, and cognitive processing in one unified flow.

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        UnifiedBrainEngine                                │
│                        (orchestrator)                                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────┐ │
│  │  ContextManager   │  │  CompanionEngine │  │ HumanDynamicsEngine    │ │
│  │  (async, Arc)     │  │  (sync, Mutex)   │  │ (sync, Mutex)         │ │
│  │                  │  │                  │  │                        │ │
│  │  collect()       │  │  update()        │  │  update()              │ │
│  │  subscribe()     │  │                  │  │                        │ │
│  └──────────────────┘  └──────────────────┘  └────────────────────────┘ │
│                                                                          │
│  ┌──────────────────┐  ┌──────────────────┐  ┌────────────────────────┐ │
│  │ FusionEngine     │  │ CognitiveEngine  │  │ LatencyTracker         │ │
│  │ (sync)           │  │ (async, Arc)     │  │ (RwLock)               │ │
│  │                  │  │                  │  │                        │ │
│  │  fuse()          │  │  process()       │  │  start_turn()          │ │
│  │                  │  │  diagnostics()   │  │  record()              │ │
│  └──────────────────┘  └──────────────────┘  └────────────────────────┘ │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────────┐│
│  │                     Event Broadcasting (tokio::sync::broadcast)     ││
│  │  BrainEvent::TurnStarted → ContextCollecting → CognitionProcessed  ││
│  │                     → TurnCompleted                                 ││
│  └──────────────────────────────────────────────────────────────────────┘│
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

## Pipeline Flow

```
BrainInput
    │
    ▼
┌─────────────────┐
│ 1. Context      │  Collect context from all providers concurrently
│    Collection    │  → ContextSnapshotSet
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. Companion    │  Update presence, greeting, micro-interactions
│    Update       │  → CompanionOutput
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 3. Human        │  Evaluate trust, behavior, protection, initiative
│    Dynamics     │  → HdrOutput
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 4. Cognition    │  Process intent, plan, reason, decide
│    Processing   │  → CognitiveResult
└────────┬────────┘
         │
         ▼
    BrainOutput
```

## Key Types

### BrainInput
Input to a single conversation turn:
- `session_id` — identifies the conversation session
- `raw_text` — user's message
- `user_presence` — Active, Idle, Away, InMeeting, etc.
- `focus_level`, `stress_level`, `is_meeting` — context signals
- `time_since_last_interaction`, `session_duration` — timing
- `errors_this_session`, `missions_completed/failed` — history

### BrainOutput
Output of a complete pipeline cycle:
- `turn_id` — unique identifier
- `response_text` — extracted cognitive response
- `cognitive_result` — intent, confidence, plan summary
- `companion` — display, silence, presence score, greetings
- `human_dynamics` — trust, relationship, behavior, protection
- `context_summary` — source count, confidence, collection time
- `pipeline_duration_ms` — total latency
- `stage_latencies` — per-stage microsecond breakdown
- `interrupted`, `errors` — error tracking

### BrainEvent (Streaming)
Events broadcast during pipeline execution:
- `TurnStarted`, `TurnCompleted`, `TurnInterrupted`, `TurnFailed`
- `ContextCollecting`, `ContextCollected`
- `CompanionUpdating`, `CompanionUpdated`
- `HdrUpdating`, `HdrUpdated`
- `CognitionProcessing`, `CognitionProcessed`
- `HealthCheck`, `PipelineLatency`

## Lifecycle

```
            init()
              │
              ▼
    ┌─────────────────┐
    │     Idle        │◄──────────────────┐
    └────────┬────────┘                   │
             │ process_turn()             │
             ▼                            │
    ┌─────────────────┐                   │
    │   Processing    │───────────────────┘
    └────────┬────────┘    (success)
             │ cancel_turn()
             ▼
    ┌─────────────────┐
    │  Interrupted    │───────────────────┐
    └─────────────────┘                   │
                                          │
            shutdown()                    │
              │                           │
              ▼                           │
    ┌─────────────────┐                   │
    │ ShuttingDown    │───────────────────┘
    └────────┬────────┘
             │
             ▼
    ┌─────────────────┐
    │    Shutdown     │
    └─────────────────┘
```

## Synchronization Model

| Component | Type | Synchronization | Reason |
|-----------|------|-----------------|--------|
| ContextManager | Async | Internal RwLock | Thread-safe async collection |
| CompanionEngine | Sync | `tokio::sync::Mutex` | `&mut self` required, not Send+Sync |
| HumanDynamicsEngine | Sync | `tokio::sync::Mutex` | `&mut self` required, not Send+Sync |
| CognitiveEngine | Async | `Arc<dyn Trait>` | Send+Sync by trait bound |
| FusionEngine | Sync | Direct (immutable during fuse) | Stateless fusion |
| LatencyTracker | Sync | `parking_lot::RwLock` | Fast reads, occasional writes |
| BrainState | Sync | `parking_lot::RwLock` | State transitions |

## Latency Tracking

The `LatencyTracker` records per-stage microsecond latencies:

```rust
pub struct LatencySnapshot {
    pub total_us: u64,      // Full pipeline
    pub context_us: u64,    // Context collection
    pub companion_us: u64,  // Companion update
    pub hdr_us: u64,        // Human dynamics update
    pub cognition_us: u64,  // Cognitive processing
    pub overhead_us: u64,   // Scheduling/lock overhead
}
```

Supports: current, average, p99, history.

## Test Coverage

| Category | Count | Description |
|----------|-------|-------------|
| Unit | 0 | (in engine modules) |
| Integration | 8 | Full pipeline, lifecycle, health, events, latency |
| **Total** | **8** | |

### Integration Tests
- `test_brain_init_and_shutdown` — lifecycle management
- `test_brain_process_turn` — full pipeline with all 4 runtimes
- `test_brain_multiple_turns` — session state accumulation
- `test_brain_health_check` — component health reporting
- `test_brain_latency_tracking` — microsecond latency tracking
- `test_brain_shutdown_blocks_processing` — shutdown guard
- `test_brain_cancel_turn` — interruption handling
- `test_brain_events` — streaming event broadcast
