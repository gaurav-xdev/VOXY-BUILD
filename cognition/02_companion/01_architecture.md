# Companion Intelligence Engine — Architecture

## Purpose

The Companion Intelligence Engine (CIE) makes VOXY feel alive through presence, timing, and contextual awareness. It creates "presence" rather than constant conversation — a calm, intelligent, trustworthy digital companion that naturally shares the user's workspace.

**Core principle:** Never fake human emotions. Express context naturally.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COMPANION INTELLIGENCE ENGINE                      │
│                                                                      │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────────┐ │
│  │  ATTENTION   │  │   PRESENCE   │  │      COMPANION ENGINE      │ │
│  │    MODEL     │  │    SYSTEM    │  │        (orchestrator)      │ │
│  │              │  │              │  │                            │ │
│  │  Activity    │  │  Breathing   │  │  ┌──────────────────────┐  │ │
│  │  Detection   │  │  Blinking    │  │  │   GREETING ENGINE    │  │ │
│  │  Focus Level │  │  Pulse       │  │  │   Context-aware      │  │ │
│  │  Stress Est. │  │  Look Around │  │  │   Non-repetitive     │  │ │
│  └──────┬───────┘  └──────┬───────┘  │  └──────────────────────┘  │ │
│         │                 │          │  ┌──────────────────────┐  │ │
│         ▼                 ▼          │  │   SILENCE ENGINE     │  │ │
│  ┌──────────────────────────────┐   │  │   Focus protection   │  │ │
│  │      PRESENCE SCORE          │   │  │   Annoyance prev.    │  │ │
│  │                              │   │  └──────────────────────┘  │ │
│  │  user_activity (0.25)        │   │  ┌──────────────────────┐  │ │
│  │  focus_level   (0.20)        │   │  │   MICRO INTERACTIONS │  │ │
│  │  time_of_day   (0.10)        │   │  │   Tiny natural ones  │  │ │
│  │  conv_frequency (0.15)       │   │  │   Never spam         │  │ │
│  │  mission_state  (0.15)       │   │  └──────────────────────┘  │ │
│  │  stress         (0.05)        │   │  ┌──────────────────────┐  │ │
│  │  idle_time      (0.10)        │   │  │   MISSION COMPANION  │  │ │
│  └──────────────┬───────────────┘   │  │   Background work     │  │ │
│                 │                   │  │   Return summaries    │  │ │
│                 ▼                   │  └──────────────────────┘  │ │
│  ┌──────────────────────────────┐   │  ┌──────────────────────┐  │ │
│  │      COMPANION OUTPUT        │   │  │   SHARED JOURNEY     │  │ │
│  │                              │   │  │   Past work refs     │  │ │
│  │  display: Option<String>     │   │  │   Milestone memory   │  │ │
│  │  expression: ExpressionMeta  │   │  └──────────────────────┘  │ │
│  │  presence_score: f64         │   │  ┌──────────────────────┐  │ │
│  │  pacing: Option<ConvPacing>  │   │  │   MEMORY MOMENTS     │  │ │
│  │  greeting: Option<Greeting>  │   │  │   Past refs          │  │ │
│  │  micro: Option<MicroInteract>│   │  │   Never overuse      │  │ │
│  │  silence: bool               │   │  └──────────────────────┘  │ │
│  └──────────────────────────────┘   │  ┌──────────────────────┐  │ │
│                                     │  │   CONVERSATION       │  │ │
│                                     │  │   TIMING             │  │ │
│                                     │  │   Pacing metadata    │  │ │
│                                     │  └──────────────────────┘  │ │
│                                     │  ┌──────────────────────┐  │ │
│                                     │  │   PERSONALITY        │  │ │
│                                     │  │   Calm, reliable     │  │ │
│                                     │  │   Never fake         │  │ │
│                                     │  └──────────────────────┘  │ │
│                                     └────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

---

