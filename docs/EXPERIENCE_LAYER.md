# VOXY Experience Layer — Phase 2 Design Document

## 1. Architecture Overview

The Experience Layer is **not a rewrite**. It is a thin orchestration layer that connects the 30+ existing subsystems into a coherent lived experience. Most crates already have traits, configs, and in-memory implementations. The work is **wiring, not building**.

```
┌─────────────────────────────────────────────────────────────────────┐
│                        EXPERIENCE LAYER                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │ Presence │ │Conversat.│ │Personali.│ │  Memory  │ │Proactive │ │
│  │  System  │ │  Engine  │ │  Engine  │ │  Synapse │ │  Beacon  │ │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ │
│       │             │             │             │             │       │
│  ┌────┴─────────────┴─────────────┴─────────────┴─────────────┴───┐ │
│  │                    Companion Engine (voxy-companion)           │ │
│  └────┬─────────────┬─────────────┬─────────────┬─────────────┬──┘ │
│       │             │             │             │             │      │
├───────┼─────────────┼─────────────┼─────────────┼─────────────┼──────┤
│  RUNTIME LAYER (existing, frozen)                                  │
│  ┌────┴────┐ ┌──────┴────┐ ┌─────┴─────┐ ┌────┴─────┐ ┌─────┴──┐  │
│  │  Brain  │ │Cognition  │ │  Memory   │ │  World   │ │  Hdr   │  │
│  │ Engine  │ │  Engine   │ │  System   │ │  Model   │ │ Engine │  │
│  └────┬────┘ └──────┬────┘ └─────┬─────┘ └────┬─────┘ └─────┬──┘  │
│       │             │             │             │             │      │
│  ┌────┴────┐ ┌──────┴────┐ ┌─────┴─────┐ ┌────┴─────┐ ┌─────┴──┐  │
│  │ Voice   │ │Orchestrat.│ │Automation │ │  Skills  │ │  Bus   │  │
│  │Pipeline │ │           │ │           │ │          │ │        │  │
│  └─────────┘ └───────────┘ └───────────┘ └──────────┘ └────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Additive only** — no crate modifications unless bug fixes
2. **Event-driven** — all communication through `voxy-event-bus`
3. **Trait-based** — every subsystem behind a trait, in-memory impl first
4. **Config-driven** — all behavior configurable via `voxy-config`
5. **Observable** — every action emits events for debugging

---

## 2. Subsystem List

### 2.1 Presence System (`crates/companion/src/purpose/presence.rs`)

**What it does:** Makes VOXY feel alive when idle. The particle system that users see.

**Already exists:** `PresenceSystem`, `PresenceState`, `PresenceSnapshot` in `voxy-companion`

**What to implement:**
- `PresenceRenderer` — procedural animation state machine
- States: `Idle`, `Listening`, `Thinking`, `Speaking`, `Celebrating`, `Concerned`
- Transition rules between states
- Emotional modulation of animation parameters

**Animation Parameters (all procedural):**
```
breathing_rate: f32        // 0.2-0.8 Hz, varies with mood
particle_density: f32      // 0.0-1.0, increases when active
particle_energy: f32       // speed of particle motion
color_temperature: f32     // warm (0.0) to cool (1.0)
glow_intensity: f32        // 0.0-1.0, pulses with speech
form_coherence: f32        // how organized the particles are
```

**Events consumed:**
- `voice::vad_started` → transition to `Listening`
- `voice::vad_ended` → transition to `Thinking`
- `cognition::result_ready` → transition to `Speaking`
- `companion::celebration` → transition to `Celebrating`

**Events emitted:**
- `presence::state_changed`
- `presence::animation_frame` (for renderer)

**Estimate:** 3-4 days

---

### 2.2 Conversation Engine (`crates/companion/src/purpose/conversation.rs`)

**What it does:** Natural turn-taking, interruptions, context carry-over.

**Already exists:** `ConversationTiming`, `BargeInManager`, `TurnManager`, `ContextTracker` in `voxy-companion` and `voxy-conversation`

**What to implement:**
- `NaturalTurnManager` — wraps `TurnManager` with human-like timing
- `InterruptionPolicy` — when to stop, when to continue
- `ContextCarryOver` — maintains conversation thread across turns

**Turn-Taking Rules:**
```
after_user_speaks:
  if confidence > 0.8:
    respond_after(100ms)  # fast, confident
  elif confidence > 0.5:
    respond_after(300ms)  # brief pause, thinking
  else:
    respond_after(800ms)  # longer pause, uncertain

