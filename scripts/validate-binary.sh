#!/usr/bin/env bash


VERSION="$1"
TARGET="$2"

echo "Testing binary preparation script..."
./scripts/prepare-binary.sh "$VERSION" "$TARGET"


echo "Testing prepared binary functionality..."
BINARY="dist/envsense-v${VERSION}-test-${TARGET}"

# Test binary functionality
if [ -f "$BINARY" ]; then
    echo "Testing binary: $BINARY"
    "./$BINARY" --help > /dev/null
    "./$BINARY" info --json | head -5
    echo "Binary functionality test passed!"
else
    echo "Binary not found: $BINARY"
    ls -la dist/
    exit 1
fi