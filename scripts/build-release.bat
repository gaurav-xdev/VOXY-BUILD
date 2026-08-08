@echo off
REM VOXY Reproducible Build Script
REM Ensures consistent, deterministic builds across environments

setlocal enabledelayedexpansion

echo ============================================
echo VOXY Reproducible Build
echo ============================================

REM Step 1: Verify Rust toolchain
echo [1/6] Verifying Rust toolchain...
rustc --version
cargo --version
if errorlevel 1 (
    echo ERROR: Rust toolchain not found
    exit /b 1
)

REM Step 2: Clean previous artifacts
echo [2/6] Cleaning previous artifacts...
cargo clean --release 2>nul

REM Step 3: Verify dependencies
echo [3/6] Verifying dependencies...
cargo fetch
if errorlevel 1 (
    echo ERROR: Failed to fetch dependencies
    exit /b 1
)

REM Step 4: Check formatting
echo [4/6] Checking formatting...
cargo fmt --all -- --check
if errorlevel 1 (
    echo WARNING: Code not formatted, running cargo fmt
    cargo fmt --all
)

REM Step 5: Run clippy
echo [5/6] Running clippy...
cargo clippy --workspace --all-targets -- -D warnings
if errorlevel 1 (
    echo ERROR: Clippy warnings found
    exit /b 1
)

REM Step 6: Build release
echo [6/6] Building release...
cargo build --release
if errorlevel 1 (
    echo ERROR: Release build failed
    exit /b 1
)

echo ============================================
echo Build Complete
echo ============================================
echo.
echo Binary: target\release\voxy-daemon.exe
echo.
echo To verify reproducibility, compare SHA-256 hash:
certutil -hashfile target\release\voxy-daemon.exe SHA256