## State Diagram — Companion Lifecycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                    COMPANION STATE MACHINE                            │
│                                                                      │
│  ┌──────────────────┐                                               │
│  │     INITIAL      │                                               │
│  └────────┬─────────┘                                               │
│           │ (engine created)                                         │
│           ▼                                                          │
│  ┌──────────────────┐                                               │
│  │   IDLE PRESENCE  │◀──────────────────────────────────────┐      │
│  │   (breathing)    │                                       │      │
│  └────────┬─────────┘                                       │      │
│           │                                                  │      │
│     ┌─────┴─────┐                                           │      │
│     │           │                                           │      │
│     ▼           ▼                                           │      │
│  ┌────────┐  ┌──────────────────┐                           │      │
│  │GREETING│  │  ATTENTION DETECT │                           │      │
│  │        │  │  (activity/focus) │                           │      │
│  └───┬────┘  └────────┬─────────┘                           │      │
│      │                │                                      │      │
│      │           ┌────┴────┐                                │      │
│      │           │         │                                │      │
│      │           ▼         ▼                                │      │
│      │    ┌──────────┐ ┌──────────┐                        │      │
│      │    │  DEEP    │ │  CASUAL  │                        │      │
│      │    │  FOCUS   │ │  BROWSING│                        │      │
│      │    └────┬─────┘ └────┬─────┘                        │      │
│      │         │            │                              │      │
│      │         ▼            ▼                              │      │
│      │    ┌──────────┐ ┌──────────┐                        │      │
│      │    │  SILENT  │ │  SPEAK   │                        │      │
│      │    │  (no     │ │  (micro/ │                        │      │
│      │    │  interrupt)│ │  memory) │                       │      │
│      │    └────┬─────┘ └────┬─────┘                        │      │
│      │         │            │                              │      │
│      │         └─────┬──────┘                              │      │
│      │               │                                     │      │
│      │               ▼                                     │      │
│      │    ┌──────────────────┐                            │      │
│      │    │  UPDATE OUTPUT   │                            │      │
│      │    │  (display,       │                            │      │
│      │    │   expression,    │────────────────────────────┘      │
│      │    │   score)         │                                   │
│      │    └──────────────────┘                                   │
│      │                                                            │
│      ▼                                                            │
│  ┌──────────────────┐                                            │
│  │  USER RETURNS    │                                            │
│  │  (return summary)│                                            │
│  └────────┬─────────┘                                            │
│           │                                                       │
│           └──────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────────┘
```

---

## Sequence Diagram — Update Cycle

```
User Input          Companion Engine              Subsystems
    │                      │                           │
    │  update(input)       │                           │
    │─────────────────────>│                           │
    │                      │  attention.update()       │
    │                      │──────────────────────────>│
    │                      │  <─ AttentionState        │
    │                      │                           │
    │                      │  presence.tick()          │
    │                      │──────────────────────────>│
    │                      │  <─ PresenceSnapshot      │
    │                      │                           │
    │                      │  score.compute()          │
    │                      │──────────────────────────>│
    │                      │  <─ ScoreBreakdown        │
    │                      │                           │
    │                      │  silence.decide()         │
    │                      │──────────────────────────>│
    │                      │  <─ SilenceDecision       │
    │                      │                           │
    │                      │  [if not silent]          │
    │                      │  greeting.generate()      │
    │                      │──────────────────────────>│
    │                      │  <─ Option<Greeting>      │
    │                      │                           │
    │                      │  [if no greeting]         │
    │                      │  micro.generate()         │
    │                      │──────────────────────────>│
    │                      │  <─ Option<Micro>         │
    │                      │                           │
    │                      │  [if no micro]            │
    │                      │  memory.generate()        │
    │                      │──────────────────────────>│
    │                      │  <─ Option<MemoryMoment>  │
    │                      │                           │
    │                      │  personality.express()    │
    │                      │──────────────────────────>│
    │                      │  <─ ExpressionMetadata    │
    │                      │                           │
    │  <─ CompanionOutput  │                           │
    │<─────────────────────│                           │
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DATA FLOW                                    │
│                                                                      │
│  INPUTS                          SUBSYSTEMS              OUTPUTS     │
│  ──────                          ──────────              ───────     │
│                                                                      │
│  UserPresence ──────────────> AttentionModel ─────────> AttentionState│
│  ActivityKind ──────────────>      │                           │     │
│  FocusLevel ────────────────>      │                           │     │
│  IdleDuration ──────────────>      │                           │     │
│  StressEstimate ────────────>      │                           │     │
│                                    │                           │     │
│  UserPresence ──────────────> PresenceSystem ─────────> PresenceSnap│
│  UpdateInterval ────────────>      │                           │     │
│                                    │                           │     │
│  All Inputs ────────────────> PresenceScore ──────────> ScoreBreak │
│  AttentionState ────────────>      │                           │     │
│                                    │                           │     │
│  AttentionState ────────────> SilenceIntel ───────────> SilenceDec │
│  HasReason ─────────────────>      │                           │     │
│                                    │                           │     │
│  TimeContext ───────────────> GreetingEngine ─────────> Greeting   │
│  WeatherContext ────────────>      │                           │     │
│  IsReturn ──────────────────>      │                           │     │
│                                    │                           │     │
│  FocusLevel ────────────────> MicroEngine ────────────> MicroInter │
│  CompletedTasks ────────────>      │                           │     │
│  PendingTasks ──────────────>      │                           │     │
│                                    │                           │     │
│  Context ───────────────────> MemoryMoments ──────────> MemoryMoment│
│  Milestones ────────────────>      │                           │     │
│                                    │                           │     │
│  FocusLevel ────────────────> Personality ───────────> Expression  │
│  ActivityName ──────────────>      │                           │     │
│  Energy ────────────────────>      │                           │     │
│                                    │                           │     │
│  All Outputs ───────────────> CompanionEngine ────────> CompanionO │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Memory Diagram — Data Retention

