# VOXY Developer Guide

## Prerequisites

- Rust 1.75+ (edition 2021)
- LLVM (for clang bindings): `C:\tools\llvm\bin`
- CMake: `C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe`

## Environment Setup

```powershell
$env:LIBCLANG_PATH="C:\tools\llvm\bin"
$env:CMAKE="C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe"
```

## Building

```bash
# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p voxy-voice

# Build in release mode
cargo build --release -p voxy-production-harden
```

## Testing

```bash
# Run all workspace tests
cargo test --workspace --lib

# Run specific crate tests
cargo test -p voxy-memory --lib

# Run specific test
cargo test -p voxy-memory --lib test_memory_store

# Run with output
cargo test -p voxy-cognitive-orchestrator --lib -- --nocapture
```

**Note**: PowerShell does not support `&&`. Use `;` to chain commands:
```powershell
# Correct
$env:LIBCLANG_PATH="C:\tools\llvm\bin"; cargo test --workspace --lib

# Incorrect
$env:LIBCLANG_PATH="C:\tools\llvm\bin" && cargo test --workspace --lib
```

## Architecture Rules

### TITAN BRAIN Rules
- NEVER rewrite working modules
- NEVER replace existing architecture
- NEVER introduce breaking changes
- NEVER reduce performance
- NEVER remove tests

### OBSIDIAN Rules
- No feature additions
- No architecture changes
- No breaking changes
- Optimize only
- Everything must compile
- All tests must pass

## Adding a New Crate

1. Create directory under `crates/`
2. Add `Cargo.toml` with workspace inheritance:
```toml
[package]
name = "voxy-my-crate"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
voxy-shared = { workspace = true }
```

3. Register in workspace `Cargo.toml` members
4. Add tests in `src/lib.rs` under `#[cfg(test)] mod tests`

## Adding a New Module to Existing Crate

1. Create file: `crates/cognitive_orchestrator/src/my_module.rs`
2. Register in `lib.rs`:
```rust
pub mod my_module;
```
3. Add tests in the module file
4. Ensure all existing tests still pass

## Event System Usage

### Publishing Events
```rust
use voxy_event_bus::EventBus;
use voxy_shared::Event;

let bus = EventBus::new(256);
let event = Event::new("topic.path", "source_subsystem", payload_bytes);
bus.publish("topic.path", event).await?;
```

### Subscribing to Events
```rust
let mut rx = bus.subscribe("topic.path").await?;
while let Ok(event) = rx.recv().await {
    // Process event
}
```

## Memory System

### Storing Memory
```rust
use voxy_memory::{LongTermMemoryV2, MemoryItemV2, MemoryCategory, ltm_v2::{MemoryId, ImportanceFactors}};

let mut mem = LongTermMemoryV2::default_memory();
let item = MemoryItemV2 {
    id: MemoryId::new(),
    category: MemoryCategory::Semantic,
    content: "Important fact".to_string(),
    summary: None,
    tags: vec!["fact".to_string()],
    importance: ImportanceFactors::default(),
    created_at: chrono::Utc::now(),
    last_accessed: chrono::Utc::now(),
    access_count: 0,
    version: 1,
    compressed: false,
    archived: false,
    project_id: None,
    related_memory_ids: Vec::new(),
    metadata: std::collections::HashMap::new(),
};
mem.store(item);
```

## Task Graph

### Building a Task Graph
```rust
use voxy_planner::task_graph::*;

let graph = TaskGraphBuilder::new("project", "main project")
    .task("design", "Design phase", TaskType::Design)
    .then("implement", "Implementation", TaskType::Code)
    .then("test", "Testing", TaskType::Test)
    .build()?;

let layers = graph.topological_layers()?;
let critical = graph.critical_path()?;
```

## Adding Tests

Every new feature MUST include tests. Tests should:
- Test the public API
- Test edge cases
- Test error conditions
- Use descriptive test names
- Be independent (no test-to-test dependencies)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_basic() {
        // Arrange
        // Act
        // Assert
    }

    #[test]
    fn test_feature_error_case() {
        // Test error conditions
    }
}
```