during_voxy_speaking:
  if user_interrupts:
    if interruption_confidence > 0.7:
      stop_after(200ms)   # quick yield
    else:
      continue(500ms)     # finish thought
```

**Context Carry-Over:**
- Last 5 turns always in context
- Topic thread tracking
- Referent resolution ("it", "that", "the one from before")

**Events consumed:**
- `voice::transcription_complete`
- `cognition::result_ready`
- `conversation::user_speaking`

**Events emitted:**
- `conversation::turn_started`
- `conversation::turn_ended`
- `conversation::context_updated`

**Estimate:** 4-5 days

---

### 2.3 Voice Personality (`crates/personality/src/purpose/voice.rs`)

**What it does:** Makes VOXY's voice sound human, not robotic.

**Already exists:** `PersonalityConfig`, `MoodState`, `CommunicationStyle` in `voxy-personality`

**What to implement:**
- `VoiceProsodyEngine` — controls TTS parameters based on emotion
- `ThinkingSoundGenerator` — "hmm", "let me see", natural pauses
- `BreathingModel` — subtle breath sounds between sentences

**Prosody Rules:**
```rust
fn prosody_for_mood(mood: &MoodState) -> ProsodyParams {
    match mood {
        MoodState::Excited => ProsodyParams {
            speed: 1.15,
            pitch_offset: 0.05,
            pause_factor: 0.7,   // shorter pauses
            energy: 0.9,
        },
        MoodState::Calm => ProsodyParams {
            speed: 0.95,
            pitch_offset: -0.02,
            pause_factor: 1.3,   // longer pauses
            energy: 0.6,
        },
        MoodState::Concerned => ProsodyParams {
            speed: 0.9,
            pitch_offset: -0.03,
            pause_factor: 1.5,
            energy: 0.5,
        },
        _ => ProsodyParams::default(),
    }
}
```

**Thinking Sounds (injected before response):**
- Short answer (< 100 tokens): no thinking sound
- Medium answer (100-500 tokens): brief "hmm" or breath
- Long answer (> 500 tokens): "let me think about that"

**Events consumed:**
- `personality::mood_changed`
- `cognition::result_ready`
- `conversation::turn_started`

**Events emitted:**
- `personality::prosody_update`
- `personality::thinking_sound`

**Estimate:** 3-4 days

---

### 2.4 Memory Synapse (`crates/companion/src/purpose/memory.rs`)

**What it does:** Makes VOXY remember like a person — not a database.

**Already exists:** `MemoryMoments` in `voxy-companion`, full `MemoryApi` in `voxy-memory`

**What to implement:**
- `MemorySynapse` — decides what to remember and when to recall
- `PreferenceTracker` — learns user preferences over time
- `ConversationRecall` — surfaces relevant past conversations

**Memory Decision Rules:**
```rust
fn should_remember(event: &Event) -> MemoryDecision {
    if event.is_user_preference() {
        MemoryDecision::Remember(Importance::High, TTL::Forever)
    } else if event.is_project_related() {
        MemoryDecision::Remember(Importance::Medium, TTL::Weeks(4))
    } else if event.is_casual() {
        MemoryDecision::Remember(Importance::Low, TTL::Days(7))
    } else {
        MemoryDecision::Forget
    }
}
```

**Recall Triggers:**
- User mentions a project → recall related context
- User returns after absence → recall last conversation
- User asks about preferences → recall learned preferences
- Time-based: morning → recall daily routine

**Events consumed:**
- `conversation::turn_ended`
- `world_model::application_changed`
- `memory::item_stored`

**Events emitted:**
- `companion::memory_relevant`
- `companion::preference_detected`

**Estimate:** 4-5 days

---

### 2.5 Desktop Presence (`crates/world_model/src/purpose/awareness.rs`)

**What it does:** VOXY understands what the user is doing without being told.

**Already exists:** `DesktopState`, `WindowInfo`, `ApplicationInfo` in `voxy-world-model`

**What to implement:**
- `DesktopWatcher` — monitors active window, clipboard, notifications
- `ActivityClassifier` — classifies user activity (coding, browsing, meeting, etc.)
- `ContextEmitter` — publishes context changes to event bus

**Monitored Signals:**
```
active_window:     WindowInfo     // title, process, bounds
clipboard:         String         // last copied text
battery:           f32            // 0.0-1.0
cpu_usage:         f32            // 0.0-1.0
gpu_usage:         Option<f32>    // if available
active_downloads:  Vec<Download>  // browser downloads
media_playing:     Option<Media>  // current media
calendar_events:   Vec<Event>     // upcoming events
notification_count: u32           // unread notifications
```

**Activity Classification:**
```rust
fn classify_activity(desktop: &DesktopState) -> Activity {
    match desktop.active_window.process.as_str() {
        "Code.exe" | "devenv.exe" => Activity::Coding,
        "chrome.exe" | "firefox.exe" => Activity::Browsing,
        "spotify.exe" | "vlc.exe" => Activity::Listening,
        "Teams.exe" | "zoom.exe" => Activity::Meeting,
        _ => Activity::Other,
    }
}
```

**Events consumed:**
- `world_model::window_changed`
- `world_model::clipboard_changed`
- `system::battery_changed`
- `system::download_completed`

**Events emitted:**
- `presence::activity_changed`
- `presence::context_updated`
- `proactive::trigger_detected`

**Estimate:** 3-4 days

---

### 2.6 Proactive Beacon (`crates/companion/src/purpose/proactive.rs`)

**What it does:** VOXY notices things and speaks up at the right moment.

**Already exists:** `CompanionEngine`, `AttentionModel` in `voxy-companion`

**What to implement:**
- `ProactiveBeacon` — decides when to speak up
- `InterruptionGuard` — prevents annoying the user
- `MomentGenerator` — creates companion moments

**Proactive Rules:**
```rust
fn should_speak_proactively(event: &Event, context: &Context) -> bool {
    // Never interrupt during meetings
    if context.activity == Activity::Meeting {
        return false;
    }
    
    // Never interrupt during deep focus
    if context.focus_duration > Duration::from_minutes(20) {
        return false;
    }
    
    // Battery warning
    if event.is_battery_low() && context.battery > 0.15 {
        return true  // only warn once
    }
    
    // Meeting reminder
    if event.is_meeting_soon() && context.minutes_until_meeting == 10 {
        return true
    }
    
    // Download completed
    if event.is_download_completed() {
        return true
    }
    
    false
}
```

**Cooldown Rules:**
- Maximum 1 proactive comment per 15 minutes
- Never during active conversation
- Never during first 5 minutes after wake
- User can say "not now" to suppress for 30 minutes

**Events consumed:**
- `presence::activity_changed`
- `system::battery_changed`
- `system::meeting_soon`
- `system::download_completed`
- `cognition::code_compiled`

**Events emitted:**
- `companion::proactive_moment`
- `companion::suggestion`

**Estimate:** 3-4 days

---

### 2.7 Companion Moments (`crates/companion/src/purpose/moments.rs`)

**What it does:** Time-aware greetings, celebrations, acknowledgments.

**Already exists:** `GreetingEngine` in `voxy-companion`

**What to implement:**
- `MomentEngine` — generates contextual moments
- `TimeAwareness` — understands time of day, day of week
- `AchievementTracker` — recognizes milestones

**Moment Types:**
```rust
enum CompanionMoment {
    // Time-based
    GoodMorning { hours_since_last: Duration },
    GoodNight { is_late_night: bool },
    WelcomeBack { absence_duration: Duration },
    
