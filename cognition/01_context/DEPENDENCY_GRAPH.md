# Context Module Dependency Graph

## Overview

This document shows how all context modules in `01_context/` interact with each other and with the broader VOXY architecture. The dependency graph is designed for a system expected to operate continuously for years — no circular dependencies, clear data flow, and graceful degradation.

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                           CONTEXT MODULE DEPENDENCY GRAPH                             │
│                                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                         CONTEXT FUSION (10_context_fusion.md)                │    │
│  │                         The brain — combines all sources                      │    │
│  │                                                                              │    │
│  │  Inputs from ALL context modules:                                            │    │
│  │  ◄── 01_environment.md    (temporal, system, network, display, power)        │    │
│  │  ◄── 02_user_context.md   (task, project, goals, attention, fatigue, mode)   │    │
│  │  ◄── 03_conversation_context.md (turns, topics, intents, summary)            │    │
│  │  ◄── 04_visual_context.md (screen, OCR, UI elements, clipboard)             │    │
│  │  ◄── 05_audio_context.md  (speech, speakers, noise, wake word)              │    │
│  │  ◄── 06_memory_context.md (retrieval, working memory, consolidation)         │    │
│  │  ◄── 07_device_context.md (USB, Bluetooth, monitors, storage, sensors)       │    │
│  │  ◄── 08_activity_context.md (coding, gaming, meetings, browsing)             │    │
│  │  ◄── 09_emotional_context.md (tone, confidence, uncertainty)                 │    │
│  │                                                                              │    │
│  │  Output: AssembledContext ──► Cognition (00_core/)                           │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                      │                                               │
│                                      │ consumes                                      │
│                                      ▼                                               │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                         COGNITION SYSTEM (00_core/)                          │    │
│  │                                                                              │    │
│  │  AssembledContext flows into:                                                │    │
│  │  ├── CognitiveLoop (cognitive_loop.md)                                       │    │
│  │  ├── DecisionPipeline (decision_pipeline.md)                                 │    │
│  │  ├── AttentionSystem (attention_system.md)                                   │    │
│  │  ├── GoalManager (goal_manager.md)                                           │    │
│  │  ├── TaskManager (task_manager.md)                                           │    │
│  │  ├── Planner (planner.md)                                                    │    │
│  │  ├── Reasoning (reasoning.md)                                                │    │
│  │  ├── Reflection (reflection.md)                                              │    │
│  │  └── SelfMonitoring (self_monitoring.md)                                     │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Inter-Module Dependencies

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    INTER-MODULE DEPENDENCY MATRIX                                     │
│                                                                                      │
│  Module              Depends On                    Depended On By                     │
│  ──────────────────────────────────────────────────────────────────────────────────  │
│  01_environment      (none — OS APIs)              08_activity, 10_fusion            │
│  02_user_context     01_environment                08_activity, 09_emotional, 10_fusion│
│  03_conversation     (none — conversation crate)   10_fusion, Cognition              │
│  04_visual           (none — vision crate)          08_activity, 10_fusion            │
│  05_audio            (none — audio/voice crates)   09_emotional, 10_fusion           │
│  06_memory           (none — memory crate)          10_fusion, Cognition              │
│  07_device           (none — OS APIs)              10_fusion                          │
│  08_activity         01_env, 04_visual, 05_audio   10_fusion, 02_user                │
│  09_emotional        03_conversation, 05_audio     10_fusion                          │
│  10_fusion           ALL modules (01-09)            Cognition (00_core)               │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                         CONTEXT DATA FLOW                                             │
│                                                                                      │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐        │
│  │   OS     │   │  Visual  │   │  Audio   │   │  Memory  │   │  Device  │        │
│  │   APIs   │   │  Crate   │   │  Crate   │   │  Crate   │   │  APIs    │        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘        │
│       │              │              │              │              │                │
│       ▼              ▼              ▼              ▼              ▼                │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐        │
│  │   01     │   │   04     │   │   05     │   │   06     │   │   07     │        │
│  │  env     │   │ visual   │   │  audio   │   │  memory  │   │  device  │        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘   └────┬─────┘        │
│       │              │              │              │              │                │
│       │         ┌────┴────┐    ┌────┴────┐         │              │                │
│       │         │         │    │         │         │              │                │
│       ▼         ▼         ▼    ▼         ▼         ▼              ▼                │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐                        │
│  │   08     │   │   02     │   │   09     │   │          │                        │
│  │ activity │◄──│  user    │   │ emotional│◄──│          │                        │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   │          │                        │
│       │              │              │          │          │                        │
│       │              │    ┌─────────┘          │          │                        │
│       │              │    │                    │          │                        │
│       ▼              ▼    ▼                    ▼          ▼                        │
│  ┌──────────────────────────────────────────────────────────────┐                  │
│  │                    CONTEXT FUSION (10)                        │                  │
│  │                                                               │                  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐  │                  │
│  │  │Priority │  │Conflict │  │Confidence│  │  Freshness   │  │                  │
│  │  │ System  │  │Resolution│  │ Scoring │  │  Management  │  │                  │
│  │  └─────────┘  └─────────┘  └─────────┘  └──────────────┘  │                  │
│  │                                                               │                  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌──────────────┐  │                  │
│  │  │ Context │  │ Context │  │ Context │  │   Context    │  │                  │
│  │  │ Merging │  │Compression│ │ Caching │  │Synchronization│ │                  │
│  │  └─────────┘  └─────────┘  └─────────┘  └──────────────┘  │                  │
│  │                                                               │                  │
│  └──────────────────────────┬───────────────────────────────────┘                  │
│                              │                                                       │
│                              ▼                                                       │
│  ┌──────────────────────────────────────────────────────────────┐                  │
│  │              AssembledContext                                  │                  │
│  │  sources: [Environment, User, Conversation, Visual, Audio,   │                  │
│  │            Memory, Device, Activity, Emotional]               │                  │
│  │  world_snapshot: WorldSnapshot                                │                  │
│  │  confidence: 0.85                                             │                  │
│  │  freshness: 1s                                                │                  │
│  └──────────────────────────┬───────────────────────────────────┘                  │
│                              │                                                       │
│                              ▼                                                       │
│  ┌──────────────────────────────────────────────────────────────┐                  │
│  │              COGNITION (00_core/)                             │                  │
│  │                                                               │                  │
│  │  CognitiveLoop ◄── AssembledContext                          │                  │
│  │  DecisionPipeline ◄── AssembledContext                       │                  │
│  │  AttentionSystem ◄── AssembledContext                        │                  │
│  │  GoalManager ◄── AssembledContext                            │                  │
│  │  Planner ◄── AssembledContext                                │                  │
│  │  Reasoning ◄── AssembledContext                              │                  │
│  │  ToolSelector ◄── AssembledContext                           │                  │
│  └──────────────────────────────────────────────────────────────┘                  │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## External Dependencies

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    EXTERNAL CRATE DEPENDENCIES                                        │
│                                                                                      │
│  Context Module          Depends On Crates                                           │
│  ──────────────────────────────────────────────────────────────────────────────────  │
│  01_environment     ◄── world_model (WorldSnapshot, UserEnvironment, DesktopState)   │
│                      ◄── platform_core (OS APIs)                                    │
│                                                                                      │
│  02_user_context    ◄── world_model (ActiveTask, DesktopState)                      │
│                      ◄── conversation (ConversationContext)                          │
│                                                                                      │
│  03_conversation    ◄── conversation (ConversationContext, ContextTracker, Turns)    │
│                      ◄── orchestrator (ExecutionContext)                             │
│                                                                                      │
│  04_visual          ◄── vision (CapturedFrame, OcrResult, UiHierarchy, Changes)     │
│                      ◄── grounding (GroundingRequest, ResolvedTarget)                │
│                      ◄── vision_provider (Platform implementations)                 │
│                                                                                      │
│  05_audio           ◄── audio (DSP, Device Management)                              │
│                      ◄── voice (VoicePipeline, VAD, WakeWord)                       │
│                      ◄── whisper (Speech Recognition)                                │
│                      ◄── voice_orchestrator (VoiceEvents, Traits)                   │
│                                                                                      │
│  06_memory          ◄── memory (MemoryManager, Episodic, Semantic, Procedural)      │
│                      ◄── embeddings (Embedding generation)                           │
│                                                                                      │
│  07_device          ◄── hardware (Device Detection)                                 │
│                      ◄── home (Smart Home Integration)                              │
│                                                                                      │
│  08_activity        ◄── world_model (DesktopState, WindowInfo)                      │
│                      ◄── vision (VisualFeatures)                                    │
│                      ◄── audio (AudioFeatures)                                      │
│                                                                                      │
│  09_emotional       ◄── conversation (TurnText)                                     │
│                      ◄── voice_orchestrator (VoiceFeatures)                         │
│                      ◄── personality (MoodState, CommunicationStyle)                │
│                                                                                      │
│  10_fusion          ◄── cognition (ContextAssembler, AssembledContext)               │
│                      ◄── All context modules (01-09)                                 │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Crate Integration Points

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    CRATE INTEGRATION POINTS                                           │
│                                                                                      │
│  VOXY Crate              Context Integration                                         │
│  ──────────────────────────────────────────────────────────────────────────────────  │
│  world_model             ◄── Provides WorldSnapshot to Context Fusion                │
│                          ◄── Provides DesktopState, UserEnvironment                 │
│                          ◄── Consumed by: 01_environment, 08_activity               │
│                                                                                      │
│  conversation            ◄── Provides ConversationContext to Context Fusion          │
│                          ◄── Provides Turn history, topic tracking                  │
│                          ◄── Consumed by: 03_conversation, Cognition               │
│                                                                                      │
│  vision                  ◄── Provides CapturedFrame, OcrResult, UiHierarchy        │
│                          ◄── Consumed by: 04_visual, 08_activity                   │
│                                                                                      │
│  voice                   ◄── Provides VoicePipeline, VAD, WakeWord                 │
│                          ◄── Consumed by: 05_audio                                  │
│                                                                                      │
│  audio                   ◄── Provides DSP, Device Management                       │
│                          ◄── Consumed by: 05_audio                                  │
│                                                                                      │
│  memory                  ◄── Provides MemoryManager, Episodic/Semantic/Procedural  │
│                          ◄── Consumed by: 06_memory                                 │
│                                                                                      │
│  grounding               ◄── Provides GroundingRequest, ResolvedTarget              │
│                          ◄── Consumed by: 04_visual                                 │
│                                                                                      │
│  personality             ◄── Provides MoodState, CommunicationStyle                 │
│                          ◄── Consumed by: 09_emotional                              │
│                                                                                      │
│  orchestrator            ◄── Provides ExecutionContext, PipelineStage               │
│                          ◄── Consumes AssembledContext from 10_fusion               │
│                                                                                      │
│  cognition               ◄── Provides ContextAssembler trait                        │
│                          ◄── Consumes AssembledContext                               │
│                          ◄── 10_fusion implements ContextAssembler                  │
│                                                                                      │
│  platform_core           ◄── Provides OS-specific APIs                              │
│                          ◄── Consumed by: 01_environment, 07_device                │
│                                                                                      │
│  hardware                ◄── Provides Device Detection                              │
│                          ◄── Consumed by: 07_device                                 │
│                                                                                      │
│  home                    ◄── Provides Smart Home Integration                        │
│                          ◄── Consumed by: 07_device                                 │
│                                                                                      │
│  embeddings              ◄── Provides Embedding generation                          │
│                          ◄── Consumed by: 06_memory                                 │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Module Interaction Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                    MODULE INTERACTION MATRIX                                           │
│                                                                                      │
│           │ 01  │ 02  │ 03  │ 04  │ 05  │ 06  │ 07  │ 08  │ 09  │ 10  │           │
│  ─────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤           │
│  01  env  │  -  │  →  │     │     │     │     │     │  →  │     │  →  │           │
│  02  user │     │  -  │     │     │     │     │     │  →  │  →  │  →  │           │
│  03  conv │     │     │  -  │     │     │     │     │     │  →  │  →  │           │
│  04  vis  │     │     │     │  -  │     │     │     │  →  │     │  →  │           │
│  05  aud  │     │     │     │     │  -  │     │     │     │  →  │  →  │           │
│  06  mem  │     │     │     │     │     │  -  │     │     │     │  →  │           │
│  07  dev  │     │     │     │     │     │     │  -  │     │     │  →  │           │
│  08  act  │  ←  │  ←  │     │  ←  │  ←  │     │     │  -  │     │  →  │           │
│  09  emo  │     │  ←  │  ←  │     │  ←  │     │     │     │  -  │  →  │           │
│  10  fus  │  ←  │  ←  │  ←  │  ←  │  ←  │  ←  │  ←  │  ←  │  ←  │  -  │           │
│                                                                                      │
│  Legend: → = depends on, ← = depended on by, - = self                               │
│                                                                                      │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

## Key Design Principles

1. **No circular dependencies**: Context modules form a DAG (Directed Acyclic Graph)
2. **Clear data flow**: Data flows from raw sources → context modules → fusion → cognition
3. **Graceful degradation**: If a context module fails, fusion continues with remaining sources
4. **Loose coupling**: Context modules communicate through well-defined interfaces
5. **Independent evolution**: Each context module can evolve independently
6. **Testability**: Each context module can be tested in isolation
7. **Performance**: Critical path is optimized (env, conversation → fusion → cognition)
8. **Privacy by design**: Each module enforces its own privacy boundaries
9. **Security by design**: Each module enforces its own access controls
10. **Extensibility**: New context sources can be added without modifying existing modules

## Critical Path

The critical path for context assembly is:

```
User Input → Conversation Context → Context Fusion → Cognition
```

This path must complete within the latency budget:
- Conversation Context: ~2ms
- Context Fusion: ~6ms
- **Total critical path: ~8ms**

Non-critical context sources (environment, device, activity, etc.) are collected asynchronously and are available when needed, but not on the critical path.
