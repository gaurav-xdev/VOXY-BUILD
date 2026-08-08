//! OPERATION TITAN FUSION — Architecture Diagrams
//!
//! # System Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        apps/daemon (main.rs)                       │
//! │                     Deterministic Boot Sequence                     │
//! └──────────────────────────────┬──────────────────────────────────────┘
//!                                │
//!                    ┌───────────▼───────────┐
//!                    │    ServiceHub         │
//!                    │  ┌─────────────────┐  │
//!                    │  │ EventBus (pub/sub)│ │
//!                    │  └─────────────────┘  │
//!                    │  ┌─────────────────┐  │
//!                    │  │  DI Container    │  │
//!                    │  └─────────────────┘  │
//!                    │  ┌─────────────────┐  │
//!                    │  │ Central Telemetry│  │
//!                    │  └─────────────────┘  │
//!                    │  ┌─────────────────┐  │
//!                    │  │ Recovery Manager │  │
//!                    │  └─────────────────┘  │
//!                    └───────────┬───────────┘
//!                                │
//!           ┌────────────────────┼────────────────────┐
//!           │                    │                    │
//!  ┌────────▼────────┐ ┌────────▼────────┐ ┌────────▼────────┐
//!  │  Voice Engine   │ │  Brain Engine   │ │  Memory System  │
//!  │  ┌───────────┐  │ │  ┌───────────┐  │ │  ┌───────────┐  │
//!  │  │ Streaming │  │ │  │ Cognitive │  │ │  │ Long Term │  │
//!  │  │ STT       │  │ │  │ Orchestr. │  │ │  │ Memory V2 │  │
//!  │  └───────────┘  │ │  └───────────┘  │ │  └───────────┘  │
//!  │  ┌───────────┐  │ │  ┌───────────┐  │ │  ┌───────────┐  │
//!  │  │ Streaming │  │ │  │ Decision  │  │ │  │ Knowledge │  │
//!  │  │ TTS       │  │ │  │ Engine    │  │ │  │ Graph     │  │
//!  │  └───────────┘  │ │  └───────────┘  │ │  └───────────┘  │
//!  │  ┌───────────┐  │ │  ┌───────────┐  │ │  ┌───────────┐  │
//!  │  │ Audio V2  │  │ │  │ Goal      │  │ │  │ Hermes    │  │
//!  │  │ Pipeline  │  │ │  │ Engine V2 │  │ │  │ Engine    │  │
//!  │  └───────────┘  │ │  └───────────┘  │ │  └───────────┘  │
//!  └─────────────────┘ └─────────────────┘ └─────────────────┘
//!           │                    │                    │
//!  ┌────────▼────────┐ ┌────────▼────────┐ ┌────────▼────────┐
//!  │ Planner         │ │ Agent Runtime   │ │ Automation      │
//!  │ ┌─────────────┐ │ │ ┌─────────────┐ │ │ ┌─────────────┐ │
//!  │ │ Task Graph  │ │ │ │ Multi-Agent │ │ │ │ Windows UIA │ │
//!  │ │ V2          │ │ │ │ Orchestr.   │ │ │ │ Integration │ │
//!  │ └─────────────┘ │ │ └─────────────┘ │ │ └─────────────┘ │
//!  │ ┌─────────────┐ │ │ ┌─────────────┐ │ │ ┌─────────────┐ │
//!  │ │ Goal        │ │ │ │ Self-Improv.│ │ │ │ Proactive   │ │
//!  │ │ Decomposer  │ │ │ │ Engine      │ │ │ │ Engine      │ │
//!  │ └─────────────┘ │ │ └─────────────┘ │ │ └─────────────┘ │
//!  └─────────────────┘ └─────────────────┘ └─────────────────┘
//!           │                    │                    │
//!  ┌────────▼────────┐ ┌────────▼────────┐ ┌────────▼────────┐
//!  │ World Model     │ │ Guardian        │ │ Companion       │
//!  │ ┌─────────────┐ │ │ ┌─────────────┐ │ │ ┌─────────────┐ │
//!  │ │ Desktop     │ │ │ │ Security    │ │ │ │ Personality │ │
//!  │ │ Context     │ │ │ │ Policies    │ │ │ │ Dynamics    │ │
//!  │ └─────────────┘ │ │ └─────────────┘ │ │ └─────────────┘ │
//!  │ ┌─────────────┐ │ │ ┌─────────────┐ │ │ ┌─────────────┐ │
//!  │ │ Device      │ │ │ │ Runtime     │ │ │ │ Emotional   │ │
//!  │ │ Tracker     │ │ │ │ Guard       │ │ │ │ State       │ │
//!  │ └─────────────┘ │ │ └─────────────┘ │ │ └─────────────┘ │
//!  └─────────────────┘ └─────────────────┘ └─────────────────┘
//! ```
//!
//! # Runtime Flow (Unified Pipeline)
//!
//! ```text
//! User Voice/Text
//!       │
//!       ▼
//! ┌─────────────┐    Topics: voice.wake, voice.input
//! │  Input       │─────────────────────────────────────────┐
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: stt.partial, stt.final       │
//! │ Streaming   │                                          │
//! │ STT         │                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: conversation.turn             │
//! │ Conversation│                                          │
//! │ Intelligence│                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: decision.request/result      │
//! │  Decision   │                                          │
//! │  Engine     │                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: plan.create/step_done         │
//! │  Planner    │                                          │
//! │  + TaskGraph│                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: task.created/completed        │
//! │ Agent       │                                          │
//! │ Orchestrator│                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: automation.trigger/complete   │
//! │ Automation  │                                          │
//! └──────┬──────┘                                          │
//!        ▼                                                 │
//! ┌─────────────┐    Topics: memory.stored/retrieved       │
//! │ Memory      │◄─────────────────────────────────────────┘
//! │ Update      │     (all stages feed events back)
//! └──────┬──────┘
//!        ▼
//! ┌─────────────┐    Topics: tts.request, tts.audio
//! │ Streaming   │
//! │ TTS         │
//! └──────┬──────┘
//!        ▼
//!    Response
//! ```
//!
//! # Event Flow
//!
//! ```text
//! ┌──────────┐     publish      ┌─────────────┐     subscribe     ┌──────────┐
//! │  Voice   │──────────────►  │             │─────────────────► │ Planner  │
//! └──────────┘                  │             │                   └──────────┘
//! ┌──────────┐     publish      │   EventBus  │     subscribe     ┌──────────┐
//! │  STT     │──────────────►  │  (pub/sub)  │─────────────────► │ Decision │
//! └──────────┘                  │             │                   └──────────┘
//! ┌──────────┐     publish      │  + Dead     │     subscribe     ┌──────────┐
//! │  LLM     │──────────────►  │  Letter     │─────────────────► │ Memory   │
//! └──────────┘                  │  Queue      │                   └──────────┘
//! ┌──────────┐     publish      │             │     subscribe     ┌──────────┐
//! │  Agent   │──────────────►  │             │─────────────────► │ Goals    │
//! └──────────┘                  └─────────────┘                   └──────────┘
//!                                                ▲
//!                                         subscribe│
//!                                          ┌──────┴──────┐
//!                                          │ Dashboard   │
//!                                          │ (monitors   │
//!                                          │  all topics)│
//!                                          └─────────────┘
//! ```
//!
//! # Memory Flow
//!
//! ```text
//! Conversation ──┐
//!                │
//! Task Results ──┤
//!                │     ┌──────────────┐     ┌──────────────┐
//! Goal Progress──┼────►│   Hermes     │────►│  Long Term   │
//!                │     │   Engine     │     │  Memory V2   │
//! Corrections ───┤     │ (importance  │     │  - Project   │
//!                │     │  scoring)    │     │  - User Prefs│
//! User Feedback──┘     └──────────────┘     │  - Relations │
//!                                           │  - Episodic  │
//!                                           └──────┬───────┘
//!                                                  │
//!                              ┌────────────────────┤
//!                              ▼                    ▼
//!                     ┌──────────────┐     ┌──────────────┐
//!                     │  Knowledge   │     │  Forgetting   │
//!                     │  Graph       │     │  Engine       │
//!                     │  (relations) │     │  (compression │
//!                     └──────────────┘     │   archival)   │
//!                                          └──────────────┘
//! ```
//!
//! # Startup Flow
//!
//! ```text
//! BootSequence::begin()
//!       │
//!       ▼ Phase 0: Kernel
//! ┌─────────────┐  Initialize tokio runtime, event bus, service registry
//! │  Kernel     │
//! └──────┬──────┘
//!       ▼ Phase 1: Config
//! ┌─────────────┐  Load AppConfig, validate settings
//! │  Config     │
//! └──────┬──────┘
//!       ▼ Phase 2: Database
//! ┌─────────────┐  SQLite, connection pools
//! │  Database   │
//! └──────┬──────┘
//!       ▼ Phase 3: Security
//! ┌─────────────┐  Guardian policies, auth
//! │  Security   │
//! └──────┬──────┘
//!       ▼ Phase 4: Providers
//! ┌─────────────┐  LLM (Ollama), STT (Whisper), TTS (Kokoro)
//! │  Providers  │
//! └──────┬──────┘
//!       ▼ Phase 5: Memory
//! ┌─────────────┐  LongTermMemoryV2, KnowledgeGraph, Hermes
//! │  Memory     │
//! └──────┬──────┘
//!       ▼ Phase 6: Planner
//! ┌─────────────┐  TaskGraphV2, GoalDecomposer
//! │  Planner    │
//! └──────┬──────┘
//!       ▼ Phase 7: Agents
//! ┌─────────────┐  MultiAgentOrchestrator, SelfImprovement
//! │  Agents     │
//! └──────┬──────┘
//!       ▼ Phase 8: Automation
//! ┌─────────────┐  WorkflowEngine, DecisionEngine
//! │  Automation │
//! └──────┬──────┘
//!       ▼ Phase 9: Voice
//! ┌─────────────┐  VoicePipeline, StreamingSTT, Barge-in
//! │  Voice      │
//! └──────┬──────┘
//!       ▼ Phase 10: Dashboard
//! ┌─────────────┐  OwnerCommandCenter, CentralTelemetry
//! │  Dashboard  │
//! └──────┬──────┘
//!       ▼ Phase 11: Ready
//!    System Ready
//!
//! On failure at any phase:
//!   RecoveryManager attempts restart (exponential backoff)
//!   Circuit breaker prevents cascading failures
//!   Failed subsystems are isolated, system continues
//! ```
//!
//! # Shutdown Flow
//!
//! ```text
//! Shutdown Signal Received
//!       │
//!       ▼
//! Stop Voice Pipeline (flush audio buffers)
//!       │
//!       ▼
//! Stop Automation (finish current workflows)
//!       │
//!       ▼
//! Stop Agents (drain task queues, save state)
//!       │
//!       ▼
//! Stop Planner (save task graph state)
//!       │
//!       ▼
//! Flush Memory (persist to disk/SQLite)
//!       │
//!       ▼
//! Stop Providers (disconnect LLM, STT, TTS)
//!       │
//!       ▼
//! Stop Security (save audit log)
//!       │
//!       ▼
//! Stop Database (flush WAL, close connections)
//!       │
//!       ▼
//! Export Diagnostics (telemetry snapshot)
//!       │
//!       ▼
//! Kernel Shutdown Complete
//! ```
//!
//! # Agent Communication Flow
//!
//! ```text
//! ┌──────────┐  assign task   ┌──────────┐
//! │ Orchestr.│──────────────► │ Agent A  │
//! │          │◄────────────── │ (Coder)  │
//! └────┬─────┘  result        └──────────┘
//!      │
//!      │  route message       ┌──────────┐
//!      ├────────────────────► │ Agent B  │
//!      │◄──────────────────── │ (QA)     │
//!      │  status update       └──────────┘
//!      │
//!      │  coordinate          ┌──────────┐
//!      ├────────────────────► │ Agent C  │
//!      │◄──────────────────── │ (Review) │
//!      │  approval            └──────────┘
//!      │
//!      ▼
//! ┌──────────┐  publish result
//! │ EventBus │──────────────► Topics: agent.completed
//! └──────────┘                 Topics: task.completed
//!                              Topics: memory.stored
//!                              Topics: goal.progress
//! ```