    // Achievement-based
    ProjectCompleted { project_name: String },
    CodeCompiled { was_struggle: bool },
    TaskFinished { was_difficult: bool },
    
    // Calendar-based
    MeetingIn10Minutes { meeting_name: String },
    Birthday { whose: String },
    ExamTomorrow { subject: String },
    
    // System-based
    BatteryLow { percent: f32 },
    DownloadComplete { filename: String },
    UpdateAvailable { app_name: String },
}
```

**Tone Rules:**
- Morning: warm, brief, not too enthusiastic
- Welcome back: acknowledge absence, no guilt
- Celebrations: genuine, not over-the-top
- Warnings: calm, informational, not alarming

**Events consumed:**
- `system::time_changed`
- `system::battery_changed`
- `world_model::session_started`
- `cognition::task_completed`
- `memory::project_completed`

**Events emitted:**
- `companion::moment`
- `companion::greeting`

**Estimate:** 2-3 days

---

### 2.8 Visual Personality (`crates/companion/src/purpose/visual.rs`)

**What it does:** The particle system that represents VOXY's presence.

**Already exists:** `PresenceSystem` in `voxy-companion`

**What to implement:**
- `ParticleSystem` — procedural particle rendering
- `EmotionMapper` — maps emotions to visual parameters
- `BreathingAnimator` — organic breathing motion

**Particle System Design:**
```
Core: 200-500 particles in a loose cluster
Motion: Perlin noise-based drift
Color: Single hue, varying saturation/brightness
Form: Organic, never geometric
Size: 2-8px, varies with energy
```

**Animation States:**
```
Idle:
  particles: 200
  drift_speed: 0.3
  breathing_rate: 0.25 Hz
  color: neutral (0.5, 0.5, 0.5)

