# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-08-02

### Added

#### TITAN BRAIN — Core Modules
- Task Graph V2: Dependency-aware task scheduling with topological sort
- Autonomous Workflows: State machine for multi-step automation
- Decision Engine: Risk/confidence-weighted action selection
- Multi-Agent Orchestrator: Agent lifecycle and message routing
- Self-Improvement: Performance tracking and parameter tuning
- Project Manager: Sprint/phase tracking with burndown
- Goal Engine V2: Persistent goals with priorities and dependencies
- LTM V2: Long-term memory with semantic search and consolidation
- Owner Command Center: Dashboard metrics and alerting
- SDK + Extensibility: Plugin framework and API surface

#### TITAN FUSION — Integration Layer
- ServiceHub: Centralized service registry and lifecycle
- EventBridge: Typed event routing between subsystems
- CentralTelemetry: Unified metrics collection and alerting
- SubsystemRecovery: Circuit breaker and restart logic
- UnifiedPipeline: Coordinated voice processing pipeline
- BootSequence: Deterministic startup ordering

#### OBSIDIAN — Production Hardening
- Removed all `panic!()`, `unwrap()`, and `expect()` from production code
- Replaced blocking calls with async alternatives (`spawn_blocking`)
- Dead code cleanup: 58 warnings → 0 warnings
- Performance optimization: Goal Create p99 -93%, Graph Sort p99 -60%
- Stress tests: 25/25 pass in release mode
- Fault injection: Cascading failure isolation verified
- Benchmark suite: Event bus, telemetry, memory, goal engine, task graph, recovery

#### Platform — Windows Integration
- Real DPI awareness: Per-Monitor V2 via `SetProcessDpiAwarenessContext`
- Multi-monitor: `EnumDisplayMonitors` with per-monitor geometry
- Window focus: `GetForegroundWindow`, `SetForegroundWindow`, `EnumWindows`
- OS version: `RtlGetVersion` for accurate Windows version reporting

#### Release Engineering
- GitHub Actions CI: Build, test, clippy, fmt, audit
- Release pipeline: LTO, strip, panic=abort, artifact generation
- Documentation: README, CONTRIBUTING, SECURITY, CHANGELOG
- Dependency auditing: cargo-deny configuration
- Reproducible builds: Cargo.lock, rust-toolchain.toml

### Fixed
- SQLite engine: Replaced `panic!()` with noop implementations for uninitialized engines
- Voice pipeline: Fixed `partial_cmp().unwrap()` in memory scoring
- Daemon: Fixed blocking filesystem operations in async context
- Model router: Migrated from `std::sync::RwLock` to `parking_lot::RwLock`

### Technical Details
- 72 crates in workspace (67 libraries, 2 apps, 3 development)
- 1488+ tests across workspace
- 0 compiler warnings in release build
- Windows 10+ (x86_64-pc-windows-msvc) primary target
- Rust 1.75+ MSRV
