# VOXY System Architecture

## Overview

VOXY is an Autonomous AI Operating System built in Rust with a modular, 72-crate workspace architecture. It transforms voice interaction into intelligent autonomous operation through cognitive orchestration, memory management, and task automation.

## Workspace Structure

```
voxy/
├── crates/
│   ├── kernel/              # Core runtime bootstrap and lifecycle
│   ├── config/              # Configuration management
│   ├── database/            # SQLite persistence layer
│   ├── memory/              # Long-term memory (LTM V2), knowledge graph, forgetting
│   ├── event_bus/           # Pub/sub event system with dead letters
│   ├── health/              # Health monitoring and circuit breakers
│   ├── security/            # Consent management, audit logging
│   ├── planner/             # Task graph V2 (DAG), workflow planning
│   ├── cognitive_orchestrator/ # Goal engine, decision engine, self-improvement
│   ├── agent_runtime/       # Multi-agent orchestrator
│   ├── voice/               # Voice pipeline (STT → LLM → TTS)
│   ├── voice_memory/        # Voice-memory binding
│   ├── audio/               # Audio capture, hot-swap, WASAPI
│   ├── model_router/        # AI model routing and provider registry
│   ├── openai/              # OpenAI API provider
│   ├── kokoro/              # Kokoro TTS provider
│   ├── whisper/             # Whisper STT provider
│   ├── automation/          # Desktop automation (mouse, keyboard, OCR)
│   ├── vision/              # Screen capture and analysis
│   ├── ipc/                 # Inter-process communication
│   ├── runtime/             # Async runtime management
│   ├── runtime_guard/       # Runtime safety guards
│   ├── integration/         # TITAN FUSION: ServiceHub, EventBridge, Telemetry
│   ├── production_harden/   # OBSIDIAN: Stress tests, fault injection, benchmarks
│   └── ... (72 total)
```

## Core Layers

### Layer 0: Kernel (`kernel`)
- Boot sequence with 12 deterministic phases
- Configuration loading and validation
- Database initialization
- Security consent verification
- Audio subsystem startup
- Voice pipeline readiness check

### Layer 1: Infrastructure (`event_bus`, `memory`, `health`, `security`)
- **Event Bus**: Tokio broadcast-based pub/sub with 200 max topics, 256 buffer size, dead letter queue, 1MB payload limit
- **Memory**: Long-term memory V2 with 6 categories (Project, Preference, Relationship, Episodic, Semantic, Procedural), importance scoring, forgetting algorithm, memory compression
- **Health**: Circuit breakers, auto-recovery, exponential backoff restart
- **Security**: User consent management, audit logging

### Layer 2: Cognitive Engine (`planner`, `cognitive_orchestrator`, `agent_runtime`)
- **Task Graph V2**: DAG-based task planning with cycle detection, topological layers, critical path analysis
- **Goal Engine V2**: Priority-based goal management with dependencies, sub-goals, auto-unblock
- **Decision Engine**: Weighted scoring with risk assessment and confidence filtering
- **Multi-Agent Orchestrator**: 9 agent roles with task assignment and message passing
- **Self-Improvement Engine**: Performance tracking, correction learning, pattern analysis

### Layer 3: Integration (`integration` — TITAN FUSION)
- **ServiceHub**: Central registry with typed DI resolution
- **EventBridge**: 25 standard topics, typed events (Voice/STT/LLM/Task/Memory/Goal/Agent/System)
- **CentralTelemetry**: Aggregated metrics per subsystem with auto-alerting
- **SubsystemRecovery**: Crash isolation, exponential backoff restart, circuit breaker
- **UnifiedPipeline**: 11-stage execution flow
- **BootSequence**: 12-phase deterministic boot

### Layer 4: Application (`voice`, `audio`, `automation`, `vision`)
- **Voice Pipeline**: Audio capture → STT → LLM → TTS with hot-swap support
- **Audio**: WASAPI backend, hot-swap device detection, ring buffer
- **Automation**: Mouse, keyboard, OCR with safety guards
- **Vision**: Screen capture and analysis

## Data Flow

```
User Speech
    ↓
Audio Capture (WASAPI)
    ↓
STT (Whisper/OpenAI)
    ↓
EventBridge → VoiceSttFinal
    ↓
Conversation Manager → LLM (OpenAI/Anthropic/local)
    ↓
EventBridge → LlmResponse
    ↓
Cognitive Orchestrator (Goal Engine + Decision Engine)
    ↓
Task Graph V2 (planning + DAG)
    ↓
Agent Runtime (multi-agent execution)
    ↓
TTS (Kokoro/OpenAI)
    ↓
Audio Output
    ↓
Memory Storage (LTM V2)
    ↓
EventBridge → TaskCompleted / MemoryStored
```

## Event System

All inter-crate communication flows through the Event Bus with 25 standard topics:

| Topic | Purpose |
|-------|---------|
| `voice.wake` | Wake word detected |
| `voice.audio` | Audio frame |
| `stt.partial` | Partial transcription |
| `stt.final` | Final transcription |
| `llm.request` | LLM request |
| `llm.response` | LLM response |
| `task.queued` | Task queued |
| `task.started` | Task execution started |
| `task.completed` | Task completed |
| `task.failed` | Task failed |
| `goal.created` | Goal created |
| `goal.progress` | Goal progress updated |
| `goal.completed` | Goal completed |
| `memory.stored` | Memory stored |
| `memory.recalled` | Memory recalled |
| `agent.spawned` | Agent spawned |
| `agent.completed` | Agent task completed |
| `system.health` | System health check |
| `system.startup` | System startup |
| `system.shutdown` | System shutdown |

## Key Design Principles

1. **Never rewrite working modules** — extend, don't replace
2. **All changes backward compatible** — existing tests must pass
3. **Isolation over integration** — crash in one subsystem doesn't bring down others
4. **Deterministic boot** — same startup sequence every time
5. **Measurable improvements only** — every change must have benchmarks
