#!/usr/bin/env bash
# VOXY Installer Build Script (Linux/WSL)
# Requires: WiX via dotnet tool or wine + WiX
set -euo pipefail

echo "════════════════════════════════════════════════════════"
echo " VOXY Installer Build (Linux/WSL)"
echo "════════════════════════════════════════════════════════"
echo

# Build release binaries (cross-compile for Windows if on Linux)
echo "[1/3] Building release binaries..."
cargo build --release --locked --target x86_64-pc-windows-msvc 2>/dev/null || \
cargo build --release --locked
echo "      OK"
echo

# Try cargo-wix if available
echo "[2/3] Building MSI..."
if command -v cargo-wix &>/dev/null || cargo wix --version &>/dev/null 2>&1; then
    cargo wix --no-build -p voxy-daemon -I wix/main.wxs
    echo "      MSI built"
else
    echo "      cargo-wix not available — skipping MSI"
    echo "      Install: cargo install cargo-wix"
fi
echo

echo "[3/3] Done."
echo
echo "════════════════════════════════════════════════════════"
echo " Artifacts:"
echo "   target/wix/voxy-*.msi"
echo "════════════════════════════════════════════════════════"
