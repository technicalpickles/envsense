#!/usr/bin/env bash
# Check if signing completed successfully without doing full verification

set -euo pipefail

RELEASE_DIR="${1:-release-files}"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "❌ Release directory $RELEASE_DIR does not exist"
    exit 1
fi

echo "🔍 Checking signing completion in $RELEASE_DIR"
cd "$RELEASE_DIR"

SIGNED_COUNT=0
MISSING_SIGS=0
TOTAL_BINARIES=0

for file in envsense-*; do
    if [[ "$file" != *.sha256 && "$file" != *.sig && "$file" != *.bundle ]]; then
        TOTAL_BINARIES=$((TOTAL_BINARIES + 1))
        echo "  📦 Checking: $file"
        
        SIG_EXISTS=false
        BUNDLE_EXISTS=false
        
        if [ -f "${file}.sig" ]; then
            SIG_SIZE=$(stat -c%s "${file}.sig" 2>/dev/null || stat -f%z "${file}.sig" 2>/dev/null || echo "unknown")
            echo "    ✅ Signature: ${file}.sig (${SIG_SIZE} bytes)"
            SIG_EXISTS=true
        else
            echo "    ❌ Missing: ${file}.sig"
        fi
        
        if [ -f "${file}.bundle" ]; then
            BUNDLE_SIZE=$(stat -c%s "${file}.bundle" 2>/dev/null || stat -f%z "${file}.bundle" 2>/dev/null || echo "unknown")
            echo "    ✅ Bundle: ${file}.bundle (${BUNDLE_SIZE} bytes)"
            BUNDLE_EXISTS=true
        else
            echo "    ❌ Missing: ${file}.bundle"
        fi
        
        if [ "$SIG_EXISTS" = true ] && [ "$BUNDLE_EXISTS" = true ]; then
            SIGNED_COUNT=$((SIGNED_COUNT + 1))
            echo "    ✅ Complete: Both signature and bundle present"
        else
            MISSING_SIGS=$((MISSING_SIGS + 1))
            echo "    ❌ Incomplete: Missing signature files"
        fi
        echo
    fi
done

echo "📊 Signing Summary:"
echo "  📦 Total binaries: $TOTAL_BINARIES"
echo "  ✅ Successfully signed: $SIGNED_COUNT"
echo "  ❌ Missing signatures: $MISSING_SIGS"

if [ $MISSING_SIGS -gt 0 ]; then
    echo
    echo "💥 Signing incomplete! Some files are missing signatures."
    exit 1
else
    echo
    echo "🎉 All files have been signed successfully!"
    echo "📁 Files ready for release:"
    ls -la *.sig *.bundle 2>/dev/null || echo "No signature files found"
fi
