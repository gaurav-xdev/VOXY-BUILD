# Contributing to VOXY

Thank you for your interest in contributing to VOXY! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.75+ (stable)
- LLVM (for clang-sys)
- CMake (for native builds)
- Windows 10+ (primary platform)

### Environment Variables

```powershell
# Windows PowerShell
$env:LIBCLANG_PATH="C:\tools\llvm\bin"
$env:CMAKE="C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe"
```

### Building

```bash
cargo build
cargo build --release
```

### Testing

```bash
# Run all library tests
cargo test --workspace --lib

# Run specific crate tests
cargo test -p voxy-audio --lib
```

### Linting

```bash
# Clippy (must pass with 0 warnings)
cargo clippy --workspace --all-targets

# Format check
cargo fmt --all -- --check

# Auto-format
cargo fmt --all
```

## Code Style

- Follow existing code conventions in each crate
- Use `parking_lot::Mutex`/`RwLock` (no poisoning)
- Prefer `async-trait` for async trait definitions
- Keep `#[allow(dead_code)]` only for intentionally reserved fields
- No `panic!()` in production code — use proper error handling
- No `unwrap()` on user input or external data

## Architecture Rules

- **NEVER** rewrite working modules
- **NEVER** replace existing architecture
- **NEVER** introduce breaking changes
- **NEVER** reduce performance
- **NEVER** remove tests
- Add features as new crates or modules, not by modifying existing ones
- Maintain backward compatibility for all public APIs

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes following the guidelines above
4. Ensure all tests pass (`cargo test --workspace --lib`)
5. Ensure clippy passes with 0 warnings
6. Ensure code is formatted (`cargo fmt --all`)
7. Submit a pull request with a clear description

## Commit Messages

Use conventional commit format:

```
feat: add new feature
fix: fix a bug
docs: update documentation
refactor: refactor code without changing behavior
test: add or update tests
chore: maintenance tasks
```

## Reporting Issues

- Use GitHub Issues for bug reports and feature requests
- Include reproduction steps for bugs
- Include environment details (OS, Rust version, etc.)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