Listening:
  particles: 350 (+75%)
  drift_speed: 0.5
  breathing_rate: 0.4 Hz
  color: warm (0.6, 0.5, 0.4)
  reaction: particles lean toward user

Thinking:
  particles: 400 (+100%)
  drift_speed: 0.8 (faster)
  breathing_rate: 0.6 Hz
  color: cool (0.4, 0.5, 0.7)
  reaction: particles swirl inward

Speaking:
  particles: 300 (+50%)
  drift_speed: 0.4
  breathing_rate: 0.35 Hz
  color: warm (0.55, 0.5, 0.45)
  reaction: particles pulse with speech

Celebrating:
  particles: 500 (+150%)
  drift_speed: 1.2 (fast)
  breathing_rate: 0.8 Hz
  color: bright (0.7, 0.6, 0.5)
  reaction: particles expand outward
```

**Events consumed:**
- `presence::state_changed`
- `personality::mood_changed`
- `voice::audio_level`

**Events emitted:**
- `visual::frame_data` (for renderer)

**Estimate:** 5-7 days

---

### 2.9 Developer Experience (`crates/skills/src/purpose/extensibility.rs`)

**What it does:** How developers add new capabilities without touching the runtime.

**Already exists:** `SkillId`, `CapabilityId` in `voxy-skills`

**What to implement:**
- `SkillManifest` — declarative skill definition
- `SkillLoader` — loads skills from TOML/JSON
- `PersonalityPlugin` — custom personality profiles
- `AnimationPlugin` — custom animation states

**Skill Manifest (TOML):**
```toml
[skill]
name = "weather"
version = "1.0.0"
description = "Get weather information"

[skill.triggers]
keywords = ["weather", "forecast", "temperature"]
patterns = ["what's the weather", "will it rain"]

[skill.capabilities]
provides = ["weather.query"]
requires = ["internet.access"]

[skill.personality]
thinking_sound = "Let me check the weather for you..."
success_response = "Here's what I found:"
error_response = "I couldn't get the weather right now."
```

**Personality Plugin:**
```toml
[personality]
id = "voxy-studio"
name = "Studio"
communication_style = "Professional"
mood_default = "Calm"

[personality.traits]
warmth = 0.6
humor = 0.3
confidence = 0.8
curiosity = 0.5

