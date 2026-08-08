# VOXY Performance Tuning Guide

## Performance Metrics

### Key Metrics to Monitor
- **Event Bus Latency**: P99 < 200μs
- **Memory Store Latency**: P99 < 100μs
- **Goal Engine Latency**: P99 < 200μs
- **Task Graph Build**: P99 < 5ms
- **Memory Query**: > 100 queries/sec
- **Throughput**: > 5000 events/sec

## Optimization Strategies

### 1. Event Bus Optimization

#### Current Performance
- P99 latency: ~110μs
- Throughput: >5000 events/sec

#### Tuning
```rust
// Increase buffer size for higher throughput
let bus = EventBus::new(512); // Default: 256

// Reduce dead letter queue size
let bus = EventBus::with_dead_letter(256, 50); // Default: 100
```

#### Best Practices
- Minimize event payload size
- Batch related events
- Use appropriate topic granularity
- Monitor dead letters

### 2. Memory System Optimization

#### Current Performance
- Store latency: P99 ~30μs
- Query throughput: >100 queries/sec

#### Tuning
```rust
// Adjust capacity for your use case
let mem = LongTermMemoryV2::new(
    10000,  // max capacity
    5000,   // preference capacity
    500,    // max compressed
    0.1     // forgetting rate
);

// Reduce forgetting rate for more persistent memory
let mem = LongTermMemoryV2::new(10000, 5000, 500, 0.05);
```

#### Best Practices
- Use appropriate memory categories
- Set meaningful importance factors
- Regular memory compression
- Monitor capacity usage

### 3. Task Graph Optimization

#### Current Performance
- Build latency: P99 ~2.5ms
- Sort latency: P99 ~1ms

#### Tuning
```rust
// Use efficient task types
let graph = TaskGraphBuilder::new("project", "description")
    .task("task1", "desc", TaskType::Code)  // Use specific types
    .then("task2", "desc", TaskType::Test)
    .build()?;

// Parallel execution for independent tasks
let executor = TaskGraphExecutor::new(8); // 8 concurrent tasks
```

#### Best Practices
- Minimize task dependencies
- Use appropriate task types
- Enable parallel execution
- Monitor critical path

### 4. Goal Engine Optimization

#### Current Performance
- Create latency: P99 ~95μs
- Update latency: P99 ~50μs

#### Tuning
```rust
// Adjust capacity for your use case
let engine = GoalEngineV2::new(1000, 100);

// Set appropriate priorities
engine.create_goal(
    "Goal".to_string(),
    "Description".to_string(),
    GoalPriority::High,  // Use appropriate priority
    None
)?;
```

#### Best Practices
- Use meaningful priorities
- Set realistic deadlines
- Regular progress updates
- Monitor goal completion

### 5. Audio System Optimization

#### WASAPI Configuration
```rust
// Use appropriate buffer size
let config = AudioConfig {
    buffer_size: 1024,  // Smaller = lower latency
    sample_rate: 44100,
    channels: 1,
    ..Default::default()
};

// Enable exclusive mode for lowest latency (if supported)
let config = AudioConfig {
    exclusive_mode: true,
    ..config
};
```

#### Best Practices
- Use appropriate sample rate
- Balance latency vs stability
- Monitor buffer underruns
- Test with actual hardware

### 6. LLM Provider Optimization

#### Model Selection
```rust
// Use local models for lower latency
router.set_mode(RoutingMode::LocalOnly);

// Use cloud models for higher quality
router.set_mode(RoutingMode::CloudOnly);

// Use hybrid approach
router.set_mode(RoutingMode::Hybrid);
```

#### Best Practices
- Cache frequent requests
- Use appropriate model size
- Monitor response times
- Implement retry logic

## Benchmarking

### Running Benchmarks
```bash
# Build with optimizations
cargo build --release -p voxy-production-harden

# Run all benchmarks
cargo test --release -p voxy-production-harden --lib bench_ -- --nocapture

# Run specific benchmark
cargo test --release -p voxy-production-harden --lib bench_event_bus_latency -- --nocapture
```

### Interpreting Results
- **P99 Latency**: 99th percentile latency (lower is better)
- **Throughput**: Operations per second (higher is better)
- **Memory Usage**: Total memory consumption (lower is better)

### Benchmark Categories
1. **Event Bus**: Publish/subscribe latency and throughput
2. **Memory**: Store/query latency and throughput
3. **Task Graph**: Build/sort latency
4. **Goal Engine**: Create/update latency
5. **Recovery**: Failure detection and restart time

## Production Tuning

### Windows-Specific
```powershell
# Set process priority
$process = Get-Process voxy
$process.PriorityClass = "High"

# Disable power saving
powercfg /setactive 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c

# Disable CPU throttling
powercfg /setacvalueindex SCHEME_CURRENT SUB_PROCESSOR THROTTLING 0
```

### Linux-Specific
```bash
# Set real-time priority
sudo chrt -f 50 ./voxy

# Disable CPU frequency scaling
sudo cpufreq-set -g performance

# Increase file descriptor limit
ulimit -n 65536
```

### Memory Tuning
```rust
// Increase memory capacity for long-running sessions
let mem = LongTermMemoryV2::new(50000, 25000, 2500, 0.05);

// Enable aggressive compression
let config = CompressionConfig {
    threshold: 0.3,  // Compress when importance < 0.3
    ratio: 0.5,      // 50% compression ratio
};
```

### Network Tuning
```rust
// Increase connection pool size
let config = NetworkConfig {
    max_connections: 100,
    timeout_ms: 30000,
    keep_alive: true,
};
```

## Monitoring in Production

### Key Metrics to Track
```rust
let snapshot = telemetry.snapshot();
println!("CPU: {}%", snapshot.aggregate_cpu_percent);
println!("Memory: {}MB", snapshot.total_memory_mb);
println!("Events/sec: {:.1}", snapshot.total_events_per_sec);
println!("Errors: {}", snapshot.total_errors);
```

### Alert Thresholds
- **CPU > 80%**: Warning
- **Memory > 90%**: Critical
- **Error Rate > 10/min**: Warning
- **Latency > 500ms**: Warning
- **Dead Letters > 100**: Warning

### Dashboard Metrics
1. **Uptime**: System availability percentage
2. **Latency**: Average response time
3. **Throughput**: Events processed per second
4. **Error Rate**: Errors per minute
5. **Memory Usage**: System memory consumption
6. **Task Completion**: Tasks completed per hour

## Performance Testing

### Load Testing
```bash
# Run stress tests
cargo test --release -p voxy-production-harden --lib stress_ -- --nocapture

# Run fault injection tests
cargo test --release -p voxy-production-harden --lib fault_ -- --nocapture
```

### Performance Regression Testing
```bash
# Before changes
cargo test --release -p voxy-production-harden --lib bench_ -- --nocapture > before.txt

# After changes
cargo test --release -p voxy-production-harden --lib bench_ -- --nocapture > after.txt

# Compare
diff before.txt after.txt
```
