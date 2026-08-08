@echo off
REM ─────────────────────────────────────────────────────────
REM  VOXY MSI Build Script
REM  Requires: WiX Toolset 3.x in PATH (candle.exe, light.exe)
REM  Or: cargo install cargo-wix && cargo wix
REM ─────────────────────────────────────────────────────────
setlocal enabledelayedexpansion

echo ════════════════════════════════════════════════════════
echo  VOXY MSI Installer Build
echo ════════════════════════════════════════════════════════
echo.

REM ── Step 1: Build release binaries ──
echo [1/5] Building release binaries...
cargo build --release --locked
if errorlevel 1 (
    echo ERROR: Release build failed.
    exit /b 1
)
echo       OK
echo.

REM ── Step 2: Check WiX availability ──
echo [2/5] Checking WiX Toolset...
where candle.exe >nul 2>&1
if errorlevel 1 (
    echo       candle.exe not found. Trying cargo-wix...
    where cargo-wix >nul 2>&1
    if errorlevel 1 (
        echo       Installing cargo-wix...
        cargo install cargo-wix
        if errorlevel 1 (
            echo ERROR: Cannot install cargo-wix. Install WiX Toolset manually:
            echo        https://wixtoolset.org/releases/
            exit /b 1
        )
    )
    echo       Using cargo-wix
    goto :build_with_cargo_wix
)
echo       Using WiX Toolset (candle + light)
goto :build_with_wix

:build_with_cargo_wix
echo [3/5] Building MSI with cargo-wix...
cargo wix --no-logo --nocapture
if errorlevel 1 (
    echo ERROR: cargo-wix build failed.
    exit /b 1
)
echo       MSI built successfully.
goto :done

:build_with_wix
echo [3/5] Compiling WiX source...
candle.exe -nologo -out installer\wix\voxy.wixobj installer\wix\voxy.wxs
if errorlevel 1 (
    echo ERROR: candle.exe compilation failed.
    exit /b 1
)
echo       OK

echo [4/5] Linking MSI...
light.exe -nologo -out voxy.msi -ext WixUIExtension installer\wix\voxy.wixobj
if errorlevel 1 (
    echo ERROR: light.exe linking failed.
    exit /b 1
)
echo       MSI built successfully.

:done
echo [5/5] Installer ready.
echo.
echo ════════════════════════════════════════════════════════
echo  Output: voxy.msi
echo ════════════════════════════════════════════════════════

endlocal