[personality.voice]
thinking_sounds = ["hmm", "let me see"]
greeting_style = "brief"
celebration_style = "understated"
```

**Estimate:** 3-4 days

---

## 3. Priority Order

| Priority | Subsystem | Depends On | Impact | Effort |
|----------|-----------|------------|--------|--------|
| **P0** | Desktop Presence | world_model | Foundation | 3-4 days |
| **P0** | Memory Synapse | memory, companion | Foundation | 4-5 days |
| **P1** | Conversation Engine | conversation, voice | Core UX | 4-5 days |
| **P1** | Voice Personality | personality, kokoro | Core UX | 3-4 days |
| **P2** | Proactive Beacon | desktop, companion | Delight | 3-4 days |
| **P2** | Companion Moments | time, memory | Delight | 2-3 days |
| **P3** | Presence System | companion, voice | Polish | 3-4 days |
| **P3** | Visual Personality | presence | Polish | 5-7 days |
| **P4** | Developer Experience | skills | Extensibility | 3-4 days |

**Total: 28-40 days**

---

## 4. Implementation Roadmap

### Sprint 1: Foundation (Week 1-2)
- [ ] Desktop Presence — `DesktopWatcher`, `ActivityClassifier`
- [ ] Memory Synapse — `MemorySynapse`, `PreferenceTracker`
- [ ] Event wiring — connect world_model → event_bus → companion

### Sprint 2: Core UX (Week 3-4)
- [ ] Conversation Engine — `NaturalTurnManager`, `InterruptionPolicy`
- [ ] Voice Personality — `VoiceProsodyEngine`, `ThinkingSoundGenerator`
- [ ] TTS parameter control — speed, pitch, pauses

### Sprint 3: Delight (Week 5-6)
- [ ] Proactive Beacon — `ProactiveBeacon`, `InterruptionGuard`
- [ ] Companion Moments — `MomentEngine`, `TimeAwareness`
- [ ] Time-based behaviors — morning, night, welcome back

### Sprint 4: Polish (Week 7-8)
- [ ] Presence System — `PresenceRenderer`, state machine
- [ ] Visual Personality — `ParticleSystem`, emotion mapping
- [ ] Breathing animation, idle animation

### Sprint 5: Extensibility (Week 9-10)
- [ ] Developer Experience — `SkillManifest`, `SkillLoader`
- [ ] Personality Plugin system
- [ ] Documentation and examples

---

## 5. Estimates

| Subsystem | Days | Parallelizable |
|-----------|------|----------------|
| Desktop Presence | 3-4 | Yes (independent) |
| Memory Synapse | 4-5 | Yes (independent) |
| Conversation Engine | 4-5 | After voice personality |
| Voice Personality | 3-4 | Yes (independent) |
| Proactive Beacon | 3-4 | After desktop presence |
| Companion Moments | 2-3 | After memory synapse |
| Presence System | 3-4 | After conversation engine |
| Visual Personality | 5-7 | After presence system |
| Developer Experience | 3-4 | Yes (independent) |
| **Total** | **28-40** | **~6 weeks parallel** |

---

## 6. User Journey

### 6.1 Opening VOXY (0:00)

```
User double-clicks VOXY icon
  ↓
Kernel starts all services (voxy-kernel)
  ↓
World Model begins monitoring (voxy-world-model)
  ↓
Desktop Watcher activates
  ↓
Memory Synapse loads recent context
  ↓
Presence System transitions to Idle
  ↓
Particle system appears — breathing at 0.25 Hz
  ↓
VOXY notices user is at computer after 30 seconds
  ↓
Companion Moment: "Good morning" (if morning)
  or "Welcome back" (if returning)
  ↓
Voice: warm, brief, not too enthusiastic
  ↓
Particle system briefly transitions to Speaking
  ↓
Returns to Idle
```

### 6.2 Working Session (0:05 - 2:00)

```
User opens VS Code
  ↓
Desktop Watcher detects: Activity::Coding
  ↓
Memory Synapse notes: "User started coding at 09:05"
  ↓
