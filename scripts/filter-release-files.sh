#!/usr/bin/env bash
# Filter release files from dist directory

set -euo pipefail

DIST_DIR="${1:-dist}"
RELEASE_DIR="${2:-release-files}"

if [ ! -d "$DIST_DIR" ]; then
    echo "❌ Distribution directory $DIST_DIR does not exist"
    exit 1
fi

echo "📁 Filtering release files..."
echo "  Source: $DIST_DIR"
echo "  Target: $RELEASE_DIR"

# Create release-files directory
mkdir -p "$RELEASE_DIR"

# Copy only non-test files to release-files directory
echo "  Copying non-test files..."
find "$DIST_DIR/" -name "envsense-*" -not -name "*-test*" -exec cp {} "$RELEASE_DIR/" \;

# Count and display results
FILE_COUNT=$(find "$RELEASE_DIR" -name "envsense-*" | wc -l)
echo
echo "✅ Filtered $FILE_COUNT release files:"
ls -la "$RELEASE_DIR/"

if [ $FILE_COUNT -eq 0 ]; then
    echo "⚠️  No files were found to filter. This might indicate an issue."
    exit 1
fi
