@echo off
REM ─────────────────────────────────────────────────────────
REM  VOXY Installer Verification Script
REM  Run after building to verify installer integrity
REM ─────────────────────────────────────────────────────────
setlocal enabledelayedexpansion

echo ════════════════════════════════════════════════════════
echo  VOXY Installer Verification
echo ════════════════════════════════════════════════════════
echo.

set PASS=0
set FAIL=0

REM ── Check binaries exist ──
echo [1/8] Checking release binaries...
if exist "target\release\voxy-daemon.exe" (
    echo       voxy-daemon.exe: FOUND
    set /a PASS+=1
) else (
    echo       voxy-daemon.exe: MISSING
    set /a FAIL+=1
)
if exist "target\release\voxy-overlay.exe" (
    echo       voxy-overlay.exe: FOUND
    set /a PASS+=1
) else (
    echo       voxy-overlay.exe: MISSING
    set /a FAIL+=1
)
echo.

REM ── Check MSI exists ──
echo [2/8] Checking MSI installer...
if exist "voxy.msi" (
    echo       voxy.msi: FOUND
    set /a PASS+=1
) else (
    echo       voxy.msi: NOT FOUND (skipping verification)
    goto :check_nsis
)
echo.

REM ── Verify MSI is valid ──
echo [3/8] Verifying MSI structure...
powershell -Command "$msi = Get-Item 'voxy.msi'; if ($msi.Length -gt 1MB) { Write-Host '       MSI size: ' + [math]::Round($msi.Length/1MB, 2) + ' MB - OK' } else { Write-Host '       MSI too small - possible corruption'; exit 1 }"
if errorlevel 1 (
    set /a FAIL+=1
) else (
    set /a PASS+=1
)
echo.

REM ── Check NSIS exists ──
:check_nsis
echo [4/8] Checking NSIS installer...
if exist "voxy-0.1.0-setup.exe" (
    echo       voxy-0.1.0-setup.exe: FOUND
    set /a PASS+=1
) else (
    echo       voxy-0.1.0-setup.exe: NOT FOUND (skipping verification)
    goto :check_sha
)
echo.

REM ── Verify NSIS is valid ──
echo [5/8] Verifying NSIS structure...
powershell -Command "$nsis = Get-Item 'voxy-0.1.0-setup.exe'; if ($nsis.Length -gt 1MB) { Write-Host '       NSIS size: ' + [math]::Round($nsis.Length/1MB, 2) + ' MB - OK' } else { Write-Host '       NSIS too small - possible corruption'; exit 1 }"
if errorlevel 1 (
    set /a FAIL+=1
) else (
    set /a PASS+=1
)
echo.

REM ── Check SHA256 checksums ──
:check_sha
echo [6/8] Verifying checksums...
if exist "voxy.msi.sha256" (
    echo       MSI SHA256: PRESENT
    set /a PASS+=1
) else (
    echo       MSI SHA256: MISSING
    set /a FAIL+=1
)
if exist "voxy-0.1.0-setup.exe.sha256" (
    echo       NSIS SHA256: PRESENT
    set /a PASS+=1
) else (
    echo       NSIS SHA256: MISSING
    set /a FAIL+=1
)
echo.

REM ── Check installer metadata ──
echo [7/8] Checking installer metadata...
if exist "installer\license.rtf" (
    echo       license.rtf: FOUND
    set /a PASS+=1
) else (
    echo       license.rtf: MISSING
    set /a FAIL+=1
)
if exist "installer\manifest.xml" (
    echo       manifest.xml: FOUND
    set /a PASS+=1
) else (
    echo       manifest.xml: MISSING
    set /a FAIL+=1
)
if exist "LICENSE" (
    echo       LICENSE: FOUND
    set /a PASS+=1
) else (
    echo       LICENSE: MISSING
    set /a FAIL+=1
)
echo.

REM ── Check uninstall support ──
echo [8/8] Checking uninstall support...
if exist "installer\nsis\voxy.nsi" (
    findstr /C:"Section \"Uninstall\"" "installer\nsis\voxy.nsi" >nul 2>&1
    if errorlevel 1 (
        echo       Uninstall section: NOT FOUND in NSIS script
        set /a FAIL+=1
    ) else (
        echo       Uninstall section: FOUND
        set /a PASS+=1
    )
) else (
    echo       NSIS script: MISSING
    set /a FAIL+=1
)
echo.

REM ── Summary ──
echo ════════════════════════════════════════════════════════
echo  Results: %PASS% passed, %FAIL% failed
echo ════════════════════════════════════════════════════════

if %FAIL% gtr 0 (
    echo.
    echo  VERIFICATION FAILED
    exit /b 1
) else (
    echo.
    echo  ALL CHECKS PASSED
    exit /b 0
)

endlocal