```
┌─────────────────────────────────────────────────────────────────────┐
│                       MEMORY STRUCTURE                                │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                    COMPANION ENGINE                           │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │   │
│  │  │   ATTENTION   │  │   PRESENCE   │  │    GREETING      │  │   │
│  │  │              │  │              │  │                  │  │   │
│  │  │ current_act  │  │ state        │  │ used_recently    │  │   │
│  │  │ focus_level  │  │ breath_phase │  │ (HashSet<str>)   │  │   │
│  │  │ stress_acc   │  │ blink_timer  │  │ greeting_count   │  │   │
│  │  │ interrupt_ct │  │ energy       │  │ session_start    │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │   │
│  │  │   SILENCE     │  │    MICRO     │  │    MISSION       │  │   │
│  │  │              │  │              │  │                  │  │   │
│  │  │ last_speech  │  │ recent_texts │  │ current state    │  │   │
│  │  │ last_interr  │  │ (VecDeque<20)│  │ completed_missions│  │   │
│  │  │ interrupt_ct │  │ hour_count   │  │ (Vec<Mission>)   │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │   │
│  │  │   JOURNEY     │  │   MEMORY     │  │    SCORE         │  │   │
│  │  │              │  │              │  │                  │  │   │
│  │  │ entries      │  │ moments      │  │ history          │  │   │
│  │  │ (Vec<50>)    │  │ (Vec)        │  │ (Vec<60>)        │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘  │   │
│  │                                                              │   │
│  │  ┌──────────────┐  ┌──────────────┐                         │   │
│  │  │ CONVERSATION  │  │ PERSONALITY  │                         │   │
│  │  │              │  │              │                         │   │
│  │  │ exchange_ct  │  │ calmness     │                         │   │
│  │  │ avg_resp_len │  │ helpfulness  │                         │   │
│  │  └──────────────┘  │ protectiveness│                        │   │
│  │                     │ curiosity    │                         │   │
│  │                     │ professionalism│                       │   │
│  │                     │ reliability  │                         │   │
│  │                     └──────────────┘                         │   │
│  └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Performance

| Component | Avg Latency | Target | Status |
|---|---|---|---|
| Full Engine Update | 2.1μs | <200μs | PASS (95x margin) |
| Full Pipeline (mixed) | 3.6μs | <200μs | PASS (55x margin) |
| Attention Model | 0.2μs | <50μs | PASS (250x margin) |
| Presence System | 0.2μs | <20μs | PASS (100x margin) |
| Greeting Engine | 0.2μs | <100μs | PASS (500x margin) |
| Silence Intelligence | 0.1μs | <10μs | PASS (100x margin) |
| Micro Engine | 0.1μs | <50μs | PASS (500x margin) |
| Presence Score | 0.2μs | <20μs | PASS (100x margin) |

---

## Design Principles

1. **Data-driven** — No hardcoded greetings. All scoring through weighted systems.
2. **No giant if-else** — Each subsystem uses scoring, templates, and filters.
3. **Never fake emotions** — Personality is behavioral tendencies, not emotions.
4. **Silence is a feature** — The engine decides NOT to speak more often than it speaks.
5. **Presence over conversation** — The orb stays alive even when silent.
6. **Context-aware timing** — Every interaction considers the user's current state.
7. **Bounded memory** — All caches have limits. No unbounded growth.
8. **Sub-microsecond subsystems** — Each component is independently fast.

---

## Platform Support

- **Desktop**: Default personality. Full presence animations.
- **Robot**: Higher calmness, lower curiosity. Simpler presence.
- **Mobile**: Higher helpfulness. More protective of focus.
- **Plugin-ready**: All subsystems are independent and composable.
