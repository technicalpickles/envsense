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

case "$TARGET" in
  "universal-apple-darwin")
    BINARY_NAME="envsense-${VERSION}-universal-apple-darwin"
    ;;
  *)
    BINARY_NAME="envsense-${VERSION}-${TARGET}"
    ;;
esac

echo "Binary name: $BINARY_NAME"

cp "target/${TARGET}/release/envsense" "dist/${BINARY_NAME}"

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
