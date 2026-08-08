VOXY Installer Infrastructure
==============================

This directory contains production installer configurations for VOXY.

Contents
--------
  wix/main.wxs           WiX v3 MSI source (cargo-wix compatible)
  wix/voxy.wxs           Standalone WiX v3 source
  wix/voxy.wixproj       WiX v4 project file
  nsis/voxy.nsi           NSIS installer script
  scripts/                Build and verification scripts
  manifest.xml            Windows application manifest
  license.rtf             RTF license for installer UI
  README.md               This file

Build Prerequisites
-------------------
  MSI:
    - cargo install cargo-wix  (recommended)
    - Or: WiX Toolset 3.x (candle.exe + light.exe)

  NSIS:
    - NSIS 3.x (makensis.exe)
    - Download: https://nsis.sourceforge.io/Download

Quick Start
-----------
  # Build MSI (requires cargo-wix + WiX)
  cargo wix --no-build -p voxy-daemon -I wix/main.wxs

  # Build NSIS (requires NSIS)
  makensis /V3 installer/nsis/voxy.nsi

  # Or use build scripts
  installer/scripts/build-msi.bat     # Windows
  installer/scripts/build-nsis.bat    # Windows
  installer/scripts/build-all.bat     # Both

  # CI builds both automatically on tag push
  # See .github/workflows/installers.yml

What Gets Installed
-------------------
  C:\Program Files\VOXY\
    voxy-daemon.exe       Main daemon binary
    voxy-overlay.exe      Desktop overlay
    config\               Configuration directory
    logs\                 Log directory

  Start Menu:
    VOXY\VOXY Daemon      Launch daemon
    VOXY\Uninstall        Remove VOXY

  System:
    PATH entry            %ProgramFiles%\VOXY

Uninstall
---------
  Via Add/Remove Programs or Start Menu → VOXY → Uninstall.
  Removes all files, shortcuts, PATH entry, and registry keys.

Code Signing
------------
  Signing is optional. Set GitHub Actions secrets:
    WINDOWS_CERTIFICATE       Base64-encoded PFX
    WINDOWS_CERTIFICATE_PASSWORD

  Or sign manually with signtool.exe (see README.md).

Version Upgrades
----------------
  Both installers detect existing installations and prompt
  the user to overwrite on upgrade.

Windows Defender
----------------
  Installers use standard MSI/NSIS structure with no
  obfuscation. Should pass Windows Defender checks.
