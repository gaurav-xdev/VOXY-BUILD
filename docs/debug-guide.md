# VOXY Debug Guide

## Common Issues

### Build Errors

#### Missing LIBCLANG_PATH
```
error: Could not find libclang
```
**Fix**: Set environment variable:
```powershell
$env:LIBCLANG_PATH="C:\tools\llvm\bin"
```

#### Missing CMake
```
error: Could not find cmake
```
**Fix**:
```powershell
$env:CMAKE="C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe"
```

#### Feature Gate Errors
```
error[E0432]: unresolved import `voxy_whisper::WhisperModel`
```
**Fix**: Enable required feature:
```toml
[features]
default = ["whisper", "openai"]
whisper = ["dep:voxy-whisper"]
```

### Test Failures

#### flaky Timing Tests
Some tests have timing assertions (e.g., `assert!(latency < 100)`). These may fail on slow machines.
**Fix**: Run in release mode or with `--release` flag.

#### Memory Capacity Tests
```
assertion failed: mem.count() <= 110
```
**Cause**: Forgetting algorithm timing varies. Usually transient.

### Runtime Errors

#### SQLite Lock Timeout
```
database is locked
```
**Cause**: Holding `tokio::sync::Mutex` across synchronous SQLite I/O.
**Fix**: Use `tokio::task::spawn_blocking` for synchronous database operations.

#### Event Bus Overflow
```
Subscriber queue full
```
**Cause**: Subscribers not draining fast enough.
**Fix**: Increase buffer size or reduce publish rate.

## Debugging Tools

### RUST_BACKTRACE
```powershell
$env:RUST_BACKTRACE=1
cargo test -p voxy-memory --lib -- --nocapture
```

### Logging
Enable tracing with:
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

```rust
use tracing::{info, warn, error, debug, trace};

info!("Subsystem started");
warn!("High latency detected");
error!("Connection failed");
debug!("Processing event: {:?}", event);
trace!("Entering function: {}", function_name);
```

### Performance Profiling
```bash
# Build with profiling
cargo build --release -p voxy-production-harden

# Run benchmarks
cargo test --release -p voxy-production-harden --lib bench_ -- --nocapture
```

## Memory Debugging

### Memory Capacity Check
```rust
let mem = LongTermMemoryV2::default_memory();
println!("Memory count: {}", mem.count());
```

### Memory Query Debug
```rust
let query = MemoryQueryV2 {
    text: Some("search term".to_string()),
    min_importance: Some(0.3),
    max_results: 10,
    ..Default::default()
};
let result = mem.query(&query);
println!("Found {} items", result.items.len());
```

## Event Bus Debugging

### Topic Count
```rust
let count = bus.topic_count().await;
println!("Active topics: {}", count);
```

### Dead Letters
```rust
let dead = bus.dead_letters().await;
println!("Dead letters: {}", dead.len());
```

### Stats
```rust
let stats = bus.stats("topic.path").await?;
println!("Messages: {}, Subscribers: {}", stats.message_count, stats.subscriber_count);
```

## Recovery Debugging

### Check Subsystem Health
```rust
let health = recovery.health("subsystem_name");
match health {
    Some(h) => println!("State: {:?}", h.state),
    None => println!("Subsystem not registered"),
}
```

### Check Circuit Breaker
```rust
let action = recovery.report_failure("subsystem_name", "reason");
match action {
    RecoveryAction::CircuitOpen { subsystem } => {
        println!("Circuit breaker tripped for {}", subsystem);
    }
    RecoveryAction::Restart { attempt, .. } => {
        println!("Restart attempt {}", attempt);
    }
    _ => {}
}
```

## Windows-Specific Issues

### WASAPI Device Access
- Ensure microphone permissions are granted
- Check if another application is using the microphone exclusively
- Try WASAPI shared mode instead of exclusive mode

### DPI Awareness
- VOXY uses HMODULE-based DPI awareness (deferred)
- On multi-monitor setups, may need manual DPI adjustment

### Antivirus Interference
- Some antivirus software blocks child process creation
- Add VOXY executable to antivirus whitelist
