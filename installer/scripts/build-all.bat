@echo off
REM ─────────────────────────────────────────────────────────
REM  VOXY Installer Build — Both MSI and NSIS
REM ─────────────────────────────────────────────────────────
setlocal

echo ════════════════════════════════════════════════════════
echo  VOXY Full Installer Build
echo ════════════════════════════════════════════════════════
echo.

REM ── Build release binaries once ──
echo [1/4] Building release binaries...
cargo build --release --locked
if errorlevel 1 (
    echo ERROR: Release build failed.
    exit /b 1
)
echo       OK
echo.

REM ── MSI ──
echo [2/4] Building MSI...
call installer\scripts\build-msi.bat
if errorlevel 1 (
    echo WARNING: MSI build failed. Continuing with NSIS...
)
echo.

REM ── NSIS ──
echo [3/4] Building NSIS installer...
call installer\scripts\build-nsis.bat
if errorlevel 1 (
    echo WARNING: NSIS build failed.
)
echo.

REM ── Summary ──
echo [4/4] Build complete.
echo.
echo ════════════════════════════════════════════════════════
echo  Artifacts:
echo    voxy.msi              — MSI installer
echo    voxy-0.1.0-setup.exe  — NSIS installer
echo ════════════════════════════════════════════════════════

endlocal
