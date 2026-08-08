# VOXY Recovery and Troubleshooting

## System Recovery

### Automatic Recovery
VOXY has built-in automatic recovery through the `SubsystemRecovery` system:

1. **Crash Detection**: Each subsystem reports health status
2. **Restart**: Failed subsystems are restarted with exponential backoff
3. **Circuit Breaker**: Prevents cascading failures
4. **Isolation**: One subsystem failure doesn't bring down others

### Manual Recovery

#### Restart a Subsystem
```rust
recovery.reset("subsystem_name");
```

#### Check Health
```rust
let health = recovery.health("subsystem_name");
println!("State: {:?}", health.state);
println!("Restart count: {}", health.restart_count);
```

#### View All Health
```rust
let all_health = recovery.all_health();
for (name, health) in all_health {
    println!("{}: {:?}", name, health.state);
}
```

## Common Failure Scenarios

### 1. Voice Pipeline Failure
**Symptoms**: No audio output, TTS errors
**Recovery**:
```rust
// Check audio device availability
let audio_system = AudioSystem::new()?;
audio_system.detect_devices()?;

// Restart voice pipeline
voice_pipeline.restart()?;
```

### 2. Memory Database Locked
**Symptoms**: "database is locked" errors
**Recovery**:
- Wait for current operation to complete
- Restart database connection
- Check for long-running synchronous operations

### 3. Event Bus Overflow
**Symptoms**: "Subscriber queue full" errors
**Recovery**:
```rust
// Increase buffer size
let bus = EventBus::new(512); // Double from 256

// Or reduce publish rate
```

### 4. LLM Provider Timeout
**Symptoms**: LLM requests failing
**Recovery**:
```rust
// Switch to backup provider
router.set_mode(RoutingMode::LocalOnly);

// Or retry with different model
router.force_model("backup-model")?;
```

### 5. Memory Capacity Exceeded
**Symptoms**: Old memories being forgotten
**Recovery**:
```rust
// Increase capacity
let mem = LongTermMemoryV2::new(20000, 10000, 1000, 0.05);

// Or compress existing memories
mem.compress_old_memories()?;
```

## Debugging Steps

### 1. Check Logs
```bash
grep "ERROR" logs/voxy.log
grep "WARN" logs/voxy.log
```

### 2. Check Health
```rust
let health = recovery.all_health();
let failed: Vec<_> = health.iter()
    .filter(|(_, h)| matches!(h.state, SubsystemState::Failed { .. }))
    .collect();
println!("Failed subsystems: {:?}", failed);
```

### 3. Check Memory
```rust
let mem = LongTermMemoryV2::default_memory();
println!("Memory count: {}", mem.count());
println!("Memory capacity: {}", mem.capacity());
```

### 4. Check Event Bus
```rust
let topic_count = bus.topic_count().await;
let dead_letters = bus.dead_letters().await;
println!("Topics: {}, Dead letters: {}", topic_count, dead_letters.len());
```

### 5. Check Task Graph
```rust
let active_tasks = task_manager.active_count();
let pending_tasks = task_manager.pending_count();
println!("Active: {}, Pending: {}", active_tasks, pending_tasks);
```

## Performance Issues

### High Latency
**Diagnosis**:
```rust
let snapshot = telemetry.snapshot();
println!("CPU: {}%", snapshot.aggregate_cpu_percent);
println!("Memory: {}MB", snapshot.total_memory_mb);
println!("Latency: {}ms", snapshot.total_events_per_sec);
```

**Solutions**:
1. Reduce memory capacity
2. Use faster LLM provider
3. Enable GPU acceleration
4. Reduce event publishing rate

### Memory Leaks
**Diagnosis**:
```rust
// Monitor memory over time
let initial = mem.count();
// ... run system for a while ...
let final_count = mem.count();
println!("Memory growth: {}", final_count - initial);
```

**Solutions**:
1. Enable forgetting algorithm
2. Compress old memories
3. Archive inactive memories
4. Check for unbounded collections

### Event Bus Bottleneck
**Diagnosis**:
```rust
let stats = bus.stats("topic.path").await?;
println!("Messages/sec: {}", stats.message_count);
println!("Subscribers: {}", stats.subscriber_count);
```

**Solutions**:
1. Increase buffer size
2. Reduce event frequency
3. Batch events
4. Use direct communication for high-frequency events

## Emergency Procedures

### System Hang
1. Check for deadlocks in database
2. Review event bus health
3. Check memory usage
4. Restart affected subsystems

### Data Loss
1. Restore from backup
2. Check memory integrity
3. Verify event bus state
4. Rebuild task graph

### Security Breach
1. Check audit logs
2. Verify consent status
3. Review access patterns
4. Update security policies

## Monitoring Dashboard

### Key Metrics
- **Uptime**: System availability
- **Latency**: Average response time
- **Throughput**: Events processed per second
- **Error Rate**: Errors per minute
- **Memory Usage**: System memory consumption
- **Task Completion**: Tasks completed per hour

### Alert Thresholds
- CPU > 80%: Warning
- Memory > 90%: Critical
- Error Rate > 10/min: Warning
- Latency > 500ms: Warning