Particle system remains Idle (don't interrupt)
  ↓
After 20 minutes of coding:
  ↓
Proactive Beacon: "You've been at this for a while. Need a break?"
  ↓
User: "No, I'm in the zone"
  ↓
VOXY: "Got it. I'll be quiet."
  ↓
Suppression timer: 30 minutes
  ↓
If user says nothing for 30 minutes:
  ↓
Proactive Beacon: (respects cooldown)
```

### 6.3 Conversation (1:30)

```
User: "Hey VOXY, what's the weather?"
  ↓
Voice Pipeline: VAD → STT → Cognition
  ↓
Conversation Engine: turn_started
  ↓
Presence System: Listening
  ↓
Cognition processes intent
  ↓
Personality Engine: mood=Calm, style=Casual
  ↓
Voice Prosody: speed=0.95, pause_factor=1.3
  ↓
LLM generates response
  ↓
Thinking Sound: brief breath
  ↓
TTS with prosody parameters
  ↓
Presence System: Speaking
  ↓
Particle system: particles pulse with speech
  ↓
Response complete
  ↓
Presence System: Idle
  ↓
Memory Synapse: stores "asked about weather"
```

### 6.4 Interruption (1:35)

```
VOXY is speaking
  ↓
User: "Actually, wait—"
  ↓
VAD detects speech during VOXY output
  ↓
Barge-In Manager: interruption_confidence=0.85
  ↓
Conversation Engine: stop_after(200ms)
  ↓
Voice Pipeline: stops TTS
  ↓
Presence System: Listening
  ↓
User: "Can you also check if my meeting is in 10 minutes?"
  ↓
VOXY processes new request
  ↓
Context carry-over: remembers previous weather query
```

### 6.5 Proactive Moment (2:00)

```
Calendar notification: "Team Standup in 10 minutes"
  ↓
Desktop Watcher detects notification
  ↓
Proactive Beacon evaluates:
  - Not in meeting ✓
  - Not in deep focus (25 min since last code change) ✓
  - Cooldown expired ✓
  ↓
Companion Moment: "Your standup is in 10 minutes"
  ↓
Voice: calm, informational
  ↓
Presence System: Speaking → Idle
  ↓
User: "Thanks, I'll wrap up"
  ↓
VOXY: "I'll remind you in 5."
  ↓
Scheduled reminder: 5 minutes later
```

### 6.6 End of Day (6:00)

```
User closes all windows
  ↓
Desktop Watcher: Activity::Idle
  ↓
Memory Synapse: consolidates day's conversations
  ↓
Time Awareness: 18:00, Friday
  ↓
Companion Moment: "Have a good evening. See you Monday."
  ↓
Presence System: Speaking → Idle
  ↓
After 5 minutes of no activity:
  ↓
Presence System: Idle → Sleep
  ↓
Particle system: breathing slows, dims
  ↓
Memory: saves session state
  ↓
System: ready for next session
```

---

## 7. What NOT to Change

- **Runtime layer** — frozen, all subsystems behind traits
- **Event bus** — already sufficient, no modifications needed
- **Memory system** — already has all stores, graph, Hermes engine
- **Cognition** — already has intent, planning, reasoning, reflection
- **Voice pipeline** — already has STT, TTS, VAD, wake word
- **Automation** — already has UIA, OpenClaw, hybrid backends
- **Kernel** — already has service registry, lifecycle management

## 8. What IS New

- **Wiring** — connecting existing crates through event bus
- **Configuration** — TOML files for personality, behavior, triggers
- **Rules engine** — decision logic for proactive behavior
- **State machines** — for presence, conversation, mood transitions
- **Timing logic** — cooldowns, delays, natural pauses

## 9. Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Time to first response | ~5s | <3s |
| Interruption detection | None | <500ms |
| Proactive suggestions/day | 0 | 3-5 |
| Conversation turns before fatigue | 3-5 | 15-20 |
| Memory recall accuracy | 0% | >70% |
| User satisfaction (qualitative) | Tool | Companion |
