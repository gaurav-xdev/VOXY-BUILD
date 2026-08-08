# VOXY — Autonomous AI Operating System

A voice-first, autonomous AI operating system built in Rust. VOXY provides a 72-crate workspace architecture with real-time voice processing, cognitive orchestration, and platform-native Windows integration.

## Architecture

```
voxy/
├── apps/                    # Application binaries (daemon, overlay)
├── crates/                  # 67 library crates
│   ├── audio/               # Audio capture, DSP, WASAPI, device management
│   ├── brain/               # Core AI reasoning engine
│   ├── cognition/           # Language understanding, intent parsing
│   ├── cognitive_orchestrator/  # Goal engine, decision making, workflows
│   ├── companion_intelligence/  # Emotional modeling, conversation
│   ├── context/             # Context fusion, memory, activity tracking
│   ├── memory/              # Long-term memory (LTM V2), SQLite storage
│   ├── orchestrator/        # Multi-agent orchestration
│   ├── platform_core/       # Platform abstraction traits
│   ├── platform_windows/    # Windows-specific Win32 implementations
│   ├── planner/             # Task graph, scheduling, dependencies
│   ├── production_harden/   # Stress tests, benchmarks, fault injection
│   ├── voice_runtime/       # Real-time voice pipeline
│   └── ...                  # 60+ more crates
├── docs/                    # Architecture, deployment, developer guides
├── plugins/                 # Plugin system
└── tools/                   # Build and development tools
```

## Quick Start

### Prerequisites

- **Rust**: 1.75+ (stable)
- **LLVM**: Required for clang-sys (set `LIBCLANG_PATH`)
- **CMake**: Required for native builds (set `CMAKE`)
- **Windows**: 10+ (primary platform)

### Build

```bash
# Set environment variables (Windows)
$env:LIBCLANG_PATH="C:\tools\llvm\bin"
$env:CMAKE="C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe"

# Debug build
cargo build

# Release build (LTO, optimized)
cargo build --release
```

### Test

```bash
# Run all library tests
cargo test --workspace --lib

# Run with output
cargo test --workspace --lib -- --nocapture
```

### Lint

```bash
# Clippy
cargo clippy --workspace --all-targets

# Format check
cargo fmt --all -- --check

# Format
cargo fmt --all
```

## Development

### Project Structure

- **67 library crates** under `crates/`
- **2 application binaries** under `apps/` (daemon, overlay)
- **72 total crates** in workspace
- **1488+ tests** across the workspace
- **25 stress tests** for production hardening

### Key Components

| Component | Description |
|-----------|-------------|
| `voxy-audio` | Cross-platform audio runtime: WASAPI, device management, DSP |
| `voxy-brain` | Core AI reasoning engine |
| `voxy-cognitive-orchestrator` | Goal engine, decision making, autonomous workflows |
| `voxy-memory` | Long-term memory with SQLite storage |
| `voxy-platform-windows` | Real Win32 API integration (DPI, multi-monitor, focus) |
| `voxy-voice-runtime` | Real-time voice pipeline |
| `voxy-production-harden` | Stress tests, benchmarks, fault injection |

### Windows APIs Used

- **DPI**: `SetProcessDpiAwarenessContext`, `GetDpiForMonitor` (per-monitor V2)
- **Multi-monitor**: `EnumDisplayMonitors`, `GetMonitorInfoW`
- **Window focus**: `GetForegroundWindow`, `SetForegroundWindow`, `EnumWindows`
- **Audio**: `cpal` (WASAPI backend), `wasapi_improvements` (drift detection)
- **Power**: `RegisterPowerSettingNotification` (planned)

## Release

### Release Profile

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

### Build a Release

```bash
cargo build --release
```

### Artifacts

Release binaries are built with full LTO, single codegen unit, and symbol stripping for maximum performance and minimum binary size.

## License

MIT License — see [LICENSE](LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## Security

See [SECURITY.md](SECURITY.md) for security policy and vulnerability reporting.
