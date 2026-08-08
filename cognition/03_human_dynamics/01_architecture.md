# Human Dynamics Runtime (HDR)

## Overview

The Human Dynamics Runtime is VOXY's behavioral operating system. It determines **HOW** VOXY behaves — not what it thinks or what it remembers, but how it expresses those thoughts in human terms.

The HDR is the last layer before output. Every decision, every word, every action passes through the HDR's 10 subsystems to determine if it's appropriate, how it should be delivered, and whether it should happen at all.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    HumanDynamicsEngine                          │
│                    (orchestrator)                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Relationship │  │    Trust     │  │     Behavior         │  │
│  │   Engine      │  │   Engine     │  │     Engine           │  │
│  │              │  │              │  │                      │  │
│  │  score()     │  │  score()     │  │  current()           │  │
│  │  level()     │  │  breakdown() │  │  suggest_state()     │  │
│  │  process()   │  │  process()   │  │  transition()        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  Protection  │  │  Initiative  │  │    Confidence        │  │
│  │   Engine     │  │   Engine     │  │    Engine            │  │
│  │              │  │              │  │                      │  │
│  │  evaluate()  │  │  decide()    │  │  calculate()         │  │
│  │              │  │              │  │  trend()             │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │    Style     │  │    Policy    │  │    Recovery          │  │
│  │   Engine     │  │   Engine     │  │    Engine            │  │
│  │              │  │              │  │                      │  │
│  │  adapt()     │  │  check()     │  │  recover()           │  │
│  │              │  │              │  │                      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Humor Engine                            │   │
│  │                                                          │   │
│  │  decide()                                                │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## The 10 Subsystems

### 1. Relationship Engine
- **Tracks**: How the relationship evolves over time
- **Score**: 0.0 (stranger) → 1.0 (long-term companion)
- **Level**: Professional → Familiar → Trusted → LongTermCompanion
- **Input**: Trust events, interaction history
- **Output**: Relationship score, level, consistency

### 2. Trust Engine
- **Tracks**: User trust in VOXY based on outcomes
- **Score**: 0.0 (untrusted) → 1.0 (fully trusted)
- **Events**: SuccessfulMission (+0.05), TaskFailed (-0.10), Correction (-0.08), FalseAlarm (-0.12), PermissionGranted (+0.03)
- **Derives**: Autonomy level, confirmation requirement, initiative permission

### 3. Behavior Engine
- **State Machine**: 12 states (Listening, Thinking, Working, Observing, Protecting, Teaching, Celebrating, Waiting, DeepFocus, MissionMode, Sleeping)
- **Transitions**: Validated against a strict transition graph
- **Cooldown**: 2s minimum between transitions
- **Suggestion**: Context-aware state recommendations

### 4. Protection Engine
- **Evaluates**: Whether an action should be allowed
- **Levels**: None, Low, Medium, High, Critical
- **Blocks**: Meeting interruptions, deep focus violations, destructive actions
- **Auto-protects**: Delete operations (moves to trash instead of permanent delete)

### 5. Initiative Engine
- **Decides**: When VOXY may speak first (unsolicited)
- **Requirements**: Sufficient trust (≥0.6), relationship level (≥Familiar), valid reason
- **Limits**: Max 4 per hour, 2min cooldown
- **Respects**: Deep focus, sleeping states

### 6. Confidence Engine
- **Calculates**: Internal confidence for responses
- **Levels**: VeryLow, Low, Medium, High, VeryHigh
- **Explains**: Automatically explains low-confidence responses
- **Tracks**: Historical trend, average confidence

### 7. Style Engine
- **Adapts**: Communication style without changing personality
- **Parameters**: Formality, pace, verbosity, initiative, sentence length
- **Contexts**: Professional (formal, terse), Familiar (balanced), Companion (relaxed)
- **Responds**: Focus level, time pressure, topic complexity

### 8. Policy Engine
- **Enforces**: Global behavioral rules
- **Rules**: Never interrupt meetings, never joke on failure, never celebrate partial success, protect before obey, always explain refusals
- **Records**: Violations with timestamps

### 9. Recovery Engine
- **Handles**: Mistakes gracefully
- **Protocol**: Acknowledge → Correct → Learn
- **Limits**: Max 3 recovery attempts, 5s cooldown
- **Tracks**: Recovery history

### 10. Humor Engine
- **Decides**: When subtle humor is appropriate
- **Requirements**: Relationship ≥0.4, confidence ≥0.7, context appropriate
- **Limits**: Max 2 per hour, 5min cooldown
- **Context**: Relationship score, appropriateness, timing

## Data Flow

```
HdrInput
  │
  ├─→ Trust Engine (process events)
  ├─→ Relationship Engine (process events)
  │
  ├─→ Trust Score ──────────────────────────────────┐
  ├─→ Relationship Score ───────────────────────────┤
  │                                                  │
  ├─→ Protection Engine ──→ ProtectionDecision       │
  ├─→ Initiative Engine ──→ InitiativeDecision       │
  ├─→ Confidence Engine ──→ ConfidenceOutput         │
  ├─→ Humor Engine ──────→ HumorDecision             │
  ├─→ Policy Engine ─────→ PolicyViolations          │
  ├─→ Recovery Engine ───→ Option<RecoveryAction>    │
  ├─→ Style Engine ──────→ InteractionStyle          │
  │                                                  │
  └─→ HdrOutput                                      │
      ├── behavior_state                              │
      ├── relationship_level                          │
      ├── trust_score                                 │
      ├── autonomy_level                              │
      ├── confirmation_level                          │
      ├── initiative_level                            │
      ├── protection_decision                         │
      ├── initiative_decision                         │
      ├── confidence                                  │
      ├── humor_decision                              │
      ├── style                                       │
      ├── recovery                                    │
      ├── policy_violations                           │
      └── update_latency_us                           │
```

## Performance

| Metric | Target | Achieved |
|--------|--------|----------|
| Full pipeline (with action) | <100μs | ~2-5μs |
| Full pipeline (no action) | <100μs | ~1-3μs |
| Pipeline with 100 events | <200μs | ~5-10μs |
| Update latency (reported) | <100μs | <10μs |

## Test Coverage

| Module | Unit Tests | Integration Tests |
|--------|-----------|-------------------|
| behavior | 5 | - |
| confidence | 4 | - |
| engine | 6 | 9 |
| humor | 4 | - |
| initiative | 5 | - |
| policy | 3 | - |
| protection | 4 | - |
| recovery | 4 | - |
| relationship | 4 | - |
| style | 5 | - |
| trust | 5 | - |
| **Total** | **51** | **9** |

Plus 3 benchmarks validating performance targets.
