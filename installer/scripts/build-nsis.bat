@echo off
REM ─────────────────────────────────────────────────────────
REM  VOXY NSIS Build Script
REM  Requires: NSIS 3.x in PATH (makensis.exe)
REM  Download: https://nsis.sourceforge.io/Download
REM ─────────────────────────────────────────────────────────
setlocal enabledelayedexpansion

echo ════════════════════════════════════════════════════════
echo  VOXY NSIS Installer Build
echo ════════════════════════════════════════════════════════
echo.

REM ── Step 1: Build release binaries ──
echo [1/3] Building release binaries...
cargo build --release --locked
if errorlevel 1 (
    echo ERROR: Release build failed.
    exit /b 1
)
echo       OK
echo.

REM ── Step 2: Check NSIS availability ──
echo [2/3] Checking NSIS...
where makensis.exe >nul 2>&1
if errorlevel 1 (
    echo ERROR: makensis.exe not found.
    echo        Install NSIS 3.x from: https://nsis.sourceforge.io/Download
    echo        Make sure makensis.exe is in your PATH.
    exit /b 1
)
echo       NSIS found
echo.

REM ── Step 3: Build installer ──
echo [3/3] Building NSIS installer...
makensis.exe /V3 installer\nsis\voxy.nsi
if errorlevel 1 (
    echo ERROR: NSIS build failed.
    exit /b 1
)
echo.
echo ════════════════════════════════════════════════════════
echo  Output: voxy-0.1.0-setup.exe
echo ════════════════════════════════════════════════════════

endlocal
