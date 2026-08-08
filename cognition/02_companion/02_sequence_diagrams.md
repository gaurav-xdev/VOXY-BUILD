# Companion Engine — Sequence Diagrams

## 1. User Returns After Absence

```
User                  Engine                Greeting            Silence           Presence
 │                      │                      │                   │                  │
 │  (opens app)         │                      │                   │                  │
 │─────────────────────>│                      │                   │                  │
 │                      │ update(Away→Active)  │                   │                  │
 │                      │─────────────────────────────────────────────────────────────>│
 │                      │                      │                   │   <─Breathing    │
 │                      │                      │                   │                  │
 │                      │ generate(is_return)  │                   │                  │
 │                      │─────────────────────>│                   │                  │
 │                      │  "Welcome back."     │                   │                  │
 │                      │<─────────────────────│                   │                  │
 │                      │                      │                   │                  │
 │                      │ decide(has_greeting) │                   │                  │
 │                      │────────────────────────────────────────>│                  │
 │                      │  Speak               │                   │                  │
 │                      │<────────────────────────────────────────│                  │
 │                      │                      │                   │                  │
 │  display:"Welcome    │                      │                   │                  │
 │  back." + breathing  │                      │                   │                  │
 │<─────────────────────│                      │                   │                  │
```

## 2. Deep Focus Protection

```
User                  Engine                Silence           Attention
 │                      │                      │                  │
 │  (coding intensely)  │                      │                  │
 │─────────────────────>│                      │                  │
 │                      │ update(Focused)      │                  │
 │                      │──────────────────────────────────────>│
 │                      │  <─focus: 0.9        │                  │
 │                      │                      │                  │
 │                      │ decide(deep_focus)   │                  │
 │                      │─────────────────────>│                  │
 │                      │  Silent              │                  │
 │                      │<─────────────────────│                  │
 │                      │                      │                  │
 │  display: None       │                      │                  │
 │  silence: true       │                      │                  │
 │<─────────────────────│                      │                  │
```

## 3. Mission Background Work

```
User                  Engine              Mission             Journey
 │                      │                    │                    │
 │  (leaves desk)       │                    │                    │
 │─────────────────────>│                    │                    │
 │  presence: Away      │                    │                    │
 │                      │                    │                    │
 │                      │ start_mission()    │                    │
 │                      │───────────────────>│                    │
 │                      │  Active:Coding     │                    │
 │                      │                    │                    │
 │  [user away 30min]   │                    │                    │
 │                      │                    │                    │
 │  (returns)           │                    │                    │
 │─────────────────────>│                    │                    │
 │  presence: Active    │                    │                    │
 │                      │                    │                    │
 │                      │ complete_mission() │                    │
 │                      │───────────────────>│                    │
 │                      │  "Code implemented"│                    │
 │                      │                    │                    │
 │                      │ record_milestone() │                    │
 │                      │───────────────────────────────────────>│
 │                      │                    │                    │
 │  display: "While you │                    │                    │
 │  were away: Coding   │                    │                    │
 │  completed in 30min" │                    │                    │
 │<─────────────────────│                    │                    │
```

## 4. Memory Moment

```
User                  Engine              Memory             Journey
 │                      │                    │                    │
 │  (working on X)      │                    │                    │
 │─────────────────────>│                    │                    │
 │                      │                    │                    │
 │                      │ generate(context)  │                    │
 │                      │───────────────────>│                    │
 │                      │  "We finished      │                    │
 │                      │  Context Fusion    │                    │
 │                      │  yesterday"        │                    │
 │                      │<───────────────────│                    │
 │                      │                    │                    │
 │  display: "We        │                    │                    │
 │  finished Context    │                    │                    │
 │  Fusion yesterday"   │                    │                    │
 │<─────────────────────│                    │                    │
```

## 5. Micro Interaction Flow

```
User                  Engine              Micro              Silence
 │                      │                    │                    │
 │  (completes task)    │                    │                    │
 │─────────────────────>│                    │                    │
 │                      │                    │                    │
 │                      │ decide(can_speak)  │                    │
 │                      │───────────────────────────────────────>│
 │                      │  Speak             │                    │
 │                      │<───────────────────────────────────────│
 │                      │                    │                    │
 │                      │ generate()         │                    │
 │                      │───────────────────>│                    │
 │                      │  "Nice."           │                    │
 │                      │<───────────────────│                    │
 │                      │                    │                    │
 │  display: "Nice."    │                    │                    │
 │<─────────────────────│                    │                    │
```
