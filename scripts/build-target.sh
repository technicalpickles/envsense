#!/usr/bin/env bash

# build-target.sh
#
# Purpose: Build a specific target (including universal binaries)
# Usage: ./build-target.sh <target> [build_type]
#
# Arguments:
#   target: The Rust target triple (e.g., x86_64-apple-darwin)
#   build_type: "universal" or "normal" (default: normal)

set -euo pipefail

TARGET="$1"
BUILD_TYPE="${2:-normal}"

echo "Building target: $TARGET (type: $BUILD_TYPE)"

case "$BUILD_TYPE" in
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
