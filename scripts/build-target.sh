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
    if ! command -v cross >/dev/null 2>&1; then
      echo "Installing cross..."
      cargo install cross --git https://github.com/cross-rs/cross
    fi
    
    echo "Attempting cross-compilation for $TARGET..."
    if cross build --release --target "$TARGET"; then
      echo "Cross-compilation successful"
    else
      echo "Cross-compilation failed for $TARGET"
      echo "This may be due to environment compatibility issues"
      exit 1
    fi
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
