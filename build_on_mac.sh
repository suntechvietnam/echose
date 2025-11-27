#!/usr/bin/env bash

set -euo pipefail

echo "==> Building echose for macOS..."

if [[ "$(uname)" != "Darwin" ]]; then
  echo "This script is intended to be run on macOS only."
  exit 1
fi

cd "$(dirname "$0")"

# Ensure we target a recent enough macOS so C++ std::filesystem is available
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.15}"

# Propagate minimum version flag to C/C++ and Rust (fixes availability errors)
export CFLAGS="-mmacosx-version-min=$MACOSX_DEPLOYMENT_TARGET ${CFLAGS:-}"
export CXXFLAGS="-mmacosx-version-min=$MACOSX_DEPLOYMENT_TARGET ${CXXFLAGS:-}"
export RUSTFLAGS="-C link-arg=-mmacosx-version-min=$MACOSX_DEPLOYMENT_TARGET ${RUSTFLAGS:-}"

# Tell CMake to use the same minimum macOS version (overrides default 10.13 from whisper-rs)
export CMAKE_OSX_DEPLOYMENT_TARGET="$MACOSX_DEPLOYMENT_TARGET"

# Make sure macOS SDK path is correctly set (helps avoid some build tool issues)
if command -v xcrun >/dev/null 2>&1; then
  export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"
fi

# Use Tauri CLI from Cargo (does not affect Windows builds)
export PATH="$HOME/.cargo/bin:$PATH"

echo "MACOSX_DEPLOYMENT_TARGET=$MACOSX_DEPLOYMENT_TARGET"
echo "CFLAGS=$CFLAGS"
echo "CXXFLAGS=$CXXFLAGS"
echo "RUSTFLAGS=$RUSTFLAGS"

# On mac we want an installable bundle (dmg) regardless of Windows config
export TAURI_BUNDLE_TARGETS="${TAURI_BUNDLE_TARGETS:-dmg}"
echo "TAURI_BUNDLE_TARGETS=$TAURI_BUNDLE_TARGETS"

tauri build

echo "==> Build finished. Check the 'src-tauri/target/release/bundle' folder for the macOS app."


