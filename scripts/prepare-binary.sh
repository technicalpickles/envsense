#!/usr/bin/env bash

# prepare-binary.sh
#
# Purpose: Prepare binary for release (copy, rename, checksum, test)
# Usage: ./prepare-binary.sh <version> <target>
#
# Arguments:
#   version: Version string (e.g., 0.1.0)
#   target: Target triple (e.g., x86_64-apple-darwin)
#
# Outputs:
#   - Binary in dist/ directory with release naming
#   - SHA256 checksum file
#   - Sets BINARY_NAME environment variable

set -euo pipefail

VERSION="$1"
TARGET="$2"

echo "Preparing binary for version $VERSION, target $TARGET"

# Create output directory
mkdir -p dist

# Handle universal binary naming
if [[ "$TARGET" == "universal-apple-darwin" ]]; then
  BINARY_NAME="envsense-v${VERSION}-universal-apple-darwin"
elif [[ "$TARGET" == *"windows"* ]]; then
  BINARY_NAME="envsense-v${VERSION}-${TARGET}.exe"
else
  BINARY_NAME="envsense-v${VERSION}-${TARGET}"
fi

echo "Binary name: $BINARY_NAME"

# Copy binary with appropriate extension
if [[ "$TARGET" == *"windows"* ]]; then
  cp "target/${TARGET}/release/envsense.exe" "dist/${BINARY_NAME}"
else
  cp "target/${TARGET}/release/envsense" "dist/${BINARY_NAME}"
fi

# Make executable (for non-Windows)
if [[ "$TARGET" != *"windows"* ]]; then
  chmod +x "dist/${BINARY_NAME}"
fi

# Generate checksum
if command -v sha256sum >/dev/null 2>&1; then
  (cd dist && sha256sum "${BINARY_NAME}" > "${BINARY_NAME}.sha256")
elif command -v shasum >/dev/null 2>&1; then
  (cd dist && shasum -a 256 "${BINARY_NAME}" > "${BINARY_NAME}.sha256")
fi

# Test binary functionality
echo "Testing binary functionality..."
BINARY_PATH="dist/${BINARY_NAME}"

# Test that binary runs and shows help
"./${BINARY_PATH}" --help > /dev/null

# Test basic functionality  
"./${BINARY_PATH}" info --json | head -5 > /dev/null

echo "Binary tests passed"

# Output for GitHub Actions
if [ -n "${GITHUB_ENV:-}" ]; then
  echo "BINARY_NAME=${BINARY_NAME}" >> "$GITHUB_ENV"
fi
echo "Binary prepared: $BINARY_NAME"
