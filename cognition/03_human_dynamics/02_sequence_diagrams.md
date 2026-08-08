# HDR Update Sequence

## Happy Path — Normal Operation

```
┌──────────┐     ┌──────────────────────────┐     ┌──────────┐
│ External │     │  HumanDynamicsEngine      │     │ Subsystem│
│  System  │     │                          │     │          │
└────┬─────┘     └──────────┬───────────────┘     └────┬─────┘
     │                      │                          │
     │    HdrInput          │                          │
     │─────────────────────>│                          │
     │                      │                          │
     │                      │──process_trust_events───>│ Trust
     │                      │                          │ Engine
     │                      │<─────────────────────────│
     │                      │                          │
     │                      │──process_trust_events───>│ Relationship
     │                      │                          │ Engine
     │                      │<─────────────────────────│
     │                      │                          │
     │                      │──evaluate(action)────────>│ Protection
     │                      │                          │ Engine
     │                      │<──ProtectionDecision─────│
     │                      │                          │
     │                      │──decide(trust,rel,...)──>│ Initiative
     │                      │                          │ Engine
     │                      │<──InitiativeDecision─────│
     │                      │                          │
     │                      │──calculate(scores)───────>│ Confidence
     │                      │                          │ Engine
     │                      │<──ConfidenceOutput───────│
     │                      │                          │
     │                      │──decide(context)─────────>│ Humor
     │                      │                          │ Engine
     │                      │<──HumorDecision──────────│
     │                      │                          │
     │                      │──check(action,state)─────>│ Policy
     │                      │                          │ Engine
     │                      │<──violations─────────────│
     │                      │                          │
     │                      │──recover(error)──────────>│ Recovery
     │                      │                          │ Engine
     │                      │<──RecoveryAction─────────│
     │                      │                          │
     │                      │──adapt(relationship)─────>│ Style
     │                      │                          │ Engine
     │                      │<──InteractionStyle───────│
     │                      │                          │
     │    HdrOutput         │                          │
     │<─────────────────────│                          │
     │                      │                          │
```

## Blocked Action — Meeting in Progress

```
┌──────────┐     ┌──────────────────────────┐
│ External │     │  HumanDynamicsEngine      │
│  System  │     │                          │
└────┬─────┘     └──────────┬───────────────┘
     │                      │
     │    HdrInput          │
     │  (pending_action,    │
     │   is_meeting=true)   │
     │─────────────────────>│
     │                      │
     │                      │──evaluate()──>│ Protection
     │                      │               │
     │                      │<──blocked─────│ "Meeting in progress"
     │                      │
     │                      │──check()──>│ Policy
     │                      │            │
     │                      │<──violation─│ "Never interrupt meetings"
     │                      │
     │    HdrOutput         │
     │  (allowed=false,     │
     │   violation=...)     │
     │<─────────────────────│
```

## Recovery — Error Handling

```
┌──────────┐     ┌──────────────────────────┐
│ External │     │  HumanDynamicsEngine      │
│  System  │     │                          │
└────┬─────┘     └──────────┬───────────────┘
     │                      │
     │    HdrInput          │
     │  (errors_this_session│
     │   = 2)               │
     │─────────────────────>│
     │                      │
     │                      │──recover()──>│ Recovery
     │                      │              │ Engine
     │                      │<──Recovery───│
     │                      │   Action     │
     │                      │   {ack,correct,desc}
     │                      │
     │    HdrOutput         │
     │  (recovery=Some(...))│
     │<─────────────────────│
```

## State Diagram — Behavior Engine

```
                    ┌─────────────┐
                    │  Sleeping   │
                    └──────┬──────┘
                           │ wake
                           ▼
┌──────────┐    ┌─────────────┐    ┌──────────┐
│ Waiting  │───>│  Observing  │───>│Listening │
└──────────┘    └─────────────┘    └────┬─────┘
     ▲              ▲                   │
     │              │                   │ think
     │              │ user              ▼
     │              │ spoke       ┌──────────┐
     │              └─────────────│ Thinking │
     │                            └────┬─────┘
     │                                 │
     │         ┌───────────────────────┼──────────┐
     │         │ work                  │ protect  │
     │         ▼                       ▼          │
┌────────┴──────────┐         ┌─────────────┐    │
│     Working       │◄───────│  Protecting  │    │
└────────┬──────────┘         └─────────────┘    │
     │         │                                  │
     │ celebrate│                                 │
     ▼         │                                  │
┌──────────┐   │                                  │
│Celebrating│  │                                  │
└────┬─────┘   │                                  │
     │         │                                  │
     └─────────┼──────────────────────────────────┘
               │
         ┌─────┴──────┐
         │ DeepFocus   │
         │ MissionMode │
         └────────────┘
```
