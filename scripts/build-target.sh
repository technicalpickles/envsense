#!/usr/bin/env bash

# build-target.sh
#
# Purpose: Build a specific target (including universal binaries)
# Usage: ./build-target.sh <target> [build_type]
#
# Arguments:
#   target: The Rust target triple (e.g., x86_64-apple-darwin)
#   build_type: "cross", "universal", or "normal" (default: normal)

set -euo pipefail

TARGET="$1"
BUILD_TYPE="${2:-normal}"

echo "Building target: $TARGET (type: $BUILD_TYPE)"

case "$BUILD_TYPE" in
  "cross")
    echo "Using cross for compilation"
    
    # Check if cross is already installed
    if ! command -v cross >/dev/null 2>&1; then
      echo "Installing cross..."
      # Use a specific version that's more stable in CI
      cargo install cross --git https://github.com/cross-rs/cross --rev 19be83481fd3e50ea103d800d72e0f8eddb1c90c
    else
      echo "Cross already installed: $(cross --version)"
    fi
    
    # Set environment variables for better CI compatibility
    export CROSS_CONTAINER_IN_CONTAINER=true
    
    echo "Attempting cross-compilation for $TARGET..."
    cross build --release --target "$TARGET"
    ;;
    
  "universal")
    echo "Building universal binary for macOS"
    
    # Build both architectures
    cargo build --release --target x86_64-apple-darwin
    cargo build --release --target aarch64-apple-darwin
    
    # Create universal binary using lipo
    mkdir -p "target/universal-apple-darwin/release"
    lipo -create \
      "target/x86_64-apple-darwin/release/envsense" \
      "target/aarch64-apple-darwin/release/envsense" \
      -output "target/universal-apple-darwin/release/envsense"
    
    # Verify the universal binary
    echo "Universal binary created:"
    lipo -info "target/universal-apple-darwin/release/envsense"
    ;;
    
  "normal"|*)
    echo "Using standard cargo build"
    cargo build --release --target "$TARGET"
    ;;
esac

echo "Build completed for $TARGET"
